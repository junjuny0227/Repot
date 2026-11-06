# Contributing to Repot

Thanks for helping. Repot reads a developer's machine and writes scripts that change another one, so the rules below are mostly about keeping it read-only, predictable and honest about what it moves.

## Before you start

- **Bugs, small fixes and package-map additions:** open a pull request directly. An issue first is welcome but not required.
- **New features, new providers or behavior changes:** open an issue first. Product decisions live in the [design spec](docs/specs/) (Korean), and a change that contradicts it needs agreement before code.
- Issues and pull requests can be written in English or Korean.

## Development setup

Requirements: **Rust stable** with rustfmt and Clippy (`rust-toolchain.toml`), **Node.js 22**, and the pnpm version pinned in `package.json`. Desktop builds also need the native [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS; the Linux packages are listed in the [README](README.md#build-from-source).

```sh
pnpm install
pnpm --filter @repot/desktop tauri dev
```

Tauri embeds `apps/desktop/dist` at compile time, so build the frontend **before** running workspace-wide `cargo` commands:

```sh
pnpm --filter @repot/desktop build
cargo test --workspace
```

To see what Repot finds on your machine without the app (read-only):

```sh
cargo run -p repot-core --example scan
cargo run -p repot-cli -- scan
```

Scan output contains your paths and configuration. Do not paste it into issues or commit it.

## Layout

- `crates/core`: providers, model, export renderers and the package map. No Tauri, no network.
- `crates/cli`: the `repot` command-line tool, built on `repot-core`.
- `apps/desktop`: the Tauri app (React frontend in `src`, Rust in `src-tauri`).
- `apps/site`: the product page.
- `scripts`: release and package-map tooling.
- `docs`: spec and plans, in Korean.

## Rules the code keeps

- The app only reads the machine. Scans read; exports write only to the chosen folder. The one exception is a signed app update the user approves.
- `repot-core` does not depend on Tauri and makes no network calls. The desktop app's only network use is the GitHub update check and download, and scan data never leaves the machine.
- Every command and file read goes through `Env`. Provider tests use `FakeEnv` and fixtures in `crates/core/tests/fixtures/<provider>/`, never real tools.
- Secrets are masked before anything is written. A new provider that reads configuration must mask the same way the existing ones do, and must have a test for it.
- `repot-core` builds no user-facing sentences; the UI translates them from `apps/desktop/src/locales/<locale>.json`. Keep all four locales (`en`, `ko`, `ja`, `zh-Hans`) on the same keys.
- New crate or npm dependencies need an issue first.
- Generated files are regenerated from their source, never edited by hand. For example, `apps/desktop/src/__fixtures__/scan.json` comes from `cargo run -p repot-core --example ui_fixture`, and export snapshots come from `INSTA_UPDATE=always cargo test`; read the diff before committing. `crates/core/data/package-map.json` may be edited by hand.

## Checks

Run what your change touches:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check                      # Biome, same rules as CI's `biome ci`
pnpm --filter @repot/desktop typecheck
pnpm --filter @repot/desktop test
pnpm --filter @repot/site test
node --test scripts/*.test.mjs scripts/package-map-gen/*.test.mjs
```

Restore scripts are also exercised on Windows and Linux in CI. If you change script output, check those jobs rather than assuming macOS results carry over.

## Pull requests

- Branch from `main` with a prefix that matches the change: `feat/`, `fix/`, `docs/`, `ci/`, `refactor/`, `test/` or `chore/`. The prefix sets the PR label and the release-notes section.
- Commit messages are English [Conventional Commits](https://www.conventionalcommits.org/): `feat(core): …`, `fix(desktop): …`, `docs: …`. Small commits are easier to review than one large one.
- Fill in the [pull request template](.github/pull_request_template.md) and tick only the checks you actually ran.
- If the change alters a product decision, update the spec in the same pull request.
- A pull request merges once every check passes and review comments are resolved.

## License

By contributing, you agree that your contribution is licensed under the [Apache License 2.0](LICENSE), the same license as the project.
