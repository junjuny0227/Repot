import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { archiveName, parseSha256, render, TARGETS, versionOf } from './cli-packages.mjs';

const script = fileURLToPath(new URL('./cli-packages.mjs', import.meta.url));
// A different hash per target, so a mixed-up target shows up.
const hashOf = (target) => String(Object.values(TARGETS).indexOf(target) + 1).repeat(64);

function shaDir(version) {
  const dir = mkdtempSync(join(tmpdir(), 'repot-cli-packages-'));
  for (const target of Object.values(TARGETS)) {
    const file = archiveName(version, target);
    writeFileSync(join(dir, `${file}.sha256`), `${hashOf(target)}  ${file}\n`);
  }
  return dir;
}

test('archive names match what the release workflow uploads', () => {
  assert.equal(
    archiveName('0.8.0', 'aarch64-apple-darwin'),
    'repot-cli-0.8.0-aarch64-apple-darwin.tar.gz',
  );
  assert.equal(
    archiveName('0.8.0', 'x86_64-pc-windows-msvc'),
    'repot-cli-0.8.0-x86_64-pc-windows-msvc.zip',
  );
});

test('only plain release tags are accepted', () => {
  assert.equal(versionOf('v0.8.0'), '0.8.0');
  for (const bad of ['0.8.0', 'v0.8', 'v0.8.0-rc.1', 'v0.8.0"; rm', undefined]) {
    assert.throws(() => versionOf(bad), /not a release tag/);
  }
});

test('sha256 lines from sha256sum and shasum are read, others rejected', () => {
  const file = 'repot-cli-0.8.0-x86_64-pc-windows-msvc.zip';
  const hash = 'A'.repeat(64);
  assert.equal(parseSha256(`${hash}  ${file}\n`, file), 'a'.repeat(64));
  assert.equal(parseSha256(`${hash} *${file}`, file), 'a'.repeat(64));
  assert.throws(() => parseSha256('', file), /no sha256 line/);
  assert.throws(() => parseSha256(`${hash}  other.zip`, file), /names other.zip/);
});

test('the formula pins each archive with its own hash', () => {
  const dir = shaDir('0.8.0');
  try {
    const rb = render('v0.8.0', dir)['Formula/repot.rb'];
    assert.match(rb, /^ {2}version "0\.8\.0"$/m);
    for (const target of [TARGETS.macArm, TARGETS.macIntel, TARGETS.linuxIntel]) {
      const url = `https://github.com/junjuny0227/Repot/releases/download/v0.8.0/repot-cli-0.8.0-${target}.tar.gz`;
      assert.ok(rb.includes(`url "${url}"\n      sha256 "${hashOf(target)}"`), target);
    }
    assert.ok(!rb.includes('windows'));
    assert.match(rb, /bin\.install "repot"/);
    assert.match(rb, /repot --version/);
  } finally {
    rmSync(dir, { recursive: true });
  }
});

test('the Scoop manifest pins the Windows zip and can update itself', () => {
  const dir = shaDir('0.8.0');
  try {
    const m = JSON.parse(render('v0.8.0', dir)['bucket/repot.json']);
    const win = m.architecture['64bit'];
    assert.equal(m.version, '0.8.0');
    assert.equal(
      win.url,
      'https://github.com/junjuny0227/Repot/releases/download/v0.8.0/repot-cli-0.8.0-x86_64-pc-windows-msvc.zip',
    );
    assert.equal(win.hash, hashOf(TARGETS.windows));
    assert.equal(win.extract_dir, 'repot-cli-0.8.0-x86_64-pc-windows-msvc');
    assert.equal(m.bin, 'repot.exe');
    assert.equal(m.checkver, 'github');
    // autoupdate with this version substituted must equal the pinned url.
    const auto = m.autoupdate.architecture['64bit'];
    assert.equal(auto.url.replaceAll('$version', '0.8.0'), win.url);
    assert.equal(auto.extract_dir.replaceAll('$version', '0.8.0'), win.extract_dir);
    assert.deepEqual(auto.hash, { url: '$url.sha256' });
  } finally {
    rmSync(dir, { recursive: true });
  }
});

test('the command writes both files and fails on a missing or wrong hash', () => {
  const dir = shaDir('0.8.0');
  const out = mkdtempSync(join(tmpdir(), 'repot-cli-packages-out-'));
  try {
    const ok = spawnSync(process.execPath, [script, 'v0.8.0', dir, out], { encoding: 'utf8' });
    assert.equal(ok.status, 0, ok.stderr);
    assert.match(readFileSync(join(out, 'Formula/repot.rb'), 'utf8'), /class Repot < Formula/);
    assert.equal(JSON.parse(readFileSync(join(out, 'bucket/repot.json'), 'utf8')).version, '0.8.0');

    const wrong = spawnSync(process.execPath, [script, 'v0.9.0', dir, out], { encoding: 'utf8' });
    assert.equal(wrong.status, 1);
    assert.match(wrong.stderr, /^error: .*0\.9\.0/);

    const usage = spawnSync(process.execPath, [script, 'v0.8.0'], { encoding: 'utf8' });
    assert.equal(usage.status, 2);
  } finally {
    rmSync(dir, { recursive: true });
    rmSync(out, { recursive: true });
  }
});
