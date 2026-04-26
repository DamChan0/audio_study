# Chapter 5 - Mixer, Bus, Master 구조

이 장은 DAW의 표준 그래프 패턴을 본다.

## 1. Channel Strip — 한 트랙의 처리 사슬

DAW에서 한 트랙은 보통 다음 사슬을 갖는다.

```text
[Source]
   │
   ▼
[Track FX (EQ, comp, etc.)]
   │
   ▼
[Track Volume / Pan]
   │
   ▼
[(Send level → 별 send bus)]
   │
   ▼
[Master Bus]
```

이걸 "channel strip"이라고 한다. 그래프 노드로 보면 다음과 같이 모델링한다.

```text
[Source Node]
[Track Effects Chain Node]   (= 여러 EQ/Comp의 직렬 = 그 자체로 sub-graph)
[Track Gain/Pan Node]
[Track Send Node]            (= 1 → 2 분기: master + send bus)
```

학습 단계에서는 Track FX 사슬을 sub-graph 또는 단순한 직렬 처리로 다룬다.

## 2. Master Bus — 모든 트랙을 합치는 노드

master bus는 graph에서 단 한 개의 노드, 모든 트랙의 출력이 흘러 들어간다.

```text
[Track 1] ─┐
[Track 2] ─┼─► [Master Bus] → [Master FX (EQ/Limiter)] → [Output (cpal)]
[Track 3] ─┤
[Track 4] ─┘
```

Master Bus 노드는 mixer node — N → 1 합산 — 다.

```rust
struct MixerNode { gain: f32 }   // 단순 합산만 (gain은 master fader)

impl AudioNode for MixerNode {
    fn process(&mut self, inputs: &[&AudioBuffer], outputs: &mut [AudioBuffer]) {
        // outputs[0] 채널마다, 각 frame마다 모든 inputs 합산
        for ch in 0..outputs[0].channels() {
            let out = outputs[0].channel_mut(ch);
            for i in 0..out.len() { out[i] = 0.0; }     // clear
            for &input in inputs {
                let in_ch = input.channel(ch);
                for i in 0..out.len() { out[i] += in_ch[i] * self.gain; }
            }
        }
    }
}
```

핵심 단순함: 단순 합산. 04 책의 limiter는 이 노드 *뒤*에 별 노드로 둔다.

## 3. Send / Return Bus

send/return은 트랙의 일부 신호를 별 그래프 경로로 보내고, 그 결과를 master에서 합치는 구조다.

```text
[Track A] ─┬──────────────────────► [Master Bus]
           │
           └─ send level ──► [Reverb FX] ──► [Reverb Return Bus] ──► [Master Bus]
```

Reverb FX 한 인스턴스를 여러 트랙이 공유한다. 이게 send/return의 본질 — "비싼 FX를 한 번만 돌려도 여러 트랙이 사용".

graph 표현.

```text
node 종류:
  Track Send Node (1입력 1출력에 send 분기)
  Reverb Send Bus (N입력 1출력 mixer)
  Reverb FX Node (1입력 1출력)
  Reverb Return Bus (1입력 1출력 fader)
```

연결.

```text
TrackA.dry_out → MasterBus.input
TrackA.send_out → ReverbSendBus.input
TrackB.send_out → ReverbSendBus.input
ReverbSendBus.out → ReverbFX.input
ReverbFX.out → ReverbReturnBus.input
ReverbReturnBus.out → MasterBus.input
```

이 구조에서 master bus 입력은 5개 (TrackA dry, TrackB dry, ..., ReverbReturn) 같은 식이 된다.

## 4. Sidechain — Audio path가 아닌 control path

sidechain은 한 노드의 입력 중 하나가 "신호 처리에 들어가는 audio"가 아니라 "처리를 어떻게 할지 결정하는 control"인 경우다.

```text
[Kick] ─┬──► [Master Bus]
        │
        └──► [Bass Compressor].sidechain_input

[Bass] ───► [Bass Compressor].audio_input → [Master Bus]
```

graph로 보면 Bass Compressor 노드가 입력 포트를 두 개 갖는다. 처리 시.

```rust
fn process(&mut self, inputs: &[&AudioBuffer], outputs: &mut [AudioBuffer]) {
    let audio = inputs[0];
    let sidechain = inputs[1];
    // envelope follower의 입력은 audio가 아닌 sidechain
    // gain reduction은 audio에 적용
}
```

