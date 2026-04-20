# Chapter 12 - 다음 단계: mod_player 아키텍처 스케치

이제 `cpal` 자체 학습은 1차로 충분하다.

다음부터는 `cpal` API를 더 파는 것보다, 그 위에 `mod_player`를 어떤 경계로 얹을지 설계하는 편이 훨씬 중요하다.

## 왜 지금 넘어가도 되나

현재까지 학습한 범위로 이미 아래 핵심은 확보됐다.

- `Host -> Device -> Stream` 구조 이해
- output callback이 실제 오디오 실행 지점이라는 점
- 실시간 오디오 스레드 규칙 이해
- 440Hz 사인파를 최소 성공 기준으로 보는 이유 이해
- DSP chain 삽입 포인트가 어디쯤 와야 하는지 이해

직접 문서화된 사실:

- `cpal` 학습 책의 복습 장은 이미 "이제 `cpal` 자체 학습은 1차 완료"라고 정리한다.
  - 출처: `study/01_cpal/src/chapter-10-review.md`
- 예제 crate에는 `silence`, `sin_sound`, `dsp_chain` 같은 실습 진입점이 이미 있다.
  - 출처: `study/01_cpal/examples/src/bin/`

근거 기반 해석:

- 지금부터 더 중요한 것은 새 `cpal` API를 더 수집하는 것이 아니라, `mod_player`의 소유권/상태/제어 흐름 경계를 정하는 것이다.

## 왜 다음 챕터가 아키텍처여야 하나

예제 crate를 보면 이미 `cpal` 기초를 넘어서 "구조를 어떻게 잡아야 하나" 문제가 보이기 시작한다.

직접 문서화된 사실:

- `examples/src/chain.rs`의 `run()`은 호출마다 `Vec`를 새로 만든다.
  - 출처: `study/01_cpal/examples/src/chain.rs:14-16`
- 프로젝트 규칙과 학습 문서에서는 오디오 스레드에서 힙 할당을 피해야 한다고 명시한다.
  - 출처: `AGENTS.md` 오디오 스레드 규칙
  - 출처: `study/01_cpal/src/chapter-5-realtime-rules.md`

근거 기반 해석:

- 즉 지금 필요한 건 `cpal` 추가 학습보다, "체인을 누가 소유하고 언제 버퍼를 준비할 것인가"를 설계하는 일이다.

## `mod_player`에서 먼저 정해야 할 것

아키텍처 스케치를 시작할 때는 아래 다섯 가지를 먼저 적으면 된다.

### 1. 스트림 소유권

- `Stream`을 누가 소유할 것인가?
- 앱 전체 매니저인가, `mod_player`인가?
- 재생/정지/장치 재선택 시 누가 생명주기를 책임지는가?

### 2. 오디오 스레드가 읽는 상태

- play / pause 상태
- sample rate
- channel count
- 현재 source 상태
- DSP on/off 또는 파라미터 스냅샷

### 3. UI -> audio thread 제어 경로

- atomic으로 충분한 값은 무엇인가?
- channel 메시지가 필요한 값은 무엇인가?
- 구조 변경은 어느 타이밍에 반영할 것인가?

### 4. source와 DSP의 경계

```text
source 생성 -> player transport 반영 -> DSP chain -> output
```

이 순서를 먼저 글로 설명할 수 있어야 한다.

### 5. 실패 상태 표현

- 장치 없음
- 설정 조회 실패
- stream 생성 실패
- 장치 재연결 필요

이 상태를 UI나 상위 앱에 어떤 식으로 올릴지 먼저 정해야 한다.

## 추천 산출물

다음 챕터에서는 코드를 바로 짜기보다 아래 산출물을 만드는 것이 좋다.

1. `mod_player` 책임 정의 1문단
2. 스레드 경계 다이어그램 1개
3. 상태 목록 표 1개
4. play / pause / stop / device change 상태 전이
5. DSP chain 소유 전략 메모

## 여기서 멈춰도 되는 `cpal` 학습과 더 보면 좋은 것

### 지금 멈춰도 되는 것

- 기본 host/device/stream 흐름
- callback 모델
- sample format / channel 처리
- 최소 사인파 출력 예제

### 필요할 때만 추가로 보면 되는 것

- 특정 backend feature (`jack`, `pipewire`, `asio`)
- 고정 buffer size 튜닝
- timestamp 기반 동기화
- input stream 처리

즉 지금 시점에서는 `cpal`을 더 파는 것보다 `mod_player` 설계로 넘어가는 판단이 맞다.

## 다음으로 바로 할 일

이 책 다음 챕터 또는 다음 작업 세션에서 바로 시작하면 좋은 질문은 이거다.

```text
[단계] Phase 1 - mod_player
[목표] cpal callback 위에 player architecture 스케치
[막힌 부분] stream 소유권, play/pause 제어, DSP chain 구조 변경
[원하는 것] 설계 방향 제시 + 상태 경계 정리
```

이 단계부터는 "cpal을 더 공부할까"보다 "cpal 위에 어떤 책임 경계를 둘까"가 핵심 질문이 된다.
