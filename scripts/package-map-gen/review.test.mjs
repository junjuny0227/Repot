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