같은 trait 시그니처에서 입력 포트의 의미를 노드가 알면 된다.

## 5. Pre-fader / Post-fader

send를 채널 fader 앞에서 보내느냐 뒤에서 보내느냐의 차이.

```text
pre-fader send  : Track Volume이 적용되기 전 신호를 send
post-fader send : Track Volume이 적용된 후 신호를 send (=DAW 기본값)
```

graph 차이.

```text
pre-fader:
  [Source] → [Track FX] ─┬─► [Track Volume] → [Master]
                          └─► [Reverb Send]

post-fader:
  [Source] → [Track FX] → [Track Volume] ─┬─► [Master]
                                            └─► [Reverb Send]
```

post-fader가 직관적 — 트랙을 mute하면 reverb도 함께 빠진다. pre-fader는 cue 전송, headphone monitor 등 특수한 경우에 쓴다.

## 6. Group / Folder — 트랙 묶음

여러 트랙을 묶어 하나의 sub-mixer로 만들고, 그 결과만 master로 보내는 패턴.

```text
[Drums Group]
  ├─ [Kick] ──┐
  ├─ [Snare] ─┼─► [Drum Bus] ──► [Drum Group FX] ──► [Master]
  └─ [Hat] ───┘

[Vocals Group]
  ├─ [Lead Voc] ─┐
  └─ [Backup] ───┴─► [Vocal Bus] ──► [Master]
```

graph로는 추가의 mixer 노드 + 그 뒤의 FX 노드. 이걸 사용자 UI에서 "group" 또는 "folder"로 추상화한다.

## 7. Master Chain

master output 직전에 마스터 FX 사슬이 들어간다.

```text
[Master Bus] → [Master EQ] → [Master Compressor] → [Master Limiter] → [Output]
```

이 사슬이 04 책의 mod_mastering 처리 사슬이다. graph로는 직선 chain — 분기/합산이 없다.

```text
mastering chain의 모든 처리는 단일 입력/단일 출력의 노드로 표현 가능
→ 그래프 모델 위에 자연스럽게 얹힘
```

## 8. RuStudio 4-track 예시

가장 단순한 mixer 그래프.

```text
[Source 1] ──► [Source 1 Strip (EQ + Vol/Pan)] ──┐
[Source 2] ──► [Source 2 Strip] ─────────────────┤
[Source 3] ──► [Source 3 Strip] ─────────────────┼──► [Master Bus] ─► [Master Limit] ─► [Output]
[Source 4] ──► [Source 4 Strip] ─────────────────┘
```

노드 수: 4(source) + 4(strip) + 1(master mix) + 1(master limit) + 1(output) = 11.

topological sort 후 process() 11번 호출하면 한 콜백 사이클 끝.

## 9. UI / engine 분리

UI에서 사용자가 보는 것은 channel strip의 fader, knob, send fader 등이다. engine에서는 graph 노드들과 atomic 파라미터들의 모음이다.

```text
UI 객체 ↔ engine 노드 매핑
  사용자가 fader 움직임 → atomic 갱신 → 다음 process() 호출 시 반영
```

graph 구조 변경(트랙 추가, send 추가)은 비-실시간 경로로. fader 움직임은 실시간 atomic으로. 이 분리가 핵심.

## 자주 하는 실수

- master limit을 master bus 안에 넣음 → mixer가 너무 많은 일을 하게 됨. 별 노드로 분리.
- send를 pre/post 둘 다 안 표시 → 사용자 혼란.
- 같은 reverb를 트랙마다 한 인스턴스씩 → CPU 폭발. send/return의 의미 무시.
- group/folder 깊이를 무한 허용 → graph 시각화/복잡도 폭증. 일반적으로 2~3단계.
- sidechain 신호의 latency를 보상 안 함 → ducking 시점이 어긋남.

## 반드시 이해해야 할 것

- channel strip = source + FX + vol/pan + send. 트랙 한 개의 표준 사슬.
- master bus = 모든 트랙의 합산 노드. 단순 mixer.
- send/return = 비싼 FX를 한 번 인스턴스로 여러 트랙이 공유.
- sidechain = audio가 아닌 control path. 노드의 별도 입력 포트로 표현.
- master chain = master bus 뒤의 직선 mastering 사슬. 04 책의 결과.
