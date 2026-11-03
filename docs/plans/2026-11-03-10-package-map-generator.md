# Repot 계획 10: 패키지 대응표 생성기

**Goal:** Homebrew 패키지의 Windows 대응을 찾아 내장 대응표를 넓히되, 모르는 것을 "없음"으로 바꾸지 않는다. 개발자가 후보를 모으고 판정·검토한 뒤 데이터 PR로 반영한다. 앱은 늘어난 정적 대응표만 사용한다.

**Architecture:** 앱과 분리된 로컬 Node.js 도구가 공식 카탈로그를 캐시하고, 저장소·URL·정확한 정규화 이름으로 후보를 만든다. 채널이 다른 후보는 제거하고 CLI/GUI 종류 차이는 검토 표시로 남긴다. 유일한 HIGH 후보는 직접 승인하고, 나머지는 jeb가 고른 후보의 확률로 auto/review를 나눈다. auto는 기존 대응표에 덧붙이고, review는 사람이 파일에 결정을 적어 적용한다. 근거는 별도 provenance 파일에 남긴다. 기존 CI는 결과 데이터의 형식과 실제 패키지 이름을 검증한다.

**Tech Stack:** Node.js 22 ESM(`.mjs`), 표준 라이브러리(`fetch`, `zlib`, `child_process`, `crypto`, 파일·스트림 API), `node --test`, git, AI Hub jeb

**Spec:** `docs/specs/2026-11-02-repot-mvp-design.md` §9·10·13·15·16. 대응표 소비 방식은 계획 9를 그대로 따른다.

## Global Constraints

- **새 의존성 없음.** Node 22 표준 라이브러리만 사용한다. YAML·tar·dotenv 라이브러리나 npm/crate 의존성을 추가하지 않는다. `pnpm-lock.yaml`, `Cargo.lock`은 바뀌지 않는다.
- **앱·CI에서 생성하지 않는다.** 생성기는 개발자 로컬에서만 네트워크와 jeb를 쓴다. `repot-core`는 계속 네트워크를 쓰지 않는다. CI에는 네트워크 없는 생성기 단위 테스트만 더하고, 기존 `package-map` 작업으로 결과를 검증한다. 앱 네트워크 권한·Rust/TS 모델은 바꾸지 않는다.
- **출발은 Homebrew, 목적지는 winget·Scoop뿐.** 출발 키는 formula의 `brew`, cask의 `brewCask`다. Scoop은 공식 `main`, `extras`, `versions`, `java`, `nerd-fonts`만 쓴다. 정확도를 측정하지 않은 choco·apt·dnf·pacman 키는 생성하지 않고 기존 값은 보존한다.
- **키는 비밀이다.** `JEB_AI_HUB_KEY` 환경변수를 우선하고, 없을 때만 저장소 루트 `.env`에서 의존성 없이 읽는다. 키·Authorization 헤더·`.env`를 로그, 오류, 캐시, fixture, PR, 커밋에 남기지 않는다. 셸로 `.env`를 실행하지 않는다. `.env`, `.cache/`는 gitignore 대상이다.
- **자동 `null` 금지.** 후보 없음, jeb의 "none of these", 오류·중단은 부재 확인이 아니다. 해당 키를 추가하지 않는다. 사람이 review의 `decision`에 `none`을 쓴 경우에만 확인된 부재로 `null`을 넣는다.
- **기존 값을 덮어쓰지 않는다.** 기존 문자열과 `null` 모두 보존한다. §10 이름 패턴(`^[A-Za-z0-9@._+/:-]+$`), 매니저별 대소문자 무시 유일성(Scoop은 생략 bucket을 `main`으로 비교), 알파벳 키 정렬을 지킨다. 새 공통 id는 brew token 소문자이며 `[a-z0-9-]+`에 맞지 않거나 충돌하면 건너뛰고 보고한다. 버전 표기를 임의로 지워 id를 만들지 않는다.
- **정확한 근거만 후보로 쓴다.** fuzzy/유사 이름 검색과 패키지별 예외는 넣지 않는다. HIGH 판정은 후보를 자르거나 필터링하기 전의 저장소 중복까지 고려한다.
- 코드 주석·CLI 출력은 영어, 문서는 한국어다. AGENTS.md·CLAUDE.md는 고치지 않고 권장 문구만 PR 본문에 남긴다.

## 결정과 이유

