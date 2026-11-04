import { sameChannel } from './filter.mjs';
import { nameKey } from './normalize.mjs';

const matchKeys = ['repos', 'web', 'names'];
// Exact id rules from documented catalog naming conventions; each id must exist verbatim in the target catalog.
const jdkIds = {
  temurin: (n) => [`java/temurin${n}-jdk`, `EclipseAdoptium.Temurin.${n}.JDK`],
  zulu: (n) => [`java/zulu${n}-jdk`, `Azul.Zulu.${n}.JDK`],
};
export function ruleHits(source, idx) {
  const hits = new Map();
  if (source.brew_kind !== 'cask') return hits;
  const jdk = /^(temurin|zulu)@(\d+)$/.exec(source.id);
  if (jdk)
    for (const id of jdkIds[jdk[1]](jdk[2]))
      if (idx.ids.has(id)) hits.set(idx.ids.get(id), 'versioned JDK id rule');
  const font = /^font-(.+)-nerd-font$/.exec(source.id);
  if (font)
    for (const i of idx.nerdFonts.get(nameKey(font[1])) ?? [])
      hits.set(i, 'Nerd Font id rule (equal normalized font name)');
  return hits;
}
export function index(records) {
  const result = Object.fromEntries(matchKeys.map((key) => [key, new Map()]));
  result.ids = new Map();
  result.nerdFonts = new Map();
  for (const [i, record] of records.entries()) {
    for (const key of matchKeys)
      for (const value of record[key]) {
        if (!result[key].has(value)) result[key].set(value, new Set());
        result[key].get(value).add(i);
      }
    result.ids.set(record.id, i);
    const font = /^nerd-fonts\/(.+)-NF$/.exec(record.id);
    if (font) {
      const key = nameKey(font[1]);
      if (!result.nerdFonts.has(key)) result.nerdFonts.set(key, []);
      result.nerdFonts.get(key).push(i);
    }
  }
  return result;
}
export function propose(source, records, idx = index(records), limit = 12) {
  const hits = Object.fromEntries(
    matchKeys.map((key) => [
      key,
      new Set(source[key].flatMap((value) => [...(idx[key].get(value) ?? [])])),
    ]),
  );
  const rules = ruleHits(source, idx);
  const proposals = [];
  for (const i of new Set([...rules.keys(), ...Object.values(hits).flatMap((set) => [...set])])) {
    const candidate = records[i];
    let tier, reason;
    if (hits.repos.has(i)) {
      tier =
        hits.repos.size === 1 &&
        ['cli', 'gui', 'mixed'].includes(source.kind) &&
        [source.kind, 'mixed'].includes(candidate.kind)
          ? 'HIGH'
          : 'MEDIUM';
      reason = `same source repo; ${tier === 'HIGH' ? 'unique + compatible kind' : 'ambiguous or kind not positively compatible'}`;
      if (tier === 'HIGH' && (!sameChannel(source.id, candidate.id) || source.id.includes('@'))) {
        tier = 'MEDIUM';
        reason = 'same source repo; channel mismatch or version-pinned source requires review';
      }
    } else if (hits.web.has(i)) {
      tier = 'MEDIUM';
      reason = 'same normalized homepage/download URL';
    } else {
      tier = 'LOW';
      reason = 'exact normalized name only';
    }
    // A rule hit is MEDIUM evidence (still judged); it never raises a candidate to HIGH.
    if (rules.has(i) && tier !== 'HIGH') {
      tier = 'MEDIUM';
      reason = rules.get(i);
    }
    proposals.push({
      ...candidate,
      tier,
      reason,
      ...(rules.has(i) ? { rule: true } : {}),
      repos: source.repos.filter((repo) => candidate.repos.includes(repo)),
    });
  }
  const rank = { HIGH: 0, MEDIUM: 1, LOW: 2 };
  // Within a tier, rule hits rank first so the cap cannot cut them behind shared-repo candidates.
  return proposals
    .sort(
      (a, b) =>
        rank[a.tier] - rank[b.tier] ||
        Number(Boolean(b.rule)) - Number(Boolean(a.rule)) ||
        (a.id.toLowerCase() < b.id.toLowerCase()
          ? -1
          : a.id.toLowerCase() > b.id.toLowerCase()
            ? 1
            : a.id < b.id
              ? -1
              : a.id > b.id
                ? 1
                : 0),
    )
    .slice(0, limit);
}
