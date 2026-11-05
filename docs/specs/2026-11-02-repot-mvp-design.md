# Repot MVP 설계

- 작성일: 2026-11-02
- 상태: 확정 (구현 계획 `docs/plans/2026-11-02-0[1-5]-*.md`, 계획 6·7(`docs/plans/2026-11-03-0[6-7]-*.md`)에서 바뀐 결정 반영)

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
- 공식 Tauri updater를 통한 서명된 앱 자동 업데이트 (계획 8)
- 앱에 내장한 대응표를 통한 OS 간 패키지 이름 대응. Windows 목적지는 winget·Scoop·Chocolatey 중 고른다 (계획 9, §9)
- 내보내기 폴더에 Repot이 다시 읽는 파일 `repot.json`을 함께 쓰고, 시작 화면에서 열어 다시 검토·내보내기 (계획 11, §10)
- 열린 `repot.json`을 이 기기 스캔과 비교해 "이미 있음 / 내용 다름 / 없음"을 보여주고 "없음"만 처음 선택 (계획 12)
- `repot.json`에 목적지에서 쓸 수 없어 빠진 항목도 "이번 내보내기에 없음" 표시와 함께 저장해, 다른 목적지로 다시 내보낼 수 있게 함 (계획 13)
- 명령줄 도구 `repot`(`scan`, `compare`, `export`). 앱과 같은 core·세션·처음 선택을 쓰고, 출력은 영어와 `--json` (계획 15)
- JetBrains IDE의 설정(코드 스타일, 키맵, 색 구성표, 라이브 템플릿, 글꼴·테마 선택)과 사용자가 설치한 플러그인 (계획 14)

### 제외 (MVP 이후 검토)

- 앱 안에서 복원 실행, 두 기기 직접 연결
- SSH(키·config)
- 온라인 정보 조회, AI 설명
- Flatpak, Snap
- 스캔 결과 전체 저장 (`repot.json`은 고른 항목과 목적지에서 쓸 수 없던 항목만 담는다. 사용자가 뺀 항목은 담지 않는다)
- 패키지 매니저 간 자동 중복 제거
- 이름 유사도나 온라인 조회로 하는 패키지 대응 (대응은 내장 표에 적힌 것만 쓴다)
- Apple/Windows 코드 서명·공증 (업데이트 자산 서명과는 별개)

## 3. 핵심 원칙

1. **설명한다.** 모든 항목은 그것이 무엇이고 왜 옮길 수 있는지/없는지를 보여준다. 설명이 없으면 "설명 없음"이라고 솔직하게 표시한다.
2. **기기를 바꾸지 않는다.** 앱은 스캔(읽기)과 내보내기 폴더 쓰기만 한다. 예외는 사용자가 동의한 앱 자체 업데이트 설치뿐이다. 시스템에 설치하거나 설정을 바꾸는 일은 사용자가 직접 실행하는 스크립트만 한다.
3. **의심스러우면 보수적으로.** 비밀정보는 기본으로 가리고, 설정 파일을 덮어쓰기 전에는 반드시 백업하며, 패키지 매니저 자체를 대신 설치하지 않는다.
4. **업데이트 외 네트워크 통신 없음.** 앱의 네트워크 기능은 GitHub의 업데이트 확인·다운로드 한 종류이며 자동 확인은 설정에서 끌 수 있다. 설명과 패키지 대응은 로컬 정보와 앱 내장 데이터만 쓰고 스캔 데이터는 기기를 떠나지 않는다.
5. **사용자가 실행 전에 검토할 수 있다.** 스크립트는 읽을 수 있는 형태로 만들고, 저장 전에 미리보기를 보여준다.

## 4. 사용자 흐름

1. **시작**: 감지된 현재 OS를 보여주고, 목적지 OS를 고른다(기본값은 현재 OS). Linux는 배포판 계열(Debian/Ubuntu, Fedora, Arch)까지 고른다.
2. **스캔 진행**: Provider별 상태(진행 중, 완료 n개, 감지 안 됨, 실패)를 실시간으로 보여준다.
3. **검토**: 항목을 확인하고 옮길 것만 남긴다. 목적지 OS를 여기서 바꿀 수도 있다.
4. **내보내기**: 형식을 고르고, 생성될 파일 트리와 내용을 미리 본 뒤, 저장 위치를 골라 저장한다.
5. **완료**: 폴더 열기 버튼과 함께 "새 기기에 폴더를 복사한 뒤 이렇게 실행하세요" 안내를 보여준다.

시작 화면에서 스캔 대신 `repot.json`을 열 수도 있다. 그러면 스캔을 건너뛰고 그 파일의 항목으로 3단계(검토)부터 시작한다. 출발 OS는 파일을 만든 기기의 OS다. 검토 화면에서 "이 기기와 비교"를 누르면 이 기기를 스캔해 각 항목을 "이미 있음 / 내용 다름 / 없음"으로 나누고, 없음만 처음 선택한다(내용 다름은 새 기기 설정을 덮어쓰므로 사용자가 직접 켠다). 비교는 목적지가 이 기기 OS일 때만 보인다(계획 12).

## 5. 아키텍처

### 저장소 구조

```
Repot/
├── Cargo.toml                    # Cargo workspace
├── package.json, pnpm-workspace.yaml
├── crates/
│   ├── cli/                      # 명령줄 도구 repot: core를 부르는 얇은 층 (계획 15)
│   └── core/                     # Tauri와 무관한 순수 Rust 라이브러리
│       └── src/
│           ├── model.rs          # Item, Category, Portability, Finding 등
│           ├── os.rs             # Os, LinuxFamily, Shell, 경로 해석
│           ├── env.rs            # 명령 실행·파일 읽기 추상화
│           ├── scan.rs           # Provider 병렬 실행, 타임아웃, 결과 취합
│           ├── availability.rs   # 목적지 OS 기준 항목 사용 가능 여부 판정
│           ├── session.rs        # 스캔 결과나 열린 파일, 미리보기, 처음 선택 (앱·CLI 공용, 계획 15)
│           ├── catalog.rs        # 내장 설명 데이터 (안정된 id + 영어 한 문장)
│           ├── providers/        # 도구별 스캐너, 도구 하나당 파일 하나
│           ├── secrets.rs        # 비밀정보 감지·마스킹
│           └── export/
│               ├── step.rs       # 추상 단계 Step
│               ├── script/       # bash, PowerShell 렌더러
│               └── manifest.rs   # 네이티브 매니페스트 생성
└── apps/
    ├── desktop/
    │   ├── src-tauri/            # Tauri 명령: core를 호출하는 얇은 층
    │   └── src/                  # React + TypeScript + Vite, locales/<locale>.json (계획 7)
    └── site/                     # 정적 제품 사이트 (GitHub Pages, 계획 6), 로캘별 페이지 (계획 7)
```

