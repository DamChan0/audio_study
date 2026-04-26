# Chapter 6 - 예제와 입력 흐름

이 장은 09 책의 모든 단계를 만져 볼 권장 예제다.

## 권장 예제 목록

```text
01_note_to_freq      : note 0~127 → frequency 변환표 출력
02_velocity_curves   : linear / square / dB perceptual 비교 표 출력
03_midir_listen      : OS 키보드 → 콘솔에 메시지 인쇄
04_simple_synth      : midir → SPSC → audio callback → SineOsc + ADSR
05_polysynth        : 04를 voice 풀(8 voice)로 확장
06_smf_player        : .mid 파일 → 내장 synth로 재생
07_record_midi       : 키보드 입력 → 콘솔 + .mid 파일 저장
```

## 디렉토리

```text
09_midi_integration/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml          ← midir, midly, cpal, ringbuf
    src/
      lib.rs            ← Voice, PolySynth, MidiQueue, Sequencer 등
      bin/
        01_note_to_freq.rs
        02_velocity_curves.rs
        03_midir_listen.rs
        04_simple_synth.rs
        05_polysynth.rs
        06_smf_player.rs
        07_record_midi.rs
```

## 01 — note → freq 표

```rust
fn main() {
    for n in 0..=127 {
        let f = 440.0 * 2f32.powf((n as f32 - 69.0) / 12.0);
        println!("note {:3} ({}) = {:.2} Hz", n, note_name(n), f);
    }
}

fn note_name(n: u8) -> String {
    let names = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
    let octave = (n / 12) as i32 - 1;
    format!("{}{}", names[(n % 12) as usize], octave)
}
```

검증.

```text
note 60 → C4 → 261.63 Hz
note 69 → A4 → 440.00 Hz
note 72 → C5 → 523.25 Hz
```

## 03 — midir로 키보드 듣기

```rust
use midir::{MidiInput, Ignore};

fn main() -> anyhow::Result<()> {
    let mut input = MidiInput::new("rustudio-listen")?;
    input.ignore(Ignore::None);

    let ports = input.ports();
    let port = &ports[0];           // 첫 번째 장치
    let _conn = input.connect(port, "rustudio-listen", |stamp, msg, _| {
        println!("[{}] {:?}", stamp, msg);
    }, ())?;

    println!("listening... (Enter to quit)");
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(())
}
```

검증.

```text
- 키보드 누름 → 두 byte (또는 세 byte) 메시지 출력
- NoteOn / NoteOff / CC가 구분되어 보임
- velocity 다양함이 숫자로 보임
```

## 04 — Simple synth

핵심 골격.

```rust
// MIDI thread (midir callback)
input.connect(port, "rustudio-synth", move |_stamp, msg, _| {
    if let Some(parsed) = parse_midi(msg) {
        let _ = midi_tx.try_push(parsed);
    }
}, ())?;

// audio thread (cpal callback)
move |data: &mut [f32], _| {
    while let Some(msg) = midi_rx.try_pop() {
        synth.handle_midi(&msg);
    }
    for f in data.chunks_mut(channels) {
        let s = synth.next_sample();
        for ch in f { *ch = s; }
    }
}
```

검증.

```text
- 키보드 누름 → 약 5~20 ms 후 사인파 발음
- 누르면 ADSR attack이 들림
- 떼면 release가 들림
- 두 키 동시에 누름 → 단음 신디라 마지막 키만 (다음 예제에서 polyphony)
```

## 05 — Polysynth

04에 voice 풀을 추가.

```rust
struct Voice { osc: SineOsc, env: Adsr, note: u8, active: bool }
struct PolySynth { voices: [Voice; 8] }
```

NoteOn 시 free voice 찾기 + voice stealing. NoteOff 시 해당 note의 voice를 release.

검증.

```text
- 8 키 동시 누름 → 모두 들림
- 9 키 → 가장 오래된 voice가 빼앗김
- 손을 떼면 release 곡선 (한 음씩 사라짐)
```

## 06 — SMF player

```rust
use midly::{Smf, Timing};

let smf = Smf::parse(&std::fs::read("song.mid")?)?;
let ppq = match smf.header.timing {
    Timing::Metrical(p) => p.as_int() as u32,
    _ => 480,
};

// 모든 트랙의 event를 시간순(absolute tick)으로 합치기
let mut events: Vec<(u64, midly::TrackEventKind)> = Vec::new();
for track in &smf.tracks {
    let mut t = 0u64;
    for ev in track {
        t += ev.delta.as_int() as u64;
        events.push((t, ev.kind));
    }
}
events.sort_by_key(|&(t, _)| t);

// audio thread cursor에서 적합한 event 꺼내 synth로
```

검증.

```text
- 짧은 .mid 파일 재생 시 듣기에 멜로디
- BPM 변화가 있는 파일은 약간 어긋날 수 있음 (단일 tempo 가정)
- 곡 끝나면 silence
```

## 07 — MIDI record

키보드 메시지를 timestamp와 함께 buffer에 저장.

```rust
struct RecordedEvent { tick: u64, msg: MidiMessage }
let recorded: Vec<RecordedEvent> = ...;

// 종료 시 midly로 SMF 저장
use midly::*;
let mut track = Track::new();
let mut prev_tick = 0u64;
for ev in &recorded {
    let delta = ev.tick - prev_tick;
    prev_tick = ev.tick;
    track.push(TrackEvent {
        delta: u28::new(delta as u32),
        kind: TrackEventKind::Midi { /* ... */ },
    });
}
let smf = Smf {
    header: Header::new(Format::SingleTrack, Timing::Metrical(u15::new(480))),
    tracks: vec![track],
};
let mut buf = Vec::new();
smf.write(&mut buf)?;
std::fs::write("recorded.mid", buf)?;
```

검증.

```text
- 녹음 후 06 player로 재생 → 동일한 멜로디
- 녹음한 SMF를 다른 DAW에서 열어도 정상 재생
- 시간 정밀도: 콜백 단위 (5~10 ms) 내외
```

## 콜백 안 sample-accurate (advanced)

학습 단계에선 buffer-rate accuracy로 시작하고, 정밀이 필요해지면 4장의 frame-offset 패턴으로 옮긴다.

```text
buffer-rate (단순):
  콜백 시작에 모든 큐 메시지를 한꺼번에 적용 → 5~10 ms 정밀도

sample-accurate (정밀):
  메시지마다 frame_offset 계산 → frame 정확도
```

## 자주 하는 실수

- midir 콜백에서 std::io::stdout().write 같은 차단 호출 → 시스템 메시지 누락.
- SPSC queue의 capacity가 너무 작아 메시지 drop → 키보드 누름이 무시됨.
- voice 풀 크기를 1로 두고 polyphony 무시 → 단음 synth.
- SMF 파싱 후 PPQ 단위 변환 누락 → 재생 속도가 이상.
- 녹음 종료 시 still active한 NoteOn에 대한 NoteOff 누락 → SMF에 stuck notes.

## 반드시 이해해야 할 것

- 모든 예제는 두 thread (MIDI input + audio callback)와 그 사이의 SPSC queue 위에 올라간다.
- note → freq, velocity → amp는 표/공식 한 줄로 끝난다. 어려운 부분은 timing.
- voice 풀 + stealing이 polyphony의 핵심.
- SMF는 delta-time + PPQ로 표현된다. tempo 변화는 별도 meta event.