**왜 CI가 아니라 로컬인가.** 카탈로그 다운로드와 유료·할당량 제한 판정은 앱 실행이나 모든 PR의 통과 조건이 아니다. 로컬 캐시·checkpoint로 비용과 중단을 제어하고, 생성 결과만 작은 데이터 PR로 제출한다. CI에서 키를 필요로 하거나 실시간 모델 응답에 테스트가 흔들리는 구조를 피한다. CI는 이름이 존재하는지 확인할 뿐, 두 패키지가 같은 제품인지 보증하지 않는다.

**왜 jeb와 0.8인가.** 2026-11-03 실험에서 baseline jeb는 정답이 포함된 106쌍 중 97쌍을 맞혔다. threshold 0.8에서 formula 31/48, cask 31/58(합계 62/106)을 승인하고 이 positive 집합의 오승인은 0이었다. 하지만 정답 후보를 제거한 인공 negative 30쌍에서는 9/30을 잘못 승인했다. 채널과 kind를 모두 제거한 실험에서는 positive 정답 98/106, 0.8 승인 78/106(formula 33/48, cask 45/58), positive 오승인 0, negative 오승인 0/30이었다. 0.8은 검사한 threshold 중 이 조건을 만족한 최저값이므로 기본값으로 채택하고 `--threshold`로 조정한다. 선택 확률은 응답의 선택된 `probabilities` 항목이지 별도 confidence 문구가 아니다.

flash-next의 선택된 paired positive에서는 두 모드 모두 31/34였고, negative는 9/12에서 10/12로 바뀐 정도여서 두 번째 모델의 이득을 입증하지 못했다. 별도 command-name 검사는 정답 154쌍에서 MISMATCH 오경보 2개, 잘못된 후보 297개에서 MATCH 41개와 UNKNOWN 252개여서 hard gate로 쓰지 않는다. 모델 추가나 명령 이름 gate 대신 단일 jeb와 보수적인 review 경계를 쓴다.

**왜 채널은 제거하고 kind는 표시만 하는가.** stable과 nightly/beta처럼 릴리스 채널이 다르면 자동 대응에 넣지 않는다. 반면 kind까지 제거한 실험은 `neovim` → `Neovim.Neovim`, `vim` → `vim.vim` 두 정답을 잃었다. CLI/GUI 힌트는 틀릴 수 있으므로 후보를 보존하고 kind 불일치로 고른 결과는 확률과 무관하게 review로 보낸다. unknown/mixed는 확정 불일치로 보지 않는다.

**측정의 한계.** 근거는 spike의 `eval-SUMMARY.md`, `eval-jeb-filtered/RESULTS.md`, 독립 재계산 기록 `eval-jeb-filtered/VERIFY.md`다. 검증 기록은 184/184 완료, 신규 요청 33, 재사용 138, 빈 목록의 결정적 none 13, 잃은 정답 2, 키 유출 검사 통과를 확인했다. positive 106쌍(formula 48/cask 58)과 인공 negative 30쌍은 작고 손으로 고른 **in-sample** 집합이며 threshold도 같은 집합에서 골랐다. HIGH-only 48/48 정답은 별도 sanity 집합이지 positive 106에 더할 표본이 아니다. 재사용 응답은 새 반복 실험이 아니다. 특히 최종 정책인 "채널 제거 + kind 표시"의 전체 성능을 "채널·kind 모두 제거"의 78/106·0/30과 같다고 주장하지 않는다. held-out 정밀도, 실사용 오승인율, 무인 승인 안전성은 아직 검증하지 못했다. HIGH 우회도 48/48의 관측을 근거로 한 정책이지 보증이 아니다. 실험 원본·캐시·개인 경로는 저장소에 복사하지 않는다.

**왜 provenance sidecar인가.** `package-map.json`은 앱에 이름만 주는 기존 형식을 유지한다. 추가된 값별로 `high-tier | jeb | human` 방법, 확률, tier, 카탈로그 snapshot 날짜를 `package-map.provenance.json`에 기록하면 근거를 추적하면서 Rust/TS 모델과 소비자를 바꾸지 않아도 된다. 모델 확률이 없는 HIGH/human은 확률을 만들어내지 않고 `null`로 기록한다(이것은 대응표의 부재 `null`과 다르다). 기존 수동 데이터의 근거를 소급해서 꾸며 쓰지 않는다.

**왜 파일 검토인가.** 터미널에서 즉시 선택하는 방식보다 후보 메타데이터·선택·확률을 나란히 읽고 diff로 남기는 방식이 실수를 줄인다. gitignored review 파일의 빈 `decision`을 사람이 채우고 `apply-review`로 적용한다. 빈 결정이나 `skip`은 아무것도 쓰지 않으며, `none`만 확인된 부재를 뜻한다.

## 파이프라인

