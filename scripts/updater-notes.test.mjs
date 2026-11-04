import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { updaterNotes } from './updater-notes.mjs';

// Shape of a published body: our highlights, tauri-action's releaseBody, GitHub's generated notes.
const published = `## Highlights

**Meslo Nerd Font now carries over to Windows.** \`font-meslo-lg-nerd-font\` maps to Scoop \`nerd-fonts/Meslo-NF\`.

- Same [upstream release](https://github.com/ryanoasis/nerd-fonts).

> **Unsigned beta.** Not code-signed or notarized yet; see the README for first launch on macOS and Windows.

### Which file do I download?

| Your computer | Download |
|---|---|
| Mac with Intel | \`Repot_0.4.2_x64.dmg\` |

<!-- Release notes generated using configuration in .github/release.yml at v0.4.2 -->

## What's Changed
### Features
* feat(data): map font-meslo-lg-nerd-font to Scoop by @junjuny0227 in https://github.com/junjuny0227/Repot/pull/68
`;

test('keeps only the Highlights section as plain text', () => {
  assert.equal(
    updaterNotes(published),
    'Meslo Nerd Font now carries over to Windows. font-meslo-lg-nerd-font maps to Scoop nerd-fonts/Meslo-NF.\n\n• Same upstream release.',
  );
});

test('returns an empty note without a Highlights section', () => {
  assert.equal(updaterNotes("> **Unsigned beta.** Not code-signed.\n## What's Changed\n* x"), '');
  assert.equal(updaterNotes(undefined), '');
});

test('CLI rewrites notes in latest.json and keeps the other fields', () => {
  const dir = mkdtempSync(join(tmpdir(), 'repot-notes-'));
  try {
    const body = join(dir, 'body.md');
    const latest = join(dir, 'latest.json');
    writeFileSync(body, published);
    writeFileSync(
      latest,
      JSON.stringify({
        version: '0.4.2',
        notes: '> **Unsigned beta.**',
        platforms: { 'darwin-aarch64': { url: 'u', signature: 's' } },
      }),
    );
    const script = fileURLToPath(new URL('./updater-notes.mjs', import.meta.url));
    const run = spawnSync(process.execPath, [script, body, latest], { encoding: 'utf8' });
    assert.equal(run.status, 0, run.stderr);
    const out = JSON.parse(readFileSync(latest, 'utf8'));
    assert.match(out.notes, /^Meslo Nerd Font now carries over/);
    assert.equal(out.version, '0.4.2');
    assert.deepEqual(out.platforms, { 'darwin-aarch64': { url: 'u', signature: 's' } });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
