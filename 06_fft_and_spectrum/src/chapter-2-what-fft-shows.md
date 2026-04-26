# Chapter 2 - FFT는 무엇을 보여주나

## 1. 한 문장 정의

> N 샘플 FFT는 시간 영역 N 샘플을 받아서 **N개의 주파수 bin**으로 바꿔주는 변환이다.

핵심 사실 몇 가지부터.

```text
입력  : 시간 영역 샘플 N개 (실수 또는 복소수)
출력  : 복소수 bin N개 (각 bin은 주파수 성분 하나의 진폭+위상)
시간  : O(N log N)
인덱스: bin k는 주파수 k · fs / N (k = 0, 1, ..., N-1)
```

여기서 `fs`는 sample rate, `N`은 FFT 크기.

## 2. bin이라는 개념

FFT는 신호를 N개의 **이산 주파수 성분**으로 분해한다. 그 한 개 한 개를 "bin"이라고 부른다.

```text
N = 1024, fs = 48000 Hz일 때

bin 0   : 0 Hz       (DC)
bin 1   : 46.875 Hz  (= 48000 / 1024)
bin 2   : 93.75 Hz
...
bin 512 : 24000 Hz   (Nyquist)
...
bin 1023: -46.875 Hz (실수 입력에서는 bin 1과 켤레, 의미 없음)
```

`bin width = fs / N`이 곧 **주파수 해상도**다. 이게 FFT 사용에서 가장 중요한 한 줄이다.

```text
N이 크면  : 해상도 좋음 (bin 사이가 좁음), 하지만 N 샘플 모이는 데 오래 걸림 (latency 증가)
N이 작으면: 해상도 거침, 하지만 빠르게 frame을 갱신할 수 있음
```

이 트레이드오프는 어쩔 수 없다. spectrum analyzer의 화면 갱신 빈도와 해상도 둘 중 하나는 양보해야 한다.

## 3. 실수 신호일 때의 N/2 + 1

오디오는 실수 신호다. 실수 신호의 FFT는 위쪽 절반(`bin N/2+1` 이후)이 아래쪽 절반의 켤레 복소수가 되어 정보가 중복된다. 그래서 실용적으로는 **bin 0 ~ N/2** 만 보면 된다.

```text
N = 1024 → 의미 있는 bin은 0 ~ 512, 총 513개
```

`rustfft`에는 `RealFft`가 있어 N/2+1 개의 복소수 bin만 직접 출력해 준다. 일반 FFT보다 약 절반의 비용이고, 사용도 더 단순하다.

## 4. complex bin = 두 가지 정보

각 bin은 복소수 한 개다.

```text
bin[k] = Re + j·Im

magnitude(k) = sqrt(Re² + Im²)         ← 그 주파수의 amplitude
phase(k)     = atan2(Im, Re)           ← 그 주파수의 위상
```

spectrum analyzer가 화면에 그리는 막대 높이는 `magnitude(k)`다. phase는 보통 사용자에게 안 보이지만 spectral processing(예: phase vocoder)에서 핵심.

## 5. 직관 — 사인파 한 개를 FFT 하면

440 Hz 사인파를 N = 1024, fs = 48000 으로 FFT 하면, magnitude 그래프는 이렇게 보인다.

```text
magnitude
   ▲
   │
   │            ●     ← 약 9.4번 bin 부근에 큰 값 (440 / 46.875 ≈ 9.39)
   │            
   │
   │
   ───●●●●●●●●●●●●●●●●●●●●●●●●●●●●─── (대부분 거의 0)
   └────────────────────────────► bin
```

피크 하나만 보여야 정상이다. 피크가 여러 bin에 퍼져 있으면 다음 장에서 볼 **leakage** 때문이다.

## 6. amplitude 추정 — 왜 한 bin만 보면 부족할 수 있나

신호 주파수가 정확히 bin 중심에 안 떨어지면, 한 사인파의 에너지가 인접 bin들로 분산된다.

```text
신호가 정확히 bin 중심   : 한 bin에 100% 에너지
신호가 bin 사이          : 인접 bin 두세 개에 분산
```

그래서 정확한 amplitude를 구하려면 단일 bin보다 인접 bin들의 합을 보거나, 보간을 한다. spectrum analyzer 시각화 단계에서는 그 정도 정밀도까진 보통 안 가지만, 측정 도구에서는 중요하다.

## 7. FFT의 latency

FFT 한 번을 하려면 N 샘플이 모여야 한다.

```text
N = 1024, fs = 48 kHz → 1024 / 48000 ≈ 21.3 ms

이 21.3 ms 동안은 다음 FFT를 못 한다 (overlap 없을 때)
```

이게 spectrum 화면의 자연스러운 frame rate 한계다. 21 ms마다 한 번 갱신 ≈ 47 fps. 이 정도면 사람이 부드럽다고 느낀다.

overlap을 도입하면 같은 N으로 더 자주 갱신할 수 있다 — 다음 장에서 본다.

## 8. RuStudio 관점

```text
mod_eq의 spectrum analyzer:
  N = 2048 ~ 4096이 흔함 (해상도와 latency의 균형)
  fs = 48 kHz → bin width 약 11 ~ 23 Hz
  → 저음역(20~200 Hz) 해상도가 부족 → 로그 스케일 시각화로 보완

mastering 미터:
  단순 RMS / LUFS 가 메인. spectrum은 보조.

debugging:
  N을 크게 (8192+) 잡고 정지 신호 분석. latency 무관.
```

## 자주 하는 실수

- bin 인덱스를 그대로 Hz라고 사용. (실은 `k * fs / N`)
- 입력 N과 sample rate를 서로 다른 값으로 두고 bin width를 잘못 계산.
- 복소수 출력을 그대로 amplitude로 시각화 (magnitude 계산 누락).
- 실수 신호인데 일반 FFT를 사용해 절반의 중복 데이터를 처리. RealFft 쓰면 즉시 절반 빠르다.
- N을 2의 거듭제곱이 아닌 값으로 둠 → 일부 FFT 구현은 느려진다 (rustfft는 그래도 동작).

## 반드시 이해해야 할 것

- N 샘플 FFT → N개 (실수 신호는 N/2+1개) 복소수 bin.
- bin width = fs / N. 해상도 vs latency 트레이드오프의 본질.
- bin은 복소수 한 개이며, `magnitude`와 `phase`로 분해해서 사용한다.
- spectrum analyzer가 그리는 것은 `magnitude(k)`다.
- N이 클수록 주파수 해상도는 좋아지지만, 그만큼 모일 때까지 기다려야 한다.
