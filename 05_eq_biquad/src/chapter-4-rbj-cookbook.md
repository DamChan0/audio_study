# Chapter 4 - RBJ Cookbook 핵심

이 장은 biquad의 5개 계수 `(b0, b1, b2, a1, a2)`를 **사용자 친화적인 손잡이 (freq, gain, Q)**에서 만들어내는 표준 공식을 본다.

## 1. RBJ Audio EQ Cookbook이란

Robert Bristow-Johnson이 1990년대 후반 정리한 "각 EQ 모양에 대해 biquad 계수를 어떻게 계산할지"의 한 페이지짜리 정리표다. 이게 사실상 디지털 오디오의 표준이다.

W3C가 Web Audio API의 BiquadFilterNode 사양에 그대로 준용한 버전을 공개해 두었다.

```text
https://www.w3.org/TR/audio-eq-cookbook/
```

이 책은 이 cookbook을 그대로 쓴다. 직접 도출하지 않는다. 디지털 오디오에선 "이미 검증된 식을 그대로 쓰는 게 정석"이다.

## 2. 모든 cookbook 계수가 공유하는 사전 변수

각 EQ 모양 공식은 다음 공통 변수에서 출발한다.

```text
f0  : center / cutoff frequency (Hz)
fs  : sample rate (Hz)
gain: dB (peaking/shelving에서만 사용)
Q   : 폭 또는 강조도

A   = 10^(gain / 40)        (shelf/peaking gain factor)
w0  = 2π · f0 / fs
cw  = cos(w0)
sw  = sin(w0)
alpha = sw / (2 · Q)
```

이 다섯 줄이 모든 공식의 머리다.

## 3. 6가지 표준 모양의 계수

각 식을 외울 필요는 없다. 이 책에서는 "어떤 식의 결과로 나오는지" 정도만 잡고, 코드에는 그대로 옮긴다.

### Low-Pass (LPF, 12 dB/oct)

```text
b0 = (1 - cos(w0)) / 2
b1 =  1 - cos(w0)
b2 = (1 - cos(w0)) / 2
a0 =  1 + alpha
a1 = -2 · cos(w0)
a2 =  1 - alpha
```

### High-Pass (HPF, 12 dB/oct)

```text
b0 =  (1 + cos(w0)) / 2
b1 = -(1 + cos(w0))
b2 =  (1 + cos(w0)) / 2
a0 =   1 + alpha
a1 =  -2 · cos(w0)
a2 =   1 - alpha
```

### Band-Pass (BPF, constant 0 dB peak gain)

```text
b0 =  alpha
b1 =  0
b2 = -alpha
a0 =  1 + alpha
a1 = -2 · cos(w0)
a2 =  1 - alpha
```

### Notch

```text
b0 =  1
b1 = -2 · cos(w0)
b2 =  1
a0 =  1 + alpha
a1 = -2 · cos(w0)
a2 =  1 - alpha
```

### Peaking EQ

```text
b0 =  1 + alpha · A
b1 = -2 · cos(w0)
b2 =  1 - alpha · A
a0 =  1 + alpha / A
a1 = -2 · cos(w0)
a2 =  1 - alpha / A
```

### Low-Shelf

```text
b0 =     A · ((A+1) - (A-1)·cos(w0) + 2·sqrt(A)·alpha)
b1 = 2 · A · ((A-1) - (A+1)·cos(w0))
b2 =     A · ((A+1) - (A-1)·cos(w0) - 2·sqrt(A)·alpha)
a0 =         (A+1) + (A-1)·cos(w0) + 2·sqrt(A)·alpha
a1 =    -2 · ((A-1) + (A+1)·cos(w0))
a2 =         (A+1) + (A-1)·cos(w0) - 2·sqrt(A)·alpha
```

### High-Shelf

```text
b0 =     A · ((A+1) + (A-1)·cos(w0) + 2·sqrt(A)·alpha)
b1 =-2 · A · ((A-1) + (A+1)·cos(w0))
b2 =     A · ((A+1) + (A-1)·cos(w0) - 2·sqrt(A)·alpha)
a0 =         (A+1) - (A-1)·cos(w0) + 2·sqrt(A)·alpha
a1 =     2 · ((A-1) - (A+1)·cos(w0))
a2 =         (A+1) - (A-1)·cos(w0) - 2·sqrt(A)·alpha
```

