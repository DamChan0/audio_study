# Chapter 5 - 시퀀싱과 타이밍

이 장은 MIDI 파일과 piano roll의 핵심 — **시간 좌표계** — 를 본다.

## 1. 시간 좌표계 4종

03 ~ 04 장에서 잠깐 언급했지만 다시 정리한다.

```text
sample : 1 / sample_rate 초
ms     : 1 / 1000 초
tick   : 1 / PPQ beat (PPQ = pulses per quarter note)
beat   : 1 / 4 of a measure (4/4 박자에서)
```

이들 사이의 변환.

```text
samples_per_beat = (60.0 / BPM) · sample_rate
ticks_per_beat   = PPQ
samples_per_tick = samples_per_beat / PPQ

샘플 t의 음악적 위치:
  t / samples_per_beat = 몇 번째 beat인가
  beat_index → bar / beat / sub-beat
```

예: BPM 120, 48 kHz, PPQ 480.

```text
samples_per_beat = (60 / 120) · 48000 = 24,000 샘플
samples_per_tick = 24,000 / 480     = 50 샘플
samples_per_ms   = 48 샘플
1 measure (4/4)  = 4 · 24,000        = 96,000 샘플
1 sixteenth      = 6,000 샘플
```

이 환산이 시퀀서 코드의 중추다.

## 2. SMF — Standard MIDI File

`.mid` 파일은 다음 구조다.

```text
Header chunk (MThd)
  format (0=single track, 1=multi-track, 2=multi-pattern)
  ntrks  (track 수)
  division (PPQ 또는 SMPTE)

Track chunks (MTrk × N)
  events: [delta_time, midi_message, ...] sequence
```

각 event 앞에 **delta-time**(직전 event부터 몇 tick 뒤)이 붙어 있다. 절대 시간이 아닌 상대.

`midly` crate가 이걸 파싱해 준다.

```toml
[dependencies]
midly = "0.5"
```

```rust
use midly::Smf;

let bytes = std::fs::read("song.mid")?;
let smf = Smf::parse(&bytes)?;
let ppq = match smf.header.timing {
    midly::Timing::Metrical(p) => p.as_int() as u32,
    midly::Timing::Timecode(_, _) => 480,    // SMPTE는 별도 처리
};

for track in &smf.tracks {
    let mut tick = 0u64;
    for ev in track {
        tick += ev.delta.as_int() as u64;
        // ev.kind: Midi(midi_message) | Meta(...) | SysEx(...)
    }
}
```

## 3. Tempo는 음악 안에 들어 있다

BPM은 보통 곡 시작에 정해지지만 곡 중간에 바뀔 수 있다. SMF에는 **Tempo Meta event**가 있다.

```text
Tempo meta event:
  microseconds per quarter note (μs/beat)
  
BPM = 60_000_000 / μs_per_beat
```

한 곡이 여러 tempo 변화를 가질 수 있어서, "tick → samples" 변환은 곡 시작부터 누적해서 계산해야 정확하다.

```text
tick 0 ~ tick A   : tempo T1
tick A ~ tick B   : tempo T2
...

각 구간마다 samples_per_tick이 다름.
누적 합으로 절대 sample 위치 계산.
```

학습 단계에서는 tempo가 한 개라고 가정해도 OK. RuStudio가 자동화/automation으로 변하는 tempo를 다루려면 위의 누적 모델이 필요.

## 4. Sequencer의 핵심 루프

시퀀서는 매 콜백 사이클마다 다음을 한다.

```text
1. 콜백 시작 시점의 tick 위치 t_start 계산
2. 콜백 끝 시점의 tick 위치 t_end 계산
3. (t_start, t_end] 사이에 떨어지는 모든 event를 가져옴
4. 각 event의 frame offset 계산
5. synth에 timestamped event로 전달 (4장 패턴)
```

