# Repot MVP 설계

- 작성일: 2026-11-02
- 상태: 확정 (구현 계획 `docs/plans/2026-11-02-0[1-5]-*.md`에서 바뀐 결정 반영)

## 1. 개요

Repot은 현재 기기의 개발 환경을 스캔해 **무엇이 설치·설정되어 있는지 설명과 함께 보여주고**, 사용자가 고른 항목만 새 기기에서 복원할 수 있는 **복원 파일/스크립트로 내보내는** 크로스플랫폼 데스크톱 앱이다. 이름은 "분갈이(repot)"에서 왔다.

### 목적

1. 만든 사람이 직접 쓰는 도구 (노트북 교체, 새 기기 세팅)
2. 오픈소스로 배포해 다른 개발자도 쓰게 하는 것

### 성공 기준

- macOS, Windows, Linux(Debian/Ubuntu, Fedora, Arch 계열) 각각에서 일반적인 개발 기기를 스캔하면 60초 안에 결과가 나온다.
- 생성된 `restore.sh`는 `shellcheck`, `restore.ps1`은 `PSScriptAnalyzer`를 경고 없이 통과한다.
- 생성된 스크립트를 깨끗한 기기(또는 VM)에서 실행하면, 선택한 항목이 설치·복사되고 실패 항목은 요약에 나온다.
- 기본 설정으로 내보낸 결과물에는 §8의 패턴에 해당하는 비밀정보가 원문으로 남지 않는다.
- 새 도구 지원은 Provider 파일 하나와 fixture 추가만으로 가능하다.

## 2. 범위

### 포함 (MVP)

- 현재 기기 스캔 → 설명과 함께 목록 표시 → 항목 선택 → 내보내기
- 출발 OS는 자동 감지, 목적지 OS는 사용자가 선택
- 내보내기 형식: **통합 스크립트** 또는 **네이티브 매니페스트 묶음** 중 하나
- 스캔 카테고리 A~F (§7)
- 비밀정보 감지·마스킹 (§8)

### 제외 (MVP 이후 검토)

- 앱 안에서 복원 실행, 두 기기 직접 연결
- SSH(키·config), JetBrains IDE
- 온라인 정보 조회, AI 설명
- Flatpak, Snap
- 스캔 결과 저장, 앱 전용 JSON 매니페스트
- 패키지 매니저 간 자동 중복 제거
- brew ↔ winget ↔ apt 같은 OS 간 패키지 이름 매핑
- 코드 서명, 자동 업데이트

## 3. 핵심 원칙

1. **설명한다.** 모든 항목은 그것이 무엇이고 왜 옮길 수 있는지/없는지를 보여준다. 설명이 없으면 "설명 없음"이라고 솔직하게 표시한다.
2. **기기를 바꾸지 않는다.** 앱은 스캔(읽기)과 내보내기 폴더 쓰기만 한다. 시스템에 설치하거나 설정을 바꾸는 일은 사용자가 직접 실행하는 스크립트만 한다.
3. **의심스러우면 보수적으로.** 비밀정보는 기본으로 가리고, 설정 파일을 덮어쓰기 전에는 반드시 백업하며, 패키지 매니저 자체를 대신 설치하지 않는다.
4. **네트워크 통신 없음.** 설명은 로컬 정보와 앱 내장 데이터만 쓴다.
5. **사용자가 실행 전에 검토할 수 있다.** 스크립트는 읽을 수 있는 형태로 만들고, 저장 전에 미리보기를 보여준다.

## 4. 사용자 흐름

1. **시작**: 감지된 현재 OS를 보여주고, 목적지 OS를 고른다(기본값은 현재 OS). Linux는 배포판 계열(Debian/Ubuntu, Fedora, Arch)까지 고른다.
2. **스캔 진행**: Provider별 상태(진행 중, 완료 n개, 감지 안 됨, 실패)를 실시간으로 보여준다.
3. **검토**: 항목을 확인하고 옮길 것만 남긴다. 목적지 OS를 여기서 바꿀 수도 있다.
4. **내보내기**: 형식을 고르고, 생성될 파일 트리와 내용을 미리 본 뒤, 저장 위치를 골라 저장한다.
5. **완료**: 폴더 열기 버튼과 함께 "새 기기에 폴더를 복사한 뒤 이렇게 실행하세요" 안내를 보여준다.

