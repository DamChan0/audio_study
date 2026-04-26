# Chapter 9 - 자주 하는 실수와 복습

## 흔한 실수 한 번 더

이 책에서 등장한 함정을 한 페이지에 모아 둔다. 다음 책으로 넘어가기 전 한 번 훑는다.

### 샘플/프레임/채널

- 샘플과 프레임을 같은 단어로 쓴다.
- 채널 수를 2로 하드코딩한다.
- `data.len()`이 프레임 수라고 가정한다 (실은 `samples = frames × channels`).

### oscillator

- phase를 매 콜백마다 0으로 초기화한다 → 클릭.
- 샘플레이트를 44100으로 하드코딩 → 다른 장치에서 음정이 어긋난다.
- 시간 t를 누적해 `sin(2π·f·t)`를 그대로 쓴다 → 정밀도 손실.

### gain / dB / pan / mixer

- dB 값을 그대로 곱한다.
- pan을 단순 `(1-x, x)`로 한다 → 센터에서 -6 dB 빠진다.
- mixer 합산 후 amplitude가 1.0을 넘는데 그냥 cpal로 보낸다 → 클리핑.
- 콜백 안에서 매 샘플 `db_to_linear()` 같은 비싼 변환 호출.

### envelope

- attack/decay/release를 sample 단위로 환산하지 않고 ms로만 들고 있다.
- `Idle` 상태가 없어서 끝난 voice도 mixer에 합산된다.
- attack=0 같은 0초 입력에서 0으로 나누기.

### delay buffer

- 콜백 안에서 `Vec::resize`/할당.
- write/read wrap 누락 → panic.
- feedback ≥ 1.0 → 발산.
- stereo 채널이 같은 buffer 공유.

## 6개 블록 입출력 상태표 (반드시 머리에 있어야 한다)

```text
oscillator   input: 없음            output: 샘플          state: phase
gain         input: 샘플            output: ×linear       state: 없음
pan          input: 모노 샘플       output: 스테레오 쌍   state: 없음
mixer        input: N 신호          output: 합 신호       state: 없음
envelope     input: 트리거          output: amp 계수      state: stage, level
delay        input: 샘플            output: 원본+과거     state: 버퍼+인덱스
```

이 표가 다음 책들의 새로운 블록(compressor, biquad, FFT)을 분석할 때 그대로 쓰인다.

## Phase 2 체크리스트

```text
□ DSP가 cpal과 무엇이 다른지 한 문장으로 설명 가능
□ 샘플/프레임/채널/버퍼를 헷갈리지 않고 사용 가능
□ phase accumulator로 사인파를 그릴 수 있음
□ dB ↔ linear 변환을 콜백 밖에서 처리할 수 있음
□ equal-power pan law를 적용할 수 있음
□ 두 source를 mixer로 합쳐 클리핑 없이 출력 가능
□ ADSR envelope를 한 voice에 적용해서 클릭 없이 발음/감쇠 가능
□ feedback delay로 짧은 echo를 콜백-안전하게 구현 가능
□ 모든 예제가 콜백 안에서 실시간 규칙을 위반하지 않음
```

이 9개가 통과하면 Phase 2의 기초 빌딩 블록은 손에 들어왔다고 봐도 된다.

## 다음 책으로 넘어가는 다리

다음 책은 `04_mod_mastering_math`다.

이 책에서 만든 6개 블록이 다음과 같이 다시 등장한다.

```text
gain / dB                  → compressor의 makeup gain, limiter ceiling
mixer                      → 마스터 버스
envelope                   → compressor envelope follower (attack/release 동일 구조)
delay buffer (look-ahead)  → look-ahead limiter, true-peak ISP
phase accumulator          → 직접 등장하지 않지만 다른 LFO/sweep test 신호 생성에 사용
```

즉 04에서 나오는 compressor/limiter/LUFS는 새로운 알고리즘이라기보다 **이 책의 블록을 다른 방식으로 조합한 것**이다. 이 시각으로 보면 04 책이 한결 가벼워진다.

## 한 줄 요약

> DSP는 콜백 안에서 매 샘플 어떤 숫자를 만들지 결정하는 일이다. 모든 처리는 (입력, 출력, 상태) 세 항목으로 분해된다. 이 책의 6개 블록이 그 분해의 기본 어휘다.
