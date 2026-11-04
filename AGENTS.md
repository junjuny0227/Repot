# AGENTS.md

<!-- omh:agent-instructions:begin -->
## 기준 문서

- 제품 결정의 기준은 `docs/specs/2026-11-02-repot-mvp-design.md`이다. 구현 순서와 인터페이스는 `docs/plans/2026-11-02-0[1-5]-*.md`를 따른다.
- 계획은 1 → 2 → (3, 4 병렬 가능) → 5 순서다. 제품 사이트(`apps/site`)는 계획 6(`docs/plans/2026-11-03-06-site.md`)을 따른다. 다국어 UI는 계획 7(`docs/plans/2026-11-03-07-i18n.md`)을, 앱 자체 업데이트는 계획 8(`docs/plans/2026-11-03-08-updater.md`)을, OS 간 패키지 대응은 계획 9(`docs/plans/2026-11-03-09-package-mapping.md`)를, 대응표 생성기는 계획 10(`docs/plans/2026-11-03-10-package-map-generator.md`)을, Repot 파일(`repot.json`)은 계획 11(`docs/plans/2026-11-04-11-repot-file.md`)을, 이 기기와 비교는 계획 12(`docs/plans/2026-11-04-12-compare.md`)를 따른다. 각 계획 첫머리의 Global Constraints와 Review Focus를 먼저 읽는다.
- 구현 중 스펙과 다른 결정을 내리면 같은 PR에서 스펙도 고친다. 계획만 고치고 스펙을 두면 두 문서가 어긋난다.

## 빌드와 테스트

아직 실행하지 못한 명령은 unverified다. 해당 계획의 Task가 끝나 처음 실행해 보면 `verified (exit 0, <날짜>)`로 바꾼다.

| 명령 | 상태 | 생기는 시점 |
|---|---|---|
| `cargo test --workspace` | verified (exit 0, 2026-11-02) | 계획 1 Task 1 |
| `cargo clippy --workspace --all-targets -- -D warnings` | verified (exit 0, 2026-11-02) | 계획 1 Task 1 |
| `cargo fmt --check` | verified (exit 0, 2026-11-02) | 계획 1 Task 1 |
| `cargo run -p repot-core --example scan` | verified (exit 0, 2026-11-02) | 계획 1 Task 13 (실제 기기를 읽기만 함) |
| `cargo run -p repot-core --example export` | verified (exit 0, 2026-11-02) | 계획 2 Task 9 |
| `pnpm --filter @repot/desktop typecheck` / `test` / `build` | verified (exit 0, 2026-11-02) | 계획 3 Task 1 |
| `pnpm --filter @repot/desktop tauri dev` | verified (exit 0, 2026-11-02) | 계획 3 Task 1 |
| `pnpm --filter @repot/site typecheck` / `test` / `build` | verified (exit 0, 2026-11-03) | 계획 6 Task 1 |
| `node --test scripts/*.test.mjs scripts/package-map-gen/*.test.mjs` | verified (exit 0, 2026-11-03) | 계획 10 (fixture와 가짜 fetch만 쓴다) |
| `pnpm package-map:gen candidates` / `run` | verified (2026-11-03, 보고할 문제가 있으면 exit 1) | 계획 10 (로컬 전용, 네트워크와 `JEB_AI_HUB_KEY` 필요, CI에서 돌리지 않는다) |
| `pnpm package-map:gen apply-review` (에이전트 결정은 `--reviewer agent`, provenance `agent-review`) | verified (2026-11-03, `--reviewer agent`로 실제 반영) | 계획 10 |
| `pnpm check` (검사만, CI는 `pnpm exec biome ci .`) / `pnpm fix` (포맷·린트 자동 수정) | verified (exit 0, 2026-11-04) | Biome 도입 |

