# Chapter 2 - mod_mastering의 목표

`mod_mastering`은 RuStudio Phase 1의 메인 모듈이다. 이름이 `mod_player`와 비슷해서 혼동될 수 있는데, 둘은 위치가 다르다.

```text
source(들) → DSP chain → [ mod_mastering ] → output buffer (cpal)
                          └ 이 책이 다루는 곳
```

`mod_player`는 재생을 **운영**하고, `mod_mastering`은 그 신호의 **마지막 단계 다듬기**를 담당한다.

## mod_mastering이 책임지는 것

```text
1. 절대 클립이 나가지 않게 한다 (safety net)
2. 음량을 일정 범위로 정렬한다 (loudness)
3. 최종 신호의 측정값을 UI에 보여준다 (metering)
```

이 셋이 mastering의 일이다. 음악적 "맛 더하기"는 EQ/이펙트의 일이고, mastering은 "안전과 표준에 맞추기"에 가깝다.

## 신호 흐름

mod_mastering 안쪽은 보통 이런 사슬이다.

```text
input
  → meter (입력 측정: peak / RMS / LUFS-pre)
  → equalizer (선택, 책 05의 영역)
  → multiband / single-band compressor
  → makeup gain
  → limiter (안전 마지막)
  → meter (출력 측정: peak / true-peak / LUFS-post)
  → output
```

이 책에서는 multiband 분기는 다루지 않고, 가장 단순한 single-band 사슬을 기준으로 본다.

```text
input → compressor → makeup gain → limiter → output
              │            │           │
              ▼            ▼           ▼
            meter        meter       meter
```

## 왜 이 사슬 순서가 거의 고정인가

순서가 바뀌면 결과가 무너진다.

- **limiter가 compressor보다 먼저** 있으면 limiter가 자주 트리거되어 신호가 항상 찌그러진 상태로 compressor에 들어간다.
- **compressor 뒤에 makeup gain이 없으면** 신호 평균이 그냥 작아진다. dynamics만 좁혀지고 음량은 떨어진다.
- **meter가 limiter 앞에만 있으면** UI는 limiter가 깎기 *전* 값을 보여줘서, 사용자는 "왜 클립인데 안 클립이라고 표시되지?"라고 본다.

이 순서 자체가 이 책의 핵심 결과물 중 하나다.

## meter와 processor의 분리

위 그림에서 볼 수 있듯이, 같은 신호 위에 미터와 프로세서가 동시에 걸린다.

```text
신호 ─────► [ meter ]   (관찰만)
       │
       ▼
       [ processor ]    (변형)
       │
       ▼
      output
```

미터는 신호를 **분기해서 측정**하지, 신호를 깎거나 더하지 않는다. 그래서 meter 코드를 짤 때 가장 흔한 실수가 "측정하면서 신호도 같이 바꾸는" 코드다.

## 실시간 vs 오프라인

mastering은 실시간(라이브 모니터)으로도, 오프라인(곡 전체 처리)으로도 쓰일 수 있다.

```text
실시간 : 버퍼 단위로 처리, look-ahead 길이만큼 latency 발생
오프라인: 곡 전체 분석 후 처리, latency 무관, 두 번 패스 가능
```

이 책의 모든 코드 골격은 **실시간 가정**이다. 즉 콜백 안에서 동작 가능해야 한다는 제약이 있다. look-ahead limiter처럼 약간의 지연이 필요하면 그건 명시적으로 모델에 들어가야 한다.

## 03 책과의 연결

이 책의 처리 블록은 03에서 본 도구들을 다른 식으로 조합한 것이다.

```text
peak meter        : amplitude의 max 추적 → envelope follower의 단순화
RMS meter         : 짧은 평균 → moving average ≈ 1차 IIR
compressor        : envelope follower + gain reduction 곡선
limiter           : compressor + 무한대 ratio + 짧은 attack
true-peak         : delay 보간 + peak 측정
LUFS              : 두 단계 IIR 필터 + 블록 평균 + 게이트
```

새 알고리즘은 사실상 없다. **익숙한 블록의 새 조합**이다.

## 반드시 이해해야 할 것

- mod_mastering = "마지막 단계 안전 + 음량 + 측정"의 모듈이다.
- 처리 사슬은 `compressor → makeup gain → limiter`가 기본이고 순서를 바꾸지 않는다.
- 메터(관찰)와 프로세서(변형)를 항상 구분해서 설계한다.
- 이 책의 블록은 03 책의 envelope/gain/delay의 응용이다. 새 어휘가 갑자기 등장하지 않는다.
