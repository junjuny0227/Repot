import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { loadAnalytics, parseOptions, run, runCandidates, selectSources } from './cli.mjs';
import { record } from './normalize.mjs';
import { applyReview } from './review.mjs';

const source = (id, kind = 'formula') => ({
  ...record(id, [id], ['https://github.com/a/' + id], kind === 'formula' ? 'cli' : 'gui'),
  brew_kind: kind,
});
const brew = [
  source('ripgrep'),
  source('docker-desktop', 'cask'),
  source('dual'),
  source('dual', 'cask'),
];
const scan = {
  scan: {
    items: [
      { name: 'ripgrep', details: { package: { manager: 'brew', kind: 'formula' } } },
      { name: 'not-brew', provider: 'npm' },
    ],
  },
};
test('merges names/scan/analytics, skips mapped including null and reports unknown/ambiguous tokens', () => {
  const selected = selectSources({
    brew,
    names: ['ripgrep', 'dual', 'missing', 'brewCask:dual'],
    scan,
    analytics: [
      {
        kind: 'cask',
        data: {
          items: [
            { cask: 'homebrew/cask/docker-desktop', count: '1,000' },
            { cask: 'other/tap/dual', count: '9,000' },
          ],
        },
      },
    ],
    top: 1,
    packages: { rg: { brew: 'ripgrep', winget: 'RG', scoop: null } },
  });
  assert.deepEqual(
    selected.sources.map((s) => [s.id, s.brew_kind]),
    [
      ['docker-desktop', 'cask'],
      ['dual', 'cask'],
    ],
  );
  assert.deepEqual(
    selected.issues.map((i) => i.type),
    ['ambiguous', 'unknown'],
  );
  assert.equal(selected.skipped[0].id, 'ripgrep');
});
test('reads brew formulae and casks from a core ScanResult (top-level items) and skips taps', () => {
  const scanResult = {
    items: [
      { name: 'ripgrep', details: { package: { manager: 'brew', kind: 'formula' } } },
      { name: 'docker-desktop', details: { package: { manager: 'brew', kind: 'cask' } } },
      { name: 'homebrew/cask-fonts', details: { package: { manager: 'brew', kind: 'tap' } } },
    ],
  };
  const selected = selectSources({ brew, scan: scanResult, packages: {} });
  assert.deepEqual(
    selected.sources.map((s) => [s.id, s.brew_kind]),
    [
      ['docker-desktop', 'cask'],
      ['ripgrep', 'formula'],
    ].sort(),
  );
  assert.deepEqual(selected.issues, []);
});
test('rejects a scan file with neither items nor scan.items', () => {
  assert.throws(
    () => selectSources({ brew, scan: { providers: [] }, packages: {} }),
    /Invalid Repot scan/,
  );
});
test('dry-run output is repeatable with channel and kind flags and no data changes', async () => {
  const catalogs = {
    brew,
    winget: [record('Docker.DockerDesktop', ['docker-desktop'], [], 'gui')],
    scoop: [],
  };
  const options = { names: ['docker-desktop'], packages: {}, catalogs };
  const before = await readFile(
    new URL('../../crates/core/data/package-map.json', import.meta.url),
    'utf8',
  );
  const a = await runCandidates(options),
    b = await runCandidates(options);
  assert.equal(JSON.stringify(a), JSON.stringify(b));
  assert.equal(a.rows.length, 2);
  assert.deepEqual(
    a.rows[0].candidates.map((c) => [c.tier, c.kindFlag, c.channelMismatch]),
    [['LOW', false, false]],
  );
  assert.equal(
    await readFile(new URL('../../crates/core/data/package-map.json', import.meta.url), 'utf8'),
    before,
  );
});
test('CLI options reject unsupported commands/flags and bad counts', () => {
  assert.equal(
    parseOptions(['candidates', '--offline', '--names', 'ripgrep,docker-desktop']).offline,
    true,
  );
  assert.throws(() => parseOptions(['candidates', '--from-analytics=-1']), /positive integer/);
  assert.throws(() => parseOptions(['run']), /Usage/);
  assert.throws(() => parseOptions(['candidates', '--oops']), /Unknown option/);
});
test('analytics is cached; offline never fetches, and missing offline analytics fails', async () => {
  const cacheDir = await mkdtemp(join(tmpdir(), 'map-analytics-'));
  try {
    const expected = [{ formula: 'ripgrep', count: '10' }];
    let calls = 0;
    const fetch = async () => {
      calls++;
      return Response.json({ items: expected });
    };
    const online = await loadAnalytics({ cacheDir, fetch });
    assert.equal(calls, 2);
    assert.equal(online[0].data.items[0].formula, 'ripgrep');
    assert.deepEqual(
      await loadAnalytics({
        cacheDir,
        offline: true,
        fetch: () => {
          throw Error('network');
        },
      }),
      online,
    );
    await rm(join(cacheDir, 'analytics-formula.json'));
    await assert.rejects(
      loadAnalytics({ cacheDir, offline: true, fetch }),
      /Offline analytics cache missing/,
    );
    assert.equal(calls, 2);
  } finally {
    await rm(cacheDir, { recursive: true, force: true });
  }
});
test('run/apply-review options validate finite thresholds and nonnegative budgets', () => {
  const options = parseOptions([
    'run',
    '--names',
    'sample',
    '--threshold',
    '0.8',
    '--max-requests',
    '2',
    '--credit-floor',
    '100',
    '--dry-run',
  ]);
  assert.equal(options.command, 'run');
  assert.equal(options.threshold, 0.8);
  assert.equal(options.maxRequests, 2);
  assert.equal(options.dryRun, true);
  assert.equal(parseOptions(['run', '--names', 'sample']).threshold, 0.95);
  assert.equal(parseOptions(['apply-review']).command, 'apply-review');
  assert.equal(parseOptions(['apply-review']).reviewer, 'human');
  assert.equal(parseOptions(['apply-review', '--reviewer', 'agent']).reviewer, 'agent');
  assert.throws(() => parseOptions(['apply-review', '--reviewer', 'robot']), /--reviewer/);
  assert.throws(
    () => parseOptions(['run', '--names', 'sample', '--reviewer', 'agent']),
    /Unknown option/,
  );
  for (const args of [
    ['--threshold', 'NaN'],
    ['--threshold', '1.1'],
    ['--max-requests', '-1'],
    ['--credit-floor', '-1'],
  ])
    assert.throws(() => parseOptions(['run', '--names', 'sample', ...args]));
});
test('end-to-end candidates → jeb → dry-run/review edit → apply → rerun: no secrets/network outside injected fetch', async () => {
  const cacheDir = await mkdtemp(join(tmpdir(), 'map-run-')),
    mapPath = join(cacheDir, 'map.json'),
    provenancePath = join(cacheDir, 'provenance.json');
  try {
    await writeFile(mapPath, JSON.stringify({ version: 1, packages: {} }));
    const catalogs = {
      brew: [source('sample')],
      winget: [record('Vendor.Sample', ['sample'], [], 'cli')],
      scoop: [record('main/sample', ['sample'], [], 'gui')],
    };
    let calls = 0;
    const options = {
      catalogs,
      names: ['sample'],
      cacheDir,
      mapPath,
      provenancePath,
      catalogSnapshot: '2026-11-03',
      key: 'synthetic',
      fetch: async () => {
        calls++;
        return Response.json(
          { answers: { q: { choice: '1', probabilities: { 1: 0.95 } } } },
          { headers: { 'x-ratelimit-remaining-credits': '900' } },
        );
      },
    };
    const before = await readFile(mapPath, 'utf8');
    const dry = await run({ ...options, dryRun: true });
    assert.equal(await readFile(mapPath, 'utf8'), before);
    assert.deepEqual(dry.counts, {
      auto: 1,
      review: 1,
      'no-candidate': 0,
      incomplete: 0,
      added: 1,
      conflicts: 0,
    });
    const real = await run({ ...options, offline: true });
    assert.equal(real.counts.added, 1);
    assert.equal(calls, 2);
    const reviewPath = join(cacheDir, 'review.json'),
      review = JSON.parse(await readFile(reviewPath));
    review.items[0].decision = 'none';
    await writeFile(reviewPath, JSON.stringify(review));
    await applyReview({ mapPath, provenancePath, reviewPath });
    assert.equal(JSON.parse(await readFile(mapPath)).packages.sample.scoop, null);
    const rerun = await run({ ...options, offline: true });
    assert.equal(rerun.counts.added, 0);
    assert.equal(rerun.skipped.length, 1);
    assert.equal(calls, 2);
  } finally {
    await rm(cacheDir, { recursive: true, force: true });
  }
});