## 5. 아키텍처

### 저장소 구조

```
Repot/
├── Cargo.toml                    # Cargo workspace
├── package.json, pnpm-workspace.yaml
├── crates/
│   └── core/                     # Tauri와 무관한 순수 Rust 라이브러리
│       └── src/
│           ├── model.rs          # Item, Category, Portability, Finding 등
│           ├── os.rs             # Os, LinuxFamily, Shell, 경로 해석
│           ├── env.rs            # 명령 실행·파일 읽기 추상화
│           ├── scan.rs           # Provider 병렬 실행, 타임아웃, 결과 취합
│           ├── availability.rs   # 목적지 OS 기준 항목 사용 가능 여부 판정
│           ├── catalog.rs        # 내장 설명 데이터 (도구·설정 파일 설명, 영어 한 문장)
│           ├── providers/        # 도구별 스캐너, 도구 하나당 파일 하나
│           ├── secrets.rs        # 비밀정보 감지·마스킹
│           └── export/
│               ├── step.rs       # 추상 단계 Step
│               ├── script/       # bash, PowerShell 렌더러
│               └── manifest.rs   # 네이티브 매니페스트 생성
└── apps/
    └── desktop/
        ├── src-tauri/            # Tauri 명령: core를 호출하는 얇은 층
        └── src/                  # React + TypeScript + Vite
```

### 계층 경계

- `core`는 Tauri, UI를 모른다. 단위 테스트와 향후 CLI 재사용이 가능하다.
- `core`는 사람이 읽는 문장을 만들지 않는다. Finding과 판정 이유는 구조화된 데이터로 돌려주고 UI가 문장으로 바꾼다. 예외는 `catalog`의 내장 설명(영어 한 문장)뿐이다.
- `src-tauri`는 `core` 호출, 직렬화, 진행 이벤트 전달만 한다.
- 프론트엔드에는 **shell 권한을 주지 않는다.** 모든 명령 실행은 Rust 쪽에서만 일어난다. 프론트엔드 권한은 저장 위치 선택 대화상자(`core:default`, `dialog:allow-open`)뿐이다.

### Tauri 명령

| 명령 | 입력 | 출력 |
|---|---|---|
| `detect_os` | - | 현재 `Os` |
| `scan` | - | `ScanResult` (진행 상황은 `scan://progress` 이벤트로 전송) |
| `availability_for` | 목적지 `Os` | 항목 id별 `Availability` |
| `preview_export` | `Selection`, 목적지 `Os`, `Format` | 파일 트리와 각 파일 내용 |
| `export` | `Selection`, 목적지 `Os`, `Format`, 저장 위치 | 생성된 폴더 경로 |
| `open_export_folder` | - | 마지막으로 내보낸 폴더를 연다 |

스캔 결과는 Rust 쪽 메모리에만 두고 디스크에 저장하지 않는다. 사용 가능 판정은 Rust가 계산해 넘기며, TypeScript로 중복 구현하지 않는다. 폴더 열기는 프론트엔드 권한이 아니라 Rust 명령이 마지막 내보내기 경로만 연다.

## 6. 도메인 모델

