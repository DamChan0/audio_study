# Chapter 3 - Note, Velocity, CC, Pitch Bend

이 장은 자주 만나는 MIDI 메시지 5종을 정리한다.

## 1. Note On / Note Off — 음의 시작과 끝

```text
Note On  : "이 음정의 음을 이 세기로 시작해라"
Note Off : "이 음정의 음을 멈춰라"

byte 1 : 0x9n / 0x8n  (n = MIDI channel 0~15)
byte 2 : note number (0~127)
byte 3 : velocity (0~127)
```

note number 표.

```text
60 = C4 (Middle C)        ← 가장 흔한 기준
69 = A4                   ← 440 Hz
72 = C5
48 = C3
```

note → frequency 변환.

```text
freq = 440 · 2^((n - 69) / 12)
```

```rust
fn note_to_freq(n: u8) -> f32 {
    440.0 * 2f32.powf((n as f32 - 69.0) / 12.0)
}
```

이 식 한 줄이 모든 신디사이저의 첫 줄이다.

note 검증.

```text
note 69 → 440.0 Hz
note 81 → 880.0 Hz   (옥타브 위)
note 57 → 220.0 Hz   (옥타브 아래)
```

## 2. Velocity — 세기

velocity는 0~127 범위의 "건반을 얼마나 세게 눌렀는가"다.

```text
0       : 일반적으로 Note Off로 해석 (사실상 침묵)
1~63    : 약함
64      : 중간
65~127  : 강함
127     : 최대
```

velocity → amplitude 매핑은 음악적 의도에 따라 다르다.

```rust
// (a) 단순 linear
fn vel_to_amp_linear(v: u8) -> f32 {
    v as f32 / 127.0
}

// (b) 제곱 (작은 velocity가 더 작게 들리도록)
fn vel_to_amp_square(v: u8) -> f32 {
    let lin = v as f32 / 127.0;
    lin * lin
}

// (c) dB-perceptual: -60 dB ~ 0 dB
fn vel_to_amp_db(v: u8) -> f32 {
    if v == 0 { return 0.0; }
    let lin = v as f32 / 127.0;
    10f32.powf((lin - 1.0) * 3.0)   // 대략적인 perceptual 곡선
}
```

물리 acoustic 악기의 청각적 동작은 dB 곡선이 가깝다. 단순 linear는 작은 velocity가 너무 잘 들리는 경향.

velocity는 amplitude뿐 아니라 다른 파라미터(filter cutoff, attack time 등)에도 매핑할 수 있다. 이게 "expression"의 핵심.

## 3. Control Change (CC) — knob/fader 등

CC는 가장 일반적인 "파라미터 변경" 메시지다.

```text
byte 1 : 0xBn
byte 2 : CC number (0~127)
byte 3 : value (0~127)
```

표준 할당.

```text
CC 1   : Modulation wheel
CC 7   : Channel volume
CC 10  : Pan
CC 11  : Expression
CC 64  : Sustain pedal (0~63 off, 64~127 on)
CC 71  : Resonance
CC 74  : Filter cutoff
CC 91  : Reverb send level
CC 93  : Chorus send level
```

CC는 노드/플러그인의 파라미터에 매핑된다.

```text
CC 74 → synth.filter.cutoff
CC 1  → synth.lfo.depth
CC 11 → channel.expression_volume
```

이 매핑은 사용자 또는 프리셋이 정한다 — DAW의 "MIDI Learn" 기능이 그 매핑이다.

CC의 해상도는 7 bit(0~127)라 미세 조작에 부족할 수 있다. 14-bit CC pair로 두 개를 합치는 표준이 있지만, 학습 단계에선 7-bit만 다뤄도 충분.

## 4. Pitch Bend — 연속적 음정 변화

```text
byte 1 : 0xEn
byte 2 : LSB (0~127)
byte 3 : MSB (0~127)

값 = (MSB << 7) | LSB    범위 0 ~ 16383
중심값(=벤딩 안 함) = 8192
```

기본 범위는 ±2 semitones이지만, MIDI에서 RPN(Registered Parameter)으로 바꿀 수 있다 (보통 ±2, 가끔 ±12).

값을 음정 변화로 변환.

```rust
fn pitch_bend_semitones(value: u16, range_semitones: f32) -> f32 {
    let centered = value as f32 - 8192.0;
    centered / 8192.0 * range_semitones
}
```

이 결과를 note의 frequency 계산에 더한다.

```rust
let bent_note = note_number as f32 + pitch_bend_semitones(value, 2.0);
let freq = 440.0 * 2f32.powf((bent_note - 69.0) / 12.0);
```

pitch bend는 키보드에서 매 ms 단위로 들어올 수 있다. parameter smoothing(05 책)이 자연스러운 변화를 만든다.

## 5. Program Change — 음색 선택

```text
byte 1 : 0xCn
byte 2 : program number (0~127)
```

신디사이저의 음색을 선택. General MIDI(GM) 표준에서는 program 0 = Acoustic Grand Piano, 1 = Bright Piano, ... 으로 정해져 있다.

플러그인 신디는 보통 program을 자기 프리셋과 연결한다.

## 6. Channel과 polyphony

MIDI 채널 16개는 같은 케이블에서 다른 악기로 routing할 때 쓴다. 보통 한 악기 = 한 채널.

각 채널은 독립적이다.

```text
channel 0: piano synth
channel 1: bass synth
channel 9: drums (GM 표준에서 channel 10 = drums, 0-indexed로 9)
```

## 7. polyphony — 동시에 몇 음을 낼 수 있나

키보드를 동시에 5개 누르면 5개의 NoteOn이 들어오고 5개의 voice가 발음한다.

신디 안에서 이를 처리하는 게 voice 관리.

```text
voice = 한 음정의 한 인스턴스
voice 풀: max_voices개 (예: 16, 32, 64)

NoteOn 들어옴:
  - free voice 있으면 거기 할당
  - 없으면 voice stealing (가장 오래된, 또는 가장 작게 들리는 voice를 빼앗음)

NoteOff 들어옴:
  - 해당 note의 voice를 release stage로 보냄 (03 envelope)
  - release 끝나면 voice가 free
```

stealing 정책은 신디마다 다르다. 단순한 정책으로 시작.

## 8. Note On with velocity 0

```text
Note On (note, velocity = 0) ≡ Note Off (note)
```

이 동치성은 표준이다. 일부 키보드는 항상 NoteOn vel=0을 보내고 NoteOff를 안 보낸다. 양쪽 모두 처리할 수 있어야 한다.

## 자주 하는 실수

- note 60을 440 Hz라고 기억 (실은 69).
- velocity 0을 그대로 amplitude 0으로 쓰고 NoteOff 처리 안 함.
- pitch bend의 중심을 0으로 해석 → 실은 8192.
- CC의 value를 0~1.0으로 normalize 안 하고 그대로 사용.
- channel 무시 → 다른 악기로 가야 할 메시지가 잘못 처리됨.
- polyphony 처리 없이 voice 한 개로 모든 NoteOn → 단음 신디.

## 반드시 이해해야 할 것

- note 69 = 440 Hz. note 60 = C4.
- velocity → amplitude 매핑은 단순 linear가 아니라 곡선이 자연스럽다.
- CC는 사용자가 노드 파라미터로 매핑하는 가장 일반 채널.
- pitch bend는 14-bit, 중심 8192. semitones 환산 후 note에 더한다.
- 동시 발음에는 voice 풀이 필요하다. 풀이 차면 stealing 정책이 결정한다.
