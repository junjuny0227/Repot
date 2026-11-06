// Runs apps/site/public/install.sh against a local stand-in for GitHub Releases, so no
// network is used. CI also runs the script against the real latest release (ci.yml,
// `install-script`). Needs a POSIX shell, so it lives outside scripts/*.test.mjs, which the
// Windows package-map job runs.
import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, before, test } from 'node:test';

const SCRIPT = new URL('../../apps/site/public/install.sh', import.meta.url).pathname;
const os = { Linux: 'unknown-linux-gnu', Darwin: 'apple-darwin' }[
  execFileSync('uname', ['-s'], { encoding: 'utf8' }).trim()
];
const arch = { x86_64: 'x86_64', arm64: 'aarch64', aarch64: 'aarch64' }[
  execFileSync('uname', ['-m'], { encoding: 'utf8' }).trim()
];
const TARGET = `${arch}-${os}`;

const work = mkdtempSync(join(tmpdir(), 'repot-install-'));
const files = new Map(); // URL path -> contents
let server;
let base;

// A release with a stand-in `repot` that prints its version, like the real one.
function release(version, { badChecksum = false } = {}) {
  const name = `repot-cli-${version}-${TARGET}`;
  const dir = join(work, 'build', version);
  mkdirSync(join(dir, name), { recursive: true });
  writeFileSync(join(dir, name, 'repot'), `#!/bin/sh\necho "repot ${version}"\n`);
  chmodSync(join(dir, name, 'repot'), 0o755);
  writeFileSync(join(dir, name, 'LICENSE'), 'license\n');
  execFileSync('tar', ['-czf', `${name}.tar.gz`, name], { cwd: dir });
  const archive = readFileSync(join(dir, `${name}.tar.gz`));
  const sum = badChecksum ? '0'.repeat(64) : createHash('sha256').update(archive).digest('hex');
  files.set(`/download/v${version}/${name}.tar.gz`, archive);
  files.set(`/download/v${version}/${name}.tar.gz.sha256`, `${sum}  ${name}.tar.gz\n`);
}

before(async () => {
  release('9.9.9');
  release('1.0.0');
  release('6.6.6', { badChecksum: true });
  server = createServer((req, res) => {
    if (req.url === '/latest') {
      res.writeHead(302, { location: `${base}/tag/v9.9.9` }).end();
    } else if (req.url === '/tag/v9.9.9') {
      res.writeHead(200).end('release page');
    } else if (files.has(req.url)) {
      res.writeHead(200).end(files.get(req.url));
    } else {
      res.writeHead(404).end();
    }
  });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  base = `http://127.0.0.1:${server.address().port}`;
});
after(() => server.closeAllConnections() || server.close());

// Async on purpose: spawnSync would block the event loop that serves the downloads.
function install(env = {}) {
  const home = mkdtempSync(join(work, 'home-'));
  const child = spawn('sh', [SCRIPT], {
    env: {
      PATH: process.env.PATH,
      HOME: home,
      SHELL: '/bin/zsh',
      REPOT_RELEASES_URL: base,
      ...env,
    },
  });
  let stdout = '';
  let stderr = '';
  child.stdout.on('data', (d) => (stdout += d));
  child.stderr.on('data', (d) => (stderr += d));
  return new Promise((resolve) =>
    child.on('close', (status) =>
      resolve({ status, stdout, stderr, home, bin: join(home, '.local', 'bin', 'repot') }),
    ),
  );
}

test('installs the latest release into ~/.local/bin and says how to add it to PATH', async () => {
  const r = await install();
  assert.equal(r.status, 0, r.stderr);
  assert.match(r.stdout, new RegExp(`Downloading repot 9\\.9\\.9 for ${TARGET}`));
  assert.equal(execFileSync(r.bin, { encoding: 'utf8' }), 'repot 9.9.9\n');
  assert.match(r.stdout, /Installed repot 9\.9\.9 to .*\/\.local\/bin\/repot/);
  assert.match(r.stdout, /is not on your PATH[\s\S]*>> ~\/\.zshrc/);
  assert.ok(!existsSync(join(r.home, '.local', 'bin', 'LICENSE')), 'only repot is copied');
});

test('REPOT_VERSION and REPOT_INSTALL_DIR choose the release and the folder', async () => {
  const dir = join(mkdtempSync(join(work, 'dir-')), 'bin');
  const r = await install({
    REPOT_VERSION: 'v1.0.0',
    REPOT_INSTALL_DIR: dir,
    PATH: `${dir}:${process.env.PATH}`,
  });
  assert.equal(r.status, 0, r.stderr);
  assert.equal(execFileSync(join(dir, 'repot'), { encoding: 'utf8' }), 'repot 1.0.0\n');
  assert.doesNotMatch(r.stdout, /not on your PATH/);
});

test('a checksum mismatch installs nothing', async () => {
  const r = await install({ REPOT_VERSION: '6.6.6' });
  assert.equal(r.status, 1);
  assert.match(r.stderr, /checksum mismatch .*nothing was installed/);
  assert.ok(!existsSync(r.bin));
});

test('a missing release fails with the URL it tried', async () => {
  const r = await install({ REPOT_VERSION: '0.0.1' });
  assert.equal(r.status, 1);
  assert.match(
    r.stderr,
    new RegExp(
      `could not download .*/download/v0\\.0\\.1/repot-cli-0\\.0\\.1-${TARGET}\\.tar\\.gz`,
    ),
  );
  assert.ok(!existsSync(r.bin));
});