```rust
pub enum Os { MacOs, Windows, Linux(LinuxFamily) }
pub enum LinuxFamily { Debian, Fedora, Arch, Other }
pub enum Shell { Zsh, Bash, Fish, PowerShell }

pub enum Category { PackageManager, Runtime, GlobalPackage, Editor, Git, Shell }

pub struct Item {
    pub id: String,                 // 예: "brew:formula:ripgrep"
    pub provider: String,
    pub category: Category,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub origin: Origin,             // Command(String) | Path(PathBuf): 어떤 명령/경로에서 얻었는지
    pub portability: Portability,
    pub details: ItemDetails,       // 종류별 정보. 복원에 필요한 정보는 모두 여기에 담는다
    pub findings: Vec<Finding>,
}

pub enum ItemDetails {
    Package { manager: PackageManager, kind: PackageKind },          // Formula, Cask, Tap, Package
    Runtime { tool: RuntimeTool, runtime: String, is_default: bool }, // runtime: "node", "python" …
    GlobalPackage { manager: GlobalManager },
    Extension { editor: Editor },
    ConfigFile { source: PathBuf, target: TargetPath },
    ShellPlugin { url: String, target: TargetPath },                  // oh-my-zsh 커스텀 플러그인·테마
}

pub enum Portability {
    Any,                  // 어떤 OS로든
    SameOs,               // 같은 OS(Linux는 같은 배포판 계열)로만
    SameShell(Shell),     // 목적지 OS에서 그 셸을 쓸 수 있을 때만
    Mapped(RuntimeTool),  // 목적지 OS에 대응 도구가 있을 때만 (런타임, §9)
}

pub struct Finding {
    pub line: Option<u32>,          // 1부터
    pub kind: FindingKind,
}

pub enum FindingKind {              // 문장은 UI가 만든다 (§5 계층 경계)
    Secret { name: Option<String> },
    OsSpecificLine { key: String }, // 예: "credential.helper"
    Duplicate { other_provider: String },
    Unreadable,
}

pub struct Selection {
    pub items: Vec<String>,                       // 체크한 Item id
    pub reveal_secret_lines: Vec<(String, u32)>,  // 사용자가 원본 내보내기를 허용한 줄
}
```

공개 모델 타입은 serde로 camelCase 직렬화해 프론트엔드에 그대로 넘긴다.

### Provider 인터페이스

```rust
pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn category(&self) -> Category;
    fn detect(&self, env: &dyn Env) -> bool;
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError>;
}
```

Provider는 스캔만 한다. 내보내기는 `ItemDetails`를 해석하는 중앙 `export` 모듈(§10)이 맡는다. Provider마다 내보내기를 구현하면 렌더러별 규칙(따옴표 처리, 백업, 순서)이 흩어지기 때문이다.

`Env`는 `run(cmd, args, timeout)`, `read_file`, `list_dir`, `home_dir`, `os`와, winget·sdkman 스캔에 필요한 `temp_file`, `remove_file`, `read_link`를 제공한다. 실제 구현(`RealEnv`)과 fixture를 돌려주는 테스트 구현(`FakeEnv`)이 있다.

## 7. 스캔

### Provider 목록

