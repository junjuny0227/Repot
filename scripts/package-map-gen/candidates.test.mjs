import test from 'node:test';
import assert from 'node:assert/strict';
import { record } from './normalize.mjs';
import { propose } from './candidates.mjs';
import { channels, preFilter } from './filter.mjs';
const pkg = (id, repo, kind = 'cli', names = [id.split(/[/.]/).at(-1)]) => record(id, names, repo ? [`https://github.com/${repo}`] : [], kind);

test('spike regressions: ripgrep HIGH, nx name collision LOW, neovim kept and flagged', () => {
  assert.equal(propose(pkg('ripgrep', 'BurntSushi/ripgrep'), [pkg('main/ripgrep', 'BurntSushi/ripgrep')])[0].tier, 'HIGH');
  assert.equal(propose(pkg('nx', 'nrwl/nx'), [pkg('SoftMaker.Office.NX', null, 'unknown')])[0].tier, 'LOW');
  const source = pkg('neovim', 'neovim/neovim');
  const result = preFilter(source, propose(source, [pkg('Neovim.Neovim', 'neovim/neovim', 'gui'), pkg('Neovim.Neovim.Nightly', 'neovim/neovim', 'gui')]));
  assert.equal(result.candidates.length, 1);
  assert.equal(result.candidates[0].tier, 'MEDIUM');
  assert.equal(result.candidates[0].kindFlag, true);
  assert.equal(result.dropped[0].channelMismatch, true);
});
test('repo ambiguity is measured before cap/filter; pinned and unknown kinds never HIGH', () => {
  const source = pkg('tool', 'a/tool');
  const targets = Array.from({ length: 14 }, (_, i) => pkg(`a.tool${i}`, 'a/tool'));
  assert.equal(propose(source, targets).length, 12);
  assert.ok(propose(source, targets).every(p => p.tier === 'MEDIUM'));
  assert.equal(propose(pkg('tool@1', 'a/tool'), [targets[0]])[0].tier, 'MEDIUM');
  assert.equal(propose(source, [pkg('a.tool', 'a/tool', 'unknown')])[0].tier, 'MEDIUM');
  assert.equal(propose(source, [pkg('a.tool-nightly', 'a/tool')])[0].tier, 'MEDIUM');
});
test('channels use identifier boundaries, all labels, and insider synonyms', () => {
  assert.deepEqual(channels('main/devtools'), []);
  assert.deepEqual(channels('a.beta.preview'), ['beta', 'preview']);
  assert.deepEqual(channels('tool-insiders'), channels('tool.insider'));
  assert.deepEqual(channels('tool-git'), ['git']);
  for (const label of ['nightly','beta','alpha','preview','dev','canary','prerelease','insider','unstable','snapshot','edge']) {
    assert.equal(preFilter(pkg('tool'), [{ ...pkg(`tool-${label}`), tier: 'LOW' }]).candidates.length, 0);
  }
  assert.equal(preFilter(pkg('tool'), [{ ...pkg('tool', null, 'mixed'), tier: 'LOW' }]).candidates[0].kindFlag, false);
});
test('URL and exact names only, tier sorting is deterministic', () => {
  const source = record('tool', ['tool'], ['https://example.org/tool/'], 'cli');
  const targets = [record('z', ['tool'], [], 'cli'), record('a', [], ['http://example.org/tool'], 'cli'), record('similar', ['tools'], [], 'cli')];
  assert.deepEqual(propose(source, targets).map(p => [p.id, p.tier]), [['a', 'MEDIUM'], ['z', 'LOW']]);
});
