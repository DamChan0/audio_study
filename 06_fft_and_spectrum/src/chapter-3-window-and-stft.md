# Chapter 3 - Window와 STFT

## 1. 왜 window가 필요한가

FFT는 입력 N 샘플이 **무한히 반복되는 주기 신호**라고 가정한다. 즉 buffer의 마지막 샘플과 첫 샘플이 자연스럽게 이어진다고 본다.

실제 오디오 buffer는 그 가정이 거의 맞지 않는다. 어떤 위치에서 잘라도 buffer 시작과 끝이 amplitude 점프를 만든다.

```text
원래 신호       : ──╱╲╱╲╱╲╱╲╱╲╱╲──
buffer (잘라낸): ──╱╲╱╲╱╲╱╲╱╲╱╲     ← 끝이 갑자기 끊김
FFT가 보는 신호 : ──╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲╱╲...    ← 끝-시작 사이에 점프 발생
                                ^ 점프 = 인공적 고주파 성분
```

이 점프가 FFT 결과에 **존재하지 않는 주파수의 에너지**를 만들어낸다. 이걸 **spectral leakage**라고 한다.

```text
순수 440 Hz 사인파 → leakage 없으면 한 bin에만 피크
                  → leakage 있으면 인접 bin들로 에너지가 새어나감
```

## 2. window 함수의 역할

window는 buffer 양 끝을 0에 가까운 값으로 부드럽게 깎는 함수다.

```text
window     w(n)
   ▲
   │       ╱──╲
1.0│      ╱    ╲
   │     ╱      ╲
   │    ╱        ╲
0.0└──●──────────●─────► n
       0          N-1
```

신호에 이걸 곱하고 FFT 한다.

```rust
let windowed: Vec<f32> = signal.iter()
    .zip(window.iter())
    .map(|(s, w)| s * w)
    .collect();
fft.process(&windowed);
```

이러면 양 끝이 0이라 끝-시작 점프가 자연스럽게 0으로 이어진다. leakage가 크게 줄어든다.

대가는 buffer 양 끝이 깎이기 때문에 amplitude가 약간 줄어든다는 점. 이건 보정 계수로 메운다.

## 3. 자주 쓰는 window 종류

```text
Rectangular   : 깎지 않음 = window 없음. leakage 큼.
Hann          : 부드러운 cos² 모양. spectrum analyzer 기본.
Hamming       : Hann과 비슷하지만 끝값이 0.08 (완전 0이 아님).
Blackman      : 더 강한 leakage 억제. 대신 main lobe가 넓음.
Kaiser        : 파라미터로 leakage vs lobe 폭 조절 가능.
Flat-top      : amplitude 측정용. peak amplitude가 정확.
```

이 책에서는 **Hann**을 기본으로 쓴다. 시각화에 거의 표준이고 단순하다.

```rust
fn hann(n: usize, len: usize) -> f32 {
    let phase = std::f32::consts::PI * 2.0 * n as f32 / (len - 1) as f32;
    0.5 * (1.0 - phase.cos())
}
```

각 window는 사용 목적이 약간 다르다.

```text
시각화 (analyzer)        : Hann
amplitude 정밀 측정       : Flat-top
좋은 다이내믹 레인지       : Blackman
range 트레이드오프 미세조절: Kaiser
```

## 4. window가 amplitude를 깎는 양 — coherent gain

Hann window의 평균 값은 약 0.5다. 즉 신호에 곱하면 평균적으로 amplitude가 절반이 된다. spectrum의 magnitude도 그만큼 작아진다.

```text
Hann coherent gain ≈ 0.5

→ 측정에서 "amplitude 1.0 사인파"의 spectrum peak이 실제로는 0.5 부근으로 보임
→ 정확한 amplitude를 알고 싶으면 결과 magnitude를 1/0.5 = 2로 곱한다
```

시각화에서는 보정 안 해도 거의 문제없다. 측정 도구라면 보정이 필요.

## 5. STFT — Short-Time Fourier Transform