| 카테고리 | Provider | 스캔 방법 | 설명 출처 | 이식성 |
|---|---|---|---|---|
| A 패키지 | `brew` | `brew leaves --installed-on-request`, `brew list --cask`, `brew tap` | `brew info --json=v2 --installed`의 `desc` | SameOs |
| | `winget` | `winget export` (JSON) | 없음 | SameOs |
| | `scoop` | `scoop export` | 로컬 bucket 매니페스트 `description` | SameOs |
| | `choco` | `choco list` (v2, 로컬) + `lib/<pkg>/<pkg>.nuspec` | nuspec `summary`/`description` | SameOs |
| | `apt` | `apt-mark showmanual` | `dpkg-query`의 Description | SameOs |
| | `dnf` | `dnf repoquery --userinstalled` | `dnf info` 로컬 결과 | SameOs |
| | `pacman` | `pacman -Qe` | `pacman -Qi` | SameOs |
| B 런타임 | `nvm` | `~/.nvm/versions/node/` 폴더와 `alias/default` 읽기 | 내장 | Mapped |
| | `nvm-windows` | `nvm list` | 내장 | Mapped |
| | `fnm`, `volta` | `fnm list`, `volta list --format plain` | 내장 | Mapped |
| | `pyenv` (pyenv-win 포함) | `pyenv versions --bare`, `pyenv global` | 내장 | Mapped |
| | `rustup` | `rustup toolchain list` (툴체인만) | 내장 | Mapped |
| | `sdkman` | `~/.sdkman/candidates/*/` 폴더 읽기 | 내장 | Mapped |
| C 전역 패키지 | `npm` | `npm ls -g --depth=0 --json` (`npm`, `corepack` 제외) | 각 패키지 `package.json` | Any |
| | `pnpm` | `pnpm ls -g --json` | 각 패키지 `package.json` | Any |
| | `pipx` | `pipx list --json` | 패키지 METADATA `Summary` | Any |
| | `cargo` | `cargo install --list` | 없음 | Any |
| D 에디터 | `vscode` (VS Code, Cursor 공용) | 확장 폴더 `extensions.json`, User 폴더 `settings.json`·`keybindings.json`·`snippets/` | 확장 `package.json`, 설정 파일은 내장 | Any |
| E Git | `git` | `~/.gitconfig`, `~/.config/git/config`, `core.excludesFile`가 가리키는 파일 | 내장 | Any |
| F 셸 | `shell` | `.zshrc`, `.zprofile`, `.bashrc`, `.bash_profile`, `.profile`, `config.fish`, PowerShell `$PROFILE`, oh-my-zsh 커스텀 플러그인·테마, `starship.toml` | 내장 | SameShell (`starship.toml`은 Any) |

### 항목 단위

- 패키지, 런타임 버전, 확장은 하나가 한 항목이다.
- 설정 파일은 파일 하나가 한 항목이고, 그 안의 비밀정보·OS 전용 줄은 줄 번호가 있는 `Finding`으로 붙는다.
- **직접 설치한 최상위 패키지만** 보여준다. 의존성으로 설치된 패키지는 숨긴다.
  - choco는 각 nuspec의 의존성을 읽어 다른 패키지의 의존성인 패키지를 제외한다.

### 실행

- Provider는 병렬로 실행하고, Provider마다 60초 타임아웃을 둔다. Provider들은 병렬 실행되며 Windows CI에서 choco/npm 스캔이 30초 제한에 한 번 걸려 이를 늘렸다.
- `detect`가 false인 Provider는 "감지되지 않은 도구" 목록에 들어간다.
- 한 Provider의 실패는 다른 Provider에 영향을 주지 않는다.
- 스캔이 끝나면 패키지 매니저 사이에 이름이 같은 항목(대소문자 무시)에 `Duplicate` Finding을 붙인다. 예: "choco로도 감지됨". 자동으로 제거하지는 않는다.
- 비밀정보 검사는 D·E·F의 설정 파일 항목에만 적용한다.

### 알려진 함정과 대응

- **macOS GUI 앱의 PATH**: Finder에서 실행하면 셸 PATH를 물려받지 못한다. 앱 시작 시 `$SHELL -ilc 'printf %s "$PATH"'`로 PATH를 가져와 `RealEnv`에 적용한다. 실패하면 `/opt/homebrew/bin`, `/usr/local/bin`, `~/.cargo/bin` 등 잘 알려진 경로를 덧붙인다.
- **nvm은 셸 함수**라 실행할 수 없다. 폴더를 직접 읽는다.
- **apt·dnf 목록 잡음**: OS 설치 시 들어간 패키지가 섞인다. apt는 `/var/log/installer/initial-status.gz`가 있으면 그 목록을 빼고, 없으면 그대로 보여준다(알려진 한계).
- **Windows 명령 출력 인코딩**: 가능한 한 JSON 출력 옵션을 쓰고, 텍스트 출력은 UTF-8로 강제하거나 손실 허용 디코딩을 한다.

## 8. 비밀정보 감지

### 대상 패턴