## 4. a0 정규화

위 공식들은 분모에 `a0`이 따로 나온다. biquad process 함수의 식은 `a0 = 1`을 가정하므로 모든 계수를 `a0`으로 나눠준다.

```rust
let inv_a0 = 1.0 / a0;
self.b0 = b0 * inv_a0;
self.b1 = b1 * inv_a0;
self.b2 = b2 * inv_a0;
self.a1 = a1 * inv_a0;
self.a2 = a2 * inv_a0;
```

이 한 단계를 잊으면 amplitude가 어긋난다.

## 5. Q의 의미는 모양마다 약간 다르다

Q라는 같은 변수가 모양마다 의미가 약간 다르다.

```text
LPF / HPF      : cutoff 부근의 강조 정도. Q=0.707이 평탄 (Butterworth).
BPF / Notch    : center 부근의 폭. Q가 크면 폭이 좁고 가파르다.
Peaking        : peak의 폭. Q가 크면 좁고 날카롭다.
Shelving       : shelving slope를 결정. Q를 직접 노출하지 않고 "S" 슬로프 변수를 쓰는 변종도 있음.
```

UI에서는 LPF/HPF의 Q 손잡이 범위와 Peaking의 Q 손잡이 범위가 일반적으로 다르게 잡힌다.

## 6. peaking EQ의 frequency response를 빠르게 가늠하는 법

가장 자주 쓰는 peaking EQ만 직관 잡아 두자.

```text
freq=1000Hz, gain=+6dB, Q=1
  → 1000 Hz에 +6 dB peak
  → 폭(전후 -3 dB 점)은 약 1 옥타브 정도

Q를 2로 올리면 폭이 약 0.5 옥타브로 좁아짐
Q를 0.5로 내리면 폭이 약 2 옥타브로 넓어짐
gain을 0 dB로 두면 곡선이 평탄해짐 (필터가 사실상 비활성)
```

이 직관이 EQ 손잡이 설계의 기본이다.

## 7. 코드 구조 권장

cookbook 계수 계산은 비싸다 (`cos`, `sin`, `pow`). 매 샘플 호출하면 안 된다.

```rust
struct BiquadCoeffs { b0: f32, b1: f32, b2: f32, a1: f32, a2: f32 }

fn peaking(f0: f32, fs: f32, gain_db: f32, q: f32) -> BiquadCoeffs {
    let a = 10f32.powf(gain_db / 40.0);
    let w0 = std::f32::consts::TAU * f0 / fs;
    let cw = w0.cos();
    let alpha = w0.sin() / (2.0 * q);

    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * cw;
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * cw;
    let a2 = 1.0 - alpha / a;

    let inv = 1.0 / a0;
    BiquadCoeffs { b0: b0 * inv, b1: b1 * inv, b2: b2 * inv, a1: a1 * inv, a2: a2 * inv }
}
```

이 함수는 **콜백 밖**에서 호출된다. 결과 계수만 콜백 안의 Biquad 인스턴스로 전달된다.

## 자주 하는 실수

- a0 정규화 누락 → amplitude가 비례적으로 어긋남.
- 파라미터(freq/gain/Q) 변경 시 매 샘플 cookbook 호출 → 콜백 안에서 `cos`/`sin`/`pow` 부담 + 클릭.
- Q를 모든 모양에 같은 의미로 매핑 → UI에서 "왜 LPF의 Q가 peaking과 다르게 들리지?"
- gain 단위를 linear로 잘못 넣음 → A 계산이 깨짐. cookbook은 항상 dB.
- f0 > fs/2 (Nyquist) 입력 → 식이 발산. UI에서 항상 clamp.

## 반드시 이해해야 할 것

- cookbook 계수는 외우지 않는다. 코드에 옮기고, 검증하고, 재사용한다.
- 모든 모양은 같은 사전 변수(`A, w0, cw, alpha`)에서 나온다.
- 계수 계산은 비싼 연산이다. 콜백 밖에서, 파라미터 변경 시점에만 한다.
- a0 정규화는 잊지 않는다.
- Q는 모양마다 의미가 약간 다르다는 점을 기억한다.
