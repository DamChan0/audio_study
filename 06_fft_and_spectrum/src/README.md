# 들어가며

이 책은 RuStudio 학습 시리즈의 여섯 번째 책이다.

지금까지 우리는 **시간 영역**에서만 신호를 다뤘다. 샘플 하나를 만들고, 합치고, 따라가고, 곱하는 일들이었다.

이 책은 그 신호를 **주파수 영역**으로 옮기는 도구를 본다.

```text
시간 영역(time domain)     : "n번째 샘플의 amplitude는 무엇인가"
주파수 영역(frequency domain): "이 신호 안에 어떤 주파수가 얼마나 들어 있는가"
```

이 두 영역은 같은 신호의 두 가지 보기다. 둘 사이를 오가는 도구가 **FFT**다.

## 이 책이 답하려는 질문

> 콜백 안에서 한 줄씩 흘러가는 `f32` 샘플들로부터, 어떻게 사용자에게 보여줄 spectrum 그래프를 만드는가?

답은 다음 흐름이다.

```text
audio thread (콜백)
   │  매 샘플 ring buffer에 push
   ▼
analysis thread (또는 주기적 task)
   │  N 샘플 모이면 → window 적용 → FFT → magnitude → dB
   ▼
UI thread
   │  결과 spectrum 데이터를 화면에 그림
```

이 흐름의 모든 단계를 본다.

## 이 책이 다루는 것

```text
1. FFT가 무엇을 출력하는가 (bin, magnitude, phase)
2. window function이 왜 필요하고 어떤 종류가 있는가
3. STFT — 시간에 따라 변하는 spectrum
4. magnitude → dBFS 변환
5. analyzer 데이터와 UI 그리기의 분리
6. 실시간 spectrum 표시의 frame rate / 해상도 트레이드오프
```

## 이 책이 다루지 않는 것

```text
✗ FFT 알고리즘 직접 구현 (rustfft crate에 위임)
✗ Goertzel, CQT, wavelet 같은 다른 분석 기법
✗ pitch detection, spectral processing 같은 분석 응용
✗ phase vocoder, time-stretch 같은 spectral 합성
```

## 이 책의 결과물이 어디에 쓰이나

```text
mod_eq의 spectrum analyzer 그래프
mod_mastering의 LUFS 측정에 부수되는 spectrum 표시
debugging — 어떤 신호가 어디로 새는지 시각 확인
```

특히 EQ 화면에서 곡선과 신호 spectrum을 동시에 보여줄 때, 그 spectrum 데이터가 이 책의 결과물이다.

## 이 책을 다 읽고 나면

- FFT가 시간 영역 N 샘플을 어떻게 N/2+1개의 frequency bin으로 바꾸는지 설명할 수 있다.
- bin width가 무엇이고 어떻게 계산되는지 말할 수 있다.
- 왜 windowing 없이 FFT 하면 "스펙트럼 누설"이 생기는지 설명할 수 있다.
- Hann window의 효과를 한 줄로 설명할 수 있다.
- STFT의 hop size / overlap의 의미를 설명할 수 있다.
- magnitude를 dBFS로 변환하는 식을 안다.
- 실시간 spectrum 화면의 frame rate를 어떻게 결정할지 설명할 수 있다.
