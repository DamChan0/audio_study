# Chapter 4 - Compressor 구조

compressor는 **큰 신호일수록 amplitude를 더 깎아 동적 범위를 좁히는 처리**다.

```text
큰 소리는 더 크게 깎고
작은 소리는 그대로 둔다
→ 결과적으로 큰소리/작은소리 차이가 줄어든다
→ 그 다음 makeup gain으로 전체 음량을 끌어올린다
→ 평균 음량이 커진 결과가 나온다
```

이 책에서는 가장 단순한 single-band feed-forward 컴프를 본다.

## 4개 핵심 파라미터

```text
threshold (dB) : 이 amplitude를 넘는 부분만 깎는다
ratio          : 넘은 부분을 몇 분의 1로 깎느냐 (예: 4:1)
attack  (ms)   : "신호가 커졌다"고 감지한 후 깎기 시작하는 데 걸리는 시간
release (ms)   : 신호가 다시 작아진 뒤 깎기를 풀어주는 데 걸리는 시간
```

여기에 보조로 두 개가 더 있다.

```text
knee   (dB)    : threshold 근처에서 부드럽게 전환되는 영역
makeup (dB)    : 깎인 만큼 다시 끌어올리는 고정 gain
```

## 4-파라미터를 한 그림으로

threshold = -20 dB, ratio = 4:1을 예로 들자.

```text
out (dB)
   0 ─────────────────────●  ← 이 위는 절대 못 가게 limiter가 잡음
                       ╱  
                     ╱     ← ratio 4:1 영역 (4 dB 들어가면 1 dB 나옴)
   -20 ────────●────       ← threshold
              ╱
            ╱             ← 1:1 영역 (그대로)
          ╱
        ╱
  -∞  ╱
       └──────────────────► in (dB)
       -∞   -20    0
```

수식으로는 이렇다.

```text
if input_db <= threshold:
    output_db = input_db                  ← 그대로
else:
    over = input_db - threshold
    output_db = threshold + over / ratio  ← 줄어든 양만 더해짐
```

ratio = ∞면 한계 위쪽이 평평한 직선, 즉 limiter가 된다.

## 블록 다이어그램

```text
input ──┬─► [ envelope follower ]──► [ static curve ]──► gain (linear)
        │           │
        │           ▼
        │       attack/release
        │
        └────────────────────────────────────────────────┐
                                                         ▼
                                                       (×) ── makeup gain ──► output
```

핵심은 흐름이 두 갈래라는 점이다.

```text
1. 신호의 amplitude를 따라가는 측정 가지 (sidechain)
2. 그 결과로 만든 gain 계수를 원 신호에 곱하는 처리 가지
```

이 분리가 컴프의 모든 변형(parallel, mix, sidechain ducking 등)의 토대다.

## envelope follower — 컴프의 심장

03 책에서 ADSR envelope를 보면서 "값이 시간에 따라 부드럽게 따라간다"는 패턴을 익혔다. 컴프 envelope follower는 그 패턴을 그대로 쓴다.

```rust
struct EnvFollower {
    state: f32,              // 현재 amplitude 추정값 (linear)
    attack_coeff: f32,       // 0.0 ~ 1.0
    release_coeff: f32,
}

impl EnvFollower {
    fn next(&mut self, sample: f32) -> f32 {
        let amp = sample.abs();
        let coeff = if amp > self.state {
            self.attack_coeff       // 빠르게 따라감
        } else {
            self.release_coeff      // 천천히 풀어줌
        };
        self.state = coeff * self.state + (1.0 - coeff) * amp;
        self.state
    }
}
```

여기서 보아야 할 점.

- `state`가 입력보다 작으면 attack로 빠르게 따라가고, 크면 release로 천천히 내려간다.
- 시간 상수 `coeff`는 `exp(-1 / (time_seconds * sr))` 형태로 계산하지만, 이 책 단계에서는 `0.0 ~ 1.0` 사이 값을 직접 튜닝해도 된다.
- 출력은 linear amplitude (0~1+ 범위). dB로 변환은 그 다음 단계에서 한다.

