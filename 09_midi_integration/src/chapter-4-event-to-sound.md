# Chapter 4 - Event에서 Sound로 연결하기

이 장은 MIDI thread → audio thread → 소리의 연결을 다룬다.

## 1. 두 thread의 만남

```text
MIDI input thread (midir 콜백):
  OS가 메시지를 줄 때마다 깨어남.
  매 5~50 ms에 한 번 정도.
  
audio thread (cpal 콜백):
  매 콜백 (예: 5 ms = 240 frames @ 48k) 호출됨.
```

이 둘은 서로 다른 시간 흐름이다. 직접 연결하면 안 된다.

```text
MIDI thread → SPSC queue → audio thread
            (lock-free)
```

queue 한 칸에는 메시지 + timestamp가 들어간다. timestamp는 sample 단위 (또는 wall clock ns)로.

## 2. Timestamp가 왜 중요한가

키보드 → 소리까지의 latency는 다음 합이다.

```text
1. OS MIDI driver 처리      : ~ 1 ms
2. midir callback 호출      : ~ 0~1 ms
3. queue 입력               : 거의 0
4. audio thread가 다음 콜백  : ~ 0~5 ms (콜백 주기)
5. 콜백 안 처리              : ~ 1 ms
6. cpal output buffer 처리   : ~ 5 ms
7. D/A + 스피커               : ~ 1~5 ms

총 latency: 약 10~20 ms (저하한 시스템에서)
```

사람 청각의 시간 정확도는 약 5~10 ms. 그래서 MIDI 메시지가 도착한 시점을 그대로 콜백 시작에 적용하면 메시지들의 상대 timing이 흩어진다.

해결: 각 메시지에 timestamp를 붙이고, 콜백 안에서 frame 단위로 정렬해서 적용한다.

## 3. Sample-accurate event 처리

콜백 한 사이클의 처리 흐름.

```text
콜백 시작 (frame 0)
  │
  ▼
queue에 쌓인 메시지를 모두 가져옴
각 메시지 → "이번 콜백의 어느 frame에 발생했는가"로 변환
  │
  ▼
frame을 0 → N으로 처리하면서:
  - 그 frame에 도달한 메시지를 synth에 전달
  - synth가 그 시점 이후 audio를 생성
  ▼
콜백 종료
```

코드 골격.

```rust
fn callback(data: &mut [f32], frames: usize) {
    let mut events: Vec<TimedMidi> = collect_events_from_queue();
    events.sort_by_key(|e| e.frame_offset);

    let mut next_event = 0;
    for f in 0..frames {
        // 이 frame에 적용할 메시지가 있으면
        while next_event < events.len()
            && events[next_event].frame_offset == f
        {
            synth.handle_midi(&events[next_event].msg);
            next_event += 1;
        }
        // 한 frame audio 생성
        let s = synth.next_sample();
        write_to_buffer(data, f, s);
    }
}
```

이게 sample-accurate timing의 표준 패턴이다. 각 event가 정확한 frame 위치에 적용된다.

## 4. Frame offset 계산

queue에 들어 있는 timestamp(ns 또는 wall clock)를 콜백 시작 시점 기준으로 frame offset으로 변환.

```text
frame_offset = (event_time - callback_start_time) · sample_rate / 1_000_000_000ns
```

간단하지만 부호와 음수 처리에 주의. 콜백 시작 *이전*에 도착한 메시지는 frame 0에 적용 (혹은 직전 콜백에 이미 적용됐어야 했음 — 이런 경우는 latency 늘리기).

## 5. 단순한 구현 — frame 정확도 포기

학습 단계에선 모든 메시지를 콜백 시작에 한꺼번에 적용해도 청각상 큰 문제가 없다.

```rust
fn callback(data: &mut [f32], frames: usize) {
    while let Some(msg) = queue.try_pop() {
        synth.handle_midi(&msg);     // 콜백 시작에 모두 처리
    }
    // 그 다음 audio 생성
    for f in 0..frames {
        let s = synth.next_sample();
        write_to_buffer(data, f, s);
    }
}
```

이걸 "buffer-rate accuracy"라고 한다. timing 오차는 콜백 buffer 길이만큼 (예: 5 ms). 키보드 연주는 충분하다. 정밀한 시퀀서나 자동 quantize 결과는 sample-accurate가 자연스럽다.

## 6. Synth voice 구성

NoteOn → Voice를 하나 활성화 → audio 생성.

