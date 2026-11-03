# Repot 계획 9: OS 간 패키지 이름 대응

**Goal:** 다른 OS로 옮길 때도 패키지를 넘길 수 있게 한다. 예를 들어 macOS의 `brew install ripgrep`이 Windows에서는 사용자가 고른 winget·Scoop·Chocolatey 이름으로, Linux에서는 배포판 계열의 apt·dnf·pacman 이름으로 바뀐다. 대응은 앱에 내장한 표에 적힌 것만 쓰고, 이름이 비슷하다고 추측하지 않는다.

**Architecture:** `repot-core`에 대응표 데이터 파일(`crates/core/data/package-map.json`)을 `include_str!`로 내장한다. 처음 쓸 때 한 번 파싱해 `(매니저, 종류, 이름) → 항목` 역색인을 만든다. 목적지는 OS만이 아니라 `Target { os, packages }`로 표현한다. `packages`는 그 목적지에서 쓸 패키지 매니저다. 사용 가능 판정은 같은 OS(Linux는 같은 계열)면 지금처럼 원래 매니저를 쓴다. 다른 OS면 대응표에서 목적지 매니저의 이름을 찾고, 찾으면 `Mapped`, 못 찾으면 `Unavailable`이다. 내보내기는 대응된 이름으로 `Step::InstallPackage`를 만들고 원래 이름을 남긴다. UI는 Windows 목적지에서만 매니저를 고르게 하고, 대응 결과를 목록과 상세 패널에 보여준다.

**Tech Stack:** Rust(serde, serde_json), React, TypeScript, Vitest, GitHub Actions

**Spec:** `docs/specs/2026-11-02-repot-mvp-design.md` §2·3·6·9·10·11·13·15·16

## Global Constraints

- **새 의존성 없음.** `serde_json`은 이미 core 의존성이다. 데이터 파싱에 다른 crate를 더하지 않는다. `Cargo.lock`, `pnpm-lock.yaml`은 바뀌지 않아야 한다.
- **core는 네트워크를 쓰지 않는다.** 대응표는 빌드에 포함되는 정적 데이터다. 실제 패키지 저장소 조회는 CI 검증 작업에서만 한다.
- **추측하지 않는다.** 대응은 표에 명시된 것만 쓴다. 이름이 같거나 비슷해도 표에 없으면 사용 불가다. 표의 `null`은 "그 매니저에는 없음"을 확인했다는 뜻이고, 키가 없으면 "아직 모름"이다. 둘 다 사용 불가지만 UI 문구가 다르다.
- **같은 OS 이동은 바뀌지 않는다.** 출발과 목적지가 같은 OS(Linux는 같은 계열)면 매니저 선택과 상관없이 원래 매니저와 이름을 그대로 쓴다. 기존 스냅샷이 바뀌면 회귀다.
- **기본 저장소만.** 대응 이름은 목적지 매니저의 기본 소스에 있어야 한다. Scoop은 공식 bucket(`main`, `extras`, `versions`, `java`, `nerd-fonts`)까지 허용하고, 필요한 bucket은 `Step::AddTap`으로 추가한다. 별도 apt 저장소, COPR, AUR, 사용자 tap에만 있는 패키지는 대응하지 않는다.
- **목적지 매니저:** macOS는 brew, Windows는 winget(기본)·Scoop·Chocolatey 중 사용자 선택, Linux는 계열별 apt(Debian)·dnf(Fedora)·pacman(Arch)이다. Linux `Other`는 매니저가 없어 다른 OS의 패키지를 받을 수 없다. Linux brew는 목적지로 쓰지 않는다.
- **값 검증:** 대응 이름도 §10의 허용 패턴(`^[A-Za-z0-9@._+/:-]+$`)을 지켜야 한다. 위반은 데이터 테스트에서 실패한다.
- **core는 문장을 만들지 않는다.** 대응표에는 이름만 둔다. 이유 문구는 UI 로캘(en·ko·ja·zh-Hans)에 둔다.
- **직렬화 변경:** `Portability`, `Availability`, 목적지 타입이 바뀌므로 `apps/desktop/src/__fixtures__/scan.json`을 `ui_fixture` 예제로 다시 생성하고 TS 타입을 손으로 맞춘다.
- AGENTS.md는 고치지 않고, 바꿀 줄이 있으면 권장 문구만 보고한다.

## 결정과 이유

**왜 내장 표인가.** core는 네트워크를 쓰지 않으므로 실행 중에 Repology 같은 외부 서비스를 조회할 수 없다. 이름 유사도로 맞추면 `python`처럼 이름은 같은데 버전 체계가 다르거나, `docker`처럼 패키지 구성이 다른 경우를 틀리게 옮긴다. 사람이 확인한 표만 쓰면 틀릴 가능성은 표의 오류로 한정되고, 그 오류는 CI 검증과 리뷰로 잡을 수 있다.

