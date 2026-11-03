import { sameChannel } from './filter.mjs';
export function index(records) {
  const result = Object.fromEntries(['repos', 'web', 'names'].map(key => [key, new Map()]));
  for (const [i, record] of records.entries()) for (const key of Object.keys(result)) for (const value of record[key]) {
    if (!result[key].has(value)) result[key].set(value, new Set());
    result[key].get(value).add(i);
  }
  return result;
}
export function propose(source, records, idx = index(records), limit = 12) {
  const hits = Object.fromEntries(Object.keys(idx).map(key => [key, new Set(source[key].flatMap(value => [...(idx[key].get(value) ?? [])]))]));
  const proposals = [];
  for (const i of new Set(Object.values(hits).flatMap(set => [...set]))) {
    const candidate = records[i];
    let tier, reason;
    if (hits.repos.has(i)) {
      tier = hits.repos.size === 1 && ['cli', 'gui', 'mixed'].includes(source.kind) && [source.kind, 'mixed'].includes(candidate.kind) ? 'HIGH' : 'MEDIUM';
      reason = `same source repo; ${tier === 'HIGH' ? 'unique + compatible kind' : 'ambiguous or kind not positively compatible'}`;
      if (tier === 'HIGH' && (!sameChannel(source.id, candidate.id) || source.id.includes('@'))) {
        tier = 'MEDIUM'; reason = 'same source repo; channel mismatch or version-pinned source requires review';
      }
    } else if (hits.web.has(i)) { tier = 'MEDIUM'; reason = 'same normalized homepage/download URL'; }
    else { tier = 'LOW'; reason = 'exact normalized name only'; }
    proposals.push({ ...candidate, tier, reason, repos: source.repos.filter(repo => candidate.repos.includes(repo)) });
  }
  const rank = { HIGH: 0, MEDIUM: 1, LOW: 2 };
  return proposals.sort((a, b) => rank[a.tier] - rank[b.tier] || (a.id.toLowerCase() < b.id.toLowerCase() ? -1 : a.id.toLowerCase() > b.id.toLowerCase() ? 1 : a.id < b.id ? -1 : a.id > b.id ? 1 : 0)).slice(0, limit);
}