```rust
struct Voice {
    note: u8,
    osc:  SineOsc,
    env:  Adsr,
    active: bool,
}

struct PolySynth {
    voices: Vec<Voice>,
}

impl PolySynth {
    fn handle_midi(&mut self, msg: &MidiMessage) {
        match msg {
            MidiMessage::NoteOn { note, vel } if *vel > 0 => {
                if let Some(v) = self.voices.iter_mut().find(|v| !v.active) {
                    v.note = *note;
                    v.osc.set_freq(note_to_freq(*note));
                    v.env.note_on();
                    v.active = true;
                } else {
                    // voice stealing
                    let oldest = self.voices.iter_mut()
                        .min_by_key(|v| v.start_time).unwrap();
                    oldest.note = *note;
                    oldest.osc.set_freq(note_to_freq(*note));
                    oldest.env.note_on();
                }
            }
            MidiMessage::NoteOn { vel: 0, note } |
            MidiMessage::NoteOff { note, .. } => {
                if let Some(v) = self.voices.iter_mut()
                    .find(|v| v.active && v.note == *note)
                {
                    v.env.note_off();
                }
            }
            _ => {}
        }
    }

    fn next_sample(&mut self) -> f32 {
        let mut sum = 0.0;
        for v in self.voices.iter_mut().filter(|v| v.active) {
            let s = v.osc.next() * v.env.next();
            sum += s;
            if v.env.is_idle() { v.active = false; }
        }
        sum
    }
}
```

이게 가장 단순한 polyphonic synth의 골격이다. 03 책의 SineOsc + Adsr을 그대로 쓴다.

## 7. CC, Pitch Bend의 적용 — parameter smoothing 위에 얹기

CC는 빠르게 들어와도 부드럽게 적용해야 한다 (knob 돌리는 동안 1~10 ms마다 메시지). 05 책의 Smoothed parameter가 그대로 쓰인다.

```rust
match msg {
    MidiMessage::CC { num: 74, value } => {
        let cutoff_target = cc_to_cutoff(*value);
        synth.filter_cutoff_target.store(cutoff_target);
    }
    MidiMessage::PitchBend { value } => {
        let semis = pitch_bend_semitones(*value, 2.0);
        synth.bend_target.store(semis);
    }
    _ => {}
}
```

target을 atomic으로 갱신. audio thread는 매 sample 또는 매 N sample 마다 smoothed value를 읽어 사용.

## 8. graph 노드로 본 synth

08 책의 AudioNode 모델에선 synth가 다음 모양이다.

```rust
impl AudioNode for SynthNode {
    fn process(&mut self, _inputs: &[&AudioBuffer], outputs: &mut [AudioBuffer]) {
        // (audio input은 안 받음. MIDI는 별도 channel로 들어옴)
        let frames = outputs[0].frames();
        for f in 0..frames {
            // sample-accurate MIDI 적용 (위 프레임 패턴)
            apply_midi_for_frame(f);
            let s = self.synth.next_sample();
            for ch in 0..outputs[0].channels() {
                outputs[0].channel_mut(ch)[f] = s;
            }
        }
    }
}
```

graph 안에서 MIDI는 audio buffer가 아닌 별도 channel을 통해 노드에 도달한다. 노드 trait는 audio만 다루고, MIDI는 노드 안의 별 메서드 또는 별 큐에서 처리.

## 자주 하는 실수

- midir 콜백 안에서 직접 audio thread의 자료구조 mutate → race.
- timestamp 없이 메시지만 큐에 넣음 → frame-accurate 처리 불가능.
- 콜백 안에서 OS API 직접 호출 → 차단 위험.
- voice stealing 없음 → 동시 N+1번째 NoteOn에서 메시지가 무시됨.
- CC를 직접 audio thread에서 매 메시지마다 적용 → 클릭 (smoothing 누락).
- NoteOn with velocity 0을 NoteOff로 처리 안 함 → 음이 영원히 안 끝남.

## 반드시 이해해야 할 것

- MIDI thread → SPSC → audio thread. 직접 다리 놓지 않는다.
- timestamp + frame offset 정렬이 sample-accurate timing의 핵심.
- 학습 단계에선 buffer-rate accuracy로 시작해도 문제없음. 정밀도가 필요해지면 sample-accurate로 확장.
- synth는 03 책의 SineOsc + Adsr 위에 voice 풀을 얹은 모양이다.
- CC와 pitch bend는 atomic + smoothing(05 책 패턴)으로 적용.