**왜 JSON 데이터 파일인가.** Rust 상수 배열은 컴파일 시점에 검사되지만, 패키지를 더하는 기여자가 Rust 코드를 고쳐야 한다. JSON은 사람이 읽고 고치기 쉽다. CI 검증 스크립트(Node)도 같은 파일을 그대로 읽는다. 파싱 실패와 형식 오류는 `cargo test`의 데이터 테스트가 잡으므로, 런타임 파싱 위험은 테스트로 막는다.

**왜 같은 OS는 그대로인가.** Windows에서 Windows로 옮기는 사용자가 winget과 Scoop을 함께 쓰고 있다면, 매니저 하나로 강제 변환하는 것은 손해다. 표에 없는 패키지가 사용 불가로 바뀌는 회귀도 생긴다. 대응은 원래 매니저를 쓸 수 없을 때만 쓴다.

**왜 Windows만 매니저를 고르는가.** macOS의 사실상 표준은 brew이고, Linux는 배포판 계열이 매니저를 정한다. Windows는 winget·Scoop·Chocolatey가 함께 널리 쓰이고 사용자 취향이 갈린다. 기본값은 OS에 내장된 winget이다.

**처음 표의 크기.** 개발자 Mac 한 대를 실제로 스캔(2026-11-03)해 보니 brew 패키지가 formula 22개, cask 31개였다. formula는 대부분 CLI 개발 도구라 다른 OS에도 있지만, cask에는 macOS 전용 앱(창 관리, 메뉴 막대 도구)이 많다. 그래서 첫 표는 개발 도구와 크로스 플랫폼 앱 약 150개로 시작한다. 후보는 공개 Homebrew 설치 통계 상위 목록과 위 스캔 결과에서 고르고, 각 매니저의 공식 저장소에서 이름을 확인한다. macOS 전용으로 확인된 cask는 다른 매니저 값을 `null`로 둬서, "대응 없음(확인됨)"으로 보여준다.

## 데이터 형식

```json
{
  "version": 1,
  "packages": {
    "ripgrep": {
      "brew": "ripgrep",
      "winget": "BurntSushi.ripgrep.MSVC",
      "scoop": "ripgrep",
      "choco": "ripgrep",
      "apt": "ripgrep",
      "dnf": "ripgrep",
      "pacman": "ripgrep"
    },
    "firefox": {
      "brewCask": "firefox",
      "winget": "Mozilla.Firefox",
      "scoop": "extras/firefox",
      "choco": "firefox",
      "apt": null,
      "dnf": "firefox",
      "pacman": "firefox"
    },
    "rectangle": {
      "brewCask": "rectangle",
      "winget": null, "scoop": null, "choco": null,
      "apt": null, "dnf": null, "pacman": null
    }
  }
}
```

- 키는 안정된 공통 id(소문자, `[a-z0-9-]+`)다. 앞으로 catalog 설명을 붙일 때 이 id를 쓸 수 있다.
- 매니저 키는 `brew`(formula), `brewCask`, `winget`, `scoop`, `choco`, `apt`, `dnf`, `pacman`이다. Scoop은 `bucket/name` 형식이고, bucket이 없으면 `main`이다.
- 같은 매니저·종류의 이름은 표 전체에서 한 번만 나와야 한다(역색인이 하나로 정해지도록). 대소문자를 구분하지 않고 비교한다. winget id와 choco id가 대소문자를 구분하지 않기 때문이다.
- 버전이 붙은 이름(`zulu@17`, `python@3.12`)은 각자 별도 항목이다.
- 키 순서는 알파벳순으로 유지한다(리뷰 diff를 작게 하기 위해). 데이터 테스트가 확인한다.

## 모델 변경 (core)

```rust
pub struct Target { pub os: Os, pub packages: Option<PackageManager> }  // packages: 목적지 패키지 매니저
impl Target { pub fn default_for(os: Os) -> Target }                    // macOS→Brew, Windows→Winget, Linux 계열→Apt/Dnf/Pacman, Other→None

pub enum Portability { Any, SameOs, SameShell(Shell), Mapped(RuntimeTool), Package }  // Package: 패키지 항목

pub enum Availability {
    Available,
    Mapped { manager: PackageManager, name: String },   // 다른 OS의 매니저 이름으로 바뀌어 사용 가능
    Unavailable(UnavailableReason),
}
pub enum UnavailableReason {
    DifferentOs { source: Os },                // 기존: 탭 등 대응 대상이 아닌 항목
    ShellUnsupported(Shell),
    NoCounterpart(RuntimeTool),
    NoPackageManager,                          // 목적지(Linux Other)에 매니저가 없음
    NotInPackageMap { manager: PackageManager },   // 표에 없음 (아직 모름)
    NoPackageInManager { manager: PackageManager }, // 표에 null (없음을 확인함)
    Unreadable,
}

// Step::InstallPackage에 원래 이름을 더한다
InstallPackage { manager, kind, name, mapped_from: Option<MappedFrom> }
pub struct MappedFrom { pub manager: PackageManager, pub kind: PackageKind, pub name: String }
```

