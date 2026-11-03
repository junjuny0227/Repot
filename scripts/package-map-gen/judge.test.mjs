import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createJudge, buildRequest, readKey } from './judge.mjs';
const source = { id: 'sample', brew_kind: 'formula', kind: 'cli', names: ['sample'], urls: ['https://example.org'], repos: [] };
const row = { source: 'sample', sourceKind: 'formula', target: 'winget', candidates: [{ id: 'Vendor.Sample', name: 'Sample', description: 'Example', urls: ['https://example.org'], kind: 'cli', tier: 'LOW' }] };
const answer = (choice = '1', probabilities = { 1: 0.8, 2: 0.2 }, credits = '900') => Response.json({ answers: { q: { choice, probabilities, confidence: 0 } } }, { headers: credits === null ? {} : { 'x-ratelimit-remaining-credits': credits } });
async function temp(fn) { const path = await mkdtemp(join(tmpdir(), 'judge-')); try { await fn(path); } finally { await rm(path, { recursive: true, force: true }); } }
test('uniform request includes final none and decodes selected probability, caches across clients offline without key', () => temp(async cacheDir => {
  const payload = buildRequest(source, row);
  assert.equal(payload.model, 'jeb'); assert.equal(payload.questions.q.criteria['2'], 'none of these is the same software');
  let calls = 0;
  const judge = createJudge({ cacheDir, key: 'synthetic-secret', fetch: async (_url, options) => { calls++; assert.equal(options.headers.Authorization, 'Bearer synthetic-secret'); return answer('1', [0.8, 0.2]); } });
  assert.deepEqual(await judge(source, row), { picked: 'Vendor.Sample', probability: 0.8, method: 'jeb', cached: false });
  assert.equal((await judge(source, row)).cached, true);
  assert.equal((await createJudge({ cacheDir, offline: true })(source, row)).cached, true);
  assert.equal(calls, 1);
  assert.ok(!(await readFile(join(cacheDir, 'checkpoint.json'), 'utf8')).includes('synthetic-secret'));
}));
test('503 backoff counts every attempt and enforces per-run budget across concurrent callers', () => temp(async cacheDir => {
  let calls = 0, active = 0, peak = 0; const delays = [];
  const judge = createJudge({ cacheDir, key: 'fake', maxRequests: 2, sleep: async ms => delays.push(ms), fetch: async () => { calls++; active++; peak = Math.max(peak, active); await new Promise(r => setTimeout(r, 2)); active--; return new Response('', { status: 503, headers: { 'x-ratelimit-remaining-credits': '900' } }); } });
  const results = await Promise.allSettled([judge(source, row), judge(source, { ...row, target: 'scoop' })]);
  assert.ok(results.every(r => r.status === 'rejected' && /limit/.test(r.reason.message)));
  assert.equal(calls, 2); assert.ok(peak <= 2); assert.deepEqual(delays, [1000, 2000]);
  assert.equal(JSON.parse(await readFile(join(cacheDir, 'checkpoint.json'))).attempts, 2);
}));
test('quota floor stops next request and same-day resumed clients; day reset permits request', () => temp(async cacheDir => {
  let calls = 0; const fetch = async () => { calls++; return answer('2', { 2: 0.9 }, '99'); };
  const options = { cacheDir, key: 'fake', fetch, now: () => new Date('2026-11-03T12:00:00Z') };
  assert.equal((await createJudge(options)(source, row)).picked, 'none');
  await assert.rejects(createJudge(options)(source, { ...row, target: 'scoop' }), /quota/);
  assert.equal(calls, 1);
  await createJudge({ ...options, now: () => new Date('2026-11-04T12:00:00Z') })(source, { ...row, target: 'scoop' });
  assert.equal(calls, 2);
}));
test('missing quota, malformed responses, HTTP body and transport exceptions never leak authorization or secret', async () => {
  for (const mode of ['missing', 'bad-header', 'bad-body', 'transport', 'http']) await temp(async cacheDir => {
    const secret = 'synthetic-private-key';
    const judge = createJudge({ cacheDir, key: secret, fetch: async (_url, options) => {
      if (mode === 'transport') throw Error(JSON.stringify(options.headers));
      if (mode === 'missing') return answer('1', { 1: 0.9 }, null);
      if (mode === 'bad-header') return answer('1', { 1: 0.9 }, 'invalid');
      if (mode === 'bad-body') return Response.json({ error: options.headers }, { headers: { 'x-ratelimit-remaining-credits': '900' } });
      return new Response(JSON.stringify(options.headers), { status: 401, headers: { 'x-ratelimit-remaining-credits': '900' } });
    } });
    await assert.rejects(judge(source, row), error => !String(error.stack).includes(secret) && !String(error.stack).includes('Authorization'));
    assert.ok(!(await readFile(join(cacheDir, 'checkpoint.json'), 'utf8')).includes(secret));
  });
});
test('env key takes precedence; dotenv quotes/comments are parsed as data, never executed', () => temp(async cacheDir => {
  const envPath = join(cacheDir, '.env');
  await writeFile(envPath, 'export JEB_AI_HUB_KEY = "$(do-not-execute)" # comment\n');
  assert.equal(await readKey({ env: { JEB_AI_HUB_KEY: 'preferred' }, envPath }), 'preferred');
  assert.equal(await readKey({ env: {}, envPath }), '$(do-not-execute)');
  await writeFile(envPath, "JEB_AI_HUB_KEY='quoted'\n");
  assert.equal(await readKey({ env: {}, envPath }), 'quoted');
  await assert.rejects(readKey({ env: {}, envPath: join(cacheDir, 'missing') }), /required/);
}));
