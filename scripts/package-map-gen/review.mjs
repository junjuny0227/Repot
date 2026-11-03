import { readFile, writeFile, mkdir, rename } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { createHash } from 'node:crypto';
import { applyDecisions } from './apply.mjs';
const identity = row => `${row.sourceKind}:${row.source}:${row.target}`;
const evidence = row => ({ source: row.source, sourceKind: row.sourceKind, target: row.target, candidates: row.candidates, picked: row.picked, probability: row.probability, catalogSnapshot: row.catalogSnapshot });
const fingerprint = row => createHash('sha256').update(JSON.stringify(evidence(row))).digest('hex');
export async function writeReview(rows, { cacheDir = '.cache/package-map-gen', reviewPath = join(cacheDir, 'review.json') } = {}) {
  let previous = { version: 1, items: [] };
  try { previous = JSON.parse(await readFile(reviewPath, 'utf8')); } catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (previous.version !== 1 || !Array.isArray(previous.items)) throw new Error('Invalid review file');
  const items = new Map(previous.items.map(item => [identity(item), item]));
  for (const row of rows.filter(row => row.status === 'review')) {
    const old = items.get(identity(row));
    items.set(identity(row), { ...evidence(row), decision: old && fingerprint(old) === fingerprint(row) ? old.decision : '' });
  }
  const review = { version: 1, items: [...items.values()].sort((a, b) => identity(a) < identity(b) ? -1 : identity(a) > identity(b) ? 1 : 0) };
  await mkdir(dirname(reviewPath), { recursive: true });
  await writeFile(reviewPath + '.tmp', JSON.stringify(review, null, 2) + '\n'); await rename(reviewPath + '.tmp', reviewPath);
  return review;
}
export async function applyReview({ cacheDir = '.cache/package-map-gen', reviewPath = join(cacheDir, 'review.json'), reviewer = 'human', ...options } = {}) {
  if (!['human', 'agent'].includes(reviewer)) throw new Error('Invalid reviewer: use human or agent');
  const method = reviewer === 'agent' ? 'agent-review' : 'human';
  const review = JSON.parse(await readFile(reviewPath, 'utf8'));
  if (review.version !== 1 || !Array.isArray(review.items)) throw new Error('Invalid review file');
  const rows = [], seen = new Set();
  for (const item of review.items) {
    if (seen.has(identity(item))) throw new Error('Duplicate review item');
    seen.add(identity(item));
    if (item.decision === '' || item.decision === 'skip') continue;
    if (typeof item.decision !== 'string' || !Array.isArray(item.candidates) || (item.decision !== 'none' && !item.candidates.some(c => c.id === item.decision))) throw new Error('Invalid review decision: choose a listed candidate, none, or skip');
    if (item.evidence !== undefined && typeof item.evidence !== 'string') throw new Error('Invalid review evidence: use a string');
    rows.push({ ...item, status: 'human', method, picked: item.decision, probability: null });
  }
  return applyDecisions(rows, options);
}
