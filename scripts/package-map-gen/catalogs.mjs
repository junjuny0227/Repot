import { createReadStream, createWriteStream } from 'node:fs';
import { readFile, writeFile, mkdir, readdir, rename, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { createGunzip } from 'node:zlib';
import { pipeline } from 'node:stream/promises';
import { Readable, Transform } from 'node:stream';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { record, strings } from './normalize.mjs';
export const BUCKETS = ['main', 'extras', 'versions', 'java', 'nerd-fonts'];
const hash = value => createHash('sha256').update(value).digest('hex');
const formulaUrl = 'https://formulae.brew.sh/api/formula.json';
const caskUrl = 'https://formulae.brew.sh/api/cask.json';
const wingetUrl = 'https://codeload.github.com/microsoft/winget-pkgs/tar.gz/refs/heads/master';
export function parseBrew(formula, cask) {
  return [...formula.map(data => ({ ...record(data.name, [data.name, ...(data.oldnames ?? [])], [data.homepage, ...Object.values(data.urls ?? {}).flatMap(value => strings(value?.url))], 'cli'), brew_kind: 'formula', desc: data.desc })),
    ...cask.map(data => ({ ...record(data.token, [data.token, ...(data.name ?? [])], [data.homepage, data.url], data.artifacts?.some(artifact => artifact.binary) ? 'mixed' : 'gui'), brew_kind: 'cask', desc: data.desc }))];
}
export function parseScoop(bucket, name, data) {
  const arch = Object.values(data.architecture ?? {});
  const gui = Boolean(data.shortcuts || arch.some(value => value.shortcuts));
  const cli = Boolean(data.bin || arch.some(value => value.bin));
  return record(`${bucket}/${name}`, [name], [...strings(data.homepage), ...strings(data.url), ...arch.flatMap(value => strings(value.url)), ...strings(data.checkver?.github)], gui && cli ? 'mixed' : gui ? 'gui' : cli ? 'cli' : 'unknown');
}
function scalar(value) {
  value = value.trim();
  if (value.startsWith('"') && value.endsWith('"')) { try { return JSON.parse(value); } catch { return value.slice(1, -1); } }
  if (value.startsWith("'") && value.endsWith("'")) return value.slice(1, -1).replace(/''/g, "'");
  return value.replace(/\s+#.*$/, '').trim();
}
// Deliberately restricted field extractor, not a general YAML parser.
function yamlFields(text) {
  const fields = {}; let key;
  for (const line of text.replace(/^\uFEFF/, '').split(/\r?\n/)) {
    const match = /^(\w+):\s*(.*)$/.exec(line);
    if (match) { key = match[1]; fields[key] = match[2] ? scalar(match[2]) : []; }
    else if (key === 'Tags' && /^\s*-\s+/.test(line)) {
      if (!Array.isArray(fields.Tags)) fields.Tags = [];
      fields.Tags.push(scalar(line.replace(/^\s*-\s+/, '')));
    }
  }
  fields.installer_urls = [...text.matchAll(/^\s*(?:-\s*)?InstallerUrl:\s*["']?(https?:\/\/[^\s"']+)/gm)].map(match => match[1]);
  return fields;
}
function compareVersions(a, b) {
  const tokenize = value => value.match(/\d+|\D+/g) ?? [];
  const x = tokenize(a), y = tokenize(b);
  for (let i = 0; i < Math.min(x.length, y.length); i++) {
    const nx = /^\d+$/.test(x[i]), ny = /^\d+$/.test(y[i]);
    if (nx !== ny) return nx ? 1 : -1;
    const p = nx ? BigInt(x[i]) : x[i].toLowerCase(), q = ny ? BigInt(y[i]) : y[i].toLowerCase();
    if (p !== q) return p > q ? 1 : -1;
  }
  return x.length - y.length;
}
// Consume tar incrementally: historical manifests are never extracted to disk.
export async function parseWinget(stream) {
  const iterator = stream[Symbol.asyncIterator](); let buffer = Buffer.alloc(0), ended = false;
  async function take(size) {
    while (buffer.length < size && !ended) {
      const next = await iterator.next(); ended = next.done;
      if (!ended) buffer = Buffer.concat([buffer, Buffer.from(next.value)]);
    }
    if (buffer.length < size) throw new Error('Truncated winget tar archive');
    const out = buffer.subarray(0, size); buffer = buffer.subarray(size); return out;
  }
  const latest = new Map(); let longName;
  while (true) {
    const header = await take(512);
    if (header.every(byte => byte === 0)) break;
    const text = (start, length) => header.subarray(start, start + length).toString().replace(/\0.*$/s, '');
    let name = longName ?? [text(345, 155), text(0, 100)].filter(Boolean).join('/'); longName = undefined;
    const size = parseInt(text(124, 12).trim() || '0', 8), type = text(156, 1);
    if (!Number.isSafeInteger(size) || size < 0) throw new Error('Invalid winget tar size');
    const relevant = (type === '0' || type === '') && name.includes('/manifests/') && name.endsWith('.yaml');
    let payload;
    if (relevant || type === 'L' || type === 'x') payload = await take(size);
    else { for (let left = size; left > 0;) { const n = Math.min(left, 65536); await take(n); left -= n; } }
    if (size % 512) await take(512 - size % 512);
    if (type === 'L') { longName = payload.toString().replace(/\0.*$/s, '').trim(); continue; }
    if (type === 'x') { longName = /(?:^|\n)\d+ path=(.*)\n/.exec(payload.toString())?.[1]; continue; }
    if (!relevant) continue;
    const parts = name.split('/'), version = parts.at(-2), filename = parts.at(-1);
    const id = filename.replace(/\.(?:installer|locale\.[^.]+)\.yaml$/, '').replace(/\.yaml$/, '');
    let item = latest.get(id); const comparison = item ? compareVersions(version, item.version) : 1;
    if (comparison < 0) continue;
    if (comparison > 0) { item = { id, version, files: new Map() }; latest.set(id, item); }
    const fields = yamlFields(payload.toString('utf8'));
    if (['defaultLocale', 'singleton', 'installer', 'version'].includes(fields.ManifestType) || fields.PackageLocale === 'en-US') item.files.set(filename, fields);
  }
  return [...latest.values()].map(item => {
    const files = [...item.files.values()];
    const primary = files.find(f => ['defaultLocale', 'singleton'].includes(f.ManifestType)) ?? files.find(f => f.PackageLocale === 'en-US') ?? {};
    const names = [item.id.split('.').at(-1), primary.PackageName, primary.Moniker];
    const tags = Array.isArray(primary.Tags) ? primary.Tags : (primary.Tags ?? '').match(/[\w-]+/g) ?? [];
    const hint = [...tags, ...names].join(' ').toLowerCase();
    const cli = /\b(cli|command[ -]?line|terminal|console|shell)\b/.test(hint), gui = /\b(gui|desktop|browser|editor|ide|player|graphical)\b/.test(hint);
    return { ...record(item.id, names, [primary.PackageUrl, primary.PublisherUrl, ...files.flatMap(f => f.installer_urls)], cli && gui ? 'mixed' : cli ? 'cli' : gui ? 'gui' : 'unknown'), version: item.version, fields: primary };
  }).sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0);
}
export async function loadCatalogs({ cacheDir = '.cache/package-map-gen', offline = false, fetch = globalThis.fetch, git = promisify(execFile) } = {}) {
  try {
    const snapshot = await readFile(join(cacheDir, 'catalogs.json'));
    const sources = JSON.parse(await readFile(join(cacheDir, 'sources.json'), 'utf8'));
    if (hash(snapshot) !== sources.snapshotSha256) throw new Error('Catalog cache hash mismatch (corrupt snapshot)');
    return JSON.parse(snapshot);
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
    if (offline) throw new Error('Offline catalog cache missing; run candidates online first');
  }
  await mkdir(cacheDir, { recursive: true });
  const sources = [];
  async function download(url, filename) {
    const response = await fetch(url);
    if (!response.ok || !response.body) throw new Error(`Catalog download failed: ${url} (${response.status})`);
    const digest = createHash('sha256'), path = join(cacheDir, filename);
    const meter = new Transform({ transform(chunk, _encoding, callback) { digest.update(chunk); callback(null, chunk); } });
    await pipeline(Readable.fromWeb(response.body), meter, createWriteStream(`${path}.partial`));
    await rename(`${path}.partial`, path);
    sources.push({ url, fetchedAt: new Date().toISOString(), sha256: digest.digest('hex') }); return path;
  }
  const formulaPath = await download(formulaUrl, 'formula.json'), caskPath = await download(caskUrl, 'cask.json');
  const brew = parseBrew(JSON.parse(await readFile(formulaPath, 'utf8')), JSON.parse(await readFile(caskPath, 'utf8')));
  const wingetPath = await download(wingetUrl, 'winget.tar.gz');
  const winget = await parseWinget(createReadStream(wingetPath).pipe(createGunzip()));
  const scoop = [];
  for (const bucket of BUCKETS) {
    const url = `https://github.com/ScoopInstaller/${bucket}.git`, path = join(cacheDir, `scoop-${bucket}`);
    // A failed prior acquisition may leave an incomplete clone, never use it as a snapshot.
    await rm(path, { recursive: true, force: true });
    await git('git', ['clone', '--depth', '1', '--quiet', url, path]);
    const digest = createHash('sha256');
    for (const filename of (await readdir(join(path, 'bucket'))).filter(f => f.endsWith('.json')).sort()) {
      const bytes = await readFile(join(path, 'bucket', filename)); digest.update(filename); digest.update('\0'); digest.update(bytes);
      scoop.push(parseScoop(bucket, filename.slice(0, -5), JSON.parse(bytes.toString('utf8').replace(/^\uFEFF/, ''))));
    }
    sources.push({ url, fetchedAt: new Date().toISOString(), sha256: digest.digest('hex'), hashScope: 'sorted bucket filenames + NUL + manifest bytes' });
  }
  const catalogs = { brew, winget, scoop }, snapshot = JSON.stringify(catalogs);
  await writeFile(join(cacheDir, 'catalogs.json.partial'), snapshot);
  await rename(join(cacheDir, 'catalogs.json.partial'), join(cacheDir, 'catalogs.json'));
  await writeFile(join(cacheDir, 'sources.json'), JSON.stringify({ snapshotDate: new Date().toISOString(), snapshotSha256: hash(snapshot), sources }, null, 2) + '\n');
  return JSON.parse(snapshot);
}
