import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { record } from './normalize.mjs';
import { selectSources, runCandidates, parseOptions, loadAnalytics } from './cli.mjs';
const source = (id, kind = 'formula') => ({ ...record(id, [id], ['https://github.com/a/' + id], kind === 'formula' ? 'cli' : 'gui'), brew_kind: kind });
const brew = [source('ripgrep'), source('docker-desktop', 'cask'), source('dual'), source('dual', 'cask')];
const scan = { scan: { items: [{ name: 'ripgrep', details: { package: { manager: 'brew', kind: 'formula' } } }, { name: 'not-brew', provider: 'npm' }] } };
test('merges names/scan/analytics, skips mapped including null and reports unknown/ambiguous tokens', () => {
  const selected = selectSources({ brew, names: ['ripgrep', 'dual', 'missing', 'brewCask:dual'], scan, analytics: [{ kind: 'cask', data: { items: [{ cask: 'homebrew/cask/docker-desktop', count: '1,000' }, { cask: 'other/tap/dual', count: '9,000' }] } }], top: 1, packages: { rg: { brew: 'ripgrep', winget: 'RG', scoop: null } } });
  assert.deepEqual(selected.sources.map(s => [s.id, s.brew_kind]), [['docker-desktop', 'cask'], ['dual', 'cask']]);
  assert.deepEqual(selected.issues.map(i => i.type), ['ambiguous', 'unknown']);
  assert.equal(selected.skipped[0].id, 'ripgrep');
});
test('dry-run output is repeatable with channel and kind flags and no data changes', async () => {
  const catalogs = { brew, winget: [record('Docker.DockerDesktop', ['docker-desktop'], [], 'gui')], scoop: [] };
  const options = { names: ['docker-desktop'], packages: {}, catalogs };
  const before = await readFile(new URL('../../crates/core/data/package-map.json', import.meta.url), 'utf8');
  const a = await runCandidates(options), b = await runCandidates(options);
  assert.equal(JSON.stringify(a), JSON.stringify(b));
  assert.equal(a.rows.length, 2);
  assert.deepEqual(a.rows[0].candidates.map(c => [c.tier, c.kindFlag, c.channelMismatch]), [['LOW', false, false]]);
  assert.equal(await readFile(new URL('../../crates/core/data/package-map.json', import.meta.url), 'utf8'), before);
});
test('CLI options reject unsupported commands/flags and bad counts', () => {
  assert.equal(parseOptions(['candidates', '--offline', '--names', 'ripgrep,docker-desktop']).offline, true);
  assert.throws(() => parseOptions(['candidates', '--from-analytics=-1']), /positive integer/);
  assert.throws(() => parseOptions(['run']), /Usage/);
  assert.throws(() => parseOptions(['candidates', '--oops']), /Unknown option/);
});
test('analytics is cached; offline never fetches, and missing offline analytics fails', async () => {
  const cacheDir = await mkdtemp(join(tmpdir(), 'map-analytics-'));
  try {
    const expected = [{ formula: 'ripgrep', count: '10' }]; let calls = 0;
    const fetch = async () => { calls++; return Response.json({ items: expected }); };
    const online = await loadAnalytics({ cacheDir, fetch });
    assert.equal(calls, 2); assert.equal(online[0].data.items[0].formula, 'ripgrep');
    assert.deepEqual(await loadAnalytics({ cacheDir, offline: true, fetch: () => { throw Error('network'); } }), online);
    await rm(join(cacheDir, 'analytics-formula.json'));
    await assert.rejects(loadAnalytics({ cacheDir, offline: true, fetch }), /Offline analytics cache missing/);
    assert.equal(calls, 2);
  } finally { await rm(cacheDir, { recursive: true, force: true }); }
});
