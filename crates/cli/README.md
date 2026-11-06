# repot command-line tool

English · [한국어](README.ko.md) · [日本語](README.ja.md) · [简体中文](README.zh-Hans.md)

`repot` moves a development environment to a new machine from a terminal. It does what
the [Repot app](../../README.md) does — scan, compare with a Repot file, export a restore
script — for machines without a desktop, over SSH, or in your own scripts.

It follows the same rules as the app:

- It only **reads** this machine. The one thing it writes is the export folder.
- It never runs the restore script for you and never asks questions.
- It never uses the network.
- Secrets in config files (tokens, keys) are always masked, and it tells you where.

Command output is in English.

## Install

| | |
| --- | --- |
| macOS, Linux (Homebrew) | `brew install junjuny0227/tap/repot` |
| Linux, macOS (install script) | `curl -fsSL https://junjuny0227.github.io/Repot/install.sh \| sh` ([source](https://github.com/junjuny0227/Repot/blob/main/apps/site/public/install.sh)) |
| Windows (Scoop) | `scoop bucket add repot https://github.com/junjuny0227/scoop-bucket` then `scoop install repot/repot` |
| Any OS, by hand | Download `repot-cli-<version>-<target>.tar.gz` / `.zip` from [Releases](https://github.com/junjuny0227/Repot/releases/latest), check it against its `.sha256` file and put `repot` on your `PATH` |
| With Rust | `cargo install --git https://github.com/junjuny0227/Repot repot-cli` |

Archives downloaded with a browser on macOS are quarantined; run
`xattr -d com.apple.quarantine repot` once. Scoop itself is not part of Windows; install it
from [scoop.sh](https://scoop.sh) first or use the zip.

Update with `brew upgrade repot` or `scoop update repot`. Check the version with
`repot --version`.

## Moving to a new machine

**On the old machine**

```sh
repot scan                       # look at what Repot finds
repot export --out ~/Desktop     # writes ~/Desktop/repot-export-<date>/
```

```text
241 items for macOS (Homebrew)
Wrote /Users/you/Desktop/repot-export-2026-11-04
Next: copy the folder to the new machine and run `bash restore.sh`.
```

The folder holds `restore.sh` (or `restore.ps1` for Windows), `repot.json`, a `files/`
folder with the config files the script puts back and, when packages were mapped to
another OS's names, a `README.md` listing them. Copy the whole folder to the new
machine (USB drive, AirDrop, a private repository — it contains your config files).

If the new machine runs another OS, say so with `--to`:

```sh
repot export --to windows --packages scoop --out ~/Desktop
```

**On the new machine**

The export already contains everything. Read the script, then run it:

```sh
bash restore.sh            # macOS, Linux
```

```powershell
powershell -ExecutionPolicy Bypass -File restore.ps1    # Windows
```

The script first lists the tools it needs (Homebrew, winget, VS Code and so on) and where
to get any that are missing, then asks `Continue? [y/N]` (`--yes` / `-Yes` skips the
question). It skips what is already installed, backs up a config file before replacing it
(`<file>.repot-backup-<time>`), and ends with a summary: **OK**, **Already done**, **Not
done** (with the reason, such as `brew not found`) and **Failed**. Running it again is
safe; install a missing tool and rerun to finish the rest.

**Only what is missing** (optional)

When the new machine already has most things — or you set it up a while ago — install
`repot` there too and export only the difference from the old `repot.json`:

```sh
repot compare repot.json
repot export repot.json --only-missing --out ~/Desktop
```

```text
# example output, shortened
repot.json (from macOS) compared with this machine

Already here: 236

Different here: 1
  .zshrc                                   shell:file:.zshrc

Missing: 4
  ripgrep 14.1.1                           brew:formula:ripgrep
  ...
```

- **Already here**: installed, or the same file is already in place.
- **Different here**: a config file exists here with other contents. It is not exported
  by `--only-missing`; add it with `--include <id>` if you want the old version back (the
  script backs up the current one first).
- **Missing**: what `--only-missing` exports.
- **Not usable on <OS>**: items from another OS that have nothing to compare with here.

`--only-missing` compares with *this* machine, so the destination must be this machine's
OS. If nothing is missing, `repot` says so and exits with 0 without writing anything.

## Commands

### `repot scan`

Lists what Repot finds on this machine, grouped by category, with each item's **id** on
the right. Progress (`brew: 62`) and providers that failed go to stderr; tools that are
not installed are listed at the end.

```text
macOS: 241 items

Packages (62)
  gh 2.102.0                               brew:formula:gh
  hashicorp/tap                            brew:tap:hashicorp/tap
  ...

Runtimes (6)
  Node.js 24.21.0                          nvm:node:24.21.0
  Rust toolchain stable                    rustup:rust:stable
  ...

Not installed: winget, scoop, choco, apt, dnf, pacman, ...
```

### `repot compare <FILE>`

Compares a Repot file (`repot.json` from an export or from the app) with this machine.
See **Only what is missing** above for what each group means.

### `repot export [FILE]`

Writes an export folder. Without `FILE` it scans this machine; with `FILE` it exports
from that Repot file instead (for example to make a Windows script from an old Mac's
`repot.json`).

| Option | Meaning |
| --- | --- |
| `--to <OS>` | Destination: `macos`, `windows`, `linux-debian`, `linux-fedora`, `linux-arch`, `linux-other`. Default: this machine's OS |
| `--packages <MANAGER>` | Package manager on the destination: `brew`, `winget`, `scoop`, `choco`, `apt`, `dnf`, `pacman`. Windows defaults to winget |
| `--format script` | One restore script (default) |
| `--format manifest` | Native files instead: `Brewfile`, winget JSON, `.tool-versions`, `npm-global.txt`, editor extension lists, and a `README.md` saying how to use them |
| `--only-missing` | With `FILE`: export only what this machine is missing |
| `--include <ID>` | Also export this item (repeatable) |
| `--exclude <ID>` | Leave this item out (repeatable) |
| `--out <DIR>` | Folder to create `repot-export-<date>/` in. Default: the current folder |
| `--dry-run` | List the files that would be written; write nothing |
| `--json` | Print the result as JSON |

**What is exported by default** is the same as what the app starts with checked:
everything the destination can use. Items it cannot use are left out and listed on stderr
with the reason, for example:

```text
note: left out boring-notch: Repot's package map has no Scoop name for it yet
note: left out aodjo/tap: only works on macOS
```

Items that a previous export could not include (marked *Not in last export* in the app)
also start unselected. `--include` adds any of them back; an item the destination really
cannot use is then skipped with a `warning:` line.

**Picking items by id.** Ids are the right-hand column of `scan` and `compare` (or `id` in
their `--json` output):

```sh
repot export --exclude brew:cask:discord --exclude vscode:code:ext:ms-python.python
repot export --to windows --include npm:typescript --dry-run
```

An unknown id is an error, so a typo never silently changes the export.

**Secrets.** Lines that look like tokens or keys are replaced with a placeholder in the
exported copy. `repot` prints where it did so (`note: masked secrets in .zshrc (lines 123,
124)`); put those values back by hand on the new machine. The app can include them if you
need that.

## Scripting

- `--json` on any command prints JSON on stdout. Notes, warnings and progress always go
  to stderr, so stdout stays parseable.
  - `scan --json`: `{ "items": [...], "providers": [...] }`, the same item shape as in
    `repot.json`.
  - `compare --json`: `{ "<id>": "present" | "differs" | "missing", ... }`.
  - `export --json`: `folder` (`null` with `--dry-run`), `target`, `selected` (ids),
    `files` (`path`, `executable`), `warnings` and `leftOut` (`id`, `reason`).
- The JSON shape can change while Repot is 0.x.
- Exit codes: `0` success (including "nothing missing"), `1` the command could not do its
  job (`error: …` on stderr), `2` wrong usage (unknown option or value).

```sh
# Which of the old machine's packages are missing here?
repot compare old/repot.json --json | jq -r 'to_entries[] | select(.value == "missing") | .key'
```

## More

- `repot --help` and `repot <command> --help` list every option.
- [Repot README](../../README.md): what Repot moves, known limitations, the app.
- Problems and ideas: [GitHub issues](https://github.com/junjuny0227/Repot/issues).
