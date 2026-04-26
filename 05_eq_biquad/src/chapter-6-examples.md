# Chapter 6 - 예제와 측정

이 장은 05 책의 핵심 코드를 만져 볼 권장 예제 목록과 그 검증 방법을 정리한다.

## 권장 예제 목록

```text
01_biquad_lpf       : cookbook LPF 한 개 적용. 사인파 sweep 입력으로 cutoff 확인
02_biquad_peaking   : peaking EQ 한 개. freq=1k, +6dB, Q=1
03_biquad_chain     : 4-band parametric EQ 직렬. 각 밴드는 사용자 손잡이 4개
04_smoothing_demo   : 같은 EQ를 smoothing on/off로 비교. 클릭 차이 듣기
05_kweighting       : 04 책에서 미뤘던 K-weighting (shelving + HPF) 직접 구현
06_multi_band_split : LPF + HPF 쌍으로 신호를 두 대역으로 쪼개고 합치면 원본인지 확인
```

이 6개를 끝내면 EQ 모듈 본체와 K-weighting까지 손에 들어온다.

## 디렉토리

03/04와 동일한 정책이다.

```text
05_eq_biquad/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml
    src/
      lib.rs               ← Biquad, BiquadCoeffs, Smoothed, EqBand, EqChain
      bin/
        01_biquad_lpf.rs
        02_biquad_peaking.rs
        03_biquad_chain.rs
        04_smoothing_demo.rs
        05_kweighting.rs
        06_multi_band_split.rs
```

## 검증 — 사인파 sweep

이 책의 핵심 검증 도구다.

```text
sweep(t) = sin(2π · f(t) · t)        f(t)가 시간에 따라 log로 증가
```

20 Hz → 20 kHz 까지 5초 동안 부드럽게 올라가는 사인파다. 이걸 EQ에 넣고 출력 amplitude 곡선을 보면 frequency response가 그대로 시각화된다.

```text
입력 sweep amplitude   : 일정 (예: 1.0)
출력 amplitude 곡선     : EQ의 amplitude response 모양

→ 출력의 amplitude envelope이 곧 EQ 곡선이다
```

각 예제의 검증 절차.

```text
01_biquad_lpf
  cutoff = 1 kHz, Q = 0.707
  → 1 kHz 위에서 amplitude가 급격히 감소하는가?
  → 약 12 dB/oct로 떨어지는가?
  → 1 kHz에서 약 -3 dB인가?

02_biquad_peaking
  freq = 1 kHz, gain = +6 dB, Q = 1
  → 1 kHz에서 amplitude가 약 2× (+6 dB)인가?
  → 폭은 약 1옥타브인가?
  → 다른 주파수는 거의 그대로인가?

03_biquad_chain
  4밴드 임의 설정 → 출력 곡선이 4개 봉우리/골이 합쳐진 모양인가?
  
04_smoothing_demo
  freq를 1초 사이에 1k → 4k 로 빠르게 변경
  → smoothing OFF: 클릭 발생, 출력 envelope에 점프
  → smoothing ON : 부드러운 변화, 클릭 없음

05_kweighting
  1 kHz 사인파 vs 100 Hz 사인파, 같은 amplitude로 입력
  → K-weighting 후 두 출력의 RMS 차이가 약 1 ~ 2 dB (1 kHz가 더 큼)

06_multi_band_split
  LPF와 HPF를 같은 cutoff로 두고, 두 출력을 다시 더하기
  → 결과가 원본과 거의 동일해야 함 (Linkwitz-Riley 4차로 갈 수도, Butterworth 2차는 phase 어긋남이 있음)
```

특히 06번은 multi-band compressor 이해의 토대다. "쪼갠 뒤 다시 합쳐도 원본이 돌아온다"는 점이 중요하다.

## 검증 보조 도구

직접 FFT로 amplitude response를 그릴 수도 있다 (06_fft_and_spectrum 책 도구).

```text
방법: white noise 1초를 EQ에 통과시키고 출력의 spectrum을 FFT로 봄
→ 그 모양이 EQ 곡선
```

이 방법이 sweep보다 빠르지만, 06 책의 도구가 갖춰진 다음에 적용하면 된다.

## 모든 예제의 콜백 골격

```rust
let stream = device.build_output_stream(
    &config,
    move |data: &mut [f32], _| {
        for frame in data.chunks_mut(channels) {
            let s = sweep.next();        // 또는 wav 입력

            // (1) 파라미터 smoothing 진행
            let f0 = freq_smooth.next();
            let g  = gain_smooth.next();
            let q  = q_smooth.next();

            // (2) 계수 갱신 (콜백 안에서 OK; 매 샘플 또는 매 N 샘플)
            let c  = peaking(f0, fs, g, q);
            band.biquad.set_coeffs(c);

            // (3) 처리
            let y  = band.biquad.process(s);

            for ch in frame.iter_mut() { *ch = y; }
        }
    },
    err_cb,
    None,
)?;
```

스테레오에선 채널마다 별도 Biquad 인스턴스에 같은 계수를 적용하는 식이다 (stereo-linked EQ).

## 자주 하는 실수

- sweep 입력의 amplitude를 1.0보다 너무 크게 잡아 EQ 출력에서 클립 → response가 평평해 보임. 입력은 -6 dB 정도.
- 좌/우 채널 같은 Biquad 인스턴스 공유 → 좌우 섞임.
- 04_smoothing_demo에서 smoothing ON 결과만 듣고 비교 → OFF도 같이 들어야 차이가 인식됨.
- 06_multi_band_split에서 LPF/HPF로 단순 Butterworth 2차 사용 후 합산 → cutoff 부근 phase 어긋남으로 dip 발생. 이건 "왜 그러는가"의 흥미로운 시작점.
- sample rate를 코드 어디선가 다르게 사용 → cutoff가 의도한 위치에 안 옴.

## 반드시 이해해야 할 것

- sweep 입력은 EQ 검증의 기본 도구다. 출력 envelope이 곧 amplitude response.
- 모든 예제는 03/04 콜백 골격에 EQ 처리만 끼워 넣는 구조다.
- 04 smoothing 예제는 OFF 결과를 일부러 듣고 클릭을 인지하는 데 의의가 있다.
- 06 multi-band split은 다음 책 단계(특히 multi-band compressor)의 기초가 된다.