```rust
fn process_block(&mut self, frames: usize) {
    let t_start = self.cursor_ticks;
    let t_end   = t_start + ticks_in_frames(frames);

    for ev in self.events_in_range(t_start, t_end) {
        let offset_ticks = ev.tick - t_start;
        let frame_offset = (offset_ticks * frames as u64) / (t_end - t_start);
        synth.queue_event(ev.message, frame_offset as usize);
    }

    self.cursor_ticks = t_end;
}
```

각 콜백마다 cursor가 앞으로 이동하면서, 그 구간에 들어 있는 event들을 정확한 frame에 적용한다. 이게 sample-accurate sequencer의 핵심.

## 5. Loop, jump, transport 제어

사용자가 transport를 stop/start/seek 하면 cursor가 이동한다.

```text
stop                  : cursor 멈춤 + 모든 voice를 NoteOff
start (from cursor)   : 그 위치부터 재개
jump to bar           : cursor를 새 위치로, voice 모두 NoteOff
loop region (A ~ B)   : cursor가 B에 도달하면 A로 점프
```

NoteOff 처리가 누락되면 voice가 영원히 매달려 있게 된다 (stuck note). transport 변화 시 항상 "all notes off"를 한 번 보내는 게 정석.

## 6. Real-time recording

키보드를 녹음해서 SMF로 저장하는 것도 시퀀서의 일부다.

```text
키보드 메시지 → MIDI thread → 큐
audio thread가 받아서:
  - synth로 보냄 (재생)
  - timestamp(현재 tick 위치)와 함께 record buffer에 저장

녹음 끝:
  buffer를 SMF event 시퀀스로 변환
  midly로 .mid 저장
```

녹음의 정밀도는 timestamp 정밀도다. 콜백 시작 시점의 tick에 그냥 모두 매핑하면 16분음표 단위 quantize와 비슷한 결과 (5 ms 해상도). 정밀하게 하려면 sample-accurate timestamp가 필요.

## 7. RuStudio 관점

```text
mod_player의 piano roll:
  사용자가 그린 노트를 자체 데이터 구조로 보존
  재생 시 위 sequencer 모델로 시간축 따라 흐림

MIDI 입력 녹음:
  키보드 메시지를 timestamp와 함께 기록
  사용자가 "stop"하면 해당 트랙에 저장

테스트/시연용 SMF 파일 재생:
  midly로 파싱 → 시퀀서가 시간축 따라 재생 → 내장 synth로 들음
```

## 8. quantize와 microtiming

녹음된 MIDI는 사람 손 떨림 때문에 정확한 grid에 안 떨어진다. quantize는 각 event를 가까운 grid 위치(예: 16분음표)로 옮기는 처리.

```rust
fn quantize_tick(tick: u64, grid_ticks: u64) -> u64 {
    let q = grid_ticks;
    let r = tick % q;
    if r < q / 2 { tick - r } else { tick + (q - r) }
}
```

정확하지만 음악적으로 너무 기계적. 그래서 0%~100% strength로 부분 quantize도 흔하다 (이 단계에선 다루지 않음).

## 자주 하는 실수

- BPM이 일정하다고 가정하고 tick → time 환산 캐싱 → tempo 변화 후 어긋남.
- transport stop 시 NoteOff 안 보냄 → stuck notes.
- delta-time을 절대 tick으로 잘못 해석 → 모든 event timing 어긋남.
- SMPTE timing을 PPQ로 처리 → 시간이 곡과 안 맞음.
- 녹음 시 wall clock 시간과 cursor tick이 어긋나는 보정 누락 → drift.

## 반드시 이해해야 할 것

- 4종 시간 단위(sample / ms / tick / beat) 사이의 환산은 시퀀서 코드의 핵심.
- SMF는 delta-time + tempo meta event로 시간을 표현한다. 절대 시간이 아니다.
- 시퀀서는 매 콜백마다 (t_start, t_end] 구간의 event들을 frame 정렬해서 보낸다.
- transport 제어 시 항상 NoteOff/All-Notes-Off로 stuck note 방지.
- 녹음의 정밀도는 timestamp의 정밀도다.