1. **입력 선택:** `--from-analytics N`은 Homebrew formula/cask의 최근 30일 설치 통계 상위 N개를 각각 읽는다. `--from-scan <scan.json>`은 제공된 Repot 스캔 JSON에서 brew 항목만, `--names a,b`는 명시한 token을 읽는다. 입력을 합쳐 중복을 제거하고 winget·Scoop 키가 이미 모두 있는 출발 항목은 건너뛴다(`null`도 기존 결정이다). 스캔 전체나 로컬 경로·설정은 모델에 보내지 않고 선택된 패키지의 공개 카탈로그 사실만 쓴다.
2. **카탈로그:** Homebrew `formula.json`·`cask.json`(formulae.brew.sh API), winget-pkgs `master` tarball, Scoop 공식 bucket을 `.cache/package-map-gen/`에 둔다. winget은 gzip/tar를 스트리밍해 id별 최신 버전만 남기고 spike와 같은 제한된 정규식 필드 추출을 쓴다(범용 YAML 파서가 아님). Scoop은 얕은 `git clone`으로 받는다. `sources.json`에 원본 URL·수집 시각·해시를 기록한다. `--offline`은 캐시만 쓰며 카탈로그·통계·jeb 어느 네트워크에도 접근하지 않는다. 캐시가 부족하면 누락을 알리고 값을 만들지 않는다.
3. **후보:** 정규화한 source repo가 같으면, 그 repo 후보가 유일하고 kind가 명확히 호환될 때만 HIGH다. repo가 여럿이거나 kind가 호환되지 않으면 MEDIUM, 같은 정규화 homepage/download URL도 MEDIUM, 정확한 정규화 이름만 같으면 LOW다. repo HIGH라도 채널 불일치나 출발 token의 `@` 버전 고정은 MEDIUM으로 낮춘다. URL의 scheme·`www.`·후행 slash·`.git` 및 repo 경로를 정규화하고, 이름은 소문자 영숫자만 비교한다. 강한 tier 순, 같은 tier는 id 대소문자 무시 순으로 최대 12개를 유지한다. 필터 후 HIGH로 승격하지 않는다.
4. **사전 필터:** 식별자 경계로 `nightly`, `beta`, `alpha`, `preview`, `dev`, `canary`, `prerelease`, `insider(s)`, `unstable`, `snapshot`, `edge`, 후행 `-git`을 감지한다. insider/insiders는 같은 label이며 빈 집합은 stable/unlabelled다. 출발·후보 label 집합이 다르면 제거한다. 설명·URL·버전으로 채널을 추측하지 않는다. formula=cli, cask=gui이나 binaries 선언이 있으면 mixed다. 확정 CLI/GUI 불일치는 flag만 단다.
5. **판정:** 유일한 HIGH이면서 kind flag가 없으면 jeb를 호출하지 않고 auto다. 나머지는 `(출발, 목적지)`당 요청 하나를 `POST https://jeb.https.gsmsv.site/v1/systemone`에 Bearer 인증으로 보낸다. 후보별 동일한 형식의 선택지와 마지막 `none of these`를 쓴다. 하루 1000요청, 동시 최대 2개, 503은 backoff 재시도다. HTTP 시도(재시도 포함)는 `--max-requests`로 제한하고 `x-ratelimit-remaining-credits`가 기본 floor 100 미만이면 새 요청을 멈춘다. 할당량 헤더가 없거나 판독 불가하면 안전하게 중단한다. 요청 해시로 응답을 캐시하고 checkpoint로 재개한다. 키는 해시 입력·캐시에 포함하지 않는다. offline에서는 캐시 없는 jeb 요청을 보내지 않는다.
6. **분류:** jeb가 후보를 고르고 확률 ≥ 기본 0.80이며 그 후보에 kind flag가 없으면 auto, 후보가 있지만 저확률·kind flag 선택·none이면 review, 후보 자체가 없으면 no-candidate다. 실패·잘못된 응답·할당량 중단은 미완료로 남겨 재개하며 auto나 부재로 처리하지 않는다.
7. **출력·검토:** auto를 `package-map.json`과 provenance에 함께 반영한다. review는 `.cache/package-map-gen/review.json`에 출발·목적지, 후보와 메타데이터, jeb 선택/확률, 빈 `decision`을 쓴다. 사람은 후보 id / `none` / `skip`을 적고 `pnpm package-map:gen apply-review`로 적용한다. 후보에 없는 id·잘못된 결정은 오류다. 적용 직전에 최신 대응표의 기존 값·id 충돌·유일성·패턴을 다시 검사한다. 적용한 값의 provenance 방법은 `human`이다.
8. **데이터 PR:** auto/review/no-candidate/미완료/충돌 수와 실제 추가 수를 나눠 보고한다. 데이터·provenance diff를 확인하고 기존 CI `package-map` 작업으로 실제 이름 존재를 검사한 뒤 merge commit으로 병합한다. CI의 존재 확인을 의미적 동일성 확인으로 표현하지 않는다.

