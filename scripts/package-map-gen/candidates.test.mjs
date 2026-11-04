import assert from 'node:assert/strict';
import test from 'node:test';
import { propose } from './candidates.mjs';
import { channels, preFilter } from './filter.mjs';
import { record } from './normalize.mjs';

const pkg = (id, repo, kind = 'cli', names = [id.split(/[/.]/).at(-1)]) =>
  record(id, names, repo ? [`https://github.com/${repo}`] : [], kind);

test('spike regressions: ripgrep HIGH, nx name collision LOW, neovim kept and flagged', () => {
  assert.equal(
    propose(pkg('ripgrep', 'BurntSushi/ripgrep'), [pkg('main/ripgrep', 'BurntSushi/ripgrep')])[0]
      .tier,
    'HIGH',
  );
  assert.equal(
    propose(pkg('nx', 'nrwl/nx'), [pkg('SoftMaker.Office.NX', null, 'unknown')])[0].tier,
    'LOW',
  );
  const source = pkg('neovim', 'neovim/neovim');
  const result = preFilter(
    source,
    propose(source, [
      pkg('Neovim.Neovim', 'neovim/neovim', 'gui'),
      pkg('Neovim.Neovim.Nightly', 'neovim/neovim', 'gui'),
    ]),
  );
  assert.equal(result.candidates.length, 1);
  assert.equal(result.candidates[0].tier, 'MEDIUM');
  assert.equal(result.candidates[0].kindFlag, true);
  assert.equal(result.dropped[0].channelMismatch, true);
});
test('repo ambiguity is measured before cap/filter; pinned and unknown kinds never HIGH', () => {
  const source = pkg('tool', 'a/tool');
  const targets = Array.from({ length: 14 }, (_, i) => pkg(`a.tool${i}`, 'a/tool'));
  assert.equal(propose(source, targets).length, 12);
  assert.ok(propose(source, targets).every((p) => p.tier === 'MEDIUM'));
  assert.equal(propose(pkg('tool@1', 'a/tool'), [targets[0]])[0].tier, 'MEDIUM');
  assert.equal(propose(source, [pkg('a.tool', 'a/tool', 'unknown')])[0].tier, 'MEDIUM');
  assert.equal(propose(source, [pkg('a.tool-nightly', 'a/tool')])[0].tier, 'MEDIUM');
});
test('channels use identifier boundaries, all labels, and insider synonyms', () => {
  assert.deepEqual(channels('main/devtools'), []);
  assert.deepEqual(channels('a.beta.preview'), ['beta', 'preview']);
  assert.deepEqual(channels('tool-insiders'), channels('tool.insider'));
  assert.deepEqual(channels('tool-git'), ['git']);
  for (const label of [
    'nightly',
    'beta',
    'alpha',
    'preview',
    'dev',
    'canary',
    'prerelease',
    'insider',
    'unstable',
    'snapshot',
    'edge',
  ]) {
    assert.equal(
      preFilter(pkg('tool'), [{ ...pkg(`tool-${label}`), tier: 'LOW' }]).candidates.length,
      0,
    );
  }
  assert.equal(
    preFilter(pkg('tool'), [{ ...pkg('tool', null, 'mixed'), tier: 'LOW' }]).candidates[0].kindFlag,
    false,
  );
});
test('URL and exact names only, tier sorting is deterministic', () => {
  const source = record('tool', ['tool'], ['https://example.org/tool/'], 'cli');
  const targets = [
    record('z', ['tool'], [], 'cli'),
    record('a', [], ['http://example.org/tool'], 'cli'),
    record('similar', ['tools'], [], 'cli'),
  ];
  assert.deepEqual(
    propose(source, targets).map((p) => [p.id, p.tier]),
    [
      ['a', 'MEDIUM'],
      ['z', 'LOW'],
    ],
  );
});
test('exact id rules: versioned JDK casks and Nerd Font casks find MEDIUM candidates only when the id exists', () => {
  const cask = (id, names = [id]) => ({ ...record(id, names, [], 'gui'), brew_kind: 'cask' });
  const scoop = [
    pkg('java/temurin17-jdk', null, 'unknown'),
    pkg('java/zulu17-jdk', null, 'unknown'),
    pkg('java/temurin17-jre', null, 'unknown'),
    pkg('nerd-fonts/Hack-NF', 'ryanoasis/nerd-fonts', 'unknown'),
    pkg('nerd-fonts/Hack-NF-Mono', 'ryanoasis/nerd-fonts', 'unknown'),
    pkg('nerd-fonts/JetBrainsMono-NF', 'ryanoasis/nerd-fonts', 'unknown'),
    pkg('nerd-fonts/JetBrains-Mono', null, 'unknown'),
  ];
  const winget = [
    pkg('EclipseAdoptium.Temurin.17.JDK', null, 'unknown'),
    pkg('EclipseAdoptium.Temurin.17.JRE', null, 'unknown'),
    pkg('Azul.Zulu.17.JDK', null, 'unknown'),
  ];
  const ids = (source, records) => propose(source, records).map((p) => [p.id, p.tier]);
  assert.deepEqual(ids(cask('temurin@17'), scoop), [['java/temurin17-jdk', 'MEDIUM']]);
  assert.deepEqual(ids(cask('temurin@17'), winget), [['EclipseAdoptium.Temurin.17.JDK', 'MEDIUM']]);
  assert.deepEqual(ids(cask('zulu@17'), scoop), [['java/zulu17-jdk', 'MEDIUM']]);
  assert.deepEqual(ids(cask('zulu@17'), winget), [['Azul.Zulu.17.JDK', 'MEDIUM']]);
  assert.match(propose(cask('zulu@17'), winget)[0].reason, /versioned JDK/);
  // The id must exist exactly: no nearest version, no unversioned or non-cask source.
  assert.deepEqual(ids(cask('temurin@21'), [...scoop, ...winget]), []);
  assert.deepEqual(ids(cask('temurin'), [...scoop, ...winget]), []);
  assert.deepEqual(ids({ ...cask('temurin@17'), brew_kind: 'formula' }, winget), []);
  // Nerd Font: exact normalized name equality with the plain -NF variant only, ranked first within its tier.
  const hack = propose(
    cask('font-hack-nerd-font', ['font-hack-nerd-font', 'Hack Nerd Font']),
    scoop,
  );
  assert.deepEqual(
    hack.map((p) => [p.id, p.tier]),
    [['nerd-fonts/Hack-NF', 'MEDIUM']],
  );
  assert.match(hack[0].reason, /Nerd Font/);
  const jb = cask('font-jetbrains-mono-nerd-font', [
    'font-jetbrains-mono-nerd-font',
    'JetBrainsMono Nerd Font',
  ]);
  jb.repos = ['github.com/ryanoasis/nerd-fonts'];
  const jbResult = propose(jb, [
    ...Array.from({ length: 14 }, (_, i) =>
      pkg(`nerd-fonts/A${i}-NF`, 'ryanoasis/nerd-fonts', 'unknown'),
    ),
    ...scoop,
  ]);
  assert.equal(jbResult.length, 12);
  assert.deepEqual([jbResult[0].id, jbResult[0].tier], ['nerd-fonts/JetBrainsMono-NF', 'MEDIUM']);
  assert.equal(jbResult.filter((p) => p.id === 'nerd-fonts/JetBrainsMono-NF').length, 1);
  assert.deepEqual(
    ids(cask('font-caskaydia-cove-nerd-font'), [
      pkg('nerd-fonts/CascadiaCode-NF', null, 'unknown'),
    ]),
    [],
  );
  assert.deepEqual(ids(cask('font-hack'), scoop), []);
});
