# repot コマンドラインツール

[English](README.md) · [한국어](README.ko.md) · 日本語 · [简体中文](README.zh-Hans.md)

`repot` は、ターミナルから開発環境を新しいマシンに移します。[Repot アプリ](../../README.ja.md)
と同じこと（スキャン、Repot ファイルとの比較、復元スクリプトの書き出し）を、画面のないマシン、
SSH 接続、自分のスクリプトから行えます。

アプリと同じルールに従います。

- このマシンを**読み取るだけ**です。書き込むのは書き出しフォルダー 1 つだけです。
- 復元スクリプトを代わりに実行することはなく、確認の質問もしません。
- ネットワークを使いません。
- 設定ファイル内の秘密情報（トークン、キー）は常に伏せ、どこを伏せたかを表示します。

コマンドの出力は英語です。このガイドの出力例も実際の出力どおり英語のままにしています。

## インストール

| | |
| --- | --- |
| macOS、Linux（Homebrew） | `brew install junjuny0227/tap/repot` |
| Linux、macOS（インストールスクリプト） | `curl -fsSL https://junjuny0227.github.io/Repot/install.sh \| sh` ([スクリプト](https://github.com/junjuny0227/Repot/blob/main/apps/site/public/install.sh)) |
| Windows（Scoop） | `scoop bucket add repot https://github.com/junjuny0227/scoop-bucket` のあと `scoop install repot/repot` |
| すべての OS（手動） | [Releases](https://github.com/junjuny0227/Repot/releases/latest) から `repot-cli-<バージョン>-<target>.tar.gz` / `.zip` をダウンロードし、`.sha256` ファイルで確認してから `repot` を `PATH` に置きます |
| Rust がある場合 | `cargo install --git https://github.com/junjuny0227/Repot repot-cli` |

macOS でブラウザからダウンロードしたアーカイブは隔離されるため、
`xattr -d com.apple.quarantine repot` を一度実行します。Scoop は Windows に標準では含まれて
いません。先に [scoop.sh](https://scoop.sh) からインストールするか、zip を使ってください。

アップデートは `brew upgrade repot` または `scoop update repot` で行います。バージョンは
`repot --version` で確認できます。

## 新しいマシンへの移行

**古いマシンで**

```sh
repot scan                       # look at what Repot finds
repot export --out ~/Desktop     # writes ~/Desktop/repot-export-<date>/
```

```text
241 items for macOS (Homebrew)
Wrote /Users/you/Desktop/repot-export-2026-11-04
Next: copy the folder to the new machine and run `bash restore.sh`.
```

フォルダーには `restore.sh`（Windows 向けなら `restore.ps1`）、`repot.json`、スクリプトが
元に戻す設定ファイルを入れた `files/` フォルダーが入っています。パッケージを別の OS の名前に
対応させた場合は、その一覧を書いた `README.md` もあります。フォルダー全体を新しいマシンに
コピーしてください（USB、AirDrop、非公開リポジトリなど。設定ファイルが入っているので公開の
場所には置かないでください）。

新しいマシンの OS が異なる場合は `--to` で指定します。

```sh
repot export --to windows --packages scoop --out ~/Desktop
```

**新しいマシンで**

書き出したフォルダーに必要なものはすべて入っています。スクリプトを読んでから実行します。

```sh
bash restore.sh            # macOS, Linux
```

```powershell
powershell -ExecutionPolicy Bypass -File restore.ps1    # Windows
```

スクリプトはまず必要なツール（Homebrew、winget、VS Code など）と、ないツールの入手先を表示し、
`Continue? [y/N]` と尋ねます（`--yes` / `-Yes` を付けると尋ねません）。インストール済みの
ものはスキップし、設定ファイルを置き換える前にバックアップ（`<file>.repot-backup-<time>`）を
残し、最後に **OK**、**Already done**、**Not done**（`brew not found` などの理由付き）、
**Failed** にまとめます。再実行しても安全なので、足りないツールを入れてからもう一度実行すれば
残りが終わります。

**ないものだけ**（任意）

新しいマシンにすでにほとんどが揃っている場合や、以前セットアップしたマシンの場合は、そのマシンにも
`repot` をインストールし、古い `repot.json` との差分だけを書き出します。

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

- **Already here**：インストール済み、または同じファイルがすでにあります。
- **Different here**：同じ設定ファイルがありますが内容が異なります。`--only-missing` では
  書き出しません。古い版に戻したい場合は `--include <id>` で追加します（スクリプトが現在の
  ファイルを先にバックアップします）。
- **Missing**：`--only-missing` が書き出すものです。
- **Not usable on <OS>**：別の OS の項目で、このマシンでは比較する対象がありません。

`--only-missing` は*この*マシンと比較するため、書き出し先はこのマシンの OS である必要が
あります。ないものが 1 つもなければそう表示し、何も書かずに 0 で終了します。

## コマンド

### `repot scan`

このマシンで見つけたものを分類ごとに表示し、右側に項目ごとの **id** を付けます。進行状況
（`brew: 62`）と失敗した Provider は stderr に出力され、インストールされていないツールは最後に
一覧表示されます。

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

Repot ファイル（書き出しやアプリで作った `repot.json`）をこのマシンと比較します。各グループの
意味は上の**ないものだけ**を参照してください。

### `repot export [FILE]`

書き出しフォルダーを作ります。`FILE` がなければこのマシンをスキャンし、あればその Repot
ファイルから書き出します（例：古い Mac の `repot.json` から Windows 用スクリプトを作る）。

| オプション | 意味 |
| --- | --- |
| `--to <OS>` | 書き出し先：`macos`、`windows`、`linux-debian`、`linux-fedora`、`linux-arch`、`linux-other`。既定はこのマシンの OS |
| `--packages <MANAGER>` | 書き出し先のパッケージマネージャー：`brew`、`winget`、`scoop`、`choco`、`apt`、`dnf`、`pacman`。Windows の既定は winget |
| `--format script` | 復元スクリプト 1 つ（既定） |
| `--format manifest` | 代わりに各ツールのファイル：`Brewfile`、winget JSON、`.tool-versions`、`npm-global.txt`、エディター拡張機能の一覧、使い方を書いた `README.md` |
| `--only-missing` | `FILE` と一緒に：このマシンにないものだけを書き出す |
| `--include <ID>` | この項目も書き出す（複数指定可） |
| `--exclude <ID>` | この項目を除く（複数指定可） |
| `--out <DIR>` | `repot-export-<date>/` を作るフォルダー。既定は現在のフォルダー |
| `--dry-run` | 書き込むファイルの一覧だけを表示し、何も書かない |
| `--json` | 結果を JSON で出力 |

**既定で書き出すもの**は、アプリで最初にチェックされている項目と同じで、書き出し先で使える
ものすべてです。使えない項目は除かれ、理由とともに stderr に一覧表示されます。

```text
note: left out boring-notch: Repot's package map has no Scoop name for it yet
note: left out aodjo/tap: only works on macOS
```

前回の書き出しに含められなかった項目（アプリで*前回は書き出していない*と表示されるもの）も
最初は選択されません。`--include` で追加でき、書き出し先で本当に使えない項目なら `warning:`
行とともにスキップされます。

**id で項目を選ぶ。** id は `scan` と `compare` の出力の右列（または `--json` 出力の `id`）
です。

```sh
repot export --exclude brew:cask:discord --exclude vscode:code:ext:ms-python.python
repot export --to windows --include npm:typescript --dry-run
```

知らない id はエラーになるため、入力ミスで書き出す内容が黙って変わることはありません。

**秘密情報。** トークンやキーに見える行は、書き出したコピーではプレースホルダーに置き換わります。
`repot` は伏せた場所を表示します（`note: masked secrets in .zshrc (lines 123, 124)`）。
新しいマシンでその値を手で入れ直してください。元の値がどうしても必要なら、アプリの「元の値で
書き出す」を使えます。

## スクリプトから使う

- どのコマンドでも `--json` は stdout に JSON を出力します。通知、警告、進行状況は常に stderr
  に出るため、stdout はそのまま解析できます。
  - `scan --json`：`{ "items": [...], "providers": [...] }`。項目の形は `repot.json` と
    同じです。
  - `compare --json`：`{ "<id>": "present" | "differs" | "missing", ... }`。
  - `export --json`：`folder`（`--dry-run` では `null`）、`target`、`selected`（id の一覧）、
    `files`（`path`、`executable`）、`warnings`、`leftOut`（`id`、`reason`）。
- JSON の形は Repot が 0.x の間は変わることがあります。
- 終了コード：`0` 成功（「ないものはない」を含む）、`1` コマンドが処理を完了できなかった
  （stderr に `error: …`）、`2` 使い方の誤り（不明なオプションや値）。

```sh
# Which of the old machine's packages are missing here?
repot compare old/repot.json --json | jq -r 'to_entries[] | select(.value == "missing") | .key'
```

## その他

- `repot --help` と `repot <command> --help` ですべてのオプションを確認できます。
- [Repot README](../../README.ja.md)：Repot が移すもの、既知の制限、アプリ。
- 問題や提案：[GitHub issues](https://github.com/junjuny0227/Repot/issues)。
