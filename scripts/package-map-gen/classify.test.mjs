import test from 'node:test';
import assert from 'node:assert/strict';
import { classify } from './classify.mjs';
const candidate = { id: 'Vendor.Tool', tier: 'MEDIUM', kindFlag: false };
test('classification: inclusive threshold, kind flag and none review, empty no-candidate, unique HIGH bypass, failures incomplete', async () => {
  for (const [candidates, outcome, threshold, expected, calls] of [
    [[], {}, 0.8, 'no-candidate', 0],
    [[{ ...candidate, tier: 'HIGH' }], {}, 0.8, 'auto', 0],
    [[candidate], { picked: candidate.id, probability: 0.8 }, 0.8, 'auto', 1],
    [[candidate], { picked: candidate.id, probability: 0.8 }, 0.81, 'review', 1],
    [[{ ...candidate, kindFlag: true }], { picked: candidate.id, probability: 1 }, 0.8, 'review', 1],
    [[candidate], { picked: 'none', probability: 1 }, 0.8, 'review', 1],
    [[candidate], null, 0.8, 'incomplete', 1],
  ]) {
    let count = 0;
    const result = await classify({}, { candidates }, { threshold, judge: async () => { count++; if (!outcome) throw new Error('failure'); return outcome; } });
    assert.equal(result.status, expected); assert.equal(count, calls);
    if (calls === 0 && expected === 'auto') { assert.equal(result.method, 'high-tier'); assert.equal(result.probability, null); }
  }
});