- 툴체인은 `rust-toolchain.toml`(stable, clippy, rustfmt)과 pnpm workspace를 따른다.
- 포맷은 Rust가 `rustfmt.toml`(stable 옵션만), TS/JS/JSON/CSS가 `biome.json`(Prettier에 가까운 설정: 100열, 작은따옴표, 세미콜론, trailing comma)이다. Markdown·YAML·SVG와 fixture·생성 데이터는 Biome 대상이 아니다. 대량 포맷 커밋은 `.git-blame-ignore-revs`에 넣는다.
- Biome 규칙을 끄거나 무시할 때는 `biome-ignore` 주석에 이유를 적는다. 규칙 전체를 끄는 것은 `biome.json`에서만 한다.
- `repot-desktop`(Tauri)은 컴파일 시점에 `apps/desktop/dist`가 필요하다. 워크스페이스 전체를 `cargo`로 돌리기 전에 프론트엔드를 먼저 빌드한다.

## 생성 파일

손으로 고치지 않는다. 원본을 고치고 다시 생성한다. 재생성 명령도 코드가 생기기 전이라 모두 미검증이다.

| 생성 파일 | 원본 | 재생성 | 확인 |
|---|---|---|---|
| `apps/desktop/src/__fixtures__/scan.json` | `crates/core/examples/ui_fixture.rs`와 core 모델 | `cargo run -p repot-core --example ui_fixture > apps/desktop/src/__fixtures__/scan.json` | Vitest (`logic.test.ts`, `state.test.ts`) |
| `crates/core/tests/snapshots/` | `export/` 렌더러 | `INSTA_UPDATE=always cargo test` (cargo-insta가 설치돼 있으면 `cargo insta review`), unverified | `cargo test` |
| `apps/desktop/src-tauri/icons/` (`icon.icns` 제외) | `apps/desktop/app-icon.svg` (여백 없는 원본, Windows·Linux용. 시작 화면도 이 파일을 쓴다) | `apps/desktop`에서 `pnpm tauri icon app-icon.svg` | 빌드 |
| `apps/desktop/src-tauri/icons/icon.icns` | `apps/desktop/app-icon-macos.svg` (macOS 템플릿: 1024 캔버스, 824 판, 100px 여백) | 위 명령 다음에 `tmp=$(mktemp -d) && pnpm tauri icon app-icon-macos.svg -o "$tmp" && cp "$tmp/icon.icns" src-tauri/icons/icon.icns && rm -rf "$tmp"` | 빌드, Dock에서 다른 앱과 크기 비교 |
| `apps/site/public/screenshots/*.webp` | 앱 화면 캡처 PNG(2200×1440, `03-review-*`, `05-export-script-*`) | 계획 6 Task 2의 `cwebp -metadata icc -q 82 -alpha_q 100` (1100폭은 `-resize 1100 0`) | 사이트 화면 확인 |
| `pnpm-lock.yaml`, `Cargo.lock` | 의존성 선언 | `pnpm install`, `cargo` | CI `--frozen-lockfile` |
| `crates/core/data/package-map.provenance.json` | 생성기가 추가한 대응 값의 근거(방법, 확률, 카탈로그 날짜) | `pnpm package-map:gen run`, 사람 확인은 `pnpm package-map:gen apply-review` | Node 테스트, `cargo test`의 데이터 테스트, CI `package-map` 작업 |

- core 모델의 직렬화 모양을 바꾸면 `scan.json`을 반드시 다시 생성한다. TS 타입은 손으로 맞추고 이 fixture 테스트로만 검증한다.
- 스냅샷 변경은 diff를 읽고 의도한 변경인지 확인한 뒤 승인한다.
- `package-map.json`은 사람이 직접 고쳐도 된다. 생성기는 기존 값을 덮어쓰지 않고, `null`은 사람 확인(`none` 결정)으로만 쓴다. 생성기가 추가한 값을 지우면 provenance의 같은 항목도 지운다.

## 지켜야 할 경계

