# Chapter 7 - Delay Buffer와 상태 보존

delay는 **과거 샘플을 기억했다가 N 샘플 뒤에 다시 꺼내 쓰는** 처리다.

```text
delay
  input  : 현재 샘플 한 개
  output : (현재 샘플) + (N 샘플 전 입력)
  state  : 과거 N 샘플을 담은 버퍼 + 쓰기 인덱스
```

이 책에서 본격적으로 등장하는 stateful DSP의 정점이다. 이걸 이해하면 reverb, comb filter, IIR 필터까지 같은 골격으로 보인다.

## 왜 일반 `Vec`이 아니라 ring buffer인가

"과거 샘플 N개를 기억"하려면 가장 단순하게는 매 샘플 `Vec::push`하고 `Vec::remove(0)`하면 된다. 하지만 그건 콜백 안에서 절대 못 한다.

- `push`: 용량을 넘기면 재할당 (`02_mod_player_architecture`에서 본 실시간 규칙 위반)
- `remove(0)`: 모든 원소를 한 칸씩 앞으로 옮김 (O(N))

이 두 문제를 동시에 푸는 자료구조가 **ring buffer**다.

```text
buffer: [_, _, _, _, _, _, _, _]   고정 크기 N
write_idx: 현재 쓸 위치
read_idx:  지금 꺼낼 위치 (write_idx - delay_samples)

쓰기 후 인덱스를 한 칸 앞으로,
끝에 도달하면 0으로 감기 (= "ring")
```

cpal 콜백 시작 시 한 번 할당해 두고, 콜백 안에서는 인덱스만 움직인다.

## 가장 단순한 delay line

```rust
struct DelayLine {
    buf: Vec<f32>,
    write: usize,
}

impl DelayLine {
    fn new(max_samples: usize) -> Self {
        Self { buf: vec![0.0; max_samples], write: 0 }
    }

    fn process(&mut self, input: f32, delay_samples: usize, mix: f32) -> f32 {
        let len = self.buf.len();
        let read = (self.write + len - delay_samples) % len;
        let delayed = self.buf[read];
        self.buf[self.write] = input;
        self.write = (self.write + 1) % len;

        input * (1.0 - mix) + delayed * mix     // dry/wet
    }
}
```

여기서 보아야 할 점.

- `buf`는 한 번 할당된다. `process()` 안에서는 인덱스 산수만 일어난다.
- `read = (write + len - delay) % len`로 "delay 샘플 전" 위치를 찾는다.
- 같은 자리에 쓰기와 읽기를 동시에 하지 않게, 보통 **읽고 쓰기** 순서를 명확히 둔다.
- `mix`는 dry/wet 비율(0.0이면 원본만, 1.0이면 지연된 신호만).

## feedback delay (echo)

진짜 echo가 되려면 delayed 신호의 일부를 다시 buffer에 써넣는다.

```rust
fn process(&mut self, input: f32, delay: usize, fb: f32, mix: f32) -> f32 {
    let len = self.buf.len();
    let read = (self.write + len - delay) % len;
    let delayed = self.buf[read];

    // 입력 + delayed의 fb 비율을 다시 저장 → echo가 점점 작아지며 반복
    self.buf[self.write] = input + delayed * fb;
    self.write = (self.write + 1) % len;

    input * (1.0 - mix) + delayed * mix
}
```

`fb < 1.0`이면 echo가 점점 줄어들며 사라진다. `fb >= 1.0`이면 발산한다 — 절대 그렇게 두면 안 된다.

## ring buffer vs delay buffer

이 두 단어가 헷갈릴 수 있는데, 관계는 이렇다.

```text
ring buffer  : "원형으로 인덱스를 감는 고정 버퍼"라는 자료구조 패턴
delay buffer : "과거 샘플을 시간 지연만큼 기억"이라는 DSP 용도

→ delay buffer는 ring buffer라는 자료구조 위에 얹어 만든다.
```

ring buffer는 콜백-스레드 사이 통신(`ringbuf`, `rtrb` 같은 SPSC 큐)에도 쓰인다. 그건 producer-consumer 구조고, delay buffer는 단일 스레드에서 자기 과거를 기억하는 구조라는 차이가 있다.

## delay 길이는 어디서 정해지나

```text
max_delay_samples = max_delay_seconds · sample_rate
```

예를 들어 최대 1초 echo를 48 kHz에서 지원하려면 48,000 샘플 버퍼다. `f32` 기준 192 kB. 콜백 시작 전에 한 번 할당해 둔다.

stereo면 채널당 별도의 delay line을 하나씩 쓰는 게 가장 깔끔하다 (`Vec<DelayLine>` 길이 = 채널 수).

## 보간 (interpolation)

실제 reverb / chorus / flanger에서는 delay 길이가 정수가 아닌 경우가 있다. 예를 들어 "234.7 샘플 전"을 읽고 싶으면 `buf[234]`와 `buf[235]` 사이를 선형 보간한다.

```rust
let pos = write as f32 - delay_f;
let i = pos.floor() as i32;
let frac = pos - i as f32;
let a = buf[wrap(i)];
let b = buf[wrap(i + 1)];
let delayed = a * (1.0 - frac) + b * frac;
```

이 책 단계에서는 정수 delay만 쓰면 된다. flanger/chorus에서 본격적으로 필요한 주제다.

## RuStudio 관점

```text
delay buffer 패턴이 직접 등장하는 RuStudio 위치:
- echo / delay / chorus / flanger 이펙트
- reverb의 comb / allpass 단계
- biquad 필터의 z⁻¹ z⁻² 상태 (05_eq_biquad)
- 콜백 ↔ UI 사이 데이터 통신용 SPSC ring buffer (02 책에서 이미 언급)
```

마지막 항목은 자료구조만 같지 용도가 다르다는 점을 다시 의식한다.

## 자주 하는 실수

- 콜백 안에서 `Vec::with_capacity` / `Vec::resize` 호출 → 실시간 규칙 위반.
- write/read 인덱스의 wrap을 빼먹어 panic → `% len` 또는 `if idx >= len { idx -= len; }` 항상.
- feedback이 1.0 이상 → 발산. `clamp(0.0, 0.99)`로 막는 습관.
- delay 길이가 buffer 길이보다 큼 → 의도와 다른 위치를 읽는다. 길이 검증 또는 `min`으로 막는다.
- stereo에서 두 채널이 같은 buffer를 공유 → 좌우가 섞여 모노가 된다. 채널당 분리.

## 반드시 이해해야 할 것

- delay의 상태는 (고정 길이 buffer, write 인덱스) 두 개다.
- ring buffer는 자료구조, delay buffer는 그 자료구조의 한 가지 사용법이다.
- 콜백 안에서는 절대 할당하지 않는다. 미리 max 크기를 잡아 둔다.
- feedback과 wrap, 두 개만 잘 다루면 echo / chorus / 간단한 reverb까지 같은 골격으로 확장된다.
