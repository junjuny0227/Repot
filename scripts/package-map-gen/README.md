# 패키지 대응표 생성기 실행

앱·CI와 분리된 개발자 도구다. 기준 정책과 한계는 계획 10을 따른다.

```sh
pnpm package-map:gen candidates --names ripgrep --offline
pnpm package-map:gen run --names brew:tool,brewCask:app --dry-run --max-requests 20
pnpm package-map:gen run --from-analytics 10 --threshold 0.95 --credit-floor 100
pnpm package-map:gen apply-review --dry-run
pnpm package-map:gen apply-review
pnpm package-map:gen apply-review --reviewer agent
```

- `--from-scan scan.json`, `--from-analytics N`, `--names a,b`는 합쳐 선택한다.
- `--dry-run`도 실제 jeb 요청을 할 수 있고 응답 캐시·checkpoint·review를 쓴다. 두 추적 데이터 파일만 쓰지 않는다. `counts.added`는 이때 **추가 예정 수**다.
- 완전 오프라인은 `--offline`이다. 카탈로그·통계·jeb 캐시가 없으면 값을 만들지 않는다. HIGH 우회는 캐시된 카탈로그만으로 가능하다.
- `run`의 `counts`는 출발/목적지 쌍별 auto/review/no-candidate/incomplete와 추가/충돌을 구분한다. incomplete 또는 입력 오류가 있으면 종료 코드 1이며, 이미 끝난 결정은 캐시로 재개한다. `jeb`는 이번 실행의 HTTP 시도 수(재시도 포함), 캐시 사용 수, 마지막으로 관측한 credits다. 잔액을 관측하지 않았으면 null이다.
- 키는 `JEB_AI_HUB_KEY` 환경변수 우선, 없으면 이 checkout 루트 `.env`에서 데이터로 읽는다. 키·헤더·응답 원문은 저장하거나 출력하지 않는다. worktree도 별도 checkout이므로 루트 `.env`가 없으면 환경변수를 사용한다.

## 캐시와 중단

기본 캐시는 `.cache/package-map-gen/`이며 `--cache-dir`로 바꾼다. 카탈로그를 갱신하려면 기존 snapshot을 별도로 보관하고 다시 취득한다. 이전 snapshot에는 Scoop 설명이 없을 수 있다. 이때 설명을 꾸미지 않고 빈 값으로 요청한다.

`responses/<sha256>.json`은 키를 제외한 공개 요청 본문의 해시로 주소를 정한다. 선택 번호와 선택 확률만 저장한다. threshold는 다시 적용하므로 캐시를 버릴 필요가 없다.

`checkpoint.json`은 UTC 날짜, 그 날짜의 HTTP 예약 수, 마지막 credits와 중단 상태를 저장한다. 요청 **전**에 예약하며 하루 최대 1000회다. `--max-requests`는 이번 실행의 추가 시도 수를 제한한다. 같은 캐시를 재사용해야 날짜별 제한과 재개가 유지된다. 다른 캐시나 다른 프로그램에서 소비한 일일 요청은 로컬에서 알 수 없으며 서버 credits 헤더가 추가 제동 장치다.

요청은 직렬(동시 최대 2보다 보수적)이며 503은 최대 7회, 1/2/4/8/16/32초 backoff로 시도한다. credits가 floor 미만이면 다음 요청을 보내지 않는다. 헤더 누락·판독 실패, transport 실패·중간 종료는 할당량을 모르는 상태라 같은 UTC 날짜에는 중단한다. 완료된 응답 캐시는 계속 재사용할 수 있다. 다음 UTC 날짜에는 로컬 예약을 초기화하고 서버 잔액을 다시 확인한다. 캐시/대응표 lock이 있으면 다른 실행이 끝난 후 재개한다. 강제 종료가 남긴 lock은 실제 실행이 없는지 확인한 뒤에만 수동 제거한다. checkpoint를 지워 예산을 초기화하지 않는다.

## 파일 검토

`review.json`의 `items`는 출발 token·종류, 목적지, 전체 후보 메타데이터, jeb 선택·확률, snapshot 날짜와 빈 문자열 `decision`을 갖는다. `--review-file path`로 다른 파일을 쓸 수 있다.

- 빈 문자열 또는 `skip`: 무변경
- 목록의 후보 id: 검토자가 고른 대응
- `none`: 검토자가 확인한 부재, 대응표에 null
- 선택적 `evidence`: 확인한 근거 문자열. 적용한 값의 provenance에 복사한다

`--reviewer human`(기본)은 provenance 방법 `human`, `--reviewer agent`는 `agent-review`로 기록한다. 에이전트 결정 규칙은 계획 10의 "누가 review를 결정하는가"를 따른다.

다른 값은 전체 적용 전에 오류다. 같은 근거로 재실행하면 편집한 결정을 보존하고, 근거가 바뀌면 그 쌍의 결정을 빈 값으로 초기화한다. 이번 실행에 안 나온 이전 항목은 보존한다. 검토 이후 대응표가 바뀌어도 apply는 최신 파일을 다시 읽고 기존 문자열/null과 case-insensitive 유일성을 지킨다. 재적용은 멱등적이다.

## provenance 스키마

처음 값을 추가할 때 `crates/core/data/package-map.provenance.json`이 생긴다. 기존 수동 대응에는 근거를 소급 생성하지 않는다. 앱의 `package-map.json` 형식은 바뀌지 않는다.

```json
{
  "version": 1,
  "packages": {
    "example": {
      "winget": {
        "method": "jeb",
        "probability": 0.9,
        "tier": "MEDIUM",
        "catalogSnapshot": "2026-11-03T00:00:00.000Z"
      },
      "scoop": {
        "method": "agent-review",
        "probability": null,
        "tier": "LOW",
        "catalogSnapshot": "2026-11-03T00:00:00.000Z",
        "evidence": "Same upstream repo: https://github.com/example/example"
      }
    }
  }
}
```

`method`는 `high-tier | jeb | human | agent-review`다. HIGH·human·agent-review의 probability는 모델 확률이 없으므로 null이며 jeb만 선택 확률을 갖는다. tier는 선택한 후보의 `HIGH | MEDIUM | LOW`, human none처럼 후보가 없으면 null이다. `evidence`는 review 항목에 비어 있지 않은 문자열이 있을 때만 생긴다. `catalogSnapshot`은 원본 카탈로그의 snapshot 날짜이며 적용 시각을 대신 쓰지 않는다. 공통 id와 provenance 매니저 키는 정렬한다.

자동 none·후보 없음·실패는 대응표 null을 만들지 않는다. 새 공통 id가 token 소문자 `[a-z0-9-]+`에 맞지 않거나 충돌하면 건너뛰어 보고한다. 잘못된 이름·전체 유일성 검증 실패는 두 파일을 쓰기 전에 오류다. 두 파일의 임시 파일을 먼저 만들고 rename하며, 두 번째 rename 실패는 sidecar를 복구한다. 프로세스/전원 강제 종료까지 두 파일의 원자적 transaction을 보장하는 것은 아니므로 데이터 PR의 두 diff를 함께 확인한다.

높은 확률·패키지 이름 존재·CI 통과는 제품 동일성이나 held-out 정밀도 보증이 아니다. 네트워크 없는 단위 테스트도 실제 jeb 성능의 검증이 아니다.
