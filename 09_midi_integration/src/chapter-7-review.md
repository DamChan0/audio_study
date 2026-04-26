# Chapter 7 - 자주 하는 실수와 복습

## MIDI vs Audio

- MIDI 파일을 audio처럼 길이 추정 (기준 단위 다름).
- "MIDI를 재생한다"고 음색까지 포함해 생각 (MIDI는 음색이 없다).
- MIDI를 audio buffer로 다루려고 시도 → 두 stream을 분리해야 한다.

## 메시지 / 변환

- note 60을 440 Hz로 기억 (실은 69).
- velocity 0을 NoteOff로 처리 안 함 → 음이 안 끝남.
- velocity → amplitude를 단순 linear로만 → 작은 velocity가 너무 잘 들림.
- pitch bend 중심을 0으로 (실은 8192).
- CC 채널/번호 무시 → 전혀 다른 파라미터가 바뀜.

## Thread / Timing

- midir 콜백에서 audio thread 자료구조 직접 mutate → race.
- timestamp 없이 메시지만 큐에 → frame-accurate 처리 불가.
- audio thread에서 OS API 직접 호출 → 차단.
- SPSC capacity 너무 작아 메시지 drop.
- transport stop 시 NoteOff 누락 → stuck notes.

## Voice / Polyphony

- voice 풀 크기 1 → 단음 신디.
- voice stealing 없음 → max+1번째 NoteOn 무시.
- envelope이 idle인 voice를 active로 표시 → CPU 낭비.
- NoteOn with velocity 0을 별 path로 처리하지 않음.

## Sequencing / SMF

- BPM 일정 가정 캐싱 → tempo 변화 후 어긋남.
- delta-time을 절대 tick으로 해석.
- SMPTE timing을 PPQ로 처리.
- quantize 없이 그대로 사용 → 사람 손 떨림이 그대로 남음.

## 처리 단계 입출력 상태표

```text
midir input          input: OS MIDI       output: 메시지 + timestamp     state: 콜백 등록
SPSC queue           input: 메시지        output: 메시지                  state: 인덱스 두 개
note_to_freq         input: u8 (0~127)    output: f32 (Hz)                state: 없음
vel_to_amp           input: u8 (0~127)    output: f32 (0~1)               state: 없음
PolySynth            input: MIDI 메시지   output: f32 샘플                state: voice 배열, env, osc
Sequencer            input: cursor frames output: timestamped events      state: cursor tick, tempo
SMF parser (midly)   input: bytes         output: tracks                  state: 없음
```

## Phase 8 체크리스트

```text
□ MIDI와 audio의 본질적 차이를 한 줄로 설명 가능
□ note ↔ freq 변환 식 (`440 · 2^((n-69)/12)`)을 외움
□ velocity / pitch bend / CC의 역할을 구분
□ MIDI thread → SPSC → audio thread 패턴을 그릴 수 있다
□ sample-accurate 처리의 frame_offset 계산을 이해
□ voice 풀 + stealing 구조를 코드로 작성 가능
□ BPM/PPQ/sample/tick 사이의 환산을 자유롭게 가능
□ SMF의 delta-time + tempo meta event 모델을 이해
□ transport 제어에서 stuck note 방지 패턴을 안다
```

## 03 ~ 08 책 도구의 재사용 지도

```text
03 SineOsc + Adsr     → Voice의 osc + env
03 phase accumulator → pitch bend가 적용된 freq를 매 sample 다르게
04 envelope follower → 별 용도 (signal-to-CC mapping 같은 advanced)
05 Smoothed parameter → CC 변경의 부드러운 반영
06 ring buffer (SPSC)→ MIDI thread ↔ audio thread 다리
08 AudioNode trait    → SynthNode 구현 (audio 입력 무시, MIDI는 별 channel)
08 graph              → synth, FX, mixer를 다 함께 연결
```

## 다음 책으로 넘어가는 다리

다음 책은 `10_plugin_architecture`다.

지금까지 모든 처리는 RuStudio 안에 있는 코드였다. 10 책은 그 경계 — **외부 플러그인을 호스팅**하거나, RuStudio가 만든 처리를 **외부 DAW에서 사용 가능한 플러그인으로** 배포하는 — 를 다룬다.

플러그인은 결국 audio buffer + MIDI events + parameters의 표준 인터페이스다. 09 책의 MIDI와 04~05 책의 audio 처리, 08 책의 graph가 모두 그 인터페이스의 양쪽 끝에 다시 등장한다.

## 한 줄 요약

> MIDI는 audio가 아닌 event stream이다. note→freq + velocity 매핑 + voice 풀 + thread 분리(SPSC) + sample-accurate timing이 이 책의 5개 골격이고, SMF + tempo는 시퀀서 한 단계 추가다.
