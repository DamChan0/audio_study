# Chapter 6 - 구조 예시와 확장 포인트

이 장은 graph 구조를 코드/그림으로 한 번 만져 볼 권장 예제다.

## 권장 예제 목록

```text
01_two_source_mixer  : 두 oscillator → mixer → output. graph 최소 모양
02_eq_master         : source → EQ → master → output. chain의 graph 표현
03_send_return       : 두 source + reverb send → master. send/return 첫 사례
04_parallel_comp     : source → (EQ + comp parallel) → master. parallel branch
05_topological_sort  : graph 노드 5~10개로 정렬 결과를 인쇄. 알고리즘 검증
06_full_4track       : 4 트랙 + 1 send/return + master limit 풀 graph
```

이 6개를 끝내면 이 책의 그래프 모델이 손에 들어온다.

## 디렉토리

```text
08_audio_graph_architecture/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml         ← petgraph, rustfft (검증 보조), hound 등
    src/
      lib.rs           ← AudioBuffer, AudioNode, MixerNode, Graph 등
      bin/
        01_two_source_mixer.rs
        02_eq_master.rs
        03_send_return.rs
        04_parallel_comp.rs
        05_topological_sort.rs
        06_full_4track.rs
```

## 01 — 두 source mixer

```text
[OscA 220Hz] ──┐
               ├─► [Mixer (sum)] ──► [Output]
[OscB 330Hz] ──┘
```

검증.

```text
- 출력 spectrum (06 책 도구)에 220Hz, 330Hz 두 피크
- 두 source의 amplitude를 0.5씩 두면 합산 amplitude가 1.0 부근
- 한 source amplitude를 0으로 두면 다른 source만 들림 (소거 검증)
```

## 02 — EQ를 graph로

```text
[Source] ──► [EqNode (peaking +6dB at 1kHz)] ──► [Output]
```

chain만 있으면 굳이 graph가 필요 없다 — 이 예제는 graph 모델 위에서 같은 결과가 나오는지 확인용.

검증.

```text
- 01번 chain 코드와 출력 비트가 동일 (graph 오버헤드 외엔 차이 없음)
- spectrum 1kHz 피크 +6dB
```

## 03 — Send / Return Reverb

```text
[Source A] ─┬───────────────────────► [Master Mixer] ─► [Output]
            │                        ▲
            └─ [Send Node] ─► [Reverb] ─┘
```

Reverb는 여기선 가장 단순한 feedback delay 한 개 (03 책의 delay buffer)면 충분하다.

검증.

```text
- send level = 0 → reverb 안 들림
- send level = 1.0 → wet 신호 추가
- reverb를 두 source가 공유하면 두 source의 wet이 같은 reverb를 거침
```

## 04 — Parallel compression

```text
[Source] ─┬─► [Light EQ] ─────────────► [Mixer] ─► [Output]
          │                            ▲
          └─► [Heavy Comp (4:1, slow)] ┘
```

검증.

```text
- 두 path의 amplitude를 각각 0.7씩 두고 합산 후 master limit 적용 가정
- comp만 강하면 transient가 죽고 sustain만 부각, 합치면 transient + sustain 둘 다 살아남
- 청각 비교: 원본 vs heavy comp만 vs parallel mix
```

## 05 — topological sort 검증

petgraph로 5~10개 노드 graph를 만들어 sort 결과 인쇄.

```rust
use petgraph::graph::DiGraph;
use petgraph::algo::toposort;

let mut g = DiGraph::<&str, ()>::new();
let a = g.add_node("A");
let b = g.add_node("B");
let c = g.add_node("C");
let d = g.add_node("D");

g.add_edge(a, b, ());
g.add_edge(a, c, ());
g.add_edge(b, d, ());
g.add_edge(c, d, ());

let order = toposort(&g, None).unwrap();
for ni in order {
    println!("{}", g[ni]);
}
// 출력: A B C D 또는 A C B D
```

검증.

```text
- 같은 graph에 cycle을 추가하면 toposort가 Err 반환 → 그것을 표출
- 노드 수가 변할 때 정렬 길이가 노드 수와 같은가
```

## 06 — Full 4-track graph

```text
4개 source (oscillator 또는 wav) → 각 channel strip → master mixer → master limit → output
1개 reverb send/return
```

검증.

```text
- 모든 트랙 동시 재생 시 output amplitude이 적정 (-3 dB ~ -6 dB peak)
- master limiter가 ceiling 보호
- reverb send가 둘 이상의 트랙에서 동시에 들어감
- 한 트랙 mute가 send도 함께 없애는가 (post-fader 가정)
```

## 그래프 시각화

학습 단계에서는 콘솔 출력이면 충분.

```text
[A] → [B] → [D]
      ↑
[C] ──┘
```

이런 ASCII 그림을 코드로 출력해 두면 디버깅에 도움이 된다.

또는 petgraph의 dot export로 graphviz 시각화도 가능하다.

```rust
use petgraph::dot::{Dot, Config};
println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));
```

## 콜백 통합 — graph를 cpal 콜백에 끼우기

graph가 만들어지고 정렬되면, 콜백 안에서는 정렬 배열을 따라 process() 호출만 하면 된다.

```rust
let stream = device.build_output_stream(
    &config,
    move |data: &mut [f32], _| {
        let frames = data.len() / channels;

        // (1) graph 모든 노드 process()
        graph.run(frames);     // 정렬 배열 따라 호출

        // (2) output 노드의 buffer를 interleaved로 cpal에 복사
        let out = graph.output_buffer();    // planar
        for f in 0..frames {
            for ch in 0..channels {
                data[f * channels + ch] = out.channel(ch)[f];
            }
        }
    },
    err_cb,
    None,
)?;
```

이 코드 안에서 `graph.run()`이 호출하는 것은 정렬된 노드들의 process() N번이다.

## 자주 하는 실수

- 콜백 안에서 graph.add_node 호출 → topological sort 부담 + 노드 할당.
- petgraph의 NodeIndex가 노드 삭제 후 invalid 됨을 인지 못함 → stale reference.
- master limit을 master mixer 안에 합침 → 06번 예제에서 분리해서 보는 의미가 사라짐.
- AudioBuffer를 매 사이클 새 Vec로 → 학습 단계에선 OK지만 production에선 미리 할당.
- send level을 audio thread에서 직접 설정 → atomic 또는 SPSC로 전달.

## 반드시 이해해야 할 것

- 작은 graph (01~04)는 chain의 graph 표현이지만, send/parallel은 chain으로 표현 안 된다.
- topological sort 결과는 한 사이클 동안 process()를 부를 순서다.
- 콜백 안에서는 graph 구조 변경 없음. 정렬 배열을 따라가는 일만.
- 06번 예제가 mod_player + mod_mastering의 합쳐진 모양이다.
