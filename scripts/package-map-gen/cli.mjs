import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import { createHash } from 'node:crypto';
import { loadCatalogs } from './catalogs.mjs';
import { index, propose } from './candidates.mjs';
import { preFilter } from './filter.mjs';
import { createJudge } from './judge.mjs';
import { classify } from './classify.mjs';
import { applyDecisions, defaultMapPath } from './apply.mjs';
import { writeReview, applyReview } from './review.mjs';
const cacheDefault = '.cache/package-map-gen';
const usage = 'Usage: pnpm package-map:gen candidates|run [--offline] [--names token,brew:token,brewCask:token] [--from-analytics N] [--from-scan scan.json] [--cache-dir path]; run [--threshold 0.95] [--max-requests 1000] [--credit-floor 100] [--dry-run]; apply-review [--review-file path] [--dry-run]';
export function parseOptions(args) {
  const command = args[0];
  if (!['candidates', 'run', 'apply-review'].includes(command)) throw new Error(usage);
  const options = { offline: { type: 'boolean', default: false }, names: { type: 'string' }, 'from-analytics': { type: 'string' }, 'from-scan': { type: 'string' }, 'cache-dir': { type: 'string' } };
  if (command !== 'candidates') Object.assign(options, { 'dry-run': { type: 'boolean', default: false }, 'review-file': { type: 'string' } });
  if (command === 'run') Object.assign(options, { threshold: { type: 'string' }, 'max-requests': { type: 'string' }, 'credit-floor': { type: 'string' } });
  const { values } = parseArgs({ args: args.slice(1), options });
  const top = values['from-analytics'] === undefined ? 0 : Number(values['from-analytics']);
  if (values['from-analytics'] !== undefined && (!Number.isSafeInteger(top) || top <= 0)) throw new Error('--from-analytics requires a positive integer');
  const names = values.names?.split(',').map(s => s.trim()).filter(Boolean) ?? [];
  if (command !== 'apply-review' && !names.length && !top && !values['from-scan']) throw new Error(usage);
  const threshold = Number(values.threshold ?? 0.95), maxRequests = Number(values['max-requests'] ?? 1000), creditFloor = Number(values['credit-floor'] ?? 100);
  if (!Number.isFinite(threshold) || threshold < 0 || threshold > 1) throw new Error('--threshold must be between 0 and 1');
  if (!Number.isSafeInteger(maxRequests) || maxRequests < 0) throw new Error('--max-requests requires a nonnegative integer');
  if (!Number.isFinite(creditFloor) || creditFloor < 0) throw new Error('--credit-floor requires a nonnegative number');
  return { command, offline: values.offline, names, top, scanPath: values['from-scan'], cacheDir: values['cache-dir'] ?? cacheDefault, threshold, maxRequests, creditFloor, dryRun: values['dry-run'] ?? false, reviewPath: values['review-file'] };
}
export async function loadAnalytics({ cacheDir = cacheDefault, offline = false, fetch = globalThis.fetch } = {}) {
  const results = [];
  for (const kind of ['formula', 'cask']) {
    const path = join(cacheDir, `analytics-${kind}.json`); let snapshot;
    try { snapshot = JSON.parse(await readFile(path, 'utf8')); }
    catch (error) {
      if (error.code !== 'ENOENT') throw error;
      if (offline) throw new Error(`Offline analytics cache missing: ${kind}`);
      const url = `https://formulae.brew.sh/api/analytics/${kind === 'formula' ? 'install-on-request' : 'cask-install'}/30d.json`;
      const response = await fetch(url);
      if (!response.ok) throw new Error(`Analytics download failed (${response.status})`);
      const text = await response.text();
      snapshot = { url, fetchedAt: new Date().toISOString(), sha256: createHash('sha256').update(text).digest('hex'), data: JSON.parse(text) };
      await mkdir(cacheDir, { recursive: true }); await writeFile(path, JSON.stringify(snapshot));
    }
    results.push({ kind, data: snapshot.data });
  }
  return results;
}
export function selectSources({ brew, names = [], scan, analytics = [], top = 0, packages = {} }) {
  const requested = [], issues = [], skipped = [], sources = [];
  for (const name of names) {
    const match = /^(brew|brewCask):(.+)$/.exec(name);
    requested.push({ id: match ? match[2] : name, kind: match ? (match[1] === 'brew' ? 'formula' : 'cask') : undefined });
  }
  if (scan) {
    // Accept a core ScanResult (`items`, as written by the scan example) and the desktop UI state (`scan.items`).
    const items = Array.isArray(scan.items) ? scan.items : scan.scan?.items;
    if (!Array.isArray(items)) throw new Error('Invalid Repot scan: expected items or scan.items');
    for (const item of items) {
      const pkg = item.details?.package;
      if (pkg?.manager === 'brew' && (pkg.kind === 'formula' || pkg.kind === 'cask')) requested.push({ id: item.name, kind: pkg.kind });
    }
  }
  for (const { kind, data } of analytics) {
    const items = Array.isArray(data.items) ? data.items : Object.values(data.items ?? {}).flat();
    const counts = new Map();
    for (const item of items) {
      let name = item[kind === 'formula' ? 'formula' : 'cask']?.replace(/\s+--.*$/, '');
      if (!name || (name.includes('/') && !name.startsWith(kind === 'formula' ? 'homebrew/core/' : 'homebrew/cask/'))) continue;
      name = name.split('/').at(-1);
      const count = Number(String(item.count ?? 0).replaceAll(',', ''));
      if (!Number.isFinite(count)) continue;
      counts.set(name, (counts.get(name) ?? 0) + count);
    }
    for (const [id] of [...counts].sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1)).slice(0, top)) requested.push({ id, kind });
  }
  const seen = new Set();
  for (const request of requested) {
    const matches = brew.filter(b => b.id === request.id && (!request.kind || b.brew_kind === request.kind));
    if (matches.length !== 1) { issues.push({ id: request.id, type: matches.length ? 'ambiguous' : 'unknown', message: matches.length ? 'Use brew:token or brewCask:token to disambiguate' : 'Token not found in Homebrew catalog' }); continue; }
    const source = matches[0], key = `${source.brew_kind}:${source.id}`;
    if (seen.has(key)) continue; seen.add(key);
    const manager = source.brew_kind === 'formula' ? 'brew' : 'brewCask';
    if (Object.values(packages).some(p => p[manager] === source.id && Object.hasOwn(p, 'winget') && Object.hasOwn(p, 'scoop'))) skipped.push({ id: source.id, kind: source.brew_kind, reason: 'winget and scoop already mapped' });
    else sources.push(source);
  }
  sources.sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : a.brew_kind < b.brew_kind ? -1 : 1);
  return { sources, skipped, issues };
}
export async function runCandidates(options) {
  const catalogs = options.catalogs ?? await loadCatalogs(options);
  const packages = options.packages ?? JSON.parse(await readFile(new URL('../../crates/core/data/package-map.json', import.meta.url), 'utf8')).packages;
  const scan = options.scan ?? (options.scanPath ? JSON.parse(await readFile(options.scanPath, 'utf8')) : undefined);
  const analytics = options.analytics ?? (options.top ? await loadAnalytics(options) : []);
  const { sources, skipped, issues } = selectSources({ ...options, brew: catalogs.brew, packages, scan, analytics });
  const indices = Object.fromEntries(['winget', 'scoop'].map(target => [target, index(catalogs[target])]));
  const rows = sources.flatMap(source => ['winget', 'scoop'].map(target => ({ source: source.id, sourceKind: source.brew_kind, target, ...preFilter(source, propose(source, catalogs[target], indices[target])) })));
  return { rows, skipped, issues };
}
export async function run(options) {
  const catalogs = options.catalogs ?? await loadCatalogs(options);
  const packages = JSON.parse(await readFile(options.mapPath ?? defaultMapPath, 'utf8')).packages;
  const proposals = await runCandidates({ ...options, catalogs, packages });
  const catalogSnapshot = options.catalogSnapshot ?? JSON.parse(await readFile(join(options.cacheDir ?? cacheDefault, 'sources.json'), 'utf8')).snapshotDate;
  const judge = options.judge ?? createJudge(options), rows = [], skipped = [...proposals.skipped];
  for (const proposal of proposals.rows) {
    const manager = proposal.sourceKind === 'formula' ? 'brew' : 'brewCask';
    if (Object.values(packages).some(p => p[manager]?.toLowerCase() === proposal.source.toLowerCase() && Object.hasOwn(p, proposal.target))) {
      skipped.push({ source: proposal.source, target: proposal.target, reason: 'existing target preserved' }); continue;
    }
    const source = catalogs.brew.find(s => s.id === proposal.source && s.brew_kind === proposal.sourceKind);
    rows.push({ ...await classify(source, proposal, { judge, threshold: options.threshold }), catalogSnapshot });
  }
  // Review evidence is checkpointed even in dry-run; only the two tracked data files are suppressed.
  await writeReview(rows, options);
  const applied = await applyDecisions(rows, options);
  const counts = Object.fromEntries(['auto', 'review', 'no-candidate', 'incomplete'].map(status => [status, rows.filter(row => row.status === status).length]));
  Object.assign(counts, { added: applied.added.length, conflicts: applied.skipped.length });
  return { dryRun: options.dryRun ?? false, catalogSnapshot, counts, added: applied.added, conflicts: applied.skipped, skipped, issues: proposals.issues, rows, ...(judge.stats ? { jeb: judge.stats } : {}) };
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  Promise.resolve().then(async () => {
    const options = parseOptions(process.argv.slice(2));
    if (options.command === 'candidates') return runCandidates(options);
    if (options.command === 'run') return run(options);
    const result = await applyReview(options);
    return { dryRun: options.dryRun, added: result.added, skipped: result.skipped };
  }).then(report => {
    console.log(JSON.stringify(report, null, 2));
    if (report.issues?.length || report.counts?.incomplete) process.exitCode = 1;
  }).catch(error => { console.error(error.message); process.exitCode = 1; });
}