- 잘 알려진 토큰 형식: AWS Access Key(`AKIA…`), GitHub(`ghp_`, `gho_`, `github_pat_`), OpenAI(`sk-…`), Anthropic(`sk-ant-…`), Slack(`xox[abpr]-…`), Google API Key(`AIza…`)
- 이름 기반: `export`/`$env:`/`set -x`로 정의된 변수 중 이름에 `TOKEN`, `SECRET`, `PASSWORD`, `PASSWD`, `API_KEY`, `ACCESS_KEY`, `PRIVATE_KEY`가 들어가고 값이 비어 있지 않은 것
- 자격 증명이 들어간 URL: `scheme://user:pass@host`, `https://<token>@host`
- 개인 키 블록: `-----BEGIN … PRIVATE KEY-----`

### 동작

- 감지된 줄에 `Secret` Finding을 붙이고 UI에서 ⚠️로 표시한다.
- 내보낼 때 **값 부분만** `<REDACTED>`로 바꾼다. 예: `export OPENAI_API_KEY="<REDACTED>"`
- 사용자가 해당 줄의 "원본으로 내보내기"를 켠 경우(`Selection.reveal_secret_lines`)에만 원문을 쓴다.
- 마스킹한 줄 목록(파일, 줄 번호, 변수 이름)을 README 또는 스크립트 실행 요약에 남긴다.
- 값이 `$OTHER_VAR`, `$(command)`처럼 다른 곳을 참조하면 비밀이 아닌 것으로 본다(오탐 방지).

## 9. 목적지 OS별 사용 가능 판정

`availability(item, target) -> Available | Unavailable(reason)`가 판정하고, UI는 사용할 수 없는 항목을 비활성화하며 이유를 표시한다.

| Portability | 판정 |
|---|---|
| Any | 항상 가능 |
| SameOs | 출발 OS와 목적지 OS가 같아야 함. Linux는 배포판 계열까지 같아야 함 |
| SameShell(s) | 목적지 OS에서 셸 s를 쓸 수 있어야 함. zsh·bash·fish는 macOS·Linux만, PowerShell은 모든 OS(`pwsh`) |
| Mapped | 아래 대응표에 목적지 OS용 도구가 있어야 함 |

### 런타임 대응표

| 출발 도구 | macOS / Linux 목적지 | Windows 목적지 |
|---|---|---|
| nvm | nvm | nvm-windows |
| nvm-windows | nvm | nvm-windows |
| fnm, volta | 동일 | 동일 |
| pyenv / pyenv-win | pyenv | pyenv-win |
| rustup | rustup | rustup |
| sdkman | sdkman | 없음 → 사용 불가 |

런타임 항목은 "Node 20.11.0"처럼 버전 단위로 다루며, 내보낼 때 대응 도구의 명령으로 바꾼다.

### OS 간 이동 시 설정 파일 변환

- **Git**: `credential.helper`(`osxkeychain` ↔ `manager` 등), `core.autocrlf`, `gpg.program`, 절대 경로가 든 `core.editor` 같은 OS 전용 줄은 주석 처리하고, 목적지 OS용 대안을 주석으로 단다.
- **VS Code/Cursor**: User 폴더 경로를 목적지 OS 기준으로 바꾼다(macOS `~/Library/Application Support/<App>/User`, Windows `%APPDATA%\<App>\User`, Linux `~/.config/<App>/User`). `keybindings.json`은 `cmd`/`ctrl` 차이에 대한 안내를 남긴다.
- **PowerShell profile**: 목적지 OS의 `$PROFILE` 위치로 바꾼다.

## 10. 내보내기

### 공통

