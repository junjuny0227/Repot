import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { applyDecisions, commonId, mergeDecisions } from './apply.mjs';

const row = (source, target, picked, extra = {}) => ({
  source,
  sourceKind: 'formula',
  target,
  picked,
  status: 'auto',
  method: 'jeb',
  probability: 0.9,
  catalogSnapshot: '2026-11-03T00:00:00Z',
  candidates: [{ id: picked, tier: 'MEDIUM' }],
  ...extra,
});
test('merge preserves existing strings/null, sorts ids, rejects collisions and invalid ids, enforces manager uniqueness including bare Scoop', () => {
  const map = {
    version: 1,
    packages: {
      zed: { brew: 'zed', winget: null },
      other: { brew: 'other', scoop: 'Tool', winget: 'Vendor.Existing' },
    },
  };
  const before = JSON.stringify(map);
  const result = mergeDecisions(map, undefined, [
    row('zed', 'winget', 'Vendor.Zed'),
    row('alpha', 'winget', 'Vendor.Alpha'),
    row('bad@2', 'winget', 'Vendor.Bad'),
    row('new', 'scoop', 'main/tool'),
    row('another', 'winget', 'vendor.existing'),
    row('OTHER', 'winget', 'Vendor.Collision'),
  ]);
  assert.equal(JSON.stringify(map), before);
  assert.deepEqual(Object.keys(result.map.packages), ['alpha', 'bad-2', 'other', 'zed']);
  assert.equal(result.map.packages.zed.winget, null);
  assert.equal(result.added.length, 2);
  assert.equal(result.skipped.length, 4);
  assert.deepEqual(result.provenance.packages.alpha.winget, {
    method: 'jeb',
    probability: 0.9,
    tier: 'MEDIUM',
    catalogSnapshot: '2026-11-03T00:00:00Z',
  });
  assert.equal(
    mergeDecisions(result.map, result.provenance, [row('alpha', 'winget', 'Vendor.Alpha')]).added
      .length,
    0,
  );
});
test('only explicit human none writes null; no-candidate and automatic none never create an entry', () => {
  const result = mergeDecisions({ version: 1, packages: {} }, undefined, [
    row('auto', 'winget', 'none'),
    row('empty', 'scoop', undefined, { status: 'no-candidate' }),
    row('human', 'scoop', 'none', { status: 'human', method: 'human' }),
  ]);
  assert.deepEqual(result.map.packages, { human: { brew: 'human', scoop: null } });
  assert.equal(result.provenance.packages.human.scoop.probability, null);
});
test('invalid names fail full preflight without changing either data file; dry-run and first apply sidecar', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'apply-map-')),
    mapPath = join(dir, 'map.json'),
    provenancePath = join(dir, 'provenance.json');
  try {
    const original = '{"version":1,"packages":{}}\n';
    await writeFile(mapPath, original);
    await assert.rejects(
      applyDecisions([row('good', 'winget', 'Vendor.Good'), row('bad', 'winget', 'bad name')], {
        mapPath,
        provenancePath,
      }),
      /Invalid.*name/,
    );
    assert.equal(await readFile(mapPath, 'utf8'), original);
    await assert.rejects(readFile(provenancePath), { code: 'ENOENT' });
    await applyDecisions([row('good', 'winget', 'Vendor.Good')], {
      mapPath,
      provenancePath,
      dryRun: true,
    });
    assert.equal(await readFile(mapPath, 'utf8'), original);
    await assert.rejects(readFile(provenancePath), { code: 'ENOENT' });
    await applyDecisions([row('good', 'winget', 'Vendor.Good')], { mapPath, provenancePath });
    assert.equal(JSON.parse(await readFile(mapPath)).packages.good.winget, 'Vendor.Good');
    assert.equal(JSON.parse(await readFile(provenancePath)).packages.good.winget.method, 'jeb');
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});
test('valid constructor id is an own package key, never an inherited object', () => {
  const result = mergeDecisions({ version: 1, packages: {} }, undefined, [
    row('constructor', 'winget', 'Vendor.Constructor'),
  ]);
  assert.ok(Object.hasOwn(result.map.packages, 'constructor'));
  assert.equal(result.map.packages.constructor.brew, 'constructor');
  assert.ok(Object.hasOwn(result.provenance.packages, 'constructor'));
  assert.equal(Object.winget, undefined);
});
test('versioned and symbol tokens get normalized ids; collisions and empty ids are skipped', () => {
  assert.deepEqual(
    [
      'python@3.15',
      'node@22',
      'temurin@17',
      'claude-code@latest',
      'powershell@preview',
      'logi-options+',
      'whisper.cpp',
      'Foo++Bar',
      'x@@1',
    ].map(commonId),
    [
      'python-3-15',
      'node-22',
      'temurin-17',
      'claude-code-latest',
      'powershell-preview',
      'logi-options-plus',
      'whisper-cpp',
      'foo-plus-plus-bar',
      'x-1',
    ],
  );
  const map = {
    version: 1,
    packages: { 'node-22': { brew: 'node-other' }, 'openssl-3': { brew: 'openssl@3' } },
  };
  const result = mergeDecisions(map, undefined, [
    row('python@3.15', 'winget', 'Python.Python.3.15'),
    row('logi-options+', 'winget', 'Logitech.OptionsPlus', { sourceKind: 'cask' }),
    row('node@22', 'winget', 'OpenJS.NodeJS.22'),
    row('openssl@3', 'winget', 'ShiningLight.OpenSSL.Light'),
    row('@', 'winget', 'Vendor.At'),
  ]);
  assert.deepEqual(result.map.packages['python-3-15'], {
    brew: 'python@3.15',
    winget: 'Python.Python.3.15',
  });
  assert.deepEqual(result.map.packages['logi-options-plus'], {
    brewCask: 'logi-options+',
    winget: 'Logitech.OptionsPlus',
  });
  assert.equal(result.map.packages['openssl-3'].winget, 'ShiningLight.OpenSSL.Light');
  assert.equal(result.provenance.packages['python-3-15'].winget.method, 'jeb');
  assert.deepEqual(result.skipped, [
    { source: 'node@22', target: 'winget', reason: 'common id collision' },
    { source: '@', target: 'winget', reason: 'invalid common id' },
  ]);
});
