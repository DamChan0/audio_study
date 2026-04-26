# Chapter 1 - 이 책을 읽는 방법

이 책에서는 두 가지 모드로 정보를 받는다.

```text
1. 구체적 사실  : "이 trait는 이 메소드를 갖는다", "DAG의 특성"
2. 설계 해석    : "이렇게 구성하면 RuStudio에서 자연스러울 것이다"
```

이 둘을 분리해서 읽는다. 후자는 변경 가능성이 있다.

## 매 챕터에서 답해야 할 질문

```text
1. 이 노드의 입력 포트 수와 출력 포트 수는?
2. 이 노드의 buffer는 누가 소유하나?
3. 이 노드는 stateful인가?
4. 이 노드 추가/제거가 graph 전체 처리 순서에 어떤 영향을 주나?
```

## 한 장 그림 — graph 처리 사이클

이 책 전체가 결국 한 콜백 사이클 동안 다음을 해내는 일이다.

```text
콜백 시작
  ↓
정렬된 순서대로 노드 process() 호출
  ↓
출력 노드(master output)의 buffer를 cpal에 넘김
  ↓
콜백 종료
```

각 노드는 자기 입력 buffer를 읽고, 처리하고, 자기 출력 buffer에 쓴다. 입력 buffer와 출력 buffer가 어떻게 연결되는가가 graph 구조다.

## 추천 독서 순서

1. `Chapter 2 - 왜 Graph가 필요한가` — chain의 한계가 무엇인지부터.
2. `Chapter 3 - AudioNode와 AudioBuffer` — 노드의 인터페이스와 버퍼 모델.
3. `Chapter 4 - DAG와 처리 순서` — 사이클 금지, topological sort.
4. `Chapter 5 - Mixer / Bus / Master` — DAW 표준 구조의 graph 표현.
5. `Chapter 6 - 예제` — 4-track 프로젝트의 graph 스케치.
6. `Chapter 7 - 복습`.

## 추가 의식 — 콜백 안 처리량

graph가 커질수록 한 콜백 안에서 호출되는 process() 횟수도 늘어난다. 노드 N개면 N번. cpal 콜백 시간 안에 끝나야 한다.

```text
N = 노드 개수
T_callback = 콜백 시간 (예: 5 ms = 240 frames @ 48k)
T_per_node = 노드 평균 처리 시간

조건: N · T_per_node < T_callback
```

이 책은 이 조건을 단순한 단일 스레드 모델 안에서 만족시키는 데 집중한다.

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- chain만으로 표현할 수 없는 구조 두 가지를 들 수 있다.
- AudioNode가 갖는 최소 메소드 세 가지를 안다.
- DAG에서 cycle이 있으면 무엇이 안 되는가?
- topological sort 결과의 길이는 항상 무엇과 같은가?
- 4-track + 1 send + master 구조에서 process() 호출 순서를 손으로 그릴 수 있다.
- graph 모델의 비용 두 가지를 들 수 있다 (간접 호출, 메모리, ...).
