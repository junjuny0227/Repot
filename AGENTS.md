# AGENTS.md

<!-- omh:agent-instructions:begin -->
## 기준 문서

- 제품 결정의 기준은 `docs/specs/2026-11-02-repot-mvp-design.md`이다. 구현 순서와 인터페이스는 `docs/plans/2026-11-02-0[1-5]-*.md`를 따른다.
- 계획은 1 → 2 → (3, 4 병렬 가능) → 5 순서다. 각 계획 첫머리의 Global Constraints와 Review Focus를 먼저 읽는다.
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

- 툴체인은 `rust-toolchain.toml`(stable, clippy, rustfmt)과 pnpm workspace를 따른다.
- `repot-desktop`(Tauri)은 컴파일 시점에 `apps/desktop/dist`가 필요하다. 워크스페이스 전체를 `cargo`로 돌리기 전에 프론트엔드를 먼저 빌드한다.

## 생성 파일

손으로 고치지 않는다. 원본을 고치고 다시 생성한다. 재생성 명령도 코드가 생기기 전이라 모두 미검증이다.

| 생성 파일 | 원본 | 재생성 | 확인 |
|---|---|---|---|
| `apps/desktop/src/__fixtures__/scan.json` | `crates/core/examples/ui_fixture.rs`와 core 모델 | `cargo run -p repot-core --example ui_fixture > apps/desktop/src/__fixtures__/scan.json` | Vitest (`logic.test.ts`, `state.test.ts`) |
| `crates/core/tests/snapshots/` | `export/` 렌더러 | `INSTA_UPDATE=always cargo test` (cargo-insta가 설치돼 있으면 `cargo insta review`), unverified | `cargo test` |
| `apps/desktop/src-tauri/icons/` | `apps/desktop/app-icon.svg` | `pnpm --filter @repot/desktop tauri icon app-icon.svg` | 빌드 |
| `pnpm-lock.yaml`, `Cargo.lock` | 의존성 선언 | `pnpm install`, `cargo` | CI `--frozen-lockfile` |

- core 모델의 직렬화 모양을 바꾸면 `scan.json`을 반드시 다시 생성한다. TS 타입은 손으로 맞추고 이 fixture 테스트로만 검증한다.
- 스냅샷 변경은 diff를 읽고 의도한 변경인지 확인한 뒤 승인한다.

## 지켜야 할 경계

- `repot-core`는 Tauri에 의존하지 않고 네트워크 통신을 하지 않는다.
- core는 사람이 읽는 문장을 만들지 않는다. 문장은 UI(`apps/desktop/src/strings.ts`)가 만든다. 예외는 `catalog.rs`의 영어 한 문장 설명이다.
- 외부 명령 실행과 파일 읽기는 모두 `Env`를 거친다. Provider 테스트는 `FakeEnv`와 `crates/core/tests/fixtures/<provider>/`로 하고, 테스트에서 실제 도구를 부르지 않는다.
- 새 crate 의존성은 계획의 Global Constraints에 없으면 계획부터 고친 뒤 추가한다.
- 프론트엔드에 shell 권한을 주지 않는다.
- 앱은 기기를 바꾸지 않는다. 스캔은 읽기만 하고, 쓰기는 내보내기 폴더뿐이다.

## 함정

- 스펙과 계획이 어긋난 채로 계획을 썼다가, 구현 전에 스펙 정리 커밋을 따로 만들어야 했다. 결정이 바뀌면 그 자리에서 스펙을 고친다.
- `shellcheck`, PSScriptAnalyzer, Windows/Linux 도구는 로컬에 없을 수 있다. 그러면 생성 스크립트와 비-macOS Provider는 계획 5의 CI에서만 실제로 검증된다. 로컬에서 확인하지 못했으면 "CI에서 검증 필요"라고 남긴다.

## 규칙

- 커밋 메시지는 영어 Conventional Commits다 (`feat(core): …`, `docs: …`, `ci: …`). 각 계획 Task의 마지막 단계에 메시지가 적혀 있다.
- 문서(`docs/`)는 한국어, 생성 파일(스크립트 출력, README)과 `catalog` 설명은 영어, UI 문구는 MVP에서 영어다.
- 원격 push와 태그 생성은 실행 전에 사용자에게 확인을 받는다. 릴리스는 draft·prerelease로만 만든다.
<!-- omh:agent-instructions:end -->
