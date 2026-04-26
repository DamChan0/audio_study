# 들어가며

이 책은 RuStudio 학습 시리즈의 여덟 번째 책이다.

지금까지 모든 신호 흐름은 **직선 사슬**이었다.

```text
source → DSP → output
decoder → DSP → encoder
```

이 책은 그 사슬을 **그래프**로 일반화한다.

```text
source A ─┬─ EQ ─┬─ master
source B ─┘      │
                  └─ analyzer
```

DAW에서 사용자가 자주 만지는 구조 — 여러 트랙, send/return, 병렬 컴프, 마스터 버스 — 가 모두 이 그래프 모델에서 자연스럽게 표현된다.

## 이 책이 답하려는 질문

```text
1. 왜 직선 chain만으로는 부족한가?
2. 그래프의 한 "노드"는 어떤 인터페이스인가?
3. 노드들의 처리 순서를 어떻게 정하는가?
4. 그 처리가 cpal 콜백 안에서 어떻게 한 사이클로 돌아가는가?
5. mixer / bus / master 같은 DAW 핵심 구조가 그래프에서 어떻게 표현되는가?
```

## 이 책이 다루는 것

```text
1. AudioNode trait의 모양 — process, latency, reset
2. AudioBuffer 구조 — 누가 소유하고 누가 빌려쓰는가
3. DAG (Directed Acyclic Graph) — 사이클 금지의 이유
4. topological sort — 정확한 처리 순서 결정
5. mixer / FX bus / master bus 구조
6. zero-copy 시도와 그 한계 (이 단계에서는 단순한 모델로 시작)
```

## 이 책이 다루지 않는 것

```text
✗ 멀티스레드 그래프 처리 (DAG 병렬화)
✗ feedback (사이클이 있는 그래프) — 별도 latency model 필요
✗ 노드 외부 메시지 시스템 (event routing, automation lane)
✗ 플러그인 호스팅 (10 책의 영역)
✗ project file format (그래프 직렬화)
```

이 책은 **싱글 스레드, 사이클 없는, 정적으로 정렬된 그래프**까지 다룬다. 그 위로 올라가는 일은 별도 주제다.

## 주의 — 이 책은 일부가 설계 단계다

지금까지의 책(03~07)은 RuStudio 안에 비교적 직접 들어가는 구체적 처리 블록을 다뤘다. 이 책은 좀 더 **추상화된 구조**를 다룬다.

따라서 일부 코드 스니펫은 "RuStudio에서 그대로 컴파일되는 코드"라기보다 **설계 가이드**에 가깝다. 실제 구현 시 trait 시그니처나 buffer 소유권 모델은 프로젝트 진척에 따라 조정될 수 있다.

읽을 때 이 점을 의식한다.

## 이 책을 다 읽고 나면

- chain과 graph의 차이를 한 그림으로 설명할 수 있다.
- AudioNode trait의 최소 인터페이스(`process`, `reset`, `latency`)를 직접 작성할 수 있다.
- 왜 cycle이 없는 graph만 정렬 가능한지 설명할 수 있다.
- topological sort가 무엇을 출력하는지 안다.
- 4-track 프로젝트의 mixer / FX bus / master 구조를 노드 그래프로 그릴 수 있다.
- 직선 사슬과 비교해 graph가 어떤 비용/이점을 가지는지 평가할 수 있다.