### 계층 경계

- `core`는 Tauri, UI를 모르며 네트워크 통신을 하지 않는다. 단위 테스트와 향후 CLI 재사용이 가능하다.
- `core`는 사람이 읽는 문장을 만들지 않는다. Finding과 판정 이유는 구조화된 데이터로 돌려주고 UI가 문장으로 바꾼다. 예외는 `catalog`의 내장 설명(영어 한 문장)뿐이다. 내장 설명에는 안정된 id(`runtime:node`, `file:zshrc` …)가 있어 `Item.catalog_id`로 함께 넘기고, UI는 이 id로 번역을 찾는다. 번역이 없으면 core의 영어 문장(`Item.description`)을 그대로 보여준다.
- `crates/cli`는 인자 해석과 영어 출력만 맡는다. 판정·선택·내보내기는 core `session`을 쓰고, 앱과 같이 기기를 바꾸지 않으며 네트워크를 쓰지 않는다.
- `src-tauri`는 `core` 호출, 직렬화, 진행 이벤트 전달과 공식 updater/process 플러그인을 통한 앱 자체 업데이트를 담당한다.
- 프론트엔드에는 **shell 권한을 주지 않는다.** 모든 명령 실행은 Rust 쪽에서만 일어난다. 프론트엔드 권한은 저장 위치 선택 대화상자(`core:default`, `dialog:allow-open`)와 앱 자체 업데이트(`updater:default`, `process:allow-restart`)로 제한한다.

### Tauri 명령

| 명령 | 입력 | 출력 |
|---|---|---|
| `detect_os` | - | 현재 `Os` |
| `scan` | - | `ScanResult` (진행 상황은 `scan://progress` 이벤트로 전송) |
| `availability_for` | 목적지 `Os` | 항목 id별 `Availability` |
| `preview_export` | `Selection`, 목적지 `Os`, `Format` | 파일 트리와 각 파일 내용 |
| `export` | `Selection`, 목적지 `Os`, `Format`, 저장 위치 | 생성된 폴더 경로 |
| `open_export_folder` | - | 마지막으로 내보낸 폴더를 연다 |
| `open_snapshot` | `repot.json` 경로 | 열린 파일의 항목(`ScanResult` 모양), 출발 `Os`, 만든 시각·앱 버전, 건너뛴 항목 수. 실패하면 오류 종류 (계획 11) |
| `scan_machine` | - | 이 기기를 스캔해 현재 세션에 비교 대상으로 넣는다. 세션 항목은 그대로 둔다 (계획 12) |
| `presence_for` | 목적지 `Target` | 항목 id별 `Presence`(이미 있음·내용 다름·없음). 목적지 OS가 이 기기와 다르거나 비교 스캔이 없으면 빈 맵 (계획 12) |

현재 세션(출발 OS, 항목, 설정 파일 내용을 어디서 읽는지)은 Rust 쪽 메모리에 하나로 묶어 둔다. 세션은 새 스캔이나 `repot.json` 열기로 바뀐다. 스캔 결과는 디스크에 따로 저장하지 않고, 내보낼 때 고른 항목과 목적지에서 쓸 수 없던 항목만 내보내기 폴더의 `repot.json`에 남는다(계획 13). 사용 가능 판정은 Rust가 계산해 넘기며, TypeScript로 중복 구현하지 않는다. 폴더 열기는 프론트엔드 권한이 아니라 Rust 명령이 마지막 내보내기 경로만 연다.

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
    pub description: Option<String>,  // 로컬 도구가 준 설명 또는 catalog 영어 문장
    pub catalog_id: Option<String>,   // 설명이 catalog에서 왔을 때 그 id (UI 번역 키)
    pub origin: Origin,             // Command(String) | Path(PathBuf): 어떤 명령/경로에서 얻었는지
    pub portability: Portability,
    pub details: ItemDetails,       // 종류별 정보. 복원에 필요한 정보는 모두 여기에 담는다
    pub findings: Vec<Finding>,
}

pub enum ItemDetails {
    Package { manager: PackageManager, kind: PackageKind,             // Formula, Cask, Tap, Package
              url: Option<String> },                                  // Scoop이 이름으로 모르는 bucket의 주소만
    Runtime { tool: RuntimeTool, runtime: String, is_default: bool,  // runtime: "node", "python" …
              components: Vec<String>, targets: Vec<String> },       // rustup만 채운다(§7), 비면 직렬화하지 않는다
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
    Package,              // 같은 OS면 원래 매니저, 다른 OS면 패키지 대응표 (§9)
}

