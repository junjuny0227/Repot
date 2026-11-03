import test from 'node:test';
import assert from 'node:assert/strict';
import { normUrl, repoKey, nameKey, record, canonical } from './normalize.mjs';

test('normalizes only exact URL/repo/name evidence and Scoop main aliases', () => {
  assert.equal(normUrl('https://www.GitHub.com/BurntSushi/ripgrep.git/'), 'github.com/burntsushi/ripgrep');
  assert.equal(repoKey('https://raw.githubusercontent.com/Owner/Repo/main/file'), 'github.com/owner/repo');
  assert.equal(repoKey('https://api.github.com/repos/Owner/Repo/releases'), 'github.com/owner/repo');
  assert.equal(repoKey('https://gitlab.com/group/sub/project/-/releases'), 'gitlab.com/group/sub/project');
  assert.equal(repoKey('https://gitlab.com/api/v4/projects/123'), null);
  assert.equal(normUrl('not a URL'), null);
  assert.equal(nameKey('Docker-Desktop'), 'dockerdesktop');
  assert.equal(canonical('scoop', 'Ripgrep'), 'main/ripgrep');
  assert.equal(canonical('scoop', 'Extras/Ripgrep'), 'extras/ripgrep');
  assert.deepEqual(record('rg', ['rip-grep'], ['https://github.com/a/b'], 'cli').repos, ['github.com/a/b']);
});
