# Repot

Move your dev environment to a new machine — explained, selective, and safe.

## What it does

Scan your current machine, review each item with its description and portability
warnings, select what you want to move, and export either a readable restore
script or a bundle of native manifests. Copy the export folder to the new machine
and review it before running anything. Repot does not run the restore for you.

Supported providers:

- **Packages:** Homebrew, winget, Scoop, Chocolatey, apt, dnf, pacman.
- **Runtimes:** nvm, nvm-windows, fnm, Volta, pyenv (including pyenv-win), rustup,
  SDKMAN!.
- **Global packages:** npm, pnpm, pipx, Cargo.
- **Editors:** VS Code and Cursor extensions, settings, keybindings and snippets.
- **Git:** user configuration and the configured global ignore file.
- **Shell:** zsh, Bash, fish and PowerShell profiles, oh-my-zsh custom plugins and
  themes, and Starship configuration.

Package-manager entries can only move to the same OS (and Linux distribution
family). Supported runtime managers are mapped between operating systems;
unavailable items are disabled rather than guessed.

## Principles

- **Explains everything:** shows local descriptions and portability warnings, or
  honestly says when no description is available.
- **Never changes your machine:** scans read-only and writes only to your chosen
  export folder; you decide whether to run the generated restore script.
- **Conservative by default:** masks detected secrets, backs up changed destination
  files, and never installs package managers for you.
- **No network:** the app uses local data and built-in descriptions. The restore
  scripts you run separately can download packages through their respective tools.
- **Review before you run:** preview the files before saving and read the exported
  script before executing it.

## Install

Download an installer for your OS from [GitHub Releases](https://github.com/junjuny0227/Repot/releases)
when a release is available. Builds are currently unsigned beta software; code
signing and automatic updates are not included. Draft releases are not public.

First launch after verifying that you trust the download:

- **macOS (Apple silicon or Intel):** open the `.dmg`, copy Repot to Applications,
  then open it. If macOS blocks it, use **System Settings › Privacy & Security ›
  Open Anyway** and confirm.
- **Windows:** open the `.msi` or setup `.exe`. If SmartScreen blocks it, choose
  **More info › Run anyway** only if you trust the file.
- **Linux:** make the downloaded AppImage executable with
  `chmod +x Repot_*.AppImage`, then open it, or install the `.deb` using your
  distribution's package installer.

## Using an export

1. Copy the **whole** `repot-export-YYYY-MM-DD/` folder to the destination machine,
   including `files/`.
2. Read the script and payloads. Secret detection is pattern-based, not a guarantee:
   check for secrets the detector may have missed. Replace `<REDACTED>` placeholders
   with the appropriate values on the destination before using the configuration.
3. Install any missing package or runtime managers yourself; Repot prints their
   installation links and skips those sections if they are absent.
4. From the export folder, run the script for the selected destination:

   **macOS / Linux**

   ```sh
   ./restore.sh
   ```

   **Windows PowerShell 5.1 or later**

   ```powershell
   powershell -ExecutionPolicy Bypass -File restore.ps1
   ```

The script asks for confirmation, skips existing items, backs up configuration
files before changing them, and summarizes successes, skips and failures. A failed
step makes its exit code nonzero. Review all output; package installation is not
transactional and is not rolled back.

For a **native manifest bundle**, follow the exported `README.md` instead. It lists
commands for the manifests present in that export and the destinations for each
configuration payload. These bundles contain only the files needed for your
selection.

## Known limitations

- apt and dnf's manually installed lists may include base-system packages.
- winget and Cargo entries have no local descriptions.
- Cross-manager duplicates are detected only by identical names (case-insensitive),
  not automatically removed; package names are not mapped between operating systems.
- Pattern-based secret detection cannot find every secret. Always review exports,
  especially if you choose to reveal a detected secret.
- WSL and Git Bash environments are not supported.
- winget handles only the `winget` source, not Microsoft Store (`msstore`) packages.
- Volta scans runtimes only, not global packages installed through Volta.
- rustup scans toolchains only, not components or targets.
- Scoop does not preserve custom bucket URLs other than its default bucket.

## Build from source

Prerequisites: **Rust stable** (with rustfmt and Clippy), **Node.js 22**, and the
pnpm version pinned in `package.json`. Desktop builds also need the native Tauri 2
build prerequisites (Xcode Command Line Tools on macOS, Microsoft C++ Build Tools
and WebView2 on Windows).

On Debian/Ubuntu, install these native build packages:
`libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.
Use equivalent Tauri 2 dependencies for other Linux distributions.

From the repository root:

```sh
pnpm install
pnpm --filter @repot/desktop tauri dev
```

For tests, build the frontend **before** compiling the Rust workspace because
Tauri embeds `apps/desktop/dist` at compile time:

```sh
pnpm --filter @repot/desktop build
cargo test --workspace
cargo run -p repot-core --example scan
```

The scan example reads your local environment and prints JSON; it does not restore
or install anything. CI runs Rust checks, real scans and restore smoke tests on
macOS, Windows and Ubuntu, plus frontend type checks, tests and build. Release
workflow builds require a matching version tag and create only draft prereleases;
publishing is a separate human decision.

## Docs

- [MVP design](docs/specs/2026-11-02-repot-mvp-design.md) (Korean)
- [Implementation plans](docs/plans/) (Korean)

## License

[Apache-2.0](LICENSE).
