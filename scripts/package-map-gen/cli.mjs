import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import { createHash } from 'node:crypto';
import { loadCatalogs } from './catalogs.mjs';
import { index, propose } from './candidates.mjs';
import { preFilter } from './filter.mjs';
const cacheDefault = '.cache/package-map-gen';
const usage = 'Usage: pnpm package-map:gen candidates [--offline] [--names token,brew:token,brewCask:token] [--from-analytics N] [--from-scan scan.json] [--cache-dir path]';
export function parseOptions(args) {
  if (args[0] !== 'candidates') throw new Error(usage);
  const { values } = parseArgs({ args: args.slice(1), options: { offline: { type: 'boolean', default: false }, names: { type: 'string' }, 'from-analytics': { type: 'string' }, 'from-scan': { type: 'string' }, 'cache-dir': { type: 'string' } } });
  const top = values['from-analytics'] === undefined ? 0 : Number(values['from-analytics']);
  if (values['from-analytics'] !== undefined && (!Number.isSafeInteger(top) || top <= 0)) throw new Error('--from-analytics requires a positive integer');
  const names = values.names?.split(',').map(s => s.trim()).filter(Boolean) ?? [];
  if (!names.length && !top && !values['from-scan']) throw new Error(usage);
  return { offline: values.offline, names, top, scanPath: values['from-scan'], cacheDir: values['cache-dir'] ?? cacheDefault };
}
export async function loadAnalytics({ cacheDir = cacheDefault, offline = false, fetch = globalThis.fetch } = {}) {
  const results = [];
  for (const kind of ['formula', 'cask']) {
    const path = join(cacheDir, `analytics-${kind}.json`); let snapshot;
    try { snapshot = JSON.parse(await readFile(path, 'utf8')); }
    catch (error) {
      if (error.code !== 'ENOENT') throw error;
      if (offline) throw new Error(`Offline analytics cache missing: ${kind}`);
      const url = `https://formulae.brew.sh/api/analytics/${kind === 'formula' ? 'install-on-request' : 'cask-install'}/30d.json`;
      const response = await fetch(url);
      if (!response.ok) throw new Error(`Analytics download failed (${response.status})`);
      const text = await response.text();
      snapshot = { url, fetchedAt: new Date().toISOString(), sha256: createHash('sha256').update(text).digest('hex'), data: JSON.parse(text) };
      await mkdir(cacheDir, { recursive: true }); await writeFile(path, JSON.stringify(snapshot));
    }
    results.push({ kind, data: snapshot.data });
  }
  return results;
}
export function selectSources({ brew, names = [], scan, analytics = [], top = 0, packages = {} }) {
  const requested = [], issues = [], skipped = [], sources = [];
  for (const name of names) {
    const match = /^(brew|brewCask):(.+)$/.exec(name);
    requested.push({ id: match ? match[2] : name, kind: match ? (match[1] === 'brew' ? 'formula' : 'cask') : undefined });
  }
  if (scan) {
    if (!Array.isArray(scan.scan?.items)) throw new Error('Invalid Repot scan: expected scan.items');
    for (const item of scan.scan.items) if (item.details?.package?.manager === 'brew') requested.push({ id: item.name, kind: item.details.package.kind });
  }
  for (const { kind, data } of analytics) {
    const items = Array.isArray(data.items) ? data.items : Object.values(data.items ?? {}).flat();
    const counts = new Map();
    for (const item of items) {
      let name = item[kind === 'formula' ? 'formula' : 'cask']?.replace(/\s+--.*$/, '');
      if (!name || (name.includes('/') && !name.startsWith(kind === 'formula' ? 'homebrew/core/' : 'homebrew/cask/'))) continue;
      name = name.split('/').at(-1);
      const count = Number(String(item.count ?? 0).replaceAll(',', ''));
      if (!Number.isFinite(count)) continue;
      counts.set(name, (counts.get(name) ?? 0) + count);
    }
    for (const [id] of [...counts].sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1)).slice(0, top)) requested.push({ id, kind });
  }
  const seen = new Set();
  for (const request of requested) {
    const matches = brew.filter(b => b.id === request.id && (!request.kind || b.brew_kind === request.kind));
    if (matches.length !== 1) { issues.push({ id: request.id, type: matches.length ? 'ambiguous' : 'unknown', message: matches.length ? 'Use brew:token or brewCask:token to disambiguate' : 'Token not found in Homebrew catalog' }); continue; }
    const source = matches[0], key = `${source.brew_kind}:${source.id}`;
    if (seen.has(key)) continue; seen.add(key);
    const manager = source.brew_kind === 'formula' ? 'brew' : 'brewCask';
    if (Object.values(packages).some(p => p[manager] === source.id && Object.hasOwn(p, 'winget') && Object.hasOwn(p, 'scoop'))) skipped.push({ id: source.id, kind: source.brew_kind, reason: 'winget and scoop already mapped' });
    else sources.push(source);
  }
  sources.sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : a.brew_kind < b.brew_kind ? -1 : 1);
  return { sources, skipped, issues };
}
export async function runCandidates(options) {
  const catalogs = options.catalogs ?? await loadCatalogs(options);
  const packages = options.packages ?? JSON.parse(await readFile(new URL('../../crates/core/data/package-map.json', import.meta.url), 'utf8')).packages;
  const scan = options.scan ?? (options.scanPath ? JSON.parse(await readFile(options.scanPath, 'utf8')) : undefined);
  const analytics = options.analytics ?? (options.top ? await loadAnalytics(options) : []);
  const { sources, skipped, issues } = selectSources({ ...options, brew: catalogs.brew, packages, scan, analytics });
  const indices = Object.fromEntries(['winget', 'scoop'].map(target => [target, index(catalogs[target])]));
  const rows = sources.flatMap(source => ['winget', 'scoop'].map(target => ({ source: source.id, sourceKind: source.brew_kind, target, ...preFilter(source, propose(source, catalogs[target], indices[target])) })));
  return { rows, skipped, issues };
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runCandidates(parseOptions(process.argv.slice(2))).then(report => {
    console.log(JSON.stringify(report, null, 2));
    if (report.issues.length) process.exitCode = 1;
  }).catch(error => { console.error(error.message); process.exitCode = 1; });
}
