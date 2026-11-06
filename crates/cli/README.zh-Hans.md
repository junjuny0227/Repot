# repot 命令行工具

[English](README.md) · [한국어](README.ko.md) · [日本語](README.ja.md) · 简体中文

`repot` 在终端中把开发环境迁移到新电脑。它完成 [Repot 应用](../../README.zh-Hans.md)所做的
事情（扫描、与 Repot 文件比较、导出恢复脚本），适用于没有图形界面的电脑、SSH 连接或你自己的
脚本。

它遵循与应用相同的规则：

- 只**读取**本机。唯一写入的是导出文件夹。
- 不会替你运行恢复脚本，也不会提问确认。
- 不使用网络。
- 配置文件中的敏感信息（令牌、密钥）始终会被遮盖，并告诉你遮盖的位置。

命令输出为英文。本指南中的输出示例也保持实际输出的英文原样。

## 安装

| | |
| --- | --- |
| macOS、Linux（Homebrew） | `brew install junjuny0227/tap/repot` |
| Linux、macOS（安装脚本） | `curl -fsSL https://junjuny0227.github.io/Repot/install.sh \| sh` ([脚本](https://github.com/junjuny0227/Repot/blob/main/apps/site/public/install.sh)) |
| Windows（Scoop） | `scoop bucket add repot https://github.com/junjuny0227/scoop-bucket`，然后 `scoop install repot/repot` |
| 任意系统（手动） | 从 [Releases](https://github.com/junjuny0227/Repot/releases/latest) 下载 `repot-cli-<版本>-<target>.tar.gz` / `.zip`，用 `.sha256` 文件校验后把 `repot` 放到 `PATH` 中 |
| 已安装 Rust | `cargo install --git https://github.com/junjuny0227/Repot repot-cli` |

在 macOS 上用浏览器下载的压缩包会被隔离，请运行一次 `xattr -d com.apple.quarantine repot`。
Scoop 并非 Windows 自带，请先从 [scoop.sh](https://scoop.sh) 安装，或使用 zip。

用 `brew upgrade repot` 或 `scoop update repot` 更新，用 `repot --version` 查看版本。

## 迁移到新电脑

**在旧电脑上**

```sh
repot scan                       # look at what Repot finds
repot export --out ~/Desktop     # writes ~/Desktop/repot-export-<date>/
```

```text
241 items for macOS (Homebrew)
Wrote /Users/you/Desktop/repot-export-2026-11-04
Next: copy the folder to the new machine and run `bash restore.sh`.
```

文件夹中有 `restore.sh`（面向 Windows 时为 `restore.ps1`）、`repot.json`，以及装有脚本要
放回的配置文件的 `files/` 文件夹。如果软件包被映射为其他系统的名称，还会有列出这些映射的
`README.md`。请把整个文件夹复制到新电脑（U 盘、隔空投送、私有仓库等。其中包含你的配置文件，
不要放在公开的地方）。

如果新电脑使用其他系统，用 `--to` 指定：

```sh
repot export --to windows --packages scoop --out ~/Desktop
```

**在新电脑上**

导出的文件夹已包含所需的一切。先阅读脚本，然后运行：

```sh
bash restore.sh            # macOS, Linux
```

```powershell
powershell -ExecutionPolicy Bypass -File restore.ps1    # Windows
```

脚本会先列出需要的工具（Homebrew、winget、VS Code 等）以及缺少的工具从哪里获取，然后询问
`Continue? [y/N]`（加上 `--yes` / `-Yes` 则不询问）。已安装的内容会跳过，替换配置文件前会先
备份（`<file>.repot-backup-<time>`），最后汇总为 **OK**、**Already done**、**Not done**
（附原因，如 `brew not found`）和 **Failed**。重复运行是安全的：安装缺少的工具后再运行一次，
即可完成剩余部分。

**只导出缺少的内容**（可选）

如果新电脑上已经有大部分内容，或者是很早以前设置的电脑，也在那台电脑上安装 `repot`，只导出
与旧 `repot.json` 的差异：

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

- **Already here**：已安装，或相同的文件已存在。
- **Different here**：存在同一个配置文件，但内容不同。`--only-missing` 不会导出它；如果想
  恢复旧版本，用 `--include <id>` 添加（脚本会先备份当前文件）。
- **Missing**：`--only-missing` 导出的内容。
- **Not usable on <OS>**：来自其他系统的项目，在本机没有可比较的对象。

`--only-missing` 与*本*机比较，因此目标系统必须是本机的系统。如果没有缺少的内容，`repot`
会说明这一点，不写入任何文件并以 0 退出。

## 命令

### `repot scan`

按类别列出在本机找到的内容，右侧是每个项目的 **id**。进度（`brew: 62`）和失败的 Provider
输出到 stderr，未安装的工具列在最后。

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

把 Repot 文件（导出或应用生成的 `repot.json`）与本机比较。各组的含义见上面的
**只导出缺少的内容**。

### `repot export [FILE]`

生成导出文件夹。没有 `FILE` 时扫描本机；有 `FILE` 时从该 Repot 文件导出（例如用旧 Mac 的
`repot.json` 生成 Windows 脚本）。

| 选项 | 含义 |
| --- | --- |
| `--to <OS>` | 目标系统：`macos`、`windows`、`linux-debian`、`linux-fedora`、`linux-arch`、`linux-other`。默认为本机系统 |
| `--packages <MANAGER>` | 目标系统的包管理器：`brew`、`winget`、`scoop`、`choco`、`apt`、`dnf`、`pacman`。Windows 默认为 winget |
| `--format script` | 一个恢复脚本（默认） |
| `--format manifest` | 改为各工具自己的文件：`Brewfile`、winget JSON、`.tool-versions`、`npm-global.txt`、编辑器扩展列表，以及说明用法的 `README.md` |
| `--only-missing` | 与 `FILE` 一起使用：只导出本机缺少的内容 |
| `--include <ID>` | 也导出此项目（可重复） |
| `--exclude <ID>` | 排除此项目（可重复） |
| `--out <DIR>` | 创建 `repot-export-<date>/` 的文件夹。默认为当前文件夹 |
| `--dry-run` | 只列出将要写入的文件，不写入任何内容 |
| `--json` | 以 JSON 输出结果 |

**默认导出的内容**与应用中初始勾选的内容相同：目标系统能用的全部内容。不能用的项目会被排除，
并连同原因列在 stderr 中，例如：

```text
note: left out boring-notch: Repot's package map has no Scoop name for it yet
note: left out aodjo/tap: only works on macOS
```

上次导出未能包含的项目（在应用中标为*上次未导出*）初始也不会选中。可以用 `--include` 加回；
如果目标系统确实无法使用，该项目会被跳过并输出 `warning:` 行。

**按 id 选择项目。** id 是 `scan` 和 `compare` 输出的右列（或 `--json` 输出中的 `id`）：

```sh
repot export --exclude brew:cask:discord --exclude vscode:code:ext:ms-python.python
repot export --to windows --include npm:typescript --dry-run
```

未知的 id 会报错，因此不会因为拼写错误而悄悄改变导出内容。

**敏感信息。** 看起来像令牌或密钥的行，在导出的副本中会被替换为占位符。`repot` 会说明遮盖的
位置（`note: masked secrets in .zshrc (lines 123, 124)`）；请在新电脑上手动填回这些值。如果
确实需要原始值，可以使用应用中的“导出原始值”。

## 在脚本中使用

- 所有命令的 `--json` 都在 stdout 输出 JSON。提示、警告和进度始终输出到 stderr，因此 stdout
  可直接解析。
  - `scan --json`：`{ "items": [...], "providers": [...] }`，项目格式与 `repot.json` 相同。
  - `compare --json`：`{ "<id>": "present" | "differs" | "missing", ... }`。
  - `export --json`：`folder`（`--dry-run` 时为 `null`）、`target`、`selected`（id 列表）、
    `files`（`path`、`executable`）、`warnings` 和 `leftOut`（`id`、`reason`）。
- 在 Repot 为 0.x 期间，JSON 格式可能变化。
- 退出码：`0` 成功（包括“没有缺少的内容”），`1` 命令未能完成（stderr 中有 `error: …`），
  `2` 用法错误（未知的选项或值）。

```sh
# Which of the old machine's packages are missing here?
repot compare old/repot.json --json | jq -r 'to_entries[] | select(.value == "missing") | .key'
```

## 更多

- `repot --help` 和 `repot <command> --help` 列出全部选项。
- [Repot README](../../README.zh-Hans.md)：Repot 迁移的内容、已知限制、应用。
- 问题与建议：[GitHub issues](https://github.com/junjuny0227/Repot/issues)。
