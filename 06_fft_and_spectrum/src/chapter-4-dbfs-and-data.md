# Chapter 4 - dBFS와 시각화용 데이터

이 장은 분석 결과를 화면이 받아 쓸 수 있는 모양으로 가공하는 단계다.

```text
[ FFT magnitude bin들 ]
     │
     ▼
  dBFS 변환
     │
     ▼
  smoothing / peak hold
     │
     ▼
  화면용 spectrum 데이터 (Vec<f32>)
```

## 1. magnitude → dBFS

FFT에서 나온 magnitude는 0 ~ ∞ 범위 linear 값이다. 이걸 사람이 보기 좋은 dB 스케일로 바꾼다.

```rust
let db = 20.0 * mag.max(1e-9).log10();
```

`max(1e-9)`는 log10(0)을 막기 위한 floor다. 결과는 보통 -100 ~ 0 dBFS 범위에 들어온다.

여기서 dBFS의 정확한 정의는 약간 미묘하다. 이상적으로 amplitude 1.0의 사인파를 N-bin FFT 하고 적절한 보정을 하면 한 bin이 0 dBFS가 된다. 측정 정밀도가 필요한 경우 다음 보정을 곁들인다.

```text
- window coherent gain 보정 (Hann이면 1/0.5 = 2)
- N에 의한 amplitude 스케일 보정 (rustfft는 N으로 나누지 않음)
- single-sided spectrum이면 위쪽 켤레 절반 보상 (×2 또는 ×sqrt(2))
```

이 책 단계에서 화면 시각화는 위 보정을 정밀히 하지 않아도 보기 좋게 나온다. 측정 도구라면 차근차근 다 적용한다.

## 2. spectrum smoothing

화면이 매 frame마다 들썩이면 보기 어렵다. 시각화에서는 보통 시간축으로 부드럽게 한다.

```rust
struct Smoother {
    state: Vec<f32>,           // bin별 현재 값
    coeff: f32,                // 0.0 ~ 1.0, 1에 가까우면 천천히
}

impl Smoother {
    fn update(&mut self, new_db: &[f32]) {
        for (s, n) in self.state.iter_mut().zip(new_db.iter()) {
            *s = self.coeff * *s + (1.0 - self.coeff) * n;
        }
    }
}
```

coeff = 0.6 ~ 0.8 정도가 자연스럽다. 너무 크면 반응이 느리고, 너무 작으면 들썩인다.

03 책의 envelope follower와 같은 구조다 — bin마다 1차 IIR이 하나씩 도는 셈이다.

## 3. peak hold

각 bin의 최근 N ms 동안의 최대값을 잠깐 잡고 천천히 떨어뜨리는 표시.

```text
실선        : 현재 (smoothed) spectrum
점선/라인   : peak hold spectrum
```

코드 골격.

```rust
struct PeakHold {
    peaks: Vec<f32>,           // bin별 peak (dB)
    fall_rate_db_per_s: f32,   // 예: -12 dB/s
}

impl PeakHold {
    fn update(&mut self, new_db: &[f32], dt: f32) {
        for (p, n) in self.peaks.iter_mut().zip(new_db.iter()) {
            *p = (*p - self.fall_rate_db_per_s * dt).max(*n);
        }
    }
}
```

비교: peak meter도 같은 패턴이었다 (04 책 chapter 3). 단지 한 값이 아닌 bin 배열에 적용된다는 차이.

## 4. 화면 매핑 — 선형 vs 로그

여기가 시각화에서 가장 중요한 단계다.

bin 인덱스는 주파수 축에서 **선형**이다.

```text
N = 2048, fs = 48 kHz → bin 0~1024
bin 0   = 0 Hz
bin 100 ≈ 2.34 kHz
bin 200 ≈ 4.68 kHz
bin 1000 ≈ 23.4 kHz
```

선형 spacing이라 저음역(0~200 Hz)이 적은 bin에, 고음역(10~20 kHz)이 많은 bin에 들어간다. 그런데 음악적으로는 그 반대다 — 사람은 옥타브(=주파수 비율) 단위로 듣는다.

