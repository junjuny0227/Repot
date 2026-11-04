import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const script = fileURLToPath(new URL('./check-version.mjs', import.meta.url));
const files = [
  'apps/desktop/src-tauri/tauri.conf.json',
  'apps/desktop/package.json',
  'apps/desktop/src-tauri/Cargo.toml',
  'crates/core/Cargo.toml',
  'crates/cli/Cargo.toml',
];

function fixture(fn) {
  const root = mkdtempSync(join(tmpdir(), 'repot-version-'));
  try {
    for (const file of files) {
      const path = join(root, file);
      mkdirSync(dirname(path), { recursive: true });
      writeFileSync(
        path,
        file.endsWith('.json')
          ? '{"version":"0.1.0"}'
          : '[package]\nname = "fixture"\nversion = "0.1.0"\n',
      );
    }
    fn(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
const run = (root, tag) =>
  spawnSync(process.execPath, [script, ...(tag === undefined ? [] : [tag])], {
    cwd: root,
    encoding: 'utf8',
  });

test('accepts matching tag and every version field', () =>
  fixture((root) => {
    assert.equal(run(root, 'v0.1.0').status, 0);
  }));

test('rejects each mismatching file and reports its value', () =>
  fixture((root) => {
    for (const file of files) {
      const path = join(root, file);
      const original = readFileSync(path, 'utf8');
      writeFileSync(path, original.replace('0.1.0', '0.2.0'));
      const result = run(root, 'v0.1.0');
      assert.equal(result.status, 1);
      assert.ok(result.stderr.includes(file), result.stderr);
      assert.ok(result.stderr.includes('0.2.0'), result.stderr);
      writeFileSync(path, original);
    }
  }));

test('rejects missing tag, missing file and missing package version', () =>
  fixture((root) => {
    assert.equal(run(root).status, 1);
    writeFileSync(join(root, files[2]), '[package]\nname = "no-version"\n');
    assert.equal(run(root, 'v0.1.0').status, 1);
    rmSync(join(root, files[0]));
    assert.equal(run(root, 'v0.1.0').status, 1);
  }));