- 결과는 사용자가 고른 위치의 `repot-export-<YYYY-MM-DD>/` 폴더다. 같은 이름의 폴더가 있으면 `-2`, `-3`을 붙인다.
- 임시 폴더에 모두 쓴 뒤 마지막에 이름을 바꾼다. 중간에 실패하면 임시 폴더를 지운다.
- 선택된 항목 중 목적지 OS에서 사용할 수 없는 항목은 내보내지 않는다.
- 스크립트·매니페스트에 들어가는 값(패키지 이름, 런타임 버전, 확장 ID, URL)은 허용 패턴(`^[A-Za-z0-9@._+/:-]+$`)에 맞아야 한다. 패키지·전역 패키지·확장은 최신 버전으로 설치하므로 스캔한 버전은 검사하지 않는다(예: brew cask 버전의 쉼표). 맞지 않으면 내보내지 않고 경고한다. 렌더러는 모든 값을 셸 규칙에 맞게 따옴표로 감싼다.
- 설정 파일 내용은 스크립트에 넣지 않고 `files/` 아래에만 둔다.
- 생성 파일(스크립트 출력 문구, README)은 영어로 쓴다. 오픈소스 배포물이라 UI 언어와 별개로 통일한다.
- 스크립트 종료 코드는 실패한 단계가 하나라도 있으면 1, 아니면 0이다.

### Step

셸 코드 대신 추상 단계를 쓰고, 렌더러가 목적지 셸에 맞게 바꾼다.

```rust
pub enum Step {
    AddTap { manager: PackageManager, name: String },     // brew tap, scoop bucket
    InstallPackage { manager: PackageManager, kind: PackageKind, name: String },
    InstallRuntime { tool: RuntimeTool, runtime: String, version: String, set_default: bool },
    InstallGlobal { manager: GlobalManager, name: String },
    InstallExtension { editor: Editor, id: String },
    CopyFile { src: String /* files/ 기준 상대 경로, '/' 구분 */, dest: TargetPath },
    GitClone { url: String, dest: TargetPath },           // oh-my-zsh 커스텀 플러그인
}
```

`export` 모듈이 `ItemDetails`를 목적지 OS 기준 `Step`으로 바꾼다. 런타임은 이때 §9 대응표로 대응 도구를 고르고, 대응 도구가 없으면 단계를 만들지 않는다.

`TargetPath`는 `Home(rel)`, `EditorUser { editor, rel }`, `PowerShellProfile` 같은 논리 경로이고, 렌더러가 목적지 OS 기준으로 해석한다. 두 항목이 같은 대상 경로를 가지면(예: PowerShell 7과 5.1 profile) 첫 항목만 내보내고 경고한다.

### ① 통합 스크립트

```
repot-export-2026-11-02/
├── restore.sh        # macOS/Linux 목적지 (Windows 목적지면 restore.ps1)
└── files/
```

- **bash 3.2 호환**: macOS 기본 bash가 3.2이므로 연관 배열 같은 bash 4 이상 기능을 쓰지 않는다.
- **PowerShell 5.1 호환**: Windows 기본 PowerShell에서 동작해야 한다.
- 실행 순서:
  1. 머리말: Repot 버전, 출발·목적지 OS, 생성 시각
  2. 사전 점검: 목적지 OS 확인, 관리자 권한이 필요한 항목(choco 등)이 있으면 권한 확인
  3. 실행 계획 요약 출력 후 `계속할까요? [y/N]` (`--yes`/`-Yes`로 생략)
  4. 패키지 매니저 → 런타임 → 전역 패키지 → 에디터 확장 → 설정 파일 순서로 실행
  5. 결과 요약: 성공, 건너뜀, 실패 목록, 직접 채워야 할 마스킹 항목
- 이미 설치된 항목은 건너뛴다(여러 번 실행해도 안전).
- 한 단계가 실패해도 계속 진행하고 요약에 기록한다.
- 패키지 매니저나 버전 관리자 자체가 없으면 설치하지 않는다. 공식 설치 방법을 안내하고 그 섹션을 건너뛴다. oh-my-zsh도 같다.
- 설정 파일을 복사할 때 대상 파일이 이미 있으면 `<파일>.repot-backup-<YYYYMMDDHHMMSS>`로 백업한 뒤 덮어쓴다.
- 런타임은 스캔한 정확한 버전으로, 전역 패키지·확장은 최신 버전으로 설치한다.

