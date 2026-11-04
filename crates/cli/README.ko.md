# repot 명령줄 도구

[English](README.md) · 한국어 · [日本語](README.ja.md) · [简体中文](README.zh-Hans.md)

`repot`은 터미널에서 개발 환경을 새 기기로 옮깁니다. [Repot 앱](../../README.ko.md)이 하는
일(스캔, Repot 파일과 비교, 복원 스크립트 내보내기)을 화면이 없는 기기, SSH 접속, 직접 만든
스크립트에서 할 수 있습니다.

앱과 같은 규칙을 따릅니다.

- 이 기기를 **읽기만** 합니다. 쓰는 것은 내보내기 폴더 하나뿐입니다.
- 복원 스크립트를 대신 실행하지 않고, 확인 질문도 하지 않습니다.
- 네트워크를 쓰지 않습니다.
- 설정 파일의 비밀 정보(토큰, 키)는 항상 가리고, 어디를 가렸는지 알려줍니다.

명령 출력은 영어입니다. 이 문서의 출력 예시도 실제 출력 그대로 영어로 둡니다.

## 설치

| | |
| --- | --- |
| macOS, Linux (Homebrew) | `brew install junjuny0227/tap/repot` |
| Windows (Scoop) | `scoop bucket add repot https://github.com/junjuny0227/scoop-bucket` 다음에 `scoop install repot/repot` |
| 모든 OS, 직접 설치 | [Releases](https://github.com/junjuny0227/Repot/releases/latest)에서 `repot-cli-<버전>-<target>.tar.gz` / `.zip`을 받아 `.sha256` 파일로 확인한 뒤 `repot`을 `PATH`에 둡니다 |
| Rust가 있으면 | `cargo install --git https://github.com/junjuny0227/Repot repot-cli` |

macOS에서 브라우저로 받은 압축 파일은 격리되므로 `xattr -d com.apple.quarantine repot`을
한 번 실행합니다. Scoop은 Windows에 기본으로 들어 있지 않습니다. 먼저
[scoop.sh](https://scoop.sh)에서 설치하거나 zip을 쓰세요.

업데이트는 `brew upgrade repot` 또는 `scoop update repot`으로 합니다. 버전은
`repot --version`으로 확인합니다.

## 새 기기로 옮기기

**옛 기기에서**

```sh
repot scan                       # look at what Repot finds
repot export --out ~/Desktop     # writes ~/Desktop/repot-export-<date>/
```

```text
241 items for macOS (Homebrew)
Wrote /Users/you/Desktop/repot-export-2026-11-04
Next: copy the folder to the new machine and run `bash restore.sh`.
```

폴더에는 `restore.sh`(Windows용이면 `restore.ps1`), `repot.json`, 스크립트가 되돌려 놓을
설정 파일이 든 `files/` 폴더가 들어 있습니다. 패키지를 다른 OS의 이름으로 대응했다면 그 목록을
적은 `README.md`도 있습니다. 폴더 전체를 새 기기로 복사하세요(USB, AirDrop, 비공개 저장소 등.
설정 파일이 들어 있으니 공개된 곳에 두지 마세요).

새 기기의 OS가 다르면 `--to`로 지정합니다.

```sh
repot export --to windows --packages scoop --out ~/Desktop
```

**새 기기에서**

내보낸 폴더에 필요한 것이 모두 들어 있습니다. 스크립트를 읽어 본 뒤 실행합니다.

```sh
bash restore.sh            # macOS, Linux
```

```powershell
powershell -ExecutionPolicy Bypass -File restore.ps1    # Windows
```

스크립트는 먼저 필요한 도구(Homebrew, winget, VS Code 등)와 없는 도구를 받을 곳을 보여 주고
`Continue? [y/N]`을 묻습니다(`--yes` / `-Yes`를 주면 묻지 않습니다). 이미 설치된 것은
건너뛰고, 설정 파일을 바꾸기 전에 백업(`<file>.repot-backup-<time>`)을 남기며, 마지막에
**OK**, **Already done**, **Not done**(`brew not found` 같은 이유와 함께), **Failed**로
요약합니다. 다시 실행해도 안전하므로, 없던 도구를 설치한 뒤 다시 실행하면 나머지가 끝납니다.

**없는 것만** (선택)

새 기기에 이미 대부분이 있거나 예전에 설정해 둔 기기라면, 그 기기에도 `repot`을 설치하고 옛
`repot.json`과의 차이만 내보냅니다.

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

- **Already here**: 이미 설치돼 있거나 같은 파일이 이미 있습니다.
- **Different here**: 같은 설정 파일이 있지만 내용이 다릅니다. `--only-missing`으로는
  내보내지 않습니다. 옛 버전으로 되돌리려면 `--include <id>`로 더하세요(스크립트가 지금 파일을
  먼저 백업합니다).
- **Missing**: `--only-missing`이 내보내는 것입니다.
- **Not usable on <OS>**: 다른 OS의 항목이라 이 기기에서 비교할 대상이 없습니다.

`--only-missing`은 *이* 기기와 비교하므로 목적지는 이 기기의 OS여야 합니다. 없는 것이 하나도
없으면 그렇게 알리고, 아무것도 쓰지 않은 채 0으로 끝납니다.

## 명령

### `repot scan`

이 기기에서 찾은 것을 분류별로 보여 주고, 오른쪽에 항목마다 **id**를 붙입니다. 진행 상황
(`brew: 62`)과 실패한 Provider는 stderr로 나가고, 설치되지 않은 도구는 마지막에 나열합니다.

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

Repot 파일(내보내기나 앱에서 만든 `repot.json`)을 이 기기와 비교합니다. 각 묶음의 뜻은 위
**없는 것만**을 보세요.

### `repot export [FILE]`

내보내기 폴더를 만듭니다. `FILE`이 없으면 이 기기를 스캔하고, 있으면 그 Repot 파일에서
내보냅니다(예: 옛 Mac의 `repot.json`으로 Windows 스크립트 만들기).

| 옵션 | 뜻 |
| --- | --- |
| `--to <OS>` | 목적지: `macos`, `windows`, `linux-debian`, `linux-fedora`, `linux-arch`, `linux-other`. 기본값은 이 기기의 OS |
| `--packages <MANAGER>` | 목적지의 패키지 매니저: `brew`, `winget`, `scoop`, `choco`, `apt`, `dnf`, `pacman`. Windows 기본값은 winget |
| `--format script` | 복원 스크립트 하나(기본값) |
| `--format manifest` | 대신 각 도구의 파일: `Brewfile`, winget JSON, `.tool-versions`, `npm-global.txt`, 에디터 확장 목록, 사용법을 적은 `README.md` |
| `--only-missing` | `FILE`과 함께: 이 기기에 없는 것만 내보냄 |
| `--include <ID>` | 이 항목도 내보냄(여러 번 가능) |
| `--exclude <ID>` | 이 항목은 뺌(여러 번 가능) |
| `--out <DIR>` | `repot-export-<date>/`를 만들 폴더. 기본값은 현재 폴더 |
| `--dry-run` | 쓸 파일 목록만 보여 주고 아무것도 쓰지 않음 |
| `--json` | 결과를 JSON으로 출력 |

**기본으로 내보내는 것**은 앱에서 처음 체크된 것과 같습니다. 목적지에서 쓸 수 있는 것은 모두
내보냅니다. 쓸 수 없는 항목은 빠지고, 이유와 함께 stderr에 나열됩니다.

```text
note: left out boring-notch: Repot's package map has no Scoop name for it yet
note: left out aodjo/tap: only works on macOS
```

이전 내보내기에 담지 못한 항목(앱에서 *이전 내보내기에 없음*으로 표시되는 것)도 처음에는
선택되지 않습니다. `--include`로 다시 더할 수 있고, 목적지에서 정말 쓸 수 없는 항목이면
`warning:` 줄과 함께 건너뜁니다.

**id로 항목 고르기.** id는 `scan`과 `compare` 출력의 오른쪽 열(또는 `--json` 출력의 `id`)
입니다.

```sh
repot export --exclude brew:cask:discord --exclude vscode:code:ext:ms-python.python
repot export --to windows --include npm:typescript --dry-run
```

모르는 id는 오류로 처리하므로, 오타 때문에 내보낼 내용이 조용히 바뀌는 일은 없습니다.

**비밀 정보.** 토큰이나 키로 보이는 줄은 내보낸 사본에서 자리 표시자로 바뀝니다. `repot`은
가린 위치를 알려 줍니다(`note: masked secrets in .zshrc (lines 123, 124)`). 새 기기에서 그
값을 직접 다시 넣으세요. 원래 값이 꼭 필요하면 앱에서 '원래 값으로 내보내기'를 쓸 수 있습니다.

## 스크립트에서 쓰기

- 모든 명령의 `--json`은 stdout에 JSON을 출력합니다. 알림, 경고, 진행 상황은 항상 stderr로
  나가므로 stdout은 그대로 파싱할 수 있습니다.
  - `scan --json`: `{ "items": [...], "providers": [...] }`. 항목 모양은 `repot.json`과
    같습니다.
  - `compare --json`: `{ "<id>": "present" | "differs" | "missing", ... }`.
  - `export --json`: `folder`(`--dry-run`이면 `null`), `target`, `selected`(id 목록),
    `files`(`path`, `executable`), `warnings`, `leftOut`(`id`, `reason`).
- JSON 모양은 Repot이 0.x인 동안 바뀔 수 있습니다.
- 종료 코드: `0` 성공("없는 것이 없음" 포함), `1` 명령이 일을 끝내지 못함(stderr에
  `error: …`), `2` 사용법 오류(모르는 옵션이나 값).

```sh
# Which of the old machine's packages are missing here?
repot compare old/repot.json --json | jq -r 'to_entries[] | select(.value == "missing") | .key'
```

## 더 보기

- `repot --help`와 `repot <command> --help`가 모든 옵션을 보여 줍니다.
- [Repot README](../../README.ko.md): Repot이 옮기는 것, 알려진 한계, 앱.
- 문제와 제안: [GitHub issues](https://github.com/junjuny0227/Repot/issues).