그래서 화면은 거의 항상 **로그 주파수 축**으로 그린다.

```text
선형 화면:
  | 0 Hz ────────────────────────── 24 kHz |
   ▲ 저음 영역이 너무 좁음

로그 화면:
  | 20 ─ 100 ─ 1k ─ 10k ─ 24k |
   ▲ 옥타브 단위로 균등
```

화면 픽셀 → 주파수 매핑.

```rust
fn pixel_to_freq(px: usize, width: usize, f_lo: f32, f_hi: f32) -> f32 {
    let log_lo = f_lo.log10();
    let log_hi = f_hi.log10();
    let log_f = log_lo + (log_hi - log_lo) * (px as f32 / width as f32);
    10f32.powf(log_f)
}
```

각 픽셀에 대해 그 주파수에 해당하는 bin을 찾고(또는 인접 bin들 사이를 보간), 그 bin의 dB 값을 화면에 그린다.

## 5. dB 축의 매핑

세로축은 dB지만, 보통 -90 ~ 0 dBFS 범위만 쓴다.

```text
y_norm = (db - db_min) / (db_max - db_min)
```

db_min = -90, db_max = 0 정도. clamp(0, 1)로 바깥은 잘라낸다.

## 6. 색

bin 값에 따라 색을 다르게 줄 수도 있고(spectrogram), 단색 라인으로 그릴 수도 있다(전형적 spectrum). 둘은 위 단계의 데이터를 어떻게 시각화하느냐의 차이일 뿐이다.

```text
spectrum line  : 1D 배열 → 화면에 한 가닥 라인
spectrogram   : 시간축 추가 → 가로 시간, 세로 주파수, 색은 dB
```

## 7. analyzer thread <-> UI thread 데이터 전달

UI 스레드가 spectrum 배열에 접근할 때마다 lock 걸면 빠르게 부하가 생긴다. 표준은 lock-free 이중 버퍼나 SPSC.

```text
analyzer 스레드:
  새 spectrum 계산 → write buffer 갱신 → atomic으로 "최신 = write" 플래그 변경

UI 스레드:
  매 frame 호출 → atomic 플래그로 read buffer 결정 → 그 buffer 그림
```

자세한 구조는 11 책에서 다룬다. 여기서는 "콜백/분석/UI 셋이 각각 별 스레드"라는 점만.

## 8. 분석 데이터 구조 권장 모양

UI에 넘기는 spectrum 데이터는 다음 정도면 충분하다.

```rust
struct SpectrumFrame {
    sample_rate: f32,
    n: usize,                  // FFT size
    bins_db: Vec<f32>,         // length = N/2 + 1, dBFS
    peak_hold_db: Vec<f32>,    // 같은 길이
    timestamp: u64,            // 분석 시점 (UI smoothing용)
}
```

UI는 화면 매핑 단계만 책임진다 — 분석 결과는 거의 그대로 받고 픽셀에 옮겨 그리는 일.

## 자주 하는 실수

- magnitude만 보고 dB 변환 누락 → 작은 값들이 안 보임.
- log10(0)에서 NaN. floor `1e-9` 또는 `1e-12` 누락.
- bin 인덱스 그대로 화면 가로축에 매핑 → 저음역이 거의 안 보임.
- smoothing을 audio 콜백 안에서 → 콜백 안에서 N개 짜리 Vec 갱신 부담.
- spectrum 배열을 매 frame `new`로 할당 → GC 없는 언어라도 매 frame heap 할당은 부담. 미리 할당된 두 버퍼 재사용.
- peak hold의 떨어지는 속도를 sample rate 기반으로 계산 → 화면 frame rate가 더 자연스러운 단위.

## 반드시 이해해야 할 것

- magnitude → dBFS 변환은 `20 · log10(max(mag, 1e-9))`.
- 시각화에서 가로축은 거의 항상 로그 주파수다. bin 그대로 매핑하면 저음이 죽는다.
- smoothing과 peak hold는 시각화 단계의 표준 도구다 (1차 IIR + 떨어지는 max).
- 분석 결과는 별도 데이터 구조로 UI에 넘긴다. UI는 매핑/렌더링만 담당.
