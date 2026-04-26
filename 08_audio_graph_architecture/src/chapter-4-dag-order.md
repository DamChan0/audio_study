# Chapter 4 - DAG와 처리 순서

## 1. 그래프를 실행하려면 순서가 필요하다

graph는 노드들과 그들의 연결이다. 매 콜백마다 모든 노드의 process()를 호출해야 한다.

문제는 호출 순서다.

```text
[A] → [B] → [C]

A의 출력이 B의 입력
B의 출력이 C의 입력

→ A를 먼저, 그 다음 B, 그 다음 C
```

A를 먼저 처리하지 않으면 B의 입력 buffer가 비어 있다 (또는 직전 사이클 값이 남아 있다). 결과가 한 사이클 늦어진다.

이 "올바른 순서"를 자동으로 만들어내는 알고리즘이 **topological sort**다.

## 2. DAG와 topological sort

DAG = Directed Acyclic Graph (방향 있고 사이클 없는 그래프).

DAG의 핵심 성질.

```text
DAG의 모든 노드를 일렬로 줄세울 수 있고,
어떤 edge (u → v)에 대해서도
u가 v보다 앞에 오는 순서가 항상 존재한다.
```

이 일렬을 만들어주는 게 topological sort. 출력은 노드 N개 짜리 순서 배열이다.

```text
[A] → [B] → [D]
  ╲       ╱
   ╲     ╱
    [C]→
       ╲
        [E]

A, B, C, D, E는 가능. 또는 A, C, B, D, E도 가능.
어느 쪽이든 "edge의 방향"은 깨지지 않는다.
```

Topological sort 알고리즘.

```text
1. in-degree 0인 노드(=들어오는 edge가 없음)를 큐에 넣음
2. 큐에서 노드 하나 꺼냄 → 결과 list에 추가
3. 그 노드의 out-edge들을 제거하고, 그 결과 in-degree 0이 된 노드들을 큐에 추가
4. 큐 빌 때까지 반복
5. 결과 list 길이가 노드 수와 같으면 성공, 아니면 cycle 존재
```

이 알고리즘은 O(V + E)다. graph 크기가 어떻든 빠르다.

## 3. cycle 검사

위 알고리즘에서 결과 길이 < 노드 수 라면 cycle이 있다는 뜻이다. graph 추가/연결 시 cycle 검사를 해서 사용자에게 거절해야 한다.

```text
사용자: "노드 X 출력 → 노드 Y 입력으로 연결" 요청
graph: 임시로 추가 → topological sort 시도 → 실패 시 거절
```

## 4. 정렬은 콜백 안이 아닌 콜백 밖에서

topological sort 자체는 빠르지만, 매 콜백마다 부를 필요는 없다. graph 구조가 바뀔 때만 한 번.

```text
graph 변경 (UI 스레드):
  변경 적용 → topological sort → 결과 배열을 audio thread로 전달

audio thread (콜백):
  미리 받은 정렬 배열을 따라 process() 호출만
```

배열 전달은 swap-pointer 또는 atomic 패턴으로. 자세한 lock-free 설계는 별도 주제.

## 5. process() 호출 사이클 한 번 그림

정렬된 graph가 있으면 콜백 한 번은 이렇게 흘러간다.

```text
콜백 시작
   │
   ▼
정렬 배열 [A, C, B, D, E] 따라 한 번씩 process() 호출
   │
   ▼
A.process(no inputs, &mut a_out)
C.process(no inputs, &mut c_out)
B.process(&[a_out], &mut b_out)
D.process(&[b_out, c_out], &mut d_out)
E.process(&[c_out], &mut e_out)
   │
   ▼
master output (= D 또는 E의 출력)을 cpal interleaved buffer로 복사
   │
   ▼
콜백 종료
```

각 노드는 자기 입력 buffer를 그래프가 미리 묶어 준 슬라이스로 받는다. 출력 buffer는 자기 소유.

## 6. graph 변경의 두 가지 종류

```text
구조 변경 (노드/edge 추가/삭제) : topological sort 다시 필요
값 변경 (파라미터, gain 등)     : sort 재실행 불필요
```

값 변경은 매 ms 발생할 수 있다 — 사용자가 fader를 움직이는 등. 이건 audio thread에서 atomic으로 받는다.

구조 변경은 비교적 드물다 — 트랙 추가/삭제, FX 삽입 등. 이건 콜백 사이의 짧은 시간에 lock 또는 swap으로 적용한다.

## 7. 단순한 model — 정적 graph

학습 단계에서는 graph가 곡 시작 전에 한 번 구성되고, 곡 진행 중에는 노드 구조가 안 바뀐다고 가정해도 된다.

```text
정적 graph:
  graph 구성 → topological sort → 정렬 배열을 audio thread로 → 곡 진행
  
동적 graph (advanced):
  곡 진행 중 노드 추가/삭제 → 매번 sort + 안전한 스왑
```

RuStudio mod_player부터 mixer까지의 단계는 정적 또는 거의 정적이다. 사용자가 트랙을 추가하는 빈도가 매 ms는 아니다.

## 8. 노드 처리 부하 추정

graph 정렬 후 각 노드의 평균 처리 시간을 합치면 한 콜백의 부하 추정이 가능하다.

```text
total = Σ T_per_node[i]

이 값이 콜백 시간보다 크면 underrun → graph 단순화 또는 buffer 키우기
```

이 측정은 audio thread 안에서 노드별 시간 측정 (별 thread 안전한 카운터)으로 해두면, UI에서 "이 노드가 가장 무겁다"를 보여줄 수 있다 — DAW의 "engine load" 표시.

## 9. 자주 쓰는 crate

```text
petgraph : graph 자료구조. 노드/edge 추가, 토폴로지 정렬, 사이클 검출 모두 제공
```

이 책은 petgraph를 그대로 쓰는 것을 권장한다. 직접 그래프 자료구조를 짜는 일은 거의 의미 없다.

```toml
[dependencies]
petgraph = "0.6"
```

## 자주 하는 실수

- topological sort를 콜백 안에서 매번 호출 → 부하 폭발.
- 사이클 검사 누락 → 사용자가 잘못된 연결을 만들 수 있음.
- 정렬 배열을 lock 잡고 audio thread에서 사용 → audio thread blocking.
- 구조 변경과 값 변경을 같은 코드 경로로 처리 → 매 fader 움직임마다 sort.
- 노드 처리 시간 모니터링 누락 → underrun 발생 시 어디가 원인인지 못 찾음.

## 반드시 이해해야 할 것

- DAG는 사이클 없는 방향 그래프. topological sort로 처리 순서를 결정한다.
- 사이클 검사는 그래프 변경 시점에 한 번. 콜백 안에서는 정렬된 배열만 따라간다.
- 구조 변경과 값 변경은 다른 종류. 후자는 atomic, 전자는 swap.
- petgraph 같은 검증된 자료구조 위에 올리는 게 정석.
