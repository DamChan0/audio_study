# Chapter 1 - 이 책을 읽는 방법

이 책은 두 가지 다른 영역의 만남을 다룬다.

```text
오디오 분석 (analysis)  : 신호를 보고 숫자(=spectrum)를 만드는 일
UI 렌더링                : 그 숫자를 화면에 픽셀로 그리는 일
```

이 둘을 한 함수에 섞어 쓰기 시작하면 빠르게 무너진다. 이 책 전체에서 끊임없이 의식할 분리다.

## 매 챕터에서 답해야 할 질문

```text
1. 이 단계의 입력은 무엇인가? (시간 영역 샘플? 주파수 bin들?)
2. 이 단계의 출력은 무엇인가? (시간 영역 샘플? bin들? dB 배열?)
3. 이 단계가 콜백 안에서 도는가, 콜백 밖에서 도는가?
4. 이 단계의 latency는 얼마인가? (FFT 한 번 하는 데 N 샘플 필요)
```

## 분석 파이프라인 한 장 그림

이 책 전체가 결국 이 그림이다.

```text
audio buffer (시간 영역 샘플)
    │
    ▼
[ ring buffer / FIFO ]            ← audio thread → analysis thread 다리
    │
    ▼
[ window function (Hann 등) ]
    │
    ▼
[ FFT ]
    │
    ▼
[ magnitude calculation ]
    │
    ▼
[ dBFS 변환 ]
    │
    ▼
[ optional: smoothing, peak hold ]
    │
    ▼
spectrum 데이터 (Vec<f32>, bin 별 dBFS)
    │
    ▼
[ UI rendering ]
```

각 단계를 별도의 chapter로 쪼갤 수도 있지만, 이 책에서는 위 6개 단계를 4개 chapter에 나눠 담는다.

## 추천 독서 순서

1. `Chapter 2 - FFT는 무엇을 보여주나` — bin, magnitude, phase, sample rate와의 관계.
2. `Chapter 3 - Window와 STFT` — leakage, window 종류, hop / overlap.
3. `Chapter 4 - dBFS와 시각화용 데이터` — magnitude → dB, smoothing, peak hold, UI 분리.
4. `Chapter 5 - 예제` — 사인파 / 두 톤 / window on-off / STFT 흐름.
5. `Chapter 6 - 복습`.

## 한 가지 주의

FFT의 알고리즘 자체를 이해할 필요는 없다. `rustfft` crate가 검증된 구현을 준다. 이 책의 목적은 "어떻게 사용하느냐"이지 "어떻게 짜느냐"가 아니다.

```toml
[dependencies]
rustfft = "6"
```

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- N 샘플 FFT의 출력은 몇 개의 복소수 bin인가? 실수 신호일 때는?
- bin 0과 bin N/2가 의미하는 주파수는 각각 무엇인가?
- 같은 신호로 N=512 FFT와 N=4096 FFT를 하면 무엇이 어떻게 다른가?
- Hann window를 곱하면 amplitude가 어떻게 변하는가? 보정이 필요한가?
- STFT의 hop=N/2(=50% overlap)는 왜 흔히 쓰는가?
- 1024-bin spectrum을 화면 1024 픽셀에 1:1 매핑하면 왜 자연스럽지 않은가?