pub struct Target {                 // 목적지
    pub os: Os,
    pub packages: Option<PackageManager>, // 목적지 패키지 매니저. macOS→Brew, Windows→Winget(기본)·Scoop·Choco, Linux 계열→Apt·Dnf·Pacman, Other→None
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
| | `scoop` | `scoop export` (bucket은 `Name`·`Source`) | 로컬 bucket 매니페스트 `description` | SameOs |
| | `choco` | `choco list` (v2, 로컬) + `lib/<pkg>/<pkg>.nuspec` | nuspec `summary`/`description` | SameOs |
| | `apt` | `apt-mark showmanual` | `dpkg-query`의 Description | SameOs |
| | `dnf` | `dnf repoquery --userinstalled` | `dnf info` 로컬 결과 | SameOs |
| | `pacman` | `pacman -Qe` | `pacman -Qi` | SameOs |
| B 런타임 | `nvm` | `~/.nvm/versions/node/` 폴더와 `alias/default` 읽기 | 내장 | Mapped |
| | `nvm-windows` | `nvm list` | 내장 | Mapped |
| | `fnm`, `volta` | `fnm list`, `volta list --format plain` | 내장 | Mapped |
| | `pyenv` (pyenv-win 포함) | `pyenv versions --bare`, `pyenv global` | 내장 | Mapped |
| | `rustup` | `rustup toolchain list`, 툴체인마다 `rustup component list --installed --toolchain <t>`·`rustup target list --installed --toolchain <t>` | 내장 | Mapped |
| | `sdkman` | `~/.sdkman/candidates/*/` 폴더 읽기 | 내장 | Mapped |
| C 전역 패키지 | `npm` | `npm ls -g --depth=0 --json` (`npm`, `corepack` 제외) | 각 패키지 `package.json` | Any |
| | `pnpm` | `pnpm ls -g --json` | 각 패키지 `package.json` | Any |
| | `pipx` | `pipx list --json` | 패키지 METADATA `Summary` | Any |
| | `cargo` | `cargo install --list` | 없음 | Any |
| D 에디터 | `vscode` (VS Code, Cursor 공용) | 확장 폴더 `extensions.json`, User 폴더 `settings.json`·`keybindings.json`·`snippets/` | 확장 `package.json`, 설정 파일은 내장 | Any |
| | `jetbrains` (IntelliJ IDEA, WebStorm, PyCharm 등, 계획 14) | IDE별 최신 버전 설정 폴더의 `codestyles/`·`keymaps/`·`colors/`·`templates/`와 글꼴·테마 선택 파일, 플러그인 폴더의 jar 속 `META-INF/plugin.xml` | plugin.xml `<description>`, 설정 파일은 내장 | Any |
| E Git | `git` | `~/.gitconfig`, `~/.config/git/config`, `core.excludesFile`가 가리키는 파일 | 내장 | Any |
| F 셸 | `shell` | `.zshrc`, `.zprofile`, `.bashrc`, `.bash_profile`, `.profile`, `config.fish`, PowerShell `$PROFILE`, oh-my-zsh 커스텀 플러그인·테마, `starship.toml` | 내장 | SameShell (`starship.toml`은 Any) |

### 항목 단위

- 패키지, 런타임 버전, 확장은 하나가 한 항목이다.
- rustup component·target은 따로 항목을 만들지 않고 툴체인 항목의 `components`·`targets`에 담는다. 툴체인 없이 component만 옮기는 일은 없고, 목록이 길어지지 않게 하기 위해서다.
  - 호스트 전용 component는 rustup이 `clippy-aarch64-apple-darwin`처럼 툴체인 호스트를 붙여 출력하므로 그 접미사를 떼고 저장한다(목적지 호스트가 다를 수 있다).
  - `rust-std-<target>`은 `rustup target list --installed`와 같은 내용이라 component에서 빼고 target으로만 담는다. 툴체인 호스트 target은 어느 툴체인에나 딸려 오고 다른 OS에서는 교차 컴파일 target이 되므로 뺀다.
  - 기본 profile에 든 component(rustc, cargo, rust-docs …)는 rustup 출력만으로 사용자가 더한 것과 구별할 수 없어 그대로 둔다. 복원할 때 이미 있으면 `component add`가 확인만 한다.
  - 이름은 `^[a-z0-9][a-z0-9._-]{0,63}$`에 맞는 것만 받는다(`thumbv8m.main-none-eabi` 같은 target에 점이 있다). 목록 명령이 실패하면 그 툴체인은 component·target 없이 담는다.
- 설정 파일은 파일 하나가 한 항목이고, 그 안의 비밀정보·OS 전용 줄은 줄 번호가 있는 `Finding`으로 붙는다.
- **직접 설치한 최상위 패키지만** 보여준다. 의존성으로 설치된 패키지는 숨긴다.
  - choco는 각 nuspec의 의존성을 읽어 다른 패키지의 의존성인 패키지를 제외한다.

### Scoop bucket

- `main`은 Scoop에 늘 있으므로 항목으로 만들지 않는다.
- Scoop이 이름만으로 추가할 수 있는 알려진 bucket(Scoop `buckets.json`: `main`, `extras`, `versions`, `nirsoft`, `sysinternals`, `php`, `nerd-fonts`, `nonportable`, `java`, `games`, 대소문자 무시)은 이름만 저장한다. 공식 주소를 저장할 이유가 없고, 복원도 `scoop bucket add <name>`이 공식 주소를 쓴다.
- 그 밖의 bucket은 `scoop export`의 `Source`를 `url`에 그대로 저장한다. 비어 있거나 없으면 `None`이다. 주소 검사는 내보내기와 `repot.json` 열기에서 한다(§10). 스캔에서 버리면 이름만으로 내보내져 복원 때 "알 수 없는 bucket"으로 실패하기 때문이다.

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

`availability(item, source, target: Target) -> Available | Mapped { manager, name } | Unavailable(reason)`가 판정하고, UI는 사용할 수 없는 항목을 비활성화하며 이유를 표시한다. `Mapped`는 다른 OS의 매니저 이름으로 바뀌어 사용할 수 있다는 뜻이다.

| Portability | 판정 |
|---|---|
| Any | 항상 가능 |
| SameOs | 출발 OS와 목적지 OS가 같아야 함. Linux는 배포판 계열까지 같아야 함 |
| SameShell(s) | 목적지 OS에서 셸 s를 쓸 수 있어야 함. zsh·bash·fish는 macOS·Linux만, PowerShell은 모든 OS(`pwsh`) |
| Mapped | 아래 대응표에 목적지 OS용 도구가 있어야 함 |
| Package | 같은 OS(Linux는 같은 계열)면 원래 매니저로 가능. 다른 OS면 아래 패키지 대응표에 목적지 매니저 이름이 있어야 함. tap·bucket 항목은 같은 OS만 |

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

### 패키지 대응표 (계획 9)

- 데이터는 `crates/core/data/package-map.json`에 두고 빌드에 포함한다. 공통 id마다 매니저별 이름(`brew`, `brewCask`, `winget`, `scoop`, `choco`, `apt`, `dnf`, `pacman`)을 적는다. `null`은 그 매니저에 없음을 확인했다는 뜻이고, 키가 없으면 아직 모른다는 뜻이다.
- 목적지 매니저는 `Target.packages`다. Windows 목적지에서만 사용자가 winget(기본)·Scoop·Chocolatey 중 고르고, 출발도 Windows면 고르지 않는다. Linux는 계열이 정하고, `Other`는 매니저가 없어 다른 OS 패키지를 받지 않는다. Linux brew는 목적지로 쓰지 않는다.
- 같은 OS 이동은 대응표를 쓰지 않는다. 대응은 원래 매니저를 쓸 수 없을 때만 쓴다.
- 대응 이름은 목적지 매니저의 기본 소스에 있어야 한다. Scoop은 공식 bucket까지 허용하고, 별도 apt 저장소·COPR·AUR·사용자 tap에만 있는 패키지는 대응하지 않는다.
- 이름 유사도로 추측하지 않는다. 표에 없으면 `NotInPackageMap`, `null`이면 `NoPackageInManager`, 목적지에 매니저가 없으면 `NoPackageManager`로 사용 불가다.
- CI가 대응표의 이름을 각 러너의 실제 매니저로 확인한다(대응표가 바뀐 PR과 주 1회).

### 패키지 대응표 생성기 (계획 10)

개발자가 로컬에서 `pnpm package-map:gen`으로 대응표 확장안을 만든다. Node.js 22 표준 라이브러리만 쓰는 별도 도구(`scripts/package-map-gen/`)이며 앱이나 CI에서 생성·AI 판정을 실행하지 않는다. CI에서는 네트워크 없는 단위 테스트와 기존 결과 데이터 검증만 한다. 출발은 Homebrew formula/cask, 목적지는 winget·Scoop 공식 bucket뿐이다. 설치 통계(30일), 제공된 스캔 JSON의 brew 항목, 명시 이름에서 입력을 고르고, 공식 카탈로그를 로컬 캐시한다. `--offline`은 캐시만 쓰고 네트워크에 접근하지 않는다. 앱에는 결과 대응표만 포함하며 core의 네트워크 금지와 기존 데이터 형식은 바뀌지 않는다.

후보는 같은 정규화 source repo·homepage/download URL·정확한 정규화 이름, 그리고 정확한 id 규칙 두 개(Nerd Font cask → Scoop `nerd-fonts/<X>-NF`, `temurin@N`·`zulu@N` → Scoop `java/…<N>-jdk`·winget `….<N>.JDK`, 카탈로그에 그 id가 있을 때만, MEDIUM)로만 만들고 최대 12개를 유지한다. repo 유일성과 명확한 kind 호환을 충족한 HIGH 후보는 jeb를 생략한다. 채널이 다른 후보는 제거하되 CLI/GUI kind 불일치는 표시만 남긴다. 나머지는 AI Hub jeb가 고른 후보의 확률이 기본 0.95 이상이고 kind 표시가 없을 때만 auto로 분류하며, 저확률·kind 불일치 선택·none은 파일 검토로 보낸다. `JEB_AI_HUB_KEY`는 환경변수 우선, 없을 때만 루트 `.env`에서 읽고 출력·커밋하지 않는다. 동시 최대 2요청, 일일 1000요청 할당량, 요청 상한과 잔여 credits floor(기본 100), 503 backoff, 요청 해시 캐시·checkpoint로 비용과 재개를 관리한다.

첫 데이터의 배포 전 spot-check에서 기존 positive 106쌍·인공 negative 30쌍에 없던 포트·대체품·변형 패키지 오선택이 0.80 이상 0.95 미만에서 발견되어 사용자 결정으로 기본값을 0.95로 올렸다. 구체적으로 `caffeine` → `ZhornSoftware.Caffeine`(0.83), `chatgpt` → `extras/chatgpt`(0.81), `libomp` → `LLVM.LLVM`(0.86), `openjdk` → `Microsoft.OpenJDK.17`(0.91), `font-jetbrains-mono` → `nerd-fonts/JetBrains-Mono`(0.93), `nvm` → `main/nvm`(0.94)이다. 이 구간의 선택은 사람이 검토하며, 0.95 이상을 확인한 표본에서 문제가 없었다는 관측도 전체 정확도 보증은 아니다.

기존 값(문자열·`null`)은 덮어쓰지 않고 §10 이름 패턴·대소문자 무시 유일성·알파벳 키 정렬을 지킨다. 새 id는 brew token 소문자에서 `+`를 `plus`로, `[a-z0-9]` 밖 문자의 연속을 `-`로 바꾸고 앞뒤 `-`를 지운 값(`openssl@3` → `openssl-3`, `logi-options+` → `logi-options-plus`)이며 비거나 충돌하면 건너뛴다. 정확도를 측정하지 않은 choco·apt·dnf·pacman 키는 추가하지 않는다. 후보 없음·모델 none·오류로 `null`을 자동 생성하지 않는다. gitignored `.cache/package-map-gen/review.json`의 빈 `decision`에 검토자가 후보 id / `none` / `skip`을 쓰고 `apply-review`로 적용한다. 검토자는 사람이 기본이고, 사용자가 맡기면 에이전트가 계획 10의 규칙으로 결정한 뒤 `apply-review --reviewer agent`로 적용한다. 에이전트는 같은 소프트웨어·같은 벤더/upstream일 때만 후보를 고르고, 포트·대체품·변형은 `skip`, 공식 Windows 버전이 아예 없을 때만 `none`을 쓴다. 검토자의 `none`만 확인된 부재로 `null`이 된다. 추가 값의 방법(`high-tier | jeb | human | agent-review`, 사람과 에이전트 결정을 구분), 항목별 선택적 `evidence` 문자열, 확률(모델 확률이 없으면 `null`), tier·snapshot 날짜는 `crates/core/data/package-map.provenance.json`에 별도 기록하며 앱에 내장하지 않는다. 생성 결과는 데이터 PR과 기존 CI `package-map` 검증을 거친다. 상세 결정·실험 근거·구현 순서는 `docs/plans/2026-11-03-10-package-map-generator.md`를 따른다.

### OS 간 이동 시 설정 파일 변환

- **Git**: `credential.helper`(`osxkeychain` ↔ `manager` 등), `core.autocrlf`, `gpg.program`, 절대 경로가 든 `core.editor` 같은 OS 전용 줄은 주석 처리하고, 목적지 OS용 대안을 주석으로 단다.
- **VS Code/Cursor**: User 폴더 경로를 목적지 OS 기준으로 바꾼다(macOS `~/Library/Application Support/<App>/User`, Windows `%APPDATA%\<App>\User`, Linux `~/.config/<App>/User`). `keybindings.json`은 `cmd`/`ctrl` 차이에 대한 안내를 남긴다.
- **PowerShell profile**: 목적지 OS의 `$PROFILE` 위치로 바꾼다.
- **JetBrains**: 설정 폴더 기준 위치(macOS `~/Library/Application Support/JetBrains`, Windows `%APPDATA%\JetBrains`, Linux `~/.config/JetBrains`)를 목적지 OS로 바꾸고, 버전 폴더는 스크립트가 실행할 때 고른다(대상에 있는 같은 IDE의 최신 버전, 없으면 출발 기기의 버전 이름). 플러그인은 `<IDE 명령> installPlugins <id>`로 설치한다(계획 14).

## 10. 내보내기

### 공통

- 결과는 사용자가 고른 위치의 `repot-export-<YYYY-MM-DD>/` 폴더다. 같은 이름의 폴더가 있으면 `-2`, `-3`을 붙인다.
- 임시 폴더에 모두 쓴 뒤 마지막에 이름을 바꾼다. 중간에 실패하면 임시 폴더를 지운다.
- 선택된 항목 중 목적지 OS에서 사용할 수 없는 항목은 내보내지 않는다.
- 스크립트·매니페스트에 들어가는 값(패키지 이름, 런타임 버전, 확장 ID, URL)은 허용 패턴(`^[A-Za-z0-9@._+/:-]+$`)에 맞아야 한다. rustup component·target 이름은 §7의 더 좁은 패턴에 맞아야 하고, 열린 `repot.json`에서 맞지 않으면 그 항목을 건너뛴다. 패키지·전역 패키지·확장은 최신 버전으로 설치하므로 스캔한 버전은 검사하지 않는다(예: brew cask 버전의 쉼표). 맞지 않으면 내보내지 않고 경고한다. 렌더러는 모든 값을 셸 규칙에 맞게 따옴표로 감싼다.
- 설정 파일 내용은 스크립트에 넣지 않고 `files/` 아래에만 둔다.
- 형식과 상관없이 폴더 맨 위에 `repot.json`을 함께 쓴다. 이번 내보내기에 실제로 들어간 항목과, 목적지에서 쓸 수 없어 빠진 항목(`notExported: true`, 계획 13)을 담고, 설정 파일 내용은 출발 기기 형태 그대로 비밀정보만 가려 넣는다(사용자가 보이기로 한 줄만 원문). 형식은 내부 모델과 분리된 버전 타입(`schemaVersion`)이며, 같은 버전 안에서는 필드·enum 값을 더하기만 하고, 지우거나 뜻을 바꾸면 버전을 올리고 옛 버전 읽기를 남긴다. 상세는 계획 11.
- `notExported` 항목은 열 때와 "이 기기와 비교" 후에 처음 선택하지 않고 "이전 내보내기에 없던 항목"으로 표시한다. 사용자가 직접 켠다. 사용자가 체크를 끈 항목, 읽을 수 없던 항목, 스크립트에 쓰기 위험한 값이 든 항목은 저장하지 않는다.
- 같은 항목과 목적지라면, 기기에서 바로 내보낸 결과와 `repot.json`을 열어 다시 내보낸 결과는 생성 시각을 빼고 같다.
- 생성 파일(스크립트 출력 문구, 매니페스트, README)은 영어로 쓴다. 터미널에서 읽는 배포물이라 UI 언어와 상관없이 통일하고, 내보내기 명령은 로캘을 받지 않는다.
- 스크립트 종료 코드는 실패한 단계가 하나라도 있으면 1, 아니면 0이다.
- 패키지 설치는 설치 명령의 종료 코드가 0이어도 설치 확인 명령으로 다시 확인한 뒤에만 성공으로 센다. Chocolatey 2.x는 저장소가 504를 돌려줘도 "installed 0/0 packages"와 함께 0으로 끝나기 때문이다(Windows 스크립트).

### Step

셸 코드 대신 추상 단계를 쓰고, 렌더러가 목적지 셸에 맞게 바꾼다.

```rust
pub enum Step {
    AddTap { manager: PackageManager, name: String,
             url: Option<String> },                        // brew tap, scoop bucket. url은 사용자 bucket만
    InstallPackage { manager: PackageManager, kind: PackageKind, name: String,
                     mapped_from: Option<MappedFrom> },             // 대응된 경우 원래 매니저·종류·이름
    InstallRuntime { tool: RuntimeTool, runtime: String, version: String, set_default: bool,
                     components: Vec<String>, targets: Vec<String> },  // rustup만 채운다
    InstallGlobal { manager: GlobalManager, name: String },
    InstallExtension { editor: Editor, id: String },
    InstallIdePlugin { ide: JetBrainsIde, id: String, folder: String }, // installPlugins, folder로 이미 있음 확인
    CopyFile { src: String /* files/ 기준 상대 경로, '/' 구분 */, dest: TargetPath },
    GitClone { url: String, dest: TargetPath },           // oh-my-zsh 커스텀 플러그인
}
```

`export` 모듈이 `ItemDetails`를 목적지 OS 기준 `Step`으로 바꾼다. 런타임은 이때 §9 대응표로 대응 도구를 고르고, 대응 도구가 없으면 단계를 만들지 않는다. 다른 OS 패키지는 §9 패키지 대응표의 이름으로 바꾸고 `mapped_from`에 원래 이름을 남긴다. 렌더러는 그 단계 위에 `# from brew: ripgrep` 같은 영어 주석을 달고, 대응에 Scoop bucket이 필요하면 `AddTap`을 먼저 만든다. 두 출발 항목이 같은 목적지 이름으로 대응되면 첫 항목만 내보내고 경고한다. 내보내기 README에는 대응된 패키지 목록을 적는다.

사용자 Scoop bucket(`url`이 있는 항목)은 `scoop bucket add '<name>' '<url>'`로, 알려진 bucket과 대응용 bucket은 `scoop bucket add '<name>'`으로 추가한다. 이미 있는지는 둘 다 `scoop bucket list`가 돌려주는 bucket의 `Name`이 정확히 같은지로 본다. `scoop.json`의 `buckets`에도 `url`이 있으면 `Source`로 적는다(`scoop import`가 그 주소로 추가한다). 주소는 `https://…` 또는 `git@host:path` 형태이고, 접두어 뒤에 `@`가 없으며(자격 증명이 든 주소를 스크립트에 쓰지 않는다), 이름과 같은 안전 문자(영숫자와 `@._+/:-`)만 쓸 때만 내보낸다. 아니면 그 bucket 항목을 `InvalidValue` 경고와 함께 뺀다. `repot.json`에서도 같은 검사를 하고, `url`은 Scoop bucket 항목에만 허용한다. 어긋나면 그 항목을 건너뛴다. `url` 필드는 0.9에 더했고 없으면 `None`이라 이전 파일도 그대로 열린다.

`TargetPath`는 `Home(rel)`, `EditorUser { editor, rel }`, `PowerShellProfile`, `JetBrains { ide, version, rel }` 같은 논리 경로이고, 렌더러가 목적지 OS 기준으로 해석한다. 두 항목이 같은 대상 경로를 가지면(예: PowerShell 7과 5.1 profile) 첫 항목만 내보내고 경고한다.

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
  3. 실행 계획 요약과 필요한 도구 확인을 출력한 뒤 `계속할까요? [y/N]` (`--yes`/`-Yes`로 생략)
     - 도구 확인은 단계에 필요한 도구(패키지 매니저, 런타임 관리자, 에디터 명령 등)를 한 번씩 나열하고 "있음 / 아직 없음(설치 URL)"을 보여준다. Windows에서는 choco가 관리자 권한 없이 실행 중이면 그것도 알린다. 앞 단계가 설치해 줄 수도 있으므로(예: Homebrew로 Node.js) 없음은 경고일 뿐 실행을 막지 않는다. nvm·SDKMAN!은 셸 함수라 init 스크립트 파일로 확인한다.
     - 앱은 새 기기의 도구를 알 수 없으므로 이 확인은 스크립트에서만 한다.
  4. 패키지 매니저 → 런타임 → 전역 패키지 → 에디터 확장 → 설정 파일 순서로 실행
  5. 결과 요약: 성공(OK), 이미 됨(Already done), 하지 않음(Not done), 실패(Failed) 목록, 직접 채워야 할 마스킹 항목
     - "이미 됨"은 이미 설치되었거나 같은 내용의 파일이라 할 일이 없는 항목이다.
     - "하지 않음"은 시도하지 못한 항목이며 항목마다 이유를 붙인다(`brew not found`, `run PowerShell as Administrator`). 도구가 없어서 하지 않은 항목이 있으면 도구별 설치 안내 URL을 한 번씩 모아 "설치한 뒤 다시 실행"하라고 안내한다. 둘을 한 목록에 섞으면 사용자가 "다 끝났다"고 오해하므로 나눈다.
     - 종료 코드는 실패가 있을 때만 1이다. 도구가 없어 하지 않은 항목은 실패가 아니며(사용자가 설치 후 다시 실행하면 이어서 진행된다) 종료 코드를 바꾸지 않는다.
- 이미 설치된 항목은 건너뛴다(여러 번 실행해도 안전).
- 한 단계가 실패해도 계속 진행하고 요약에 기록한다.
- 패키지 매니저나 버전 관리자 자체가 없으면 설치하지 않는다. 공식 설치 방법을 안내하고 그 섹션을 건너뛴다. oh-my-zsh도 같다.
- 설정 파일을 복사할 때 대상 파일이 이미 있으면 `<파일>.repot-backup-<YYYYMMDDHHMMSS>`로 백업한 뒤 덮어쓴다.
- 런타임은 스캔한 정확한 버전으로, 전역 패키지·확장은 최신 버전으로 설치한다.
- "이미 됨" 확인은 목록에서 이름·버전이 통째로 같아야 한다. 부분 문자열로 찾으면 `java` bucket이 `java-extra`나 다른 bucket 주소에, `20.1.1`이 `20.1.10`에, `eslint`가 `eslint-plugin-react`에 맞아 설치하지 않은 항목을 "이미 됨"으로 남긴다. brew tap·pyenv 버전·에디터 확장은 줄 전체(`grep -qxF`, 확장은 대소문자 무시), fnm·volta·nvm-windows 목록은 공백으로 나눈 낱말, pnpm·pipx·cargo 목록은 줄의 첫 낱말, Scoop bucket은 `Name`으로 비교한다. npm 전역 패키지는 두 스크립트 모두 `npm ls -g --depth=0 <name>`의 종료 코드로 본다. 패키지(winget·choco `--exact`, `scoop prefix`, `brew list`, `dpkg -s`, `rpm -q`, `pacman -Q`), nvm(`nvm ls <정확한 버전>`), SDKMAN!·JetBrains 플러그인(폴더 확인)은 처음부터 그 이름 하나만 본다.
- rustup 툴체인은 툴체인과 담긴 component·target이 모두 있어야 "이미 됨"이다(bash `rustup_has`, PowerShell `Test-RustupToolchain`). 확인할 때 목적지 component 목록에서도 툴체인 호스트 접미사를 뗀다. 툴체인 이름은 `<채널>-<영문자로 시작하는 호스트>`로 찾아 `nightly`가 날짜 붙은 nightly와 맞지 않게 한다. 툴체인이 이미 있으면 다시 설치하지 않고(채널이 업데이트된다) `rustup component add --toolchain <t> …`, `rustup target add --toolchain <t> …`만 실행한다. 단계 이름에 component·target을 적는다(`rustup install rust stable (components: clippy, rust-src; targets: wasm32-unknown-unknown)`).

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
├── jetbrains-plugins.txt   # <IDE> <플러그인 id> 줄 (계획 14)
└── files/                  # 설정 파일, README에 복사할 위치 안내
```

- 선택된 항목이 있는 파일만 만든다.

## 11. UI

- 스택: React + TypeScript + Vite. 상태는 React 내장 기능(`useReducer`, Context)으로 시작한다. UI 라이브러리(컴포넌트 킷, 상태 관리 라이브러리)는 쓰지 않는다.
- 다국어 UI(계획 7): 영어(원본), 한국어(`ko`), 일본어(`ja`), 중국어 간체(`zh-Hans`). i18n 라이브러리 없이 로캘마다 JSON 파일 하나(`src/locales/<locale>.json`)에 메시지를 두고, 타입은 `en.json`에서 끌어낸다. 자리표시자는 `{name}`, 복수형은 `Intl.PluralRules` 범주를 키로 쓰는 객체, 숫자는 `Intl.NumberFormat`. 문장 조각을 이어 붙이지 않고 문장 전체를 메시지 하나로 둔다. 언어를 더하는 일은 JSON 파일 하나와 로캘 목록 한 줄이다.
- 화면 컴포넌트는 `src/strings.ts`의 함수와 `S`로 문구를 얻고, `strings.ts`가 현재 로캘 메시지를 읽는다.
- 로캘 결정: 앱에서 고른 언어(`localStorage`) > `navigator.languages`에서 처음 맞는 언어 > 영어. `ko*` → 한국어, `ja*` → 일본어, 간체 문자권 중국어(`zh`, `zh-CN`, `zh-SG`, `zh-Hans-*`) → 중국어 간체. 번체(`zh-TW`, `zh-HK`, `zh-Hant-*`)는 아직 지원하지 않아 맞는 언어로 치지 않는다. 고른 로캘은 `<html lang>`에 쓴다.
- 언어 선택은 시작 화면 아래와 검토 화면 사이드바 맨 아래의 팝업 버튼이다. 각 언어 이름은 그 언어로 쓴다.
- 영어로 남는 것: 내보내는 파일(§10), 로컬 도구가 준 패키지 설명, Provider 실패 메시지와 stderr 원문, 앱 이름과 도구·파일 이름, 실행 명령.
- 글꼴은 시스템 글꼴만 쓴다. `:lang()`별로 macOS(Apple SD Gothic Neo, Hiragino, PingFang SC)와 Windows·Linux(Malgun Gothic, Yu Gothic UI, Microsoft YaHei UI, Noto Sans CJK) 글꼴을 고른다.
- 화면 4개: 시작, 스캔 진행, 검토, 내보내기(+완료).
- 시작 화면에는 스캔 버튼 왼쪽에 "Repot 파일 열기…" 보조 버튼이 있다. 파일을 열면 검토 화면 사이드바 아래(스캔의 "감지 안 됨/실패" 자리)에 출발 OS와 만든 날짜, 건너뛴 항목 수를 보여준다. 비밀정보 줄은 보이기 토글 없이 "Repot 파일에 가린 채 저장됨, 복원한 뒤 값을 채우세요"로 설명한다.
- 열린 파일의 검토 화면에는 사이드바 아래에 "이 기기와 비교" 버튼이 있다. 비교 후 목록 항목에 "이미 있음"·"내용 다름" 표시를 붙이고, "이 기기에 없는 것만" 필터와 요약을 보여준다. 상세 패널은 판정 이유를 보여주고 설정 파일 내용 차이는 보여주지 않는다.
- 디자인: macOS 기본 앱(시스템 설정·Finder) 스타일. 시스템 색·글꼴·컨트롤 모양을 따르고 다크 모드를 지원한다. macOS에서는 제목 표시줄을 콘텐츠와 합치고(overlay) 사이드바에 시스템 반투명 재질을 쓴다(`tauri.macos.conf.json`). Windows·Linux는 같은 배치를 불투명 배경과 기본 제목 표시줄로 보여준다. 투명도 줄이기 설정에서도 불투명으로 바뀐다.

### 앱 업데이트 (계획 8)

- 시작 시 자동 확인(기본 켜짐), 언어 선택 옆 설정에서 자동 확인 끄기/수동 확인을 제공한다.
- 시작·완료 화면에서만 새 버전과 짧은 노트를 시트로 제안하고 설치 및 재시작/나중에를 선택한다. 스캔·검토·내보내기 중 결과는 보류한다.
- 다운로드 진행률과 오류를 표시하고 설치 중에는 새 작업을 막는다. Linux 비-AppImage는 패키지 매니저나 사이트 다운로드를 안내한다.

### 검토 화면

- 왼쪽: 사이드바. 카테고리 목록과 항목 수, 각 줄 끝의 카테고리 전체 선택/해제 체크박스
- 가운데: 항목 목록. 체크박스, 이름, (JetBrains 항목은) IDE 이름, 버전(길면 말줄임), 설명 한 줄, 배지(비밀정보·중복·목적지 OS에서 사용 불가를 SF 스타일 아이콘으로 표시하고, 각각 접근성 이름을 단다). 검색과 필터("사용 가능만", "경고 있는 것만")
- 오른쪽: 상세 패널. 설명, 원본 위치, 이식성 판정 이유, Finding 목록, 비밀정보 줄별 "원본으로 내보내기" 토글
- 기본 체크 상태: 목적지 OS에서 사용 가능한 항목은 모두 체크된 상태로 시작한다.
- 상단에서 목적지 OS를 바꾸면 사용 가능 여부를 즉시 다시 판정한다. 사용 불가가 된 항목의 체크는 해제된다.
- 목적지가 Windows이고 출발이 Windows가 아니면 목적지 옆에 패키지 매니저 팝업(winget/Scoop/Chocolatey)을 보여준다. 바꾸면 즉시 다시 판정한다. 대응된 항목은 목록에 대응 이름을 보조 텍스트로, 상세 패널에 "brew `ripgrep` → winget `BurntSushi.ripgrep.MSVC`"와 내장 대응표 기준이니 실행 전에 확인하라는 안내를 보여준다.
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
| `repot.json` 열기 실패 | 5 MB 초과, Repot 파일 아님, 더 새 버전(새 Repot 필요), 형식 오류, 읽기 실패를 구분해 표시. 현재 세션은 그대로 둔다. 형식이 틀린 항목만 있으면 그 항목을 건너뛰고 개수를 알린다 |

## 13. 테스트

- **Provider 단위 테스트**: OS별로 녹화한 명령 출력과 파일을 fixture로 두고 `FakeEnv`로 실행한다. 어떤 OS에서든 모든 Provider 테스트를 돌릴 수 있다.
- **비밀정보**: 감지해야 하는 사례와 감지하면 안 되는 사례(변수 참조, 주석 속 설명, 빈 값 등)를 표 기반으로 테스트한다.
- **사용 가능 판정**: Portability × 목적지 OS 조합을 표 기반으로 테스트한다.
- **패키지 대응표**: 데이터 형식(알려진 매니저 키, 정렬, 빈 값, 허용 패턴)과 역색인 유일성(대소문자 무시)을 테스트하고, CI `package-map` 작업이 이름을 실제 매니저로 확인한다.
- **렌더러**: `restore.sh`, `restore.ps1`, 각 매니페스트를 `insta` 스냅샷으로 고정한다.
- **Repot 파일**: 커밋한 v1 fixture는 고치지 않고 모든 이후 버전이 읽어야 한다. 모르는 필드·enum 값, 새 버전, 크기·경로·URL·값 검증, 바로 내보내기와 파일을 거친 내보내기의 결과 동일성을 테스트한다.
- **이 기기 비교**: 같은 OS와 다른 OS(Windows winget·Scoop)에서 패키지·런타임 버전·전역 패키지·확장·플러그인·설정 파일(같음·다름·비밀 줄만 다름) 판정을 표 기반으로 테스트한다.
- **CI (GitHub Actions)**:
  - macOS, Windows, Ubuntu 러너에서 `cargo test`, `clippy`, `fmt`
  - 프론트엔드·스크립트: `biome ci`(포맷, 린트, import 정렬)
  - 각 러너에서 실제 스캔을 실행하는 통합 테스트(러너에 brew, choco, apt 등이 설치되어 있음)
  - 생성된 스크립트를 `shellcheck`, `PSScriptAnalyzer`로 검사
  - `restore-e2e`: 새 macOS·Ubuntu·Windows 호스티드 러너에서 실제 `build_export`/`write_export`로 만든 스크립트를 실행한다. 사전 미설치 확인 후 brew/apt의 `sl`, choco의 `dos2unix`, 격리된 npm prefix의 `is-odd` 설치와 설정 파일 복사를 검증한다. 존재하지 않는 패키지 하나만 실패 목록에 남고 종료 코드가 1인지 확인하며, 두 번째 실행에서는 성공했던 세 항목이 모두 "이미 됨" 목록에 남고 "하지 않음" 목록은 비어 있어야 한다.
- **CLI**: 실행 함수를 `FakeEnv`로 불러 `scan`·`compare`·`export`의 출력과 종료 코드, `--dry-run`이 아무것도 쓰지 않는지를 테스트하고, 실행 파일로 `--help`·`--version`·잘못된 인자를 확인한다. 처음 선택 규칙은 core `session` 테스트와 Vitest에 같은 경우를 둔다.
- **프론트엔드**: 선택 상태, 목적지 OS 변경 시 비활성화 로직을 Vitest로 테스트한다.

## 14. 배포

- `tauri-action`으로 macOS(`.dmg`, Apple silicon·Intel), Windows(`.msi`/설치 파일), Linux(`.AppImage`, `.deb`)를 빌드해 GitHub Releases에 올린다.
- CLI는 같은 릴리스에 `repot-cli-<버전>-<target>.tar.gz`(macOS arm64·x64, Linux x64)와 `.zip`(Windows x64)으로 올린다. 자체 Homebrew tap(`brew install junjuny0227/tap/repot`)과 Scoop bucket(`junjuny0227/scoop-bucket`)은 릴리스를 공개하면 워크플로가 갱신한다. `cargo install --git`도 안내한다. crates.io 게시는 하지 않는다. (계획 15)
- 코드 서명이 없으므로 README에 첫 실행 방법(macOS "확인 없이 열기", Windows "추가 정보 → 실행")을 안내한다.
- 업데이트 자산은 별도의 updater 키로 서명한다. 공개 키만 앱에 포함하고 비밀 키/암호는 GitHub secrets로 전달한다. `bundle.createUpdaterArtifacts`와 tauri-action `uploadUpdaterJson`을 켠다. Windows 설치 모드는 passive다.
- updater endpoint는 `https://github.com/junjuny0227/Repot/releases/latest/download/latest.json`이다. draft를 사람이 공개해야 접근 가능하다. 0.1.0 사용자는 updater가 포함된 버전을 한 번 수동 설치해야 한다.
- 업데이트 시트의 "짧은 노트"는 `latest.json`의 `notes`이고, 공개된 릴리스 본문의 `## Highlights` 절을 평문으로 바꾼 것이다. 릴리스를 공개하면 `updater-notes.yml`이 `scripts/updater-notes.mjs`로 고쳐 다시 올린다. 서명 없음 안내와 PR 목록은 릴리스 페이지에만 둔다. Highlights 절이 없으면 노트는 비고, 시트는 노트 상자를 그리지 않는다.
- 라이선스: Apache-2.0 (저장소의 `LICENSE`)
- 제품 사이트: `apps/site`의 정적 페이지를 GitHub Pages(`https://junjuny0227.github.io/Repot/`)에 올린다. 다운로드 링크는 빌드 시점에 앱 버전으로 만든 릴리스 자산 URL이고, 사이트도 런타임 네트워크 호출을 하지 않는다. (계획 6)
- 사이트는 로캘마다 정적 페이지가 있다: `/Repot/`(영어), `/Repot/ko/`, `/Repot/ja/`, `/Repot/zh-Hans/`. 각 페이지는 맞는 `lang`, 번역된 제목·설명, canonical, hreflang 대체 링크와 `x-default`를 가진다. 자동 리디렉션은 하지 않고, `/Repot/`에 처음 왔을 때 브라우저가 다른 지원 언어를 원하면 닫을 수 있는 제안만 보여준다. (계획 7)

## 15. 알려진 한계

- apt·dnf의 "직접 설치" 목록에 기본 시스템 패키지가 섞일 수 있다.
- winget, cargo 항목은 로컬에 설명이 없어 "설명 없음"으로 표시된다.
- 패키지 매니저 간 중복은 이름이 정확히 같을 때만 감지한다.
- OS 간 패키지 대응은 내장 대응표에 있는 패키지만 된다. 표에 없는 패키지는 다른 OS로 옮길 수 없다. 대응 이름은 앱 버전 시점의 저장소 기준이라 이후 이름이 바뀔 수 있다.
- 로컬 생성기(계획 10)는 Homebrew → winget·Scoop만 넓힌다. 초기 threshold 0.8의 근거는 positive 106쌍·인공 negative 30쌍의 작은 in-sample 평가이며, kind 제거를 표시로 바꾼 최종 정책의 held-out 성능이나 무인 승인 안전성은 검증하지 못했다. 높은 확률과 CI 이름 존재 확인은 의미적 동일성의 보증이 아니며, 모르는 대응은 계속 키를 비워 둔다.
- 비밀정보 감지는 패턴 기반이라 모든 비밀을 잡지는 못한다. UI와 README에 이 점을 명시한다.
- WSL, Git Bash 환경은 고려하지 않는다.
- winget은 `winget` 소스 패키지만 다룬다(msstore 제외).
- volta는 런타임만 다룬다(volta로 설치한 전역 패키지 제외).
- rustup component·target은 이름으로만 옮긴다. 목적지 호스트에 없는 component·target이면 그 툴체인 단계는 실패로 기록된다. 0.8 이하 앱은 `repot.json`의 `components`·`targets`를 모르고 툴체인만 복원한다.
- Scoop의 알려진 bucket 이름(`extras` 등)을 다른 주소(포크)로 추가했어도 이름만 저장하므로 복원하면 공식 bucket이 추가된다. 로컬 경로나 `http://`·`ssh://`·자격 증명이 든 주소의 bucket은 내보내지 않는다. 0.8 이하 앱은 `repot.json`의 bucket `url`을 몰라 이름만으로 추가하므로 사용자 bucket 복원이 실패한다.
- `repot.json`과 이 기기 비교는 내보낼 스크립트가 쓰는 매니저 기준이다. 다른 매니저로 설치한 패키지와 다른 버전의 런타임은 "없음"으로 본다. rustup 툴체인은 파일에 적힌 component·target이 이 기기에 모두 있어야 "이미 있음"이다(스크립트 확인과 같다).
- `repot.json`은 고른 항목과 목적지에서 쓸 수 없던 항목만 담는다. 사용자가 체크를 끈 항목은 나중에 다시 내보낼 수 없다. 파일 안에서 가린 비밀정보는 되살릴 수 없다. 0.6 이하 앱은 `notExported` 표시를 몰라 그 항목을 보통 항목처럼 보여준다.
- JetBrains는 IDE별 최신 버전 설정 폴더만 읽는다. 플러그인은 JetBrains Marketplace에서 `installPlugins`로 받으므로 사용자 지정 저장소나 디스크에서 설치한 플러그인은 복원되지 않고, 설치 중에는 IDE를 닫아야 한다. IDE 자체는 설치하지 않는다(없으면 Toolbox 안내). 0.6 이하 앱은 JetBrains 항목이 든 `repot.json`을 열 때 그 항목을 건너뛴다.

## 16. 향후 방향

- "새 기기에서 앱으로 복원" 기능 (`repot.json`과 `Step` 모델을 그대로 재사용)
- SSH(암호화 내보내기 포함), Flatpak, Snap Provider
- 사용자 동의 하의 온라인 설명 보강
- CLI의 crates.io 게시와 homebrew-core·공식 Scoop bucket 등록 (core 공개 API가 안정된 뒤)