## static curve — gain 계수 만들기

envelope이 만든 amplitude를 dB로 변환해 위의 식을 적용하고, 그걸 다시 linear gain으로 바꾼다.

```rust
fn compute_gain(env_amp: f32, threshold_db: f32, ratio: f32) -> f32 {
    let env_db = 20.0 * env_amp.max(1e-6).log10();
    let target_db = if env_db <= threshold_db {
        env_db
    } else {
        threshold_db + (env_db - threshold_db) / ratio
    };
    let gain_db = target_db - env_db;     // 깎아야 할 양
    10f32.powf(gain_db / 20.0)
}
```

`gain_db`는 항상 0 이하다 (안 깎으면 0, 깎으면 음수). 이 값을 linear로 바꿔 원 신호에 곱한다.

## makeup gain은 고정이다

threshold = -20, ratio = 4 컴프가 -10 dB 신호를 -10 → -12.5 dB로 깎았다고 하자. 이걸 다시 평균 음량으로 끌어올리려면 +x dB 정도의 makeup gain을 마지막에 곱한다.

```rust
let out = input * compressor_gain * makeup_gain_linear;
```

makeup gain은 입력 amplitude와 무관한 고정 곱셈이다. 사용자(또는 자동 makeup 알고리즘)가 정하는 값.

## 한 콜백 안 처리 흐름

```rust
for frame in data.chunks_mut(channels) {
    for sample in frame.iter_mut() {
        let in_x   = *sample;
        let env    = follower.next(in_x);
        let g      = compute_gain(env, threshold_db, ratio);
        *sample    = in_x * g * makeup_linear;
    }
}
```

이게 single-band peak-detection feed-forward compressor의 골격이다. 60줄도 안 된다.

## 변형들 (이 책 범위 너머의 풍경)

같은 골격에 옵션을 끼우는 식으로 다양한 변형이 나온다.

```text
RMS detection      : envelope follower 입력에 절대값 대신 sample^2 사용
look-ahead         : 신호를 N 샘플 지연시켜서 envelope 측정과 시간 정렬 (delay 사용)
soft knee          : threshold 근처에서 부드러운 곡선
multi-band         : 신호를 EQ로 N개 대역으로 쪼개고 각 대역에 컴프 1개씩
sidechain          : envelope follower 입력을 다른 신호로 (예: 킥에 맞춰 베이스 ducking)
parallel           : 컴프 결과를 원 신호와 일정 비율로 섞기
```

전부 위 골격의 일부만 바꾼 것이다.

## 자주 하는 실수

- envelope follower 출력을 그대로 신호에 곱한다 → 신호가 envelope 모양으로 변형된다. envelope은 입력이고, 출력은 그것을 통한 *gain 계수*다.
- threshold를 linear 값으로 두고 dB와 비교 → 단위가 안 맞는다.
- ratio = 1 컴프를 "안 깎는다"고 생각 → 0이 아니라 1이 "안 깎음"이다. 0으로 두면 0으로 나누기.
- log10(0) → -∞. envelope amplitude가 0일 때 NaN이 된다. `.max(1e-6)` 같은 floor 필요.
- attack/release를 ms 단위로만 들고 다니고 콜백마다 매번 `exp` 호출 → 비싸다. 파라미터 변경 시점에 한 번만 coeff로 변환.

## 반드시 이해해야 할 것

- 컴프는 input을 보면서 "amplitude를 따라가는 가지"와 "그 결과로 만든 gain을 곱하는 가지" 두 갈래로 동작한다.
- envelope follower는 03 책의 envelope와 같은 구조다. attack/release 시간상수가 곧 IIR 계수다.
- ratio = ∞ + 빠른 attack = limiter다. 다음 장에서 그 변형을 본다.
- 컴프는 신호를 "찌그러뜨리는" 게 아니라 "신호 amplitude를 따라가는 gain 신호를 만들어 곱하는" 것이다.