## 파일 구조

- `scripts/package-map-gen/cli.mjs`: 진입점, 입력·옵션, 실행 요약
- `scripts/package-map-gen/catalogs.mjs`: 다운로드·스트리밍·공식 bucket·snapshot 캐시
- `scripts/package-map-gen/normalize.mjs`: URL/repo/name 정규화
- `scripts/package-map-gen/candidates.mjs`: 결정적 후보·tier·정렬·상한
- `scripts/package-map-gen/filter.mjs`: 채널 제거·kind flag
- `scripts/package-map-gen/judge.mjs`: 키 읽기·jeb 요청/해석·quota·응답 캐시·checkpoint
- `scripts/package-map-gen/classify.mjs`: HIGH 우회·확률·flag별 분류
- `scripts/package-map-gen/apply.mjs`: 덮어쓰기 없는 병합·검증·provenance
- `scripts/package-map-gen/review.mjs`: review 생성·사람 결정 적용
- `scripts/package-map-gen/*.test.mjs`, `fixtures/`: 작고 비밀정보 없는 합성 카탈로그·응답 fixture, 주입한 fetch/git 실행기로 네트워크 없는 테스트
- `package.json`: `"package-map:gen": "node scripts/package-map-gen/cli.mjs"`
- `.gitignore`: `.cache/`, `.env` 보호(이미 있으면 중복 추가하지 않음)
- `.github/workflows/ci.yml`: 기존 Node 테스트 명령에 `scripts/package-map-gen/*.test.mjs` 추가. 생성기 실호출은 추가하지 않음
- `crates/core/data/package-map.json`: 기존 형식의 앱 내장 대응표
- `crates/core/data/package-map.provenance.json`: 추가 값별 근거 sidecar, 앱에는 내장하지 않음
- `.cache/package-map-gen/{sources.json,review.json,...}`: 로컬 카탈로그·통계·후보·응답·checkpoint, 커밋하지 않음

## Review Focus

1. **잘못된 kind 힌트·채널 경계(Task 1·2):** neovim/vim 유형의 후보는 남고 review로 가야 한다. stable/nightly는 제외되며 unknown/mixed를 확정 불일치로 보지 않아야 한다. 식별자 경계와 insider 동의어를 fixture로 확인한다.
2. **잘린 후보가 HIGH로 승격되는 경우(Task 1):** 같은 repo가 여러 개인데 cap·필터 후 하나만 남아도 HIGH로 바꾸면 안 된다. 버전 고정·repo 충돌 fixture로 확인한다.
3. **오프라인·할당량·비밀정보(Task 2):** offline은 fetch/git 네트워크 0회, 503 재시도도 요청 상한에 포함되어야 한다. 헤더 누락·잘못된 응답·키가 포함된 예외가 승인·키 출력으로 이어지지 않는지 주입한 transport와 로그 검사가 판정한다.
4. **검토 사이 데이터 변경(Task 2):** 오래된 review가 기존 값·null을 덮거나 같은 매니저 이름을 중복 추가하면 안 된다. apply 직전 변경·대소문자 충돌·Scoop `main/` 별칭 fixture로 확인한다.
5. **측정과 실제 안전성 혼동(Task 3):** 후보 존재·높은 확률·CI 이름 확인만으로 동일 제품이라 단정하지 않는다. 실제 데이터 diff의 출처·채널·종류와 미완료 수는 사람이 확인하고, in-sample 수치를 운영 보증으로 쓰지 않는다. 최종 정책의 held-out 성능은 미검증으로 남긴다.

### Task 1: 카탈로그와 후보 (PR `feat/package-map-gen-candidates`)

