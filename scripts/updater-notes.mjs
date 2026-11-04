// Turns a published release body into the short plain-text note shown in the
// app's update sheet (latest.json "notes"). Only the "## Highlights" section is
// used; the install warning and GitHub's generated PR list are for the release
// page, not for someone who already has Repot installed.
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export function updaterNotes(body) {
  const lines = (body ?? '').replace(/\r\n/g, '\n').split('\n');
  const start = lines.findIndex(line => /^##\s+Highlights\s*$/i.test(line));
  if (start === -1) return '';
  const section = [];
  for (const line of lines.slice(start + 1)) {
    if (/^#{1,2}\s/.test(line)) break;
    if (/^\s*>/.test(line)) continue;
    section.push(line);
  }
  return section
    .join('\n')
    .replace(/<!--[\s\S]*?-->/g, '')
    .split('\n')
    .map(line => line
      .replace(/^#{3,6}\s+/, '')
      .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
      .replace(/(\*\*|__)(.+?)\1/g, '$2')
      .replace(/`([^`]+)`/g, '$1')
      .replace(/^(\s*)[-*]\s+/, '$1• ')
      .trimEnd())
    .join('\n')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
}

// node scripts/updater-notes.mjs <release-body.md> <latest.json>
// Rewrites latest.json's "notes" in place.
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [bodyFile, latestFile] = process.argv.slice(2);
  if (!bodyFile || !latestFile) {
    console.error('Usage: node scripts/updater-notes.mjs <release-body.md> <latest.json>');
    process.exit(1);
  }
  const latest = JSON.parse(readFileSync(latestFile, 'utf8'));
  latest.notes = updaterNotes(readFileSync(bodyFile, 'utf8'));
  writeFileSync(latestFile, `${JSON.stringify(latest, null, 2)}\n`);
  console.log(latest.notes || '(no Highlights section; notes cleared)');
}