### ② 네이티브 매니페스트 묶음

```
repot-export-2026-11-02/
├── README.md               # 목적지 OS 기준 파일별 복원 명령, 마스킹 목록
├── Brewfile                # brew bundle --file=Brewfile
├── winget.json             # winget import -i winget.json
├── scoop.json              # scoop import scoop.json
├── choco-packages.config   # choco install choco-packages.config
├── apt-packages.txt        # dnf-packages.txt, pacman-packages.txt
├── .tool-versions          # 런타임 버전 (mise/asdf 형식)
├── npm-global.txt
├── pnpm-global.txt
├── pipx.txt
├── cargo.txt
├── vscode-extensions.txt
├── cursor-extensions.txt
└── files/                  # 설정 파일, README에 복사할 위치 안내
```

- 선택된 항목이 있는 파일만 만든다.

## 11. UI

- 스택: React + TypeScript + Vite. 상태는 React 내장 기능(`useReducer`, Context)으로 시작한다. UI 라이브러리(컴포넌트 킷, 상태 관리 라이브러리)는 쓰지 않는다.
- UI 문구는 `src/strings.ts` 한 파일에 모은다. MVP는 영어이고, 다른 언어는 이 파일만 바꾸면 되게 한다.
- 화면 4개: 시작, 스캔 진행, 검토, 내보내기(+완료).
- 디자인: macOS 기본 앱(시스템 설정·Finder) 스타일. 시스템 색·글꼴·컨트롤 모양을 따르고 다크 모드를 지원한다. macOS에서는 제목 표시줄을 콘텐츠와 합치고(overlay) 사이드바에 시스템 반투명 재질을 쓴다(`tauri.macos.conf.json`). Windows·Linux는 같은 배치를 불투명 배경과 기본 제목 표시줄로 보여준다. 투명도 줄이기 설정에서도 불투명으로 바뀐다.

### 검토 화면

- 왼쪽: 사이드바. 카테고리 목록과 항목 수, 각 줄 끝의 카테고리 전체 선택/해제 체크박스
- 가운데: 항목 목록. 체크박스, 이름, 버전, 설명 한 줄, 배지(비밀정보·중복·목적지 OS에서 사용 불가를 SF 스타일 아이콘으로 표시하고, 각각 접근성 이름을 단다). 검색과 필터("사용 가능만", "경고 있는 것만")
- 오른쪽: 상세 패널. 설명, 원본 위치, 이식성 판정 이유, Finding 목록, 비밀정보 줄별 "원본으로 내보내기" 토글
- 기본 체크 상태: 목적지 OS에서 사용 가능한 항목은 모두 체크된 상태로 시작한다.
- 상단에서 목적지 OS를 바꾸면 사용 가능 여부를 즉시 다시 판정한다. 사용 불가가 된 항목의 체크는 해제된다.
- "감지되지 않은 도구"와 "스캔 실패" 목록은 사이드바 아래쪽에 접힌 상태로 보여준다.

### 내보내기 화면

- 형식 선택(통합 스크립트 / 네이티브 매니페스트)은 세그먼트 컨트롤로 한다.
- 미리보기: 생성될 파일 트리, 파일을 누르면 줄 번호가 붙은 고정폭 글꼴로 내용 표시
- 저장 위치 선택 후 저장 → 완료 화면(폴더 열기, 새 기기에서 실행하는 방법 안내)

## 12. 오류 처리