- `Target`은 `availability_for`, `preview_export`, `export` Tauri 명령의 `target: Os`를 대신한다. `packages`가 목적지 OS에서 쓸 수 없는 값(예: macOS에 winget)이면 명령이 오류를 돌려준다.
- brew tap, scoop bucket 같은 `PackageKind::Tap` 항목은 다른 OS로 옮기지 않는다(`DifferentOs`). 필요한 bucket은 대응된 패키지가 스스로 `AddTap`을 만든다.
- 둘 이상의 출발 항목이 같은 목적지 이름으로 대응되면, 첫 항목만 내보내고 기존 중복 경고 방식으로 알린다.

## Review Focus

1. 같은 OS 이동의 결과가 그대로인가: 기존 `insta` 스냅샷과 Vitest가 바뀌지 않아야 한다.
2. 표에 없거나 `null`인 패키지가 절대 내보내지지 않는가. 비슷한 이름 매칭 코드가 없는가.
3. 대응 이름이 허용 패턴을 지키고, 렌더러가 따옴표로 감싸며, Scoop bucket이 패키지보다 먼저 추가되는가.
4. 역색인이 유일한가(대소문자 무시). 데이터 테스트가 중복, 정렬, 알 수 없는 매니저 키, 빈 문자열을 잡는가.
5. Windows 매니저 선택이 목적지 OS를 바꿀 때 올바르게 초기화되고, 선택이 사용 가능 판정과 미리보기에 즉시 반영되는가.
6. CI 검증이 실제 저장소에서 이름을 확인하되, 일시적인 네트워크 실패와 "패키지 없음"을 구분하는가.
7. 네 로캘의 새 키가 모두 있고, 이유 문구가 "모름"과 "없음 확인"을 구분하는가.

## 파일 구조

- `crates/core/data/package-map.json`: 대응표 데이터
- `crates/core/src/package_map.rs`: 로딩, 역색인, `lookup(source_manager, kind, name, target_manager)`
- `crates/core/src/{model.rs,availability.rs,os.rs}`: `Target`, `Portability::Package`, `Availability::Mapped`, 새 이유
- `crates/core/src/providers/*.rs`: 패키지 항목의 portability를 `Package`로
- `crates/core/src/export/{step.rs,mod.rs,manifest.rs,script/{bash,powershell}.rs}`: 대응된 `InstallPackage`, Scoop bucket, 원래 이름 주석, 매니페스트 반영
- `crates/core/tests/package_map_test.rs`: 데이터 형식 테스트, 판정 표 테스트
- `crates/core/tests/snapshots/`: 다른 OS 대상 스크립트·매니페스트 스냅샷 추가
- `scripts/check-package-map.mjs`(+`.test.mjs`): 러너의 실제 매니저로 이름 확인
- `.github/workflows/ci.yml`: `package-map` 작업(대응표가 바뀐 PR과 주 1회 예약 실행)
- `apps/desktop/src/{api.ts,types.ts,state.ts,logic.ts}`, `src/screens/{TargetSelect.tsx,ReviewScreen.tsx,review/*}`, `src/locales/*.json`: Windows 매니저 선택, 대응 표시, 이유 문구
- `README*.md`, `apps/site/src/locales/*.json`: "What it moves"와 알려진 한계 갱신

### Task 1: 계획과 스펙 (PR `docs/plan-9-package-mapping`)

- [ ] 이 계획을 추가한다. 스펙 §2(포함으로 이동), §3(내장 데이터만), §6(모델), §9(패키지 대응), §10(Step, 주석), §11(검토 화면), §13(테스트), §15·§16을 고친다.
- [ ] `git diff --check`, 스펙과 계획이 같은 결정을 말하는지 확인한다.
- 커밋: `docs: plan cross-OS package mapping`

### Task 2: 대응표 로딩과 판정 (PR `feat/package-map-core`)

