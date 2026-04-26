# Chapter 7 - 자주 하는 실수와 복습

## Filter / Biquad 일반

- "EQ는 그래픽 도구"라고만 이해. 시간 영역 차분 방정식이 그 그래프를 만든다는 사실을 잊는다.
- z1, z2를 콜백 시작마다 0으로 reset → 매 콜백 시작에 클릭.
- 채널마다 별도 인스턴스가 필요한데 공유 → 좌우가 섞여 모노가 됨.
- a0 정규화 누락 → amplitude가 비례적으로 어긋남.

## Cookbook

- 매 샘플 cookbook 호출로 cos/sin/pow를 부른다 → CPU 폭발.
- gain을 linear로 주고 cookbook을 호출 → A 계산이 깨진다 (cookbook은 dB).
- f0 > Nyquist → 식 발산. UI에서 항상 clamp 필요.
- Q를 모든 모양에 같은 직관으로 매핑 → LPF/HPF의 Q와 peaking의 Q가 의미가 다르다.

## Smoothing

- 사용자 슬라이더 값을 콜백에서 직접 읽음 → atomic이 없으면 race.
- target이 바뀌면 current도 같이 점프 → smoothing 의미 무시.
- 매 샘플이 아니라 매 콜백마다만 smoothing → 콜백 크기에 의존하는 동작.
- gain만 smoothing하고 freq/Q는 그대로 → freq/Q 클릭만 남음.

## 처리 블록 입출력 상태표

```text
Biquad (DF2T)        input: 샘플       output: 샘플          state: z1, z2
EqBand               input: 샘플       output: 샘플          state: Biquad 상태 + 계수
EqChain              input: 샘플       output: 샘플          state: Vec<EqBand>
Smoothed param       input: target     output: smoothed val  state: current
peaking() (cookbook) input: f0/g/Q     output: 계수 5개      state: 없음 (pure 함수)
```

## Phase 4 체크리스트

```text
□ frequency response를 그림으로 그릴 수 있다
□ biquad의 차분 방정식을 보고 어떤 항이 입력/출력 메모리인지 짚을 수 있다
□ Direct Form II Transposed 6줄 process 함수를 외움 없이 쓸 수 있다
□ peaking EQ의 freq=1k, +6dB, Q=1 곡선을 머리로 그릴 수 있다
□ cookbook 호출이 콜백 밖이어야 하는 이유를 설명할 수 있다
□ 파라미터 smoothing이 왜 필요한지 클릭 발생 메커니즘으로 설명할 수 있다
□ 4-band parametric EQ 직렬 사슬을 그림으로 그릴 수 있다
□ K-weighting이 biquad 두 단이라는 점을 직접 코드로 확인했다
□ multi-band split (LPF + HPF) 후 합산이 원본과 같음을 검증했다 (또는 차이의 이유를 안다)
```

## 03/04 책 블록의 재사용 지도

```text
03 phase accumulator → sweep tone 생성에 사용 (검증용)
03 envelope          → parameter smoothing의 기반 패턴 (1차 IIR)
03 delay buffer      → biquad의 z1/z2 자체가 1샘플 지연을 두 번 사용한 형태
03 gain              → cookbook 결과의 a0 정규화에 사용
04 K-weighting       → 이 책에서 직접 구현 (biquad 2단)
04 multi-band 컴프 split → 06 예제로 시연
```

## 다음 책으로 넘어가는 다리

다음 책은 `06_fft_and_spectrum`이다.

이 책에서 만든 EQ의 frequency response를 직접 시각화하려면 FFT가 필요하다. white noise를 EQ에 넣고 출력의 spectrum을 보면 그 모양이 정확히 EQ 곡선이다.

또한 06 책은 spectrum analyzer UI 구성요소의 데이터 생성 흐름을 다룬다. 11 책의 UI와 합쳐 사용자 EQ 화면이 완성된다.

## 한 줄 요약

> EQ는 frequency response 그래프이고, biquad는 그 그래프를 만드는 6줄짜리 시간 영역 코드다. cookbook이 freq/gain/Q에서 계수 5개를 만들고, smoothing이 사용자 손잡이 변동을 부드럽게 받아 준다.