| 상황 | 처리 |
|---|---|
| 도구 없음 | "감지되지 않은 도구"로 분류, 오류 아님 |
| 명령 실행 실패 / 0이 아닌 종료 코드 | Provider를 "스캔 실패"로 표시. 사람이 읽을 수 있는 문장 + "자세히"에서 stderr 원문 |
| 출력 해석 실패 | 같은 방식. 가능한 만큼만 해석한 항목은 보여주지 않는다(부분 결과로 오해 방지) |
| 시간 초과 | "시간 초과(60초)"로 표시 |
| 파일 읽기 권한 없음 | 항목은 보여주되 `Unreadable` Finding을 붙이고 내보내기에서 제외 |
| 내보내기 중 쓰기 실패 | 임시 폴더 정리 후 오류 표시. 기존 파일은 건드리지 않음 |

## 13. 테스트

- **Provider 단위 테스트**: OS별로 녹화한 명령 출력과 파일을 fixture로 두고 `FakeEnv`로 실행한다. 어떤 OS에서든 모든 Provider 테스트를 돌릴 수 있다.
- **비밀정보**: 감지해야 하는 사례와 감지하면 안 되는 사례(변수 참조, 주석 속 설명, 빈 값 등)를 표 기반으로 테스트한다.
- **사용 가능 판정**: Portability × 목적지 OS 조합을 표 기반으로 테스트한다.
- **렌더러**: `restore.sh`, `restore.ps1`, 각 매니페스트를 `insta` 스냅샷으로 고정한다.
- **CI (GitHub Actions)**:
  - macOS, Windows, Ubuntu 러너에서 `cargo test`, `clippy`, `fmt`
  - 각 러너에서 실제 스캔을 실행하는 통합 테스트(러너에 brew, choco, apt 등이 설치되어 있음)
  - 생성된 스크립트를 `shellcheck`, `PSScriptAnalyzer`로 검사
  - `restore-e2e`: 새 macOS·Ubuntu·Windows 호스티드 러너에서 실제 `build_export`/`write_export`로 만든 스크립트를 실행한다. 사전 미설치 확인 후 brew/apt의 `sl`, choco의 `dos2unix`, 격리된 npm prefix의 `is-odd` 설치와 설정 파일 복사를 검증한다. 존재하지 않는 패키지 하나만 실패 목록에 남고 종료 코드가 1인지 확인하며, 두 번째 실행에서는 성공했던 세 항목이 모두 건너뜀 목록에 남아야 한다.
- **프론트엔드**: 선택 상태, 목적지 OS 변경 시 비활성화 로직을 Vitest로 테스트한다.

## 14. 배포

- `tauri-action`으로 macOS(`.dmg`, Apple silicon·Intel), Windows(`.msi`/설치 파일), Linux(`.AppImage`, `.deb`)를 빌드해 GitHub Releases에 올린다.
- 코드 서명이 없으므로 README에 첫 실행 방법(macOS "확인 없이 열기", Windows "추가 정보 → 실행")을 안내한다.
- 라이선스: Apache-2.0 (저장소의 `LICENSE`)

## 15. 알려진 한계

- apt·dnf의 "직접 설치" 목록에 기본 시스템 패키지가 섞일 수 있다.
- winget, cargo 항목은 로컬에 설명이 없어 "설명 없음"으로 표시된다.
- 패키지 매니저 간 중복은 이름이 정확히 같을 때만 감지한다.
- 비밀정보 감지는 패턴 기반이라 모든 비밀을 잡지는 못한다. UI와 README에 이 점을 명시한다.
- WSL, Git Bash 환경은 고려하지 않는다.
- winget은 `winget` 소스 패키지만 다룬다(msstore 제외).
- volta는 런타임만 다룬다(volta로 설치한 전역 패키지 제외).
- rustup은 툴체인만 다룬다(component·target 제외).
- scoop은 기본 bucket 이외의 사용자 지정 bucket URL을 보존하지 않는다.

## 16. 향후 방향

- 앱 전용 JSON 매니페스트와 "새 기기에서 앱으로 복원" 기능 (`Step` 모델을 그대로 재사용)
- SSH(암호화 내보내기 포함), JetBrains, Flatpak, Snap Provider
- 사용자 동의 하의 온라인 설명 보강
- `core`를 재사용하는 CLI (`cargo install repot`)
