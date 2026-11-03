// CI-only repository queries. repot-core remains offline and dependency-free.
import { spawnSync } from 'node:child_process';
import { readFileSync, appendFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';

const managers = ['brew', 'brewCask', 'winget', 'scoop', 'choco', 'apt', 'dnf', 'pacman'];
const escaped = value => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
export function commandFor(manager, name) {
  if (!/^[A-Za-z0-9@._+/:-]+$/.test(name) || name.startsWith('-')) throw new Error(`Invalid name: ${name}`);
  switch (manager) {
    case 'brew': case 'brewCask': return ['brew', ['info', '--json=v2', manager === 'brew' ? '--formula' : '--cask', name]];
    case 'winget': return ['winget', ['show', '--id', name, '--exact', '--source', 'winget', '--accept-source-agreements', '--disable-interactivity']];
    // Scoop is a PowerShell command, not an executable. Names are validated above.
    case 'scoop': return ['pwsh', ['-NoProfile', '-Command', `$ErrorActionPreference='Stop'; scoop info '${name}'; exit $LASTEXITCODE`]];
    case 'choco': return ['choco', ['search', name, '--exact', '--limit-output', '--source=https://community.chocolatey.org/api/v2/']];
    case 'apt': return ['apt-cache', ['show', name]];
    case 'dnf': return ['dnf', ['info', '--available', name]];
    case 'pacman': return ['pacman', ['-Si', name]];
    default: throw new Error(`Unknown manager: ${manager}`);
  }
}

export function classify(manager, name, result) {
  if (result.error || result.status === null) return 'transient';
  const text = `${result.stdout ?? ''}\n${result.stderr ?? ''}`.replace(/\x1b\[[0-9;]*m/g, '');
  // Errors can coexist with a misleading empty-result message or partial metadata.
  if (/HTTP\s*[45]\d\d|timed?\s*out|timeout|TLS|SSL|certificate|network|could not resolve|connection|failed to download|failed to fetch|source unavailable|service unavailable|error.*source/i.test(text)) return 'transient';
  const missing = {
    brew: /No available formula with the name|No formulae? found/i,
    brewCask: /No Cask with this name exists|No available cask/i,
    winget: /No package found matching input criteria/i,
    scoop: /Could not find manifest for .* in local buckets/i,
    choco: /(?:^|\n)0 packages found\.?\s*(?:\n|$)/i,
    apt: /No packages found|Unable to locate package/i,
    dnf: /No matching Packages to list|No matching packages/i,
    pacman: /error: package .* was not found/i,
  };
  if (!managers.includes(manager)) throw new Error(`Unknown manager: ${manager}`);
  if (missing[manager].test(text)) return 'missing';
  if (result.status !== 0) return 'transient';
  const n = escaped(name);
  if (manager === 'brew' || manager === 'brewCask') {
    try {
      const data = JSON.parse(result.stdout);
      return (manager === 'brew' ? data.formulae?.some(f => f.name === name || f.full_name === name) : data.casks?.some(c => c.token === name)) ? 'found' : 'transient';
    } catch { return 'transient'; }
  }
  if (manager === 'scoop') {
    const [bucket, app] = name.includes('/') ? name.split('/') : ['main', name];
    return new RegExp(`^Name\\s*:\\s*${escaped(app)}\\s*$`, 'mi').test(text) && new RegExp(`^Source\\s*:\\s*${escaped(bucket)}\\s*$`, 'mi').test(text) ? 'found' : 'transient';
  }
  const positive = {
    winget: new RegExp(`^Found .+ \\[${n}\\]\\s*$`, 'mi'),
    choco: new RegExp(`^${n}\\|\\S+\\s*$`, 'mi'),
    apt: new RegExp(`^Package:\\s*${n}\\s*$`, 'mi'),
    dnf: new RegExp(`^Name\\s*:\\s*${n}\\s*$`, 'mi'),
    pacman: new RegExp(`^Name\\s*:\\s*${n}\\s*$`, 'mi'),
  };
  return positive[manager].test(text) ? 'found' : 'transient';
}

function execute(manager, name) {
  const [cmd, args] = commandFor(manager, name);
  return spawnSync(cmd, args, { encoding: 'utf8', timeout: 120_000, maxBuffer: 8 * 1024 * 1024,
    env: { ...process.env, LC_ALL: 'C', LANG: 'C', NO_COLOR: '1', HOMEBREW_NO_AUTO_UPDATE: '1' } });
}
export async function verify(manager, name, { run = execute, sleep = ms => new Promise(resolve => setTimeout(resolve, ms)), log = () => {} } = {}) {
  for (let attempt = 1; attempt <= 3; attempt++) {
    const result = await run(manager, name);
    const status = classify(manager, name, result);
    log({ manager, name, attempt, status, exit: result.status, stdout: result.stdout, stderr: result.stderr, error: result.error?.message });
    if (status !== 'transient' || attempt === 3) return status;
    await sleep(attempt * 5000);
  }
}
export function selectNames(data, selected) {
  return selected.flatMap(manager => [...new Set(Object.values(data.packages).map(p => p[manager]).filter(n => typeof n === 'string'))].map(name => [manager, name]));
}
async function main() {
  const selected = process.argv.slice(2);
  if (!selected.length || selected.some(m => !managers.includes(m))) throw new Error(`Usage: node scripts/check-package-map.mjs <${managers.join('|')}> ...`);
  const data = JSON.parse(readFileSync(new URL('../crates/core/data/package-map.json', import.meta.url), 'utf8'));
  const results = [];
  for (const [manager, name] of selectNames(data, selected)) {
    const status = await verify(manager, name, { log: record => console.log(JSON.stringify(record)) });
    results.push({ manager, name, status });
  }
  const counts = Object.fromEntries(selected.map(manager => [manager, results.filter(r => r.manager === manager && r.status === 'found').length]));
  const report = { counts, results };
  writeFileSync(`package-map-${selected.join('-')}.json`, `${JSON.stringify(report, null, 2)}\n`);
  console.log(JSON.stringify({ verified: counts, failed: results.filter(r => r.status !== 'found') }, null, 2));
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, `### Package map repository verification\n\n\`\`\`json\n${JSON.stringify({ verified: counts, failed: results.filter(r => r.status !== 'found') }, null, 2)}\n\`\`\`\n`);
  if (results.some(r => r.status !== 'found')) process.exitCode = 1;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => { console.error(error); process.exitCode = 1; });
}