지금까지는 buffer **하나**를 분석하는 이야기였다. 실시간 spectrum analyzer는 buffer를 시간에 따라 계속 업데이트해야 한다.

STFT는 그 흐름이다.

```text
신호 ───────────────────────────►

frame 1: |───── N 샘플 ─────|
frame 2:        |───── N 샘플 ─────|
frame 3:               |───── N 샘플 ─────|
frame 4:                      |───── N 샘플 ─────|

      hop          hop          hop

각 frame에 window 곱하고 FFT 적용 → spectrum 한 장씩 출력
```

핵심 파라미터.

```text
N (frame size) : FFT 크기
H (hop size)   : 다음 frame 시작이 몇 샘플 뒤인가
overlap        : (N - H) / N (예: H = N/2 → 50% overlap)
```

## 6. 왜 overlap이 필요한가

Hann window는 양 끝을 0으로 깎는다. 그래서 frame 끝에 들어간 신호 정보는 거의 무시된다. overlap 없이 hop = N으로 가면, frame 경계의 신호가 절반쯤 사라진 채 분석된다.

```text
no overlap (H = N):
  frame 1   |▆▇█▇▆▅▄▃▂▁  ▁▂▃▄▅▆▇█▇▆| frame 2
                       ▲
                  여기 신호가 분석에서 약해짐

50% overlap (H = N/2):
  frame 1   |▆▇█▇▆▅▄▃▂▁ |
        frame 2   |▆▇█▇▆▅▄▃▂▁ |
              frame 3   |▆▇█▇▆▅▄▃▂▁ |
        → 어느 시점이든 두 frame이 같이 cover
```

50% overlap (= H = N/2)이 Hann과 가장 잘 맞는 표준이다.

## 7. frame rate와 화면 갱신

화면을 매 frame마다 그릴 필요는 없다. 사람 눈이 인지하는 frame rate는 30 fps 정도면 충분하다.

```text
N = 2048, fs = 48 kHz, hop = 1024
→ 1024 / 48000 ≈ 21.3 ms마다 새 frame
→ 약 47 fps

이 중 30 fps만 화면에 보낸다고 하면, 1.5 frame마다 한 번 그리면 됨.
```

UI는 이렇게 갱신 빈도를 따로 정해서 audio analysis와 분리한다.

## 8. 콜백과 분석의 분리

audio thread (cpal 콜백)에서 직접 FFT 하면 안 된다. FFT는 비싸고 가변적이다.

표준 패턴.

```text
audio thread (콜백):
  매 샘플 → ring buffer에 push (lock-free SPSC)

analysis thread (또는 timer task):
  ring buffer에 N 샘플 모였는지 검사
  모였으면 → window → FFT → magnitude → dB → 결과 큐로
  hop 단위로 반복

UI thread:
  결과 큐에서 최신 spectrum 가져와 그림
```

ring buffer는 02 책에서 본 SPSC 큐다. 03 책의 delay buffer와 같은 자료구조지만 용도가 다르다 — 스레드 간 통신.

## 자주 하는 실수

- buffer를 그냥 FFT (window 미적용) → 사인파인데 leakage로 broadband 모양.
- window 보정 누락 → magnitude가 일관되게 0.5배.
- hop = N으로 overlap 없음 → 화면이 끊겨 보이고 transient 놓침.
- audio 콜백 안에서 FFT 호출 → 비싼 작업이라 underrun 가능.
- 매 frame마다 화면 그리려고 시도 → 60 Hz 모니터에선 의미 없는 부담.
- ring buffer를 lock-protected로 → SPSC lock-free가 표준.

## 반드시 이해해야 할 것

- window가 없으면 leakage가 생긴다. spectrum analyzer는 거의 항상 Hann.
- STFT는 windowed FFT를 hop 간격으로 반복하는 흐름이다.
- 50% overlap (Hann + H = N/2)이 시각화 표준 조합.
- audio 콜백 / FFT 분석 / UI 렌더링은 스레드를 분리한다.
