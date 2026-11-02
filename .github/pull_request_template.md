## Summary

<!-- What changes and why. Link the issue, plan task (docs/plans/...) or spec section (docs/specs/...). -->

## Type

- [ ] feat
- [ ] fix
- [ ] refactor / chore
- [ ] docs
- [ ] ci / build

## Changes

-

## Checks

<!-- Tick what you ran. Build the frontend before workspace cargo commands. -->

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `pnpm --filter @repot/desktop typecheck` / `test` / `build`
- [ ] Ran the real flow if behavior changed (`--example scan` / `--example export`, or `tauri dev`)
- [ ] Not checked locally, CI only (say which, e.g. Windows/Linux providers, shellcheck):

## Spec and generated files

- [ ] Spec updated in this PR if a product decision changed, or no decision changed
- [ ] Generated files regenerated from their source, not edited by hand (`scan.json`, `snapshots/`, `icons/`)
- [ ] Snapshot diffs read and intended

## Boundaries

- [ ] `repot-core` does not depend on Tauri and does no network calls
- [ ] Commands and file reads go through `Env`; tests use `FakeEnv` and fixtures, not real tools
- [ ] No new crate or npm dependency, or the plan/spec was updated first
- [ ] The app still only reads the device; it writes only to the export folder

## Screenshots

<!-- UI changes: before and after, light and dark. Delete this section otherwise. -->
