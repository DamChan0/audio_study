# Chapter 2 - 왜 Graph가 필요한가

## 1. Chain만으로 가능한 일

지금까지 우리가 본 모든 신호 흐름은 chain이었다.

```text
source → DSP1 → DSP2 → DSP3 → output
```

이 모델로 표현 가능한 일.

```text
✓ 한 source에 EQ + compressor + limiter 직렬 적용
✓ 파일 → SRC → DSP → 인코더
✓ 마이크 → effect → 헤드폰
```

문제는 DAW에서 자주 등장하는 "분기"와 "합산"이 직선이 아니라는 점이다.

## 2. Chain으로 표현하기 힘든 일

### (a) 여러 source가 한 master로

여러 트랙이 한 마스터 버스에서 합쳐진다.

```text
track A ─┐
track B ─┼─► master
track C ─┘
```

chain으로는 표현이 안 된다 — chain은 입력 1, 출력 1이다. 합산이라는 N→1 연산이 필요하다.

### (b) Send / Return FX

한 트랙이 자기 신호를 dry로 master에 보내면서, 동시에 일부를 reverb로 돌려서 처리한 wet 신호를 다른 경로로 master에 합치는 구조.

```text
track A ──┬──────────────────► master (dry)
          │
          └─► reverb send ──► reverb FX ──► master (wet)
```

같은 신호가 두 곳으로 나뉘는 1→N 분기가 필요하다.

### (c) Parallel compression

원 신호와 강하게 컴프된 신호를 합치는 기법.

```text
source ──┬─► EQ ─────────────────────┐
         │                            ├─► master
         └─► strong compressor ──────┘
```

같은 source가 두 갈래로 처리된 뒤 다시 합쳐진다.

### (d) Sidechain

킥 드럼 신호를 베이스 컴프의 sidechain 입력으로 넣는다. 같은 신호가 audio path가 아닌 control path로도 분기된다.

```text
kick ──┬──► master
       │
       └──► (sidechain input)
                 ▼
bass ────────► compressor ──► master
```

이 모든 구조의 공통점.

```text
1. 분기 (1→N)        : 한 신호가 여러 destination
2. 합산 (N→1)         : 여러 신호가 한 destination
3. 비-선형 연결        : 직선 chain 위에 얹지 못함
```

이걸 일반화한 모델이 그래프다.

## 3. 그래프 모델의 정의

```text
node : 처리 블록 (oscillator, EQ, compressor, mixer, output, ...)
edge : 한 node의 출력 buffer를 다른 node의 입력 buffer로 연결
```

위의 4가지 사례가 어떻게 표현되는지 그래프로 보면.

```text
(a) 합산
  [Track A] → [Master Mix]
  [Track B] →
  [Track C] →

(b) Send/Return
  [Track A] ─┬─► [Master Mix]
             └─► [Reverb] ──► [Master Mix]

(c) Parallel comp
  [Source] ─┬─► [EQ] ──────────────► [Master Mix]
            └─► [Strong Comp] ─────► [Master Mix]

(d) Sidechain
  [Kick] ──► [Master Mix]
  [Kick] ──► (sidechain) → [Bass Comp] ─► [Master Mix]
                                    ↑
  [Bass] ─────────────────────────  ┘
```

graph는 이런 분기/합산을 같은 한 가지 모델로 자연스럽게 표현한다.

## 4. Graph의 대가 — 복잡도 증가

대가는 분명하다.

```text
chain:
  - 호출 순서가 자명
  - 각 단계 buffer가 단순
  - 디버깅 쉬움

graph:
  - 호출 순서를 계산해야 함 (topological sort)
  - 노드 간 buffer 라우팅이 필요
  - 무한 루프 가능성 (사이클 검사)
  - 동적 변경 시 재정렬 비용
```

그래서 단순한 case에는 그래프를 안 쓰는 게 낫다.

```text
"한 source에 EQ + compressor + limiter"
  → chain으로 충분 (graph로 만들면 오버엔지니어링)

"4 트랙 + 2 FX bus + master"
  → graph가 필요
```

## 5. 사이클은 왜 안 되는가

graph가 사이클을 허용하면 latency 모델이 갑자기 복잡해진다.

```text
A → B → C → A
            ↑
   "A의 출력이 B의 입력인데
    B의 출력이 C의 입력이고
    C의 출력이 A의 입력"

→ 한 사이클의 결과를 알려면 다음 사이클이 필요. 무한 의존.
```

해결 방법은 cycle 안에 1샘플 또는 N샘플 delay를 강제로 끼워 "현재 사이클 vs 다음 사이클"로 시간 분리를 하는 것이지만, 이건 별도 주제다 (feedback delay network 등).

이 책에서는 **DAG (Directed Acyclic Graph)**, 즉 사이클 없는 그래프만 다룬다.

```text
DAG의 특성:
  cycle 없음 → topological sort 가능 → 정확한 처리 순서 결정
```

## 6. RuStudio에서 graph가 필요해지는 시점

mod_player와 03~07까지의 chain 구조로도 한참 갈 수 있다.

graph가 본격 필요해지는 시점.

```text
- 사용자가 트랙을 N개 만들고 동시에 재생할 때 (mixer)
- send/return FX UI를 노출할 때
- multi-band compressor 또는 parallel comp UI를 노출할 때
- 플러그인이 sidechain 입력을 갖는 시점 (10 책)
```

처음부터 graph로 시작할 필요는 없다. 단순 chain으로 출발해서, 위 요구가 나타날 때 graph로 확장하는 게 자연스럽다.

## 자주 하는 실수

- 모든 처리를 graph로 모델링 → 단순 case도 무거워짐.
- graph 위에 cycle을 허용하는 식으로 시작 → 처리 순서가 결정 안 됨.
- node 간 buffer를 매 사이클 새로 할당 → 메모리 폭발.
- graph 변경(노드 추가/삭제)을 audio thread에서 직접 → race / lock 문제.

## 반드시 이해해야 할 것

- chain이 표현 못 하는 구조 — 합산, 분기, send/return, parallel, sidechain — 가 graph가 필요해지는 진짜 이유다.
- graph는 chain의 일반화. chain으로 충분하면 graph로 넘어가지 않는다.
- DAG는 cycle 없는 graph. 이 책은 DAG만 다룬다.
- graph는 비용이 있다 — 정렬, buffer 라우팅, 동적 변경.