- [ ] TDD: 작은 formula/cask, winget 다중 버전·locale/installer, Scoop bucket fixture를 만들고 파서·최신 버전 선택·sources 해시·offline 캐시 누락을 테스트한다. 실제 저장소나 jeb를 부르지 않는다.
- [ ] `catalogs`, `normalize`, `candidates`, `filter`를 구현한다. repo/URL/name 정규화, HIGH 중복·버전 고정, tier 순·cap 12, 채널 drop·kind flag를 테스트로 고정한다. spike 로직만 이식하고 캐시·결과는 복사하지 않는다.
- [ ] CLI의 입력 선택·후보 출력, 루트 `package-map:gen` 명령, 캐시 ignore를 연결한다. analytics/scan/names 합치기, 이미 대응된 항목 건너뛰기, 알 수 없는 token과 formula/cask 동명이인을 명시적으로 보고하는 테스트를 둔다.
- [ ] CI Node 테스트 glob을 확장한다. `node --test scripts/*.test.mjs scripts/package-map-gen/*.test.mjs`, `git diff --check`가 exit 0이고 lockfile이 바뀌지 않아야 한다. fixture 실행을 두 번 돌려 후보 출력이 같은지 확인한다. 외부 판정자는 Node 테스트와 PR CI다.
- 커밋: `feat: add package catalog cache and normalization`, `feat: generate deterministic package map candidates`, `ci: test package map generator modules`

### Task 2: jeb 판정과 파일 검토 (PR `feat/package-map-gen-judge`)

- [ ] TDD: 주입한 fetch·타이머로 선택 확률 해석, 마지막 none, 잘못된 응답, 503 backoff, 최대 동시 2, 요청 상한·quota floor·헤더 누락 중단, 요청 해시 재사용·checkpoint 재개를 검증한다. `.env` 파서는 값 실행 없이 환경변수 우선·따옴표 처리를 테스트하고 키가 로그·오류·캐시에 없는지 검사한다.
- [ ] `judge`, `classify`를 구현한다. 유일 HIGH 호출 0회, 0.80 경계 포함, kind flag 선택은 review, none도 review, 빈 후보는 no-candidate, 오류는 미완료임을 표 기반 테스트로 확인한다. offline은 캐시만 재사용한다.
- [ ] `apply`, `review`와 CLI 실행·`apply-review`를 연결한다. 기존 값/null 보존, 잘못된 id·이름·결정 거부, case-insensitive 유일성·정렬, auto가 null을 쓰지 않음, 사람 none만 null, human provenance, 빈 결정·skip 무변경, 재적용 멱등성을 테스트한다. 쓰기 전 전체 검증 실패 시 두 데이터 파일 모두 기존 내용이 유지되는지 확인한다.
- [ ] 비밀정보 없는 합성 fixture로 후보 → 판정 → review 편집 → 적용 → 재실행을 끝까지 검증한다. Node 전체 테스트·`git diff --check`·PR CI를 통과한다. 실 jeb 성능은 이 PR의 mocked 테스트로 검증됐다고 하지 않는다.
- 커밋: `feat: judge package map candidates with jeb`, `feat: apply package map decisions with provenance`, `feat: support file-based package map review`

### Task 3: 첫 데이터 실행 (별도 데이터 PR)

- [ ] 구현 PR이 main에 병합된 뒤 새 브랜치에서 통계·명시 이름 또는 제공된 스캔 JSON으로 실행한다. 키를 출력하지 않고 실제 요청 수·잔여 credits·snapshot 날짜·캐시 재사용·미완료를 기록한다. 중단되면 같은 checkpoint로 재개하고 실패를 none으로 바꾸지 않는다.
- [ ] 첫 PR은 auto 승인 데이터만 반영한다. review는 로컬 파일에 남겨 후속 사람이 검토하고, no-candidate와 미검토 none은 표에 쓰지 않는다. 추가 값별 provenance와 기존 값 보존을 diff로 확인한다. 실제 auto 추가 수를 보고하며 목표 수를 맞추려고 threshold를 낮추지 않는다.
- [ ] `node --test scripts/*.test.mjs scripts/package-map-gen/*.test.mjs`, `pnpm --filter @repot/desktop build` 후 `cargo test --workspace`, `git diff --check`를 실행한다. 실제 데이터 형식의 판정자는 Rust 데이터 테스트, 이름 존재의 판정자는 기존 CI `package-map` 작업, 의미적 동일성의 판정자는 공개 카탈로그 근거를 읽는 사람이다. CI green이 후자의 대체물은 아니다.
- [ ] PR 본문에 실제 명령 결과·auto/review/no-candidate/미완료/추가 수·정책 한계를 남긴다. 키·review·카탈로그·스캔 원본은 커밋하지 않는다. checks에 pending·실패가 없으면 merge commit으로 병합하고 브랜치·worktree를 정리한다.
- 커밋: `data: expand Windows package mappings with generator provenance`

위 명령과 생성기 동작은 구현 전이므로 **unverified**다. 이 계획 문서 PR의 완료는 스펙 일치·`git diff --check`·docs PR checks·merge commit으로만 판정하며, 구현·실제 jeb 실행·데이터 확대 완료를 뜻하지 않는다.
