# Chapter 7 - 자주 하는 실수와 복습

## Graph 일반

- 단순 chain 대신 graph로 모든 걸 모델링 → 오버엔지니어링.
- cycle을 허용하는 graph로 시작 → 처리 순서 결정 안 됨.
- 매 콜백마다 topological sort → 부하 폭발.
- graph 구조 변경을 audio thread에서 직접 수행 → race / lock.

## AudioNode / AudioBuffer

- planar/interleaved 가정이 노드마다 다름 → 채널 섞임.
- buffer pool 모델로 처음부터 시작 → 복잡도 폭증.
- 매 사이클 AudioBuffer 새로 할당 → GC 없는 언어라도 부담.
- latency_samples()를 무시하고 PDC 적용 안 함 → look-ahead 노드 들어가면 시간 어긋남.
- reset()을 매 콜백마다 호출 → 신호 끊김.

## DAG / 정렬

- cycle 검사 누락 → 사용자가 잘못된 연결 가능.
- 정렬 배열을 lock 잡고 audio thread에서 사용 → 콜백 blocking.
- 구조 변경과 값 변경을 같은 코드 경로 → 매 fader마다 sort.
- 노드 처리 시간 모니터링 누락 → underrun 원인 추적 어려움.

## Mixer / Bus / Master

- 같은 reverb를 트랙마다 한 인스턴스씩 → CPU 폭발.
- master limit을 master mixer 안에 합침 → 책임 섞임.
- send를 pre/post 표시 안 함 → 사용자 혼란.
- sidechain 신호의 latency 보상 누락 → ducking 시점 어긋남.

## 처리 단계 입출력 상태표

```text
AudioNode trait      input: &[&AudioBuffer]   output: &mut [AudioBuffer]   state: 노드별
AudioBuffer (planar) input: -                 output: -                    state: Vec<Vec<f32>>
Graph                input: 콜백 frames       output: master out buffer    state: 노드들 + 정렬 배열
toposort             input: graph             output: Vec<NodeIndex>       state: 없음 (pure)
MixerNode            input: N inputs          output: 1 output             state: gain
SendNode             input: 1                 output: 2 (main + send)      state: send level
SidechainNode        input: 2 (audio + sc)    output: 1                    state: env follower
```

## Phase 7 체크리스트

```text
□ chain이 표현 못 하는 구조 두 가지 이상 들 수 있다
□ AudioNode trait의 최소 메소드를 외운다
□ planar AudioBuffer 모델을 채택한 이유를 설명할 수 있다
□ DAG에서 cycle이 있으면 무엇이 안 되는지 한 줄로 설명 가능
□ topological sort의 결과가 무엇인지 안다
□ 4-track + 1 send/return + master 구조의 process() 호출 순서를 손으로 그릴 수 있다
□ petgraph로 toposort, cycle 감지가 가능
□ 콜백 안에서 graph.run()의 의미를 안다
□ graph 구조 변경 vs 값 변경을 분리해서 처리하는 패턴을 안다
```

## 03~07 책 도구의 재사용 지도

```text
03 oscillator      → SourceNode 구현
03 envelope        → ParameterSmoother로 노드 파라미터 변경 흡수
03 delay buffer    → DelayNode / ReverbNode
04 compressor      → CompressorNode (sidechain 입력 옵션)
05 EQ              → EqNode (channel strip 안)
06 ring buffer     → SourceNode가 디코더 thread에서 받는 다리
07 file decoder    → FileSourceNode
```

## 다음 책으로 넘어가는 다리

다음 책은 `09_midi_integration`이다.

지금까지 graph의 입력은 audio buffer였다. 09에서는 별 종류의 입력 — **MIDI event** — 가 들어온다.

```text
MIDI event (note_on, note_off, CC, pitch bend, ...)
   │
   ▼
event router
   │
   ▼
synth/sampler node가 받아 audio buffer 생성
```

graph 모델은 변하지 않는다 — 단지 source 노드 종류가 한 가지 더 늘어난다 (MIDI 입력에서 audio를 만드는 노드). 09 책 끝에서는 키보드 → graph → cpal output까지 한 사이클이 완성된다.

## 한 줄 요약

> graph는 chain의 일반화다. 합산/분기/send/sidechain을 표현하기 위한 도구이며, DAG + topological sort 위에 올린다. 노드 trait (process/latency/reset)와 buffer 소유권 모델이 핵심 구조다.
