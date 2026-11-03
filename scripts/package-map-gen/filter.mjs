export function channels(id) {
  const labels = id.toLowerCase().split(/[/._@-]/).filter(part => /^(nightly|beta|alpha|preview|dev|canary|prerelease|insiders?|unstable|snapshot|edge)$/.test(part)).map(part => part === 'insiders' ? 'insider' : part);
  if (id.toLowerCase().endsWith('-git')) labels.push('git');
  return [...new Set(labels)].sort();
}
export const sameChannel = (a, b) => JSON.stringify(channels(a)) === JSON.stringify(channels(b));
export const kindMismatch = (a, b) => ['cli', 'gui'].includes(a) && ['cli', 'gui'].includes(b) && a !== b;
export function preFilter(source, proposals) {
  const candidates = [], dropped = [];
  for (const proposal of proposals) {
    const candidate = { ...proposal, channels: channels(proposal.id), channelMismatch: !sameChannel(source.id, proposal.id), kindFlag: kindMismatch(source.kind, proposal.kind) };
    (candidate.channelMismatch ? dropped : candidates).push(candidate);
  }
  return { candidates, dropped };
}
