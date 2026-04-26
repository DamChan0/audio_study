# Chapter 3 - 오실레이터와 위상 누산

이 챕터의 목표: **Sine / Saw / Square / Triangle 네 파형을 같은 골격으로 만들 수 있다.**

## 핵심 아이디어: 위상 누산기

모든 기본 파형은 **"0~1 사이를 반복하는 값"** 하나만 있으면 만들 수 있다. 그 값이 phase 다.

```text
phase  : 0.00 -> 0.25 -> 0.50 -> 0.75 -> (wrap) -> 0.00 ...
sine   : sin(2π * phase)
saw    : 2.0 * phase - 1.0
square : if phase < 0.5 { 1.0 } else { -1.0 }
triangle: 1 - 4 * |phase - 0.5|   (대략)
```

## 위상 증가량

```text
phase_increment = frequency / sample_rate
```

- sample_rate 48000, frequency 440 이면 샘플마다 phase 를 `440/48000 ≈ 0.009166` 만큼 진행.
- 이 값을 매 샘플 더하고, 1.0 을 넘으면 `phase -= 1.0` 또는 `phase = phase.fract()`.

## `f32` vs `f64`

`01_cpal` 의 사인파 예제는 `f32` 로 `phase` 를 누산했다. 그래도 3초 재생에는 문제 없었다. 왜일까?

- `f32` 의 유효 자릿수: 약 7 자리 10진.
- 48000 Hz 에서 1 초 = 48000 샘플, 10 분 = 2,880,000 샘플.
- 누산 오차가 쌓이면서 주파수 드리프트가 귀로 들린다.

그래서 **Phase 2 부터는 `phase: f64`** 로 간다. 출력만 `as f32` 로 내린다.

## 공통 골격

각 파형은 아래 골격을 공유한다.

```rust
pub struct Oscillator {
    phase: f64,
    phase_inc: f64,
    sample_rate: f64,
}

impl Oscillator {
    pub fn set_frequency(&mut self, freq: f64) {
        self.phase_inc = freq / self.sample_rate;
    }

    fn advance(&mut self) -> f64 {
        let p = self.phase;
        self.phase += self.phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        p
    }
}
```

`advance()` 가 돌려주는 `phase` 하나로 모든 파형을 계산할 수 있다. 사인만 `phase * TAU` 해서 `.sin()` 에 넣으면 된다.

## 이 챕터에서 구현할 것

- `examples/src/oscillator/sine.rs`
- `examples/src/oscillator/saw.rs`
- `examples/src/oscillator/square.rs`
- `examples/src/oscillator/triangle.rs`
- 재생 진입점: `examples/src/bin/play_osc.rs`

네 타입이 같은 `DspNode` trait 를 구현하고, `play_osc` 에서 하나씩 바꿔가며 귀로 비교한다.

## 답할 질문

- `set_frequency` 를 오디오 스레드 밖에서 호출하면 어떤 race 가 가능한가? atomic 으로 충분한가?
- 왜 `phase.fract()` 와 `phase -= 1.0` 중 후자를 선호하는가? (힌트: fract 의 정밀도)
- Saw wave 단순 구현 `2*phase - 1` 을 스펙트럼으로 보면 왜 위쪽에 알리아싱이 보일까?
- Square wave 의 DC 성분을 0 으로 유지하려면 어떻게 해야 하는가?

## 완료 기준

- 네 파형을 각각 440Hz 로 3 초 재생.
- 귀로 들었을 때 각 파형의 음색 차이가 명확히 구별된다.
- 파형 전환 시 클릭이 발생하지 않는다 (위상을 이어받기).
