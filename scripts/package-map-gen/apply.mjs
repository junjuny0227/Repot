import { open, readFile, rename, rm, writeFile } from 'node:fs/promises';
import { canonical } from './normalize.mjs';
export const defaultMapPath = new URL('../../crates/core/data/package-map.json', import.meta.url);
export const defaultProvenancePath = new URL(
  '../../crates/core/data/package-map.provenance.json',
  import.meta.url,
);
const managers = ['brew', 'brewCask', 'winget', 'scoop', 'choco', 'apt', 'dnf', 'pacman'];
const sorted = (object) =>
  Object.fromEntries(Object.entries(object).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)));
// Common id from a brew token: keep version marks, map '+' to 'plus', every other run outside [a-z0-9] to '-'.
export const commonId = (token) =>
  token
    .toLowerCase()
    .replaceAll('+', '-plus-')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
const validName = (name) =>
  typeof name === 'string' && /^[A-Za-z0-9@._+/:-]+$/.test(name) && !name.startsWith('-');
function validate(map) {
  if (
    map.version !== 1 ||
    !map.packages ||
    typeof map.packages !== 'object' ||
    Array.isArray(map.packages)
  )
    throw new Error('Invalid package map');
  const seen = new Set();
  for (const [id, values] of Object.entries(map.packages)) {
    if (!/^[a-z0-9-]+$/.test(id) || !values || typeof values !== 'object' || Array.isArray(values))
      throw new Error('Invalid package map entry');
    for (const [manager, name] of Object.entries(values)) {
      if (!managers.includes(manager) || (name !== null && !validName(name)))
        throw new Error('Invalid package name or manager');
      if (name === null) continue;
      const key = `${manager}:${canonical(manager, name)}`;
      if (seen.has(key)) throw new Error('Duplicate package manager name');
      seen.add(key);
    }
  }
}
export function mergeDecisions(original, previous = { version: 1, packages: {} }, rows = []) {
  validate(original);
  if (
    previous.version !== 1 ||
    !previous.packages ||
    typeof previous.packages !== 'object' ||
    Array.isArray(previous.packages)
  )
    throw new Error('Invalid provenance');
  const map = structuredClone(original),
    provenance = structuredClone(previous),
    added = [],
    skipped = [];
  for (const row of rows) {
    if (!['auto', 'human'].includes(row.status)) continue;
    // Reviewed decisions: a person (`human`) or an agent following the review rules (`agent-review`).
    const human = row.status === 'human' && ['human', 'agent-review'].includes(row.method);
    if (!['formula', 'cask'].includes(row.sourceKind) || !['winget', 'scoop'].includes(row.target))
      throw new Error('Invalid decision source or target');
    if (row.picked === 'none' && !human) continue;
    if (row.picked !== 'none' && !validName(row.picked))
      throw new Error('Invalid target package name');
    if (!validName(row.source)) throw new Error('Invalid source package name');
    const manager = row.sourceKind === 'formula' ? 'brew' : 'brewCask';
    let id = Object.keys(map.packages).find(
      (id) => map.packages[id][manager]?.toLowerCase() === row.source.toLowerCase(),
    );
    const proposedId = commonId(row.source);
    let reason;
    if (!id && !/^[a-z0-9-]+$/.test(proposedId)) reason = 'invalid common id';
    if (!id && Object.hasOwn(map.packages, proposedId)) reason = 'common id collision';
    id ??= proposedId;
    if (Object.hasOwn(map.packages[id] ?? {}, row.target)) reason = 'existing target preserved';
    const value = row.picked === 'none' ? null : row.picked;
    if (
      value !== null &&
      Object.values(map.packages).some(
        (p) =>
          typeof p[row.target] === 'string' &&
          canonical(row.target, p[row.target]) === canonical(row.target, value),
      )
    )
      reason = 'duplicate manager name';
    if (reason) {
      skipped.push({ source: row.source, target: row.target, reason });
      continue;
    }
    if (!human && !['high-tier', 'jeb'].includes(row.method))
      throw new Error('Invalid provenance method');
    const probability = human || row.method === 'high-tier' ? null : row.probability;
    if (
      probability !== null &&
      (!Number.isFinite(probability) || probability < 0 || probability > 1)
    )
      throw new Error('Invalid decision probability');
    if (
      typeof row.catalogSnapshot !== 'string' ||
      !Number.isFinite(Date.parse(row.catalogSnapshot))
    )
      throw new Error('Invalid catalog snapshot date');
    const tier = row.candidates?.find((c) => c.id === value)?.tier ?? null;
    if (tier !== null && !['HIGH', 'MEDIUM', 'LOW'].includes(tier))
      throw new Error('Invalid candidate tier');
    if (!Object.hasOwn(map.packages, id)) map.packages[id] = { [manager]: row.source };
    map.packages[id][row.target] = value;
    if (!Object.hasOwn(provenance.packages, id)) provenance.packages[id] = {};
    provenance.packages[id][row.target] = {
      method: row.method,
      probability,
      tier,
      catalogSnapshot: row.catalogSnapshot,
      ...(human && typeof row.evidence === 'string' && row.evidence.trim()
        ? { evidence: row.evidence.trim() }
        : {}),
    };
    added.push({ id, target: row.target, value });
  }
  map.packages = sorted(map.packages);
  provenance.packages = sorted(provenance.packages);
  for (const id of Object.keys(provenance.packages))
    provenance.packages[id] = sorted(provenance.packages[id]);
  validate(map);
  return { map, provenance, added, skipped };
}
export async function applyDecisions(
  rows,
  { mapPath = defaultMapPath, provenancePath = defaultProvenancePath, dryRun = false } = {},
) {
  const mapName = mapPath instanceof URL ? mapPath.pathname : mapPath;
  const provenanceName = provenancePath instanceof URL ? provenancePath.pathname : provenancePath;
  // Serialize cooperating writers and always re-read the latest files inside the lock.
  const lock = await open(mapName + '.lock', 'wx');
  try {
    const original = await readFile(mapPath, 'utf8');
    let previous;
    try {
      previous = await readFile(provenancePath, 'utf8');
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
    }
    const result = mergeDecisions(
      JSON.parse(original),
      previous === undefined ? undefined : JSON.parse(previous),
      rows,
    );
    if (!dryRun && result.added.length) {
      const mapTmp = mapName + '.tmp',
        provenanceTmp = provenanceName + '.tmp';
      try {
        await writeFile(mapTmp, JSON.stringify(result.map, null, 2) + '\n');
        await writeFile(provenanceTmp, JSON.stringify(result.provenance, null, 2) + '\n');
        await rename(provenanceTmp, provenanceName);
        try {
          await rename(mapTmp, mapName);
        } catch (error) {
          if (previous === undefined) await rm(provenanceName);
          else await writeFile(provenanceName, previous);
          throw error;
        }
      } finally {
        await rm(mapTmp, { force: true });
        await rm(provenanceTmp, { force: true });
      }
    }
    return result;
  } finally {
    await lock.close();
    await rm(mapName + '.lock');
  }
}
