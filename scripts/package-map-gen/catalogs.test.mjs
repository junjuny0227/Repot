import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import { parseBrew, parseScoop, parseWinget, loadCatalogs, BUCKETS } from './catalogs.mjs';
const fixture = JSON.parse(await readFile(new URL('./fixtures/catalogs.json', import.meta.url), 'utf8'));
export function tar(files) {
  return Buffer.concat([...files.flatMap(([name, text]) => {
    const header = Buffer.alloc(512); header.write(name); header.write(Buffer.byteLength(text).toString(8).padStart(11, '0'), 124); header[156] = 48;
    return [header, Buffer.from(text), Buffer.alloc((512 - Buffer.byteLength(text) % 512) % 512)];
  }), Buffer.alloc(1024)]);
}
const manifests = [
  ['root/manifests/a/Acme/Tool/1.9/Acme.Tool.yaml', 'PackageIdentifier: Acme.Tool\nPackageVersion: 1.9\nManifestType: singleton\nPackageName: Old\n'],
  ['root/manifests/a/Acme/Tool/1.10/Acme.Tool.locale.fr-FR.yaml', 'ManifestType: locale\nPackageLocale: fr-FR\nPackageName: French\n'],
  ['root/manifests/a/Acme/Tool/1.10/Acme.Tool.installer.yaml', 'ManifestType: installer\nInstallers:\n- InstallerUrl: https://github.com/acme/tool/releases/tool.exe\n'],
  ['root/manifests/a/Acme/Tool/1.10/Acme.Tool.locale.en-US.yaml', 'ManifestType: defaultLocale\nPackageName: "Acme Tool"\nPackageUrl: https://example.org/tool\nTags:\n- cli\n'],
];
test('small catalog fixtures preserve source kinds, architecture URLs and latest winget locale/installer', async () => {
  assert.deepEqual(parseBrew(fixture.formula, fixture.cask).map(r => r.kind), ['cli', 'mixed', 'gui']);
  const scoop = parseScoop('main', 'tool', fixture.scoop);
  assert.equal(scoop.id, 'main/tool'); assert.equal(scoop.kind, 'mixed');
  assert.ok(scoop.repos.includes('github.com/acme/tool'));
  const records = await parseWinget((async function* () { for (const byte of tar(manifests)) yield Buffer.from([byte]); })());
  assert.equal(records.length, 1); assert.equal(records[0].version, '1.10');
  assert.equal(records[0].kind, 'cli'); assert.ok(records[0].names.includes('acmetool'));
  assert.deepEqual(records[0].repos, ['github.com/acme/tool']);
});
test('catalog acquisition records hashes; offline reuses snapshot with zero transports and detects missing/corrupt cache', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'map-gen-'));
  let fetches = 0, clones = 0;
  try {
    const fetch = async url => { fetches++; return new Response(url.endsWith('formula.json') ? JSON.stringify(fixture.formula) : url.endsWith('cask.json') ? JSON.stringify(fixture.cask) : gzipSync(tar(manifests))); };
    const git = async (_cmd, args) => { clones++; if (args.at(-2).includes('nerd-fonts')) assert.equal(args.at(-2), 'https://github.com/matthewjberger/scoop-nerd-fonts.git'); const target = args.at(-1); await mkdir(join(target, 'bucket'), { recursive: true }); await writeFile(join(target, 'bucket/tool.json'), JSON.stringify(fixture.scoop)); };
    const online = await loadCatalogs({ cacheDir: dir, fetch, git });
    assert.equal(fetches, 3); assert.equal(clones, BUCKETS.length);
    assert.equal(online.scoop.length, 5);
    const sources = JSON.parse(await readFile(join(dir, 'sources.json'), 'utf8'));
    assert.equal(sources.sources.length, 8);
    assert.ok(sources.sources.every(s => s.url.startsWith('https://') && /^[a-f0-9]{64}$/.test(s.sha256) && !Number.isNaN(Date.parse(s.fetchedAt))));
    const snapshot = await readFile(join(dir, 'catalogs.json'));
    assert.equal(sources.snapshotSha256, createHash('sha256').update(snapshot).digest('hex'));
    const forbidden = () => { throw new Error('network called'); };
    assert.deepEqual(await loadCatalogs({ cacheDir: dir, offline: true, fetch: forbidden, git: forbidden }), online);
    await writeFile(join(dir, 'catalogs.json'), '{}');
    await assert.rejects(loadCatalogs({ cacheDir: dir, offline: true, fetch: forbidden }), /corrupt|hash/i);
    await rm(join(dir, 'catalogs.json'));
    await assert.rejects(loadCatalogs({ cacheDir: dir, offline: true, fetch: forbidden }), /offline.*missing/i);
  } finally { await rm(dir, { recursive: true, force: true }); }
});
