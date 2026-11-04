# Repot 계획 13: Repot 파일에 내보내지 못한 항목도 담기

**Goal:** 목적지에서 쓸 수 없어 내보내기에서 빠진 항목도 `repot.json`에 "이번 내보내기에 없음" 표시와 함께 저장한다. 그 파일을 다른 OS에서 열면 그 항목도 다시 내보낼 수 있다. 사용자가 직접 뺀 항목은 지금처럼 저장하지 않는다.

**Architecture:** v1 형식에 선택 필드 `notExported`(기본 false)를 더한다. 내보내기는 고른 항목 외에 목적지에서 사용할 수 없는 항목을 표시와 함께 저장한다. 열 때 그 id 목록을 세션과 UI로 넘기고, UI는 처음 선택과 비교 후 선택에서 그 항목을 켜지 않는다.

**Tech Stack:** Rust, React, TypeScript, Vitest

**Spec:** `docs/specs/2026-11-02-repot-mvp-design.md` §2·10·11·15

## Global Constraints

- 새 의존성 없음.
- v1 안의 변경은 선택 필드 추가뿐이다. 고정 예시 `tests/fixtures/snapshot/v1-basic.json`은 고치지 않고 그대로 열려야 한다.
- 이전 버전 앱(0.5~0.6)은 모르는 필드를 무시하므로 `notExported` 항목을 보통 항목처럼 보여준다. 이 차이는 받아들인다(형식을 깨지 않는 쪽이 낫다).
- 기기를 바꾸지 않는다. 쓰는 곳은 내보내기 폴더뿐이다.

## 결정과 이유

**무엇을 더 저장하나.** 이번 목적지에서 사용할 수 없는 항목(다른 OS 전용 앱, 대응표에 없는 패키지, 다른 셸 설정 등)만 더한다. 이런 항목은 화면에서 고를 수 없었으므로 사용자가 뺀 것이 아니다. 사용자가 체크를 끈 항목은 일부러 뺀 것이라 저장하지 않는다. 읽을 수 없던 항목과 스크립트에 쓰기 위험한 값이 든 항목도 저장하지 않는다.

**설정 파일.** 내보내지 않은 설정 파일도 내용이 있어야 나중에 다시 내보낼 수 있다. 내보낸 파일과 같은 방식(출발 기기 형태, 비밀정보 가림)으로 저장한다. 원래 값 보이기는 고를 수 없었으므로 모두 가린다.

**열 때 처음 선택(2026-11-04 사용자 결정).** `notExported` 항목은 사용 가능해져도 처음에 켜지 않고 "이전 내보내기에 없던 항목" 표시만 붙인다. "이 기기와 비교" 후의 처음 선택에서도 켜지 않는다. 사용자가 그 항목을 고른 적이 없으므로, 원칙 3("의심스러우면 보수적으로")에 따라 직접 켜게 한다.

**다시 내보낼 때.** 열린 파일에서 내보내도 같은 규칙을 쓴다. 고른 항목은 표시 없이, 이번 목적지에서 쓸 수 없는 항목은 표시와 함께 저장한다. 표시된 항목을 켜지 않고 내보내면 사용자가 뺀 것으로 보고 새 파일에서는 빠진다.

## 형식 변경

```json
{ "id": "brew:cask:rectangle", "...": "...", "notExported": true }
```

- `notExported`는 true일 때만 쓴다. 없으면 false다.

## 모델·명령 변경

- core: `snapshot::StoredItem { item, config, not_exported }`, `render_snapshot(.., &[StoredItem])`, `Snapshot.not_exported: BTreeSet<String>`.
- 데스크톱: `OpenedFile.notExported: string[]`.

## Review Focus

1. 고른 항목, 사용자가 뺀 항목, 목적지에서 쓸 수 없는 항목이 각각 맞게 저장·제외되는가(같은 OS, 다른 OS).
2. 저장한 설정 파일의 비밀정보가 가려져 있는가.
3. 고정 v1 예시가 그대로 열리고, `notExported` 왕복이 유지되는가.
4. 열었을 때와 비교 후에 `notExported` 항목이 켜지지 않고, 사용자가 켤 수 있는가.
5. 네 로캘에 새 문구가 있는가.

### Task 1: 계획·스펙과 core (PR `feat/snapshot-not-exported`)

- 커밋: `docs: plan storing items an export left out`, `feat(core): keep items an export could not include in repot.json`

### Task 2: 데스크톱 (PR `feat/opened-not-exported-ui`)

- 커밋: `feat(desktop): leave items from outside the last export unchecked`
