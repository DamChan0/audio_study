# Chapter 4 - Oscillator와 위상 누산기

oscillator는 **입력 없이 매 샘플마다 파형 값을 만들어내는 소스**다.

```text
oscillator
  input  : (없음)
  output : f32 샘플 한 개
  state  : 현재 위상(phase, 0.0 ~ 1.0)
```

이 책에서 가장 먼저 등장하는 stateful DSP다.

## 위상이 무엇인가

사인파를 그릴 때 우리가 흔히 보는 식은 이거다.

```text
y(t) = sin(2π · f · t)
```

여기서 `2π · f · t` 부분이 위상(phase)이다. 시간 t가 흐를수록 위상이 선형으로 증가하고, sin이 그 위상을 받아서 -1~+1 사이를 오간다.

문제는 t를 그대로 쓰면 안 된다는 점이다.

```rust
// 나쁜 예
let t = elapsed_seconds_since_start;
let y = (2.0 * PI * 440.0 * t).sin();
```

이 코드는 시간이 지날수록 t가 커진다. `f64`라도 결국 정밀도가 깎이면서 위상이 어긋나기 시작한다. 게다가 주파수를 바꾸려면 t의 의미가 깨진다.

해결책은 t를 더 누적하지 않고, **위상 자체를 0~1 범위 안에서 누적**하는 것이다.

## 위상 누산기(phase accumulator)

```text
매 샘플마다:
  phase += frequency / sample_rate
  phase = phase - floor(phase)        // 0~1 범위로 환원
  output = sin(2π · phase)
```

핵심은 두 줄이다.

```rust
phase += freq / sr;
phase = phase.fract();
```

이게 왜 안전한가?

- `freq / sr`은 "한 샘플당 위상이 얼마나 늘어나는가"를 의미한다. 예를 들어 440 Hz / 48000 Hz = 0.00917, 즉 한 샘플마다 위상이 0.917% 진행한다.
- `fract()`로 정수 부분을 떼어내면 `phase`는 영원히 0~1 사이에 머문다. 정밀도가 망가지지 않는다.
- 주파수를 바꾸려면 `freq` 변수만 바꾸면 끝이다. 과거 시간을 다시 계산할 필요가 없다.

## sine oscillator 최소 구현

```rust
struct SineOsc {
    phase: f64,
    sr: f64,
}

impl SineOsc {
    fn new(sr: f64) -> Self {
        Self { phase: 0.0, sr }
    }

    fn next(&mut self, freq: f64) -> f32 {
        let s = (self.phase * std::f64::consts::TAU).sin() as f32;
        self.phase = (self.phase + freq / self.sr).fract();
        s
    }
}
```

여기서 주의할 점.

- `phase`는 `f64`. `f32`로 쓰면 누적 오차가 빨리 보인다.
- `sr`은 oscillator가 처음 생성될 때 받는 값. 콜백 안에서 매번 묻지 않는다.
- `freq`는 매 샘플 인자로 받는다. 즉 vibrato나 pitch envelope을 적용해도 그대로 동작한다.

## 다른 파형은 어떻게?

`phase`(0~1) 하나만 정확하면 모든 파형을 그 위에 올릴 수 있다.

```rust
fn saw(phase: f64) -> f32 {
    (2.0 * phase - 1.0) as f32
}

fn square(phase: f64) -> f32 {
    if phase < 0.5 { 1.0 } else { -1.0 }
}

fn triangle(phase: f64) -> f32 {
    let p = phase * 4.0;
    let v = if p < 2.0 { p - 1.0 } else { 3.0 - p };
    v as f32
}
```

다만 saw / square처럼 불연속이 있는 파형을 그대로 만들면 **앨리어싱**이 생긴다. 이건 PolyBLEP 같은 보정으로 다음 단계에서 다룰 주제고, 지금 단계에서는 "이건 그냥 두면 고주파 노이즈가 끼는구나" 정도만 인지하면 된다.

## RuStudio 관점

oscillator는 두 가지 위치에서 등장한다.

```text
1. 신디사이저 source       : note_on 시점에 freq를 받아 발음
2. LFO (Low Frequency Osc) : 다른 파라미터(amplitude, pitch, filter)를 흔드는 변조원
```

둘 다 위상 누산기 패턴은 동일하다. 다만 LFO는 보통 주파수가 0.1~20 Hz 수준으로 낮고, 그 출력이 직접 들리는 게 아니라 다른 값을 변조한다.

## 자주 하는 실수

- `phase` 변수를 콜백 안에서 매번 새로 만든다 → 매번 0에서 시작해서 소리가 클릭으로 끊긴다.
- 샘플레이트를 44100으로 하드코딩 → 48000 장치에서 음정이 8% 정도 어긋난다.
- 위상을 `f32`로 누적 → 몇 초만 지나도 정밀도가 깨진다.
- 주파수를 바꾸겠다고 매번 phase를 0으로 reset → 클릭 노이즈가 매번 발생한다. 주파수만 바꾸고 phase는 유지해야 한다.

## 반드시 이해해야 할 것

- oscillator의 상태는 phase 하나다. 시간(t)이 아니다.
- 위상은 0~1 범위로 누적하고 `fract()`로 환원한다. 이 패턴이 모든 oscillator의 골격이다.
- 주파수를 바꾸는 일과 phase를 reset하는 일은 다른 일이다. 섞으면 클릭이 난다.
- LFO든 audio-rate oscillator든 같은 구조를 쓴다. 주파수 범위만 다르다.