- `repot-core`는 Tauri에 의존하지 않고 네트워크 통신을 하지 않는다. 데스크톱 앱의 네트워크는 GitHub 업데이트 확인·다운로드뿐이고(설정에서 끌 수 있다), 스캔 데이터는 전송하지 않는다.
- core는 사람이 읽는 문장을 만들지 않는다. 문장은 UI(`apps/desktop/src/locales/<locale>.json`, `strings.ts`)가 만든다. 예외는 `catalog.rs`의 영어 한 문장 설명이고, 각 설명의 id(`Item.catalog_id`)로 UI가 번역을 찾는다.
- 외부 명령 실행과 파일 읽기는 모두 `Env`를 거친다. Provider 테스트는 `FakeEnv`와 `crates/core/tests/fixtures/<provider>/`로 하고, 테스트에서 실제 도구를 부르지 않는다.
- 새 crate 의존성은 계획의 Global Constraints에 없으면 계획부터 고친 뒤 추가한다.
- 프론트엔드에 shell 권한을 주지 않는다. 업데이트용 추가 권한은 `updater:default`, `process:allow-restart`뿐이다.
- 앱은 기기를 바꾸지 않는다. 스캔은 읽기만 하고, 쓰기는 내보내기 폴더뿐이다. 예외는 사용자가 동의한 서명된 앱 자체 업데이트 설치다.
- updater 비밀 키(`~/.tauri/repot-updater.key`)와 암호(키체인 `repot-updater-key-password`)는 출력하거나 커밋하지 않는다. 공개 키만 `tauri.conf.json`에 둔다.
- AI Hub 키 `JEB_AI_HUB_KEY`는 저장소 루트 `.env`에만 둔다(git이 무시한다). 출력하거나 커밋하지 않는다. 생성기 캐시 `.cache/package-map-gen/`과 사람 확인 파일, 스캔 원본도 커밋하지 않는다.

## 함정

- 스펙과 계획이 어긋난 채로 계획을 썼다가, 구현 전에 스펙 정리 커밋을 따로 만들어야 했다. 결정이 바뀌면 그 자리에서 스펙을 고친다.
- `shellcheck`, PSScriptAnalyzer, Windows/Linux 도구는 로컬에 없을 수 있다. 그러면 생성 스크립트와 비-macOS Provider는 계획 5의 CI에서만 실제로 검증된다. 로컬에서 확인하지 못했으면 "CI에서 검증 필요"라고 남긴다.

- `tauri dev`/`tauri build`가 `apps/desktop/src-tauri/Cargo.toml`을 다시 쓰면 의도한 의존성(updater, process 플러그인)을 지우지 않도록 그 줄만 되돌린다. 파일 전체를 `git checkout`하지 않는다.
- jeb 자동 반영 기준 0.8은 평가 세트에서 오답이 0이었지만, 실제 첫 실행에서 다른 회사의 비슷한 프로그램, 포트판, 버전 고정판을 골라 0.95로 올렸다(계획 10). 기준을 낮추려면 새 평가 결과와 실제 데이터 표본 확인이 먼저다.

## 규칙

- 커밋 메시지는 영어 Conventional Commits다 (`feat(core): …`, `docs: …`, `ci: …`). 각 계획 Task의 마지막 단계에 메시지가 적혀 있다.
- 문서(`docs/`)는 한국어, 생성 파일(스크립트 출력, README)과 `catalog` 설명은 영어다. UI와 사이트 문구는 영어 원본(`en.json`)과 ko·ja·zh-Hans 번역을 로캘 JSON에 둔다(계획 7). 문구를 더하면 모든 로캘 파일에 같은 키를 넣는다.
- 원격 push와 태그 생성은 실행 전에 사용자에게 확인을 받는다. 릴리스 워크플로는 draft로 만들고, 사용자가 확인한 뒤 공개한다. 0.x 버전은 prerelease가 아닌 정식 릴리스로 내고 이름에 `(beta)`를 붙인다.
<!-- omh:agent-instructions:end -->
