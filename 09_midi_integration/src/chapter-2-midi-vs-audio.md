# Chapter 2 - MIDI는 오디오가 아니다

## 1. 본질적 차이

```text
audio:
  sample stream. 끊임없이 흐른다.
  data: f32 (또는 i16) 한 줄에 한 개. 1초에 48,000개.
  의미: amplitude 변화 그 자체.

MIDI:
  event stream. 사건이 있을 때만 메시지가 온다.
  data: 보통 3 byte (status + data1 + data2).
  의미: "지금 무엇이 일어났는가" (note pressed, knob moved, ...).
```

이 둘은 **같은 신호의 두 가지 보기가 아니다**. 다른 종류의 정보다.

```text
audio "C4 사인파 1초"  → 1초 동안 48,000개의 f32 샘플
MIDI  "C4 1초간 누름"  → 0초에 NoteOn 1개, 1초에 NoteOff 1개. 끝.
```

MIDI 파일 1초가 audio WAV 1초보다 수천 배 작은 이유.

## 2. MIDI는 "어떻게 소리가 날지"가 아니다

MIDI 자체는 무음이다. 키보드가 NoteOn 60을 보내도, 그걸 받아서 소리로 만들어줄 무언가가 없으면 아무 일도 안 난다.

```text
NoteOn (60, velocity 100) 메시지
   │
   ▼
어떤 음색으로 만들지는 받는 쪽이 결정
   │
   ▼
synth가 받으면     → sine 또는 saw 등 합성
sampler가 받으면   → 미리 녹음한 샘플 재생
다른 신디가 받으면 → 다른 음색
```

같은 MIDI 메시지가 다른 결과를 낸다. 이게 MIDI의 강점이자 약점.

## 3. MIDI의 timing 단위

```text
ms      : "사건이 몇 ms 전에 일어났는가" (실시간 입력)
sample  : "사건이 audio buffer의 몇 번째 frame에서 일어났는가" (sample-accurate 처리)
tick    : "사건이 곡 시작 후 몇 tick에 일어났는가" (음악적 timing)
beat    : tick / PPQ. 음악 박자 단위.
```

이 네 가지 단위를 적절히 변환하는 것이 MIDI 다루는 코드의 절반이다.

```text
1 beat = PPQ ticks (보통 PPQ = 480 또는 960)
1 beat = 60 / BPM 초
1 beat = (60 / BPM) · sample_rate 샘플
```

예: BPM 120, 48 kHz, PPQ 480 일 때.

```text
1 beat = 0.5 초 = 24,000 샘플 = 480 ticks
1 tick = 50 샘플
1 sixteenth (1/4 beat) = 6,000 샘플
```

이 환산을 자유롭게 할 수 있어야 시퀀서가 정확히 동작한다.

## 4. 왜 audio buffer가 아니라 별도 path인가

같은 graph에서 MIDI도 buffer로 다루고 싶을 수도 있지만, 다음 이유로 분리한다.

```text
1. 발생 빈도가 다르다
   audio: 매 sample (48k Hz)
   MIDI : 키 한 번 누를 때마다 (수 Hz ~ 수십 Hz)

2. 데이터 형이 다르다
   audio: f32 stream
   MIDI : 가변 길이 메시지 + timestamp

3. timing 정밀도 요구가 다르다
   audio: sample-accurate가 자연스럽다
   MIDI : 사람 인지 한계는 약 5~10 ms이지만, sample-accurate가 가능하면 더 좋다

4. routing이 다르다
   audio: graph edge
   MIDI : note는 한 synth로, CC는 그 synth의 특정 파라미터로
```

그래서 MIDI는 audio graph와 **나란히 흐르는 별도 channel**로 모델링한다.

```text
MIDI events ─────────────────► synth params (frame-aligned timestamp)

audio ───► graph nodes ───► output
                ▲
              synth는 두 입력 (audio = 없음, MIDI events) 받음
```

## 5. RuStudio 관점

```text
mod_player의 MIDI 입력:
  midir로 키보드 받기 → SPSC queue → audio callback이 매 사이클 시작에 처리

내부 시퀀서 (track의 piano roll):
  MIDI 파일 또는 자체 데이터 → 시간축으로 스케줄 → 같은 audio callback에서 처리

플러그인 (10 책):
  MIDI 입력 받는 플러그인 (synth)에 메시지 전달
  audio 처리 콜백과 같은 단위 시간에 함께 적용
```

## 6. MIDI 1.0 메시지 골격

이 책에서 다루는 메시지는 대부분 **MIDI 1.0**이다 (가장 흔하다).

```text
[status byte] [data1] [data2]

status byte = type(4 bits) + channel(4 bits)
data1, data2 = 0 ~ 127

예시:
  Note On  (channel 0): 0x90  60  100        ← C4를 velocity 100으로
  Note Off (channel 0): 0x80  60  0          ← 또는 Note On with velocity 0
  CC       (channel 0): 0xB0  cc#  value
  PitchBend(channel 0): 0xE0  lsb  msb
```

3 byte 안에 거의 모든 의미가 들어 있다.

## 자주 하는 실수

- MIDI 파일 길이를 audio WAV처럼 추정 → 단위가 다르다.
- "MIDI를 재생한다"고 말하면서 음색까지 포함해서 생각 → MIDI는 음색을 갖지 않는다.
- audio thread에서 MIDI message를 OS API로 직접 받는다 → 차단/IO 위험.
- timestamp 없이 메시지만 큐에 넣음 → frame-accurate timing 불가능.
- MIDI 입력의 latency를 audio latency와 합쳐서 안 다룸 → 키보드 반응이 느려짐.

## 반드시 이해해야 할 것

- MIDI = event stream. audio = sample stream. 데이터 모양이 본질적으로 다르다.
- MIDI는 음색을 갖지 않는다. 받는 쪽(synth/sampler)이 결정한다.
- timing 단위 4종 (ms / sample / tick / beat)을 자유롭게 환산할 수 있어야 한다.
- MIDI는 audio graph와 나란히 흐르는 별도 channel로 모델링한다.
