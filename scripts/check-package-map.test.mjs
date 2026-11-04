import assert from 'node:assert/strict';
import test from 'node:test';
import { classify, commandFor, selectNames, verify } from './check-package-map.mjs';

test('recognizes exact identities, not successful empty or unrelated output', () => {
  const cases = [
    ['brew', 'jq', '{"formulae":[{"name":"jq"}],"casks":[]}'],
    ['brewCask', 'firefox', '{"formulae":[],"casks":[{"token":"firefox"}]}'],
    ['winget', 'Git.Git', 'Found Git [Git.Git]\nVersion: 2.0'],
    ['scoop', 'extras/firefox', 'Name : firefox\nSource : extras\nVersion : 1'],
    ['choco', 'jq', 'jq|1.7'],
    ['apt', 'jq', 'Package: jq\nVersion: 1.7'],
    ['dnf', 'jq', 'Name         : jq\nVersion      : 1.7'],
    ['pacman', 'jq', 'Repository      : extra\nName            : jq\nVersion         : 1.7'],
  ];
  for (const [manager, name, stdout] of cases) {
    assert.equal(classify(manager, name, { status: 0, stdout }), 'found', manager);
    assert.equal(classify(manager, name, { status: 0, stdout: '' }), 'transient', manager);
    assert.equal(classify(manager, 'different', { status: 0, stdout }), 'transient', manager);
  }
});

test('metadata mentioning networking, SSL and certificates is not a transport error', () => {
  assert.equal(
    classify('pacman', 'curl', {
      status: 0,
      stdout:
        'Name : curl\nDescription : network transfer with SSL\nDepends On : openssl ca-certificates ruby-timeout',
    }),
    'found',
  );
});

test('release notes are metadata, not diagnostics', () => {
  assert.equal(
    classify('winget', 'GitHub.GitLFS', {
      status: 0,
      stdout:
        'Found Git LFS [GitHub.GitLFS]\nRelease Notes:\nFixed authentication timeouts and network error handling',
    }),
    'found',
  );
});

test('distinguishes authoritative absence from transport and command failures', () => {
  const cases = [
    ['brew', 'Error: No available formula with the name "missing"'],
    ['brewCask', "Error: Cask 'missing' is unavailable: No Cask with this name exists."],
    ['winget', 'No package found matching input criteria.'],
    ['scoop', "Could not find manifest for 'missing' in local buckets."],
    ['choco', '0 packages found.'],
    ['apt', 'E: No packages found'],
    ['dnf', 'No matches found.'],
    ['dnf', 'Error: No matching Packages to list'],
    ['pacman', "error: package 'missing' was not found"],
  ];
  for (const [manager, stdout] of cases) {
    assert.equal(classify(manager, 'missing', { status: 1, stdout }), 'missing');
    assert.equal(
      classify(manager, 'missing', {
        status: 1,
        stdout: `HTTP 503 service unavailable\n${stdout}`,
      }),
      'transient',
    );
    assert.equal(
      classify(manager, 'missing', { status: 1, stdout: 'TLS handshake failed' }),
      'transient',
    );
    assert.equal(
      classify(manager, 'missing', { status: null, error: 'ENOENT', stdout }),
      'transient',
    );
  }
  assert.equal(classify('choco', 'jq', { status: 1, stdout: 'jq|1.7' }), 'transient');
  assert.equal(
    classify('choco', 'jq', { status: 0, stdout: 'jq|1.7\nERROR: source unavailable' }),
    'transient',
  );
});

test('retries only transient failures and never passes an exhausted retry', async () => {
  let calls = 0;
  const run = () =>
    ++calls < 3 ? { status: 1, stdout: 'HTTP 503' } : { status: 0, stdout: 'Package: jq' };
  assert.equal(await verify('apt', 'jq', { run, sleep: async () => {} }), 'found');
  assert.equal(calls, 3);
  calls = 0;
  assert.equal(
    await verify('apt', 'missing', {
      run: () => {
        calls++;
        return { status: 100, stdout: 'E: No packages found' };
      },
      sleep: async () => {},
    }),
    'missing',
  );
  assert.equal(calls, 1);
  calls = 0;
  assert.equal(
    await verify('apt', 'jq', {
      run: () => {
        calls++;
        return { status: 1, stdout: 'network failure' };
      },
      sleep: async () => {},
    }),
    'transient',
  );
  assert.equal(calls, 3);
});

test('selects only known names for requested managers, preserving explicit absence', () => {
  assert.deepEqual(
    selectNames({ packages: { a: { brew: 'a', apt: null }, b: { apt: 'b' } } }, ['apt']),
    [['apt', 'b']],
  );
  assert.deepEqual(commandFor('winget', 'Git.Git'), [
    'winget',
    [
      'show',
      '--id',
      'Git.Git',
      '--exact',
      '--source',
      'winget',
      '--accept-source-agreements',
      '--disable-interactivity',
    ],
  ]);
  assert.throws(() => commandFor('apt', 'x;rm'), /Invalid/);
  assert.throws(() => commandFor('unknown', 'x'), /manager/);
});