- [ ] TDD: `package_map_test.rs`를 먼저 쓴다. 형식(버전, 알려진 매니저 키, 정렬, 빈 값 없음, 허용 패턴), 역색인 유일성(대소문자 무시), `null`과 키 없음의 구분을 확인한다. 작은 시험용 표로 판정을 표 기반 테스트한다: 같은 OS, macOS→Windows(winget/scoop/choco), macOS→Linux 세 계열, Linux Debian→Fedora, Linux Other, tap, 사용자 tap 이름(`owner/tap/formula`).
- [ ] `package_map.rs`, `Target`, `Portability::Package`, `Availability::Mapped`, 새 이유를 구현한다. 첫 데이터는 테스트에 필요한 몇 항목(`ripgrep`, `git`, `firefox`, `rectangle`)만 넣는다.
- [ ] Provider의 패키지 항목을 `Portability::Package`로 바꾸고, `ui_fixture`로 `scan.json`을 다시 생성한다.
- [ ] `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- 커밋: `feat(core): add package map data and lookup`, `feat(core): judge package availability across OSes`

### Task 3: 내보내기 (같은 PR)

- [ ] `InstallPackage.mapped_from`, Scoop bucket `AddTap`, 중복 대상 경고를 구현한다. 스크립트는 단계 위에 `# from brew: ripgrep` 같은 영어 주석을 단다. 매니페스트(`winget.json`, `scoop.json`, `choco-packages.config`, `*-packages.txt`)에 대응 이름을 넣고, 내보내기 README에 대응된 패키지 목록을 적는다.
- [ ] `insta` 스냅샷: macOS 출발 → Windows(세 매니저), Ubuntu, Fedora, Arch 목적지 스크립트와 매니페스트. 기존 스냅샷이 바뀌지 않는지 확인한다.
- [ ] 생성 스크립트를 `shellcheck`/PSScriptAnalyzer로 검사한다(로컬에 없으면 CI에서 검증 필요라고 남긴다).
- 커밋: `feat(core): export mapped packages`

### Task 4: 데스크톱 UI (PR `feat/package-map-ui`)

- [ ] Tauri 명령의 `target`을 `Target`으로 바꾼다.
- [ ] 검토 화면 상단 목적지 선택 옆에 Windows 목적지일 때만 패키지 매니저 팝업(winget/Scoop/Chocolatey)을 보여준다. 출발도 Windows면 숨긴다(같은 OS는 원래 매니저를 쓰므로).
- [ ] 목록에서 대응된 항목에는 대응 이름을 보조 텍스트로, 상세 패널에는 "brew `ripgrep` → winget `BurntSushi.ripgrep.MSVC`"와 "내장 대응표 기준, 실행 전에 확인하세요" 안내를 보여준다. 이유 문구 3개(매니저 없음, 표에 없음, 없음 확인)를 네 로캘에 넣는다.
- [ ] Vitest: 목적지 변경 시 매니저 초기화, 매니저 변경 시 재판정과 체크 해제, 대응 표시, 이유 문구 선택.
- [ ] desktop `typecheck`/`test`/`build`, 그다음 cargo 검사.
- 커밋: `feat(desktop): choose Windows package manager`, `feat(desktop): show package mappings`

### Task 5: 첫 대응표와 CI 검증 (PR `feat/package-map-data`)

- [ ] `scripts/check-package-map.mjs`: 러너 OS에 맞는 매니저 키만 골라 실제 이름을 확인한다. macOS는 `brew info --json=v2 --formula/--cask`, Windows는 `winget show --id <id> --exact --source winget`, `scoop info`(필요한 bucket 추가 후), `choco search <id> --exact`, Ubuntu는 `apt-cache show`, Fedora 컨테이너는 `dnf info`, Arch 컨테이너는 `pacman -Si`. 명령 실패와 "없음"을 구분하고 재시도한다. 단위 테스트(`node --test`)로 출력 해석을 검증한다.
- [ ] `ci.yml`에 `package-map` 작업을 더한다. 대응표나 스크립트가 바뀐 PR, 그리고 주 1회 `schedule`에서 돈다. 예약 실행 실패는 이름이 바뀐 패키지를 알려 준다.
- [ ] 첫 대응표 약 150개를 채운다. 근거(공식 저장소 페이지나 CI 출력)를 PR 본문에 남긴다.
- [ ] `restore-e2e`에 다른 OS 경우를 하나 더한다. macOS 출발 fixture로 Windows 목적지(winget) 스크립트를 만들어 Windows 러너에서 실제로 설치되는지 확인한다.
- 커밋: `feat(core): seed package map`, `ci: verify package map against real repositories`

### Task 6: 문서와 실제 확인

- [ ] README 네 언어의 "What it moves", "Known limitations", 사이트 네 로캘 문구를 갱신한다(같은 OS로만 옮긴다는 문장 수정).
- [ ] 이 Mac에서 실제 스캔 후 Windows(winget/Scoop/Chocolatey)·Ubuntu 목적지로 미리보기를 만들어, 대응된 수와 대응되지 않은 수를 기록한다.
- [ ] 프론트엔드 먼저 빌드한 뒤 전체 검사, PR CI 통과 후 merge commit, 브랜치·worktree 정리.
- 커밋: `docs: describe cross-OS package mapping`
