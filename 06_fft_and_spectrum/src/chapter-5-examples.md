# Chapter 5 - 예제와 성능 감각

이 장은 06 책의 처리 흐름을 실제로 만지고 검증할 권장 예제 목록을 정리한다.

## 권장 예제 목록

```text
01_fft_single_tone   : 440 Hz 사인파 → N=2048 FFT → 한 bin 부근 피크 확인
02_fft_two_tones     : 440 + 880 Hz → 두 피크가 분리되는가
03_window_compare    : 같은 신호를 Rectangular / Hann / Blackman 비교
04_stft_pipeline     : ring buffer + STFT + smoothing + dB 출력 흐름
05_spectrogram_dump  : STFT 결과를 텍스트로 덤프해서 시간 변화 확인
```

이 5개를 끝내면 이 책이 다룬 모든 단계가 손에 들어온다.

## 디렉토리

```text
06_fft_and_spectrum/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml          ← rustfft = "6", ringbuf = "0.4" 정도
    src/
      lib.rs            ← Window, Smoother, PeakHold, AnalyzerFrame 등
      bin/
        01_fft_single_tone.rs
        02_fft_two_tones.rs
        03_window_compare.rs
        04_stft_pipeline.rs
        05_spectrogram_dump.rs
```

## 모든 예제의 골격

이 책 예제는 실시간 입력이 필수가 아니다. 대부분 미리 만들어 둔 신호를 분석하는 것이 학습에 더 좋다.

```rust
fn main() {
    let fs: f32 = 48_000.0;
    let n: usize = 2048;

    // 1) 합성 신호
    let signal: Vec<f32> = make_test_signal(fs, n);

    // 2) window 적용
    let win: Vec<f32> = hann_window(n);
    let windowed: Vec<f32> = signal.iter().zip(&win)
        .map(|(s, w)| s * w).collect();

    // 3) FFT
    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let mut buf: Vec<rustfft::num_complex::Complex<f32>> =
        windowed.iter().map(|&x| (x, 0.0).into()).collect();
    fft.process(&mut buf);

    // 4) magnitude → dB
    let db: Vec<f32> = buf.iter().take(n / 2 + 1)
        .map(|c| (20.0 * (c.norm()).max(1e-9).log10()))
        .collect();

    // 5) 결과 출력
    for (k, v) in db.iter().enumerate().take(50) {
        println!("bin {:4} = {:.2} dB", k, v);
    }
}
```

이 골격이 01~03번 예제의 base다. 04~05번은 ring buffer와 hop을 추가한다.

## 검증 — 정량과 시각

### 01번 — 한 톤

입력: 1.0 amplitude × 440 Hz 사인파, N = 2048, fs = 48 kHz, Hann window.

```text
bin width = fs / N = 48000 / 2048 ≈ 23.4 Hz
440 Hz → bin 약 18.78
→ bin 18, 19에 피크가 인접 형태로 보여야 함
→ 피크 dB는 약 -6 dB (Hann coherent gain 보정 안 했을 때)
→ 다른 bin은 -60 dB 이하
```

피크가 여러 bin으로 너무 넓게 퍼져 있으면 window 적용이 빠진 것이다.

### 02번 — 두 톤

같은 N, 1 kHz와 1.05 kHz 두 톤. bin width 23 Hz라서 두 피크가 따로 보일 락 말 듯한 거리.

```text
1 kHz   → bin 약 42.66 (42, 43에 피크)
1.05 kHz → bin 약 44.8 (44, 45에 피크)
→ 둘이 거의 붙어 보이거나 살짝 분리
```

N을 4096으로 키우면 bin width 11.7 Hz. 두 피크가 또렷이 분리된다 — **해상도와 latency의 트레이드오프**의 직관 잡는 데 가장 좋은 예제.

### 03번 — window 비교

같은 한 톤 신호를 Rectangular / Hann / Blackman 으로 처리한 결과를 한 그래프에 겹쳐 그림.

```text
Rectangular: 피크는 좁고 높지만 옆 bin들에 leakage 큼
Hann       : 피크가 약간 넓고 leakage가 거의 보이지 않음
Blackman   : Hann보다 더 넓은 main lobe, 더 깊은 leakage 억제
```

leakage가 시각화에서 어떻게 보이는지 직접 확인하는 게 목적.

### 04번 — STFT 파이프라인

cpal 입력(또는 미리 만든 1초 신호)을 ring buffer로 받아, hop = N/2마다 한 번 FFT하는 흐름. 출력은 각 frame의 spectrum dB 배열.

```text
검증: 시간에 따라 신호 주파수를 sweep시키면 (예: 100 Hz → 5 kHz)
      frame별 spectrum의 피크 위치가 부드럽게 이동하는가?
```

### 05번 — spectrogram dump

04번 결과를 시간축으로 쌓아서 텍스트 파일에 저장(또는 ppm/png). 가로 = frame 인덱스, 세로 = bin, 색 = dB. 진짜 spectrogram의 단순 버전.

## 성능 감각

대략 다음 수치를 머릿속에 두자.

```text
N = 2048, f32 RealFft, 한 채널 → 한 frame 약 30 ~ 100 µs (현대 CPU)
N = 4096                        → 약 60 ~ 200 µs

콜백 호출 주기 (예: 5 ms = 240 frames@48k)에 비해 여유는 큼.
하지만 매 콜백에서 무조건 FFT는 비효율 — hop 단위로만.
```

stereo, multi-channel, 또는 여러 트랙의 spectrum이 동시에 필요하면 그 만큼 곱해진다.

## 콜백 안에서 지켜야 할 규칙

```text
✗ FFT 직접 호출
✗ Vec 새로 할당
✗ smoothing 갱신 (Vec를 만진다는 의미에서)
✓ ring buffer에 push만 (lock-free)

별도 thread / task에서:
✓ FFT, smoothing, dB 변환, peak hold 모두 처리
✓ 결과를 UI 측 double buffer 또는 다른 SPSC로 전달
```

## 자주 하는 실수

- 콜백 안에서 FFT 호출 → underrun.
- ring buffer overflow를 안 처리 → 갑자기 분석이 어긋남.
- N과 fs를 서로 일관 안 두기 → bin width 계산이 다 어긋남.
- window 적용 후 Vec를 매 frame 새로 할당 → 미리 할당해 두면 충분.
- spectrogram을 8-bit PPM으로 dump하면서 dB → 픽셀 매핑을 잊고 magnitude 그대로 → 어둡거나 단색.

## 반드시 이해해야 할 것

- 실제 분석 흐름은 콜백 → ring buffer → 분석 thread → UI 큐 → UI 그리기 다.
- N의 변화가 해상도와 latency에 미치는 영향을 한 톤/두 톤 예제로 직접 본다.
- window 효과는 Rectangular와 비교해야 직관이 잡힌다. 03번 예제가 이 책의 가장 중요한 직관 도구다.
- 성능은 N=2048~4096 한 채널 정도면 큰 부담은 아니다. 채널/트랙이 늘어날수록 hop 정책이 중요해진다.
