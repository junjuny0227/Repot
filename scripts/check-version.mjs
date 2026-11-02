import { readFileSync } from 'node:fs';

const tag = process.argv[2];
if (!tag) {
  console.error('Usage: node scripts/check-version.mjs <tag>');
  process.exit(1);
}
const expected = tag.replace(/^v/, '');
const files = [
  'apps/desktop/src-tauri/tauri.conf.json',
  'apps/desktop/package.json',
  'apps/desktop/src-tauri/Cargo.toml',
  'crates/core/Cargo.toml',
];
let failed = false;
for (const file of files) {
  try {
    const contents = readFileSync(file, 'utf8');
    const version = file.endsWith('.json')
      ? JSON.parse(contents).version
      : contents.match(/^version = "(.+)"/m)?.[1];
    if (version !== expected) {
      console.error(`${file}: version ${version ?? '<missing>'}; tag requires ${expected}`);
      failed = true;
    }
  } catch (error) {
    console.error(`${file}: ${error.message}`);
    failed = true;
  }
}
process.exitCode = failed ? 1 : 0;
