# Chapter 5 - Gain, dB, Pan, Mixer

이 장은 **amplitude를 직접 다루는 가장 기초 연산** 4개를 모은다.

- `gain`     : 한 신호의 크기를 곱셈으로 조절
- `dB`       : gain을 사람 귀에 맞는 로그 스케일로 표현
- `pan`      : 한 신호를 좌우 채널에 분배
- `mixer`    : 여러 신호를 합산

이 4개는 전부 stateless다. 곱셈/덧셈만 있으면 된다.

## 1. Gain — 그냥 곱셈이다

```rust
let out = input * gain;
```

이게 전부다.

- `gain == 1.0` → 입력 그대로
- `gain == 0.5` → 절반 크기
- `gain == 0.0` → 무음
- `gain == 2.0` → 두 배 크기 (`-1.0..1.0` 범위를 넘어가면 클리핑됨)

이 줄이 단순해 보여서 우습지만, **DAW의 모든 채널 페이더가 결국 이 한 줄**이다. 페이더 하나당 곱셈 하나.

## 2. dB — gain을 로그로 본 것

사람의 청각은 amplitude에 대해 **로그적**으로 반응한다. linear 0.5는 0.25보다 "두 배 크다"고 느껴지지 않고, 약간 큰 정도로 느껴진다. 그래서 amplitude를 직관적으로 다루려면 로그 스케일이 필요하다.

dB는 그 로그 스케일이다.

```text
dB = 20 · log10(linear)

linear = 10^(dB / 20)
```

외울 필요는 거의 없고, 자주 쓰는 환산표 몇 개면 된다.

```text
+6 dB   ≈  ×2.0    (대략 두 배)
 0 dB   =  ×1.0    (변화 없음)
-6 dB   ≈  ×0.5    (대략 절반)
-12 dB  ≈  ×0.25
-20 dB  =  ×0.1
-∞  dB  =  ×0.0    (무음)
```

코드로는 두 줄이다.

```rust
fn db_to_linear(db: f32) -> f32 { 10f32.powf(db / 20.0) }
fn linear_to_db(lin: f32) -> f32 { 20.0 * lin.log10() }
```

### dB 값을 더하면 안 되고, linear 값을 곱한다

dB는 **표시용**이다. 실제 곱셈은 linear에서 한다.

```rust
// 나쁜 예
let new_db = current_db * 0.5;        // 의미 없음

// 좋은 예
let lin = db_to_linear(current_db);
let out = input * lin;
```

dB끼리 "더하기"는 의미가 있다. `+6 dB`와 `-3 dB`를 합치면 `+3 dB`다 (= linear 곱셈 ×1.41).

## 3. Pan — 채널 분배에는 "법칙"이 있다

mono 신호를 stereo 출력으로 보낼 때, 단순히 L=신호, R=신호로 보내면 안 된다.

```rust
// 나쁜 예: 가운데(센터)일 때 신호가 ×2.0이 됨
let left  = input * (1.0 - pan);   // pan = 0.0이면 left=1.0
let right = input * pan;           // pan = 1.0이면 right=1.0
// pan = 0.5일 때 left=0.5, right=0.5 → 합쳐 들으면 절반 amplitude
```

이걸 그대로 쓰면 가운데 위치(`pan = 0.5`)에서 소리가 양쪽 모두 -6 dB로 줄어든다. 좌/우 끝에서는 풀 amplitude. 결과적으로 패닝이 움직일 때 음량이 흔들린다.

이 문제를 해결하는 방식이 **pan law**다. 가장 흔한 두 가지.

```text
-3 dB equal-power pan law:
  left  = cos(pan · π/2)
  right = sin(pan · π/2)

  pan = 0.0 → (1.0, 0.0)
  pan = 0.5 → (0.707, 0.707)   ← 두 채널 합 power 일정
  pan = 1.0 → (0.0, 1.0)
```

```rust
fn equal_power_pan(pan: f32) -> (f32, f32) {
    let p = pan.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2;
    (p.cos(), p.sin())
}
```

DAW마다 -3 dB / -4.5 dB / -6 dB 등 약간씩 다르다. 핵심은 "센터에서 양쪽 합산 음량이 끝과 같아야 한다"는 원칙이다.

## 4. Mixer — 그냥 덧셈이다

여러 source를 한 버퍼에 합치려면 더하면 된다.

```rust
for i in 0..frames {
    out[i] = src_a[i] + src_b[i] + src_c[i];
}
```

이게 끝이다. 다만 두 가지 함정이 있다.

### Headroom이 사라진다

각 source가 `-1.0..1.0` 범위라도, 셋을 더하면 최악의 경우 `-3.0..3.0` 범위가 된다. 그게 cpal로 그대로 나가면 클리핑되고, "지직"하는 디지털 왜곡이 생긴다.

대처법은 두 가지.

```rust
// (a) 각 source에 미리 gain 적용 (대표적으로 1/N 또는 사용자가 정함)
out[i] = src_a[i] * 0.33 + src_b[i] * 0.33 + src_c[i] * 0.33;

// (b) 마스터에 limiter / compressor를 둔다  ← 04_mod_mastering_math
```

전문적으로는 (b)다. 입력 단에서 amplitude를 깎으면 정보가 줄어든다. 마스터에서 dynamics 처리로 잡는 것이 보통이다.

### 실시간 합산은 항상 같은 길이여야 한다

source가 끝나서 길이가 안 맞을 때, source 쪽에서 0으로 패딩해서 줘야 mixer가 단순해진다. mixer가 source 길이를 분기 처리하기 시작하면 코드가 빠르게 망한다.

## RuStudio 관점

```text
mod_player 콜백 안:
  source[0..N] (oscillator/sample 등)
       │
       ├─ envelope · gain · pan
       ▼
  mixer (sum)
       │
  master gain (linear, 사용자 dB → linear 변환된 값)
       │
  output buffer
```

이 그림에서 dB는 UI에 표시되는 값이고, 실제 콜백에서는 항상 linear gain으로만 곱한다. dB ↔ linear 변환은 콜백 밖, 파라미터 갱신 시점에 한 번만.

## 자주 하는 실수

- dB 값을 그대로 곱한다 → 무조건 잘못된 결과.
- pan을 `(1-x, x)`로 한다 → 센터에서 음량이 -6 dB 빠진다.
- mixer 출력이 1.0을 넘는데 그냥 cpal에 넘긴다 → 디지털 클리핑.
- 콜백 안에서 매 샘플 `db_to_linear()`를 호출 → `powf`는 비싸다. 파라미터 변경 시점에 한 번만 변환해서 atomic 등으로 전달한다.

## 반드시 이해해야 할 것

- gain은 그냥 곱셈이다. 모든 페이더는 곱셈 하나다.
- dB는 표시용 로그 스케일. 실제 처리에서는 항상 linear로 변환해서 곱한다.
- pan은 "왼쪽 비율, 오른쪽 비율"이 아니라 "양쪽을 합쳤을 때 power가 일정해야 한다"는 제약이 있다.
- mixer는 단순 합산이지만 headroom 관리가 따라온다. 마지막 단계에 dynamics가 있어야 안전하다.
