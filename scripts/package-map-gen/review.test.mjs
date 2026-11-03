import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { writeReview, applyReview } from './review.mjs';
const row = { source: 'sample', sourceKind: 'formula', target: 'winget', status: 'review', picked: 'none', probability: 0.9, catalogSnapshot: '2026-11-03', candidates: [{ id: 'Vendor.Sample', tier: 'LOW', kindFlag: true }] };
test('review persists blank decisions and metadata, preserves edits on rerun, human choice/none, skip, invalid decisions and stale map safety', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'review-')), reviewPath = join(dir, 'review.json'), mapPath = join(dir, 'map.json'), provenancePath = join(dir, 'provenance.json');
  const options = { reviewPath, mapPath, provenancePath };
  try {
    await writeFile(mapPath, JSON.stringify({ version: 1, packages: {} }));
    await writeReview([row], options);
    let review = JSON.parse(await readFile(reviewPath)); assert.equal(review.items[0].decision, ''); assert.equal(review.items[0].candidates[0].kindFlag, true);
    assert.equal((await applyReview(options)).added.length, 0);
    review.items[0].decision = 'unknown'; await writeFile(reviewPath, JSON.stringify(review));
    await assert.rejects(applyReview(options), /Invalid review decision/);
    review.items[0].decision = 'Vendor.Sample'; await writeFile(reviewPath, JSON.stringify(review));
    await writeReview([row], options);
    assert.equal(JSON.parse(await readFile(reviewPath)).items[0].decision, 'Vendor.Sample');
    assert.equal((await applyReview(options)).added.length, 1);
    assert.equal(JSON.parse(await readFile(provenancePath)).packages.sample.winget.method, 'human');
    assert.equal((await applyReview(options)).added.length, 0);
    review.items[0].decision = 'none'; review.items.push({ ...row, source: 'absent', target: 'scoop', decision: 'none' }, { ...row, source: 'ignored', decision: 'skip' });
    await writeFile(reviewPath, JSON.stringify(review));
    assert.equal((await applyReview(options)).added.length, 1);
    const map = JSON.parse(await readFile(mapPath));
    assert.equal(map.packages.sample.winget, 'Vendor.Sample'); assert.equal(map.packages.absent.scoop, null); assert.ok(!map.packages.ignored);
    // Changed candidate evidence must not inherit an old human decision.
    await writeReview([{ ...row, candidates: [{ ...row.candidates[0], tier: 'MEDIUM' }] }], options);
    assert.equal(JSON.parse(await readFile(reviewPath)).items.find(r => r.source === 'sample').decision, '');
  } finally { await rm(dir, { recursive: true, force: true }); }
});
test('apply-review records the reviewer: human and agent-review stay distinct, evidence is copied, none is allowed for both', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'review-agent-')), reviewPath = join(dir, 'review.json'), mapPath = join(dir, 'map.json'), provenancePath = join(dir, 'provenance.json');
  const options = { reviewPath, mapPath, provenancePath };
  const item = (source, target, decision, evidence) => ({ ...row, source, target, decision, ...(evidence === undefined ? {} : { evidence }) });
  try {
    await writeFile(mapPath, JSON.stringify({ version: 1, packages: {} }));
    await writeFile(reviewPath, JSON.stringify({ version: 1, items: [item('agent-pick', 'winget', 'Vendor.Sample', ' Same upstream: https://example.org/sample '), item('agent-none', 'scoop', 'none', 'macOS only: https://example.org/faq'), item('agent-bare', 'scoop', 'none')] }));
    await assert.rejects(applyReview({ ...options, reviewer: 'robot' }), /Invalid reviewer/);
    assert.equal((await applyReview({ ...options, reviewer: 'agent' })).added.length, 3);
    let map = JSON.parse(await readFile(mapPath)), provenance = JSON.parse(await readFile(provenancePath)).packages;
    assert.equal(map.packages['agent-pick'].winget, 'Vendor.Sample'); assert.equal(map.packages['agent-none'].scoop, null);
    assert.deepEqual(provenance['agent-pick'].winget, { method: 'agent-review', probability: null, tier: 'LOW', catalogSnapshot: '2026-11-03', evidence: 'Same upstream: https://example.org/sample' });
    assert.equal(provenance['agent-none'].scoop.method, 'agent-review'); assert.equal(provenance['agent-none'].scoop.evidence, 'macOS only: https://example.org/faq');
    assert.ok(!Object.hasOwn(provenance['agent-bare'].scoop, 'evidence'));
    await writeFile(reviewPath, JSON.stringify({ version: 1, items: [item('human-none', 'winget', 'none', 'Checked the vendor site'), item('human-bad', 'scoop', 'none', 42)] }));
    await assert.rejects(applyReview(options), /Invalid review evidence/);
    await writeFile(reviewPath, JSON.stringify({ version: 1, items: [item('human-none', 'winget', 'none', 'Checked the vendor site')] }));
    assert.equal((await applyReview(options)).added.length, 1);
    provenance = JSON.parse(await readFile(provenancePath)).packages; map = JSON.parse(await readFile(mapPath));
    assert.equal(map.packages['human-none'].winget, null);
    assert.deepEqual(provenance['human-none'].winget, { method: 'human', probability: null, tier: null, catalogSnapshot: '2026-11-03', evidence: 'Checked the vendor site' });
    assert.equal(provenance['agent-pick'].winget.method, 'agent-review');
  } finally { await rm(dir, { recursive: true, force: true }); }
});
