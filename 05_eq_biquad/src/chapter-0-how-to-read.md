# Chapter 1 - 이 책을 읽는 방법

이 책은 식이 많아 보이지만, 읽는 동안 머리에 담아야 할 것은 **그래프 한 장**과 **차분 방정식 한 줄**뿐이다.

## 그래프 한 장 — frequency response

이 그래프 한 장이 모든 EQ 책의 출발점이다.

```text
gain (dB)
   ▲
+6 │              ╱╲
   │             ╱  ╲
 0 ─────────────╱────╲─────────────
   │           ╱      ╲
-6 │
   └─────────────────────────► freq (log)
       100Hz   1kHz   10kHz
```

이건 "어떤 주파수에서 얼마만큼 깎거나 올리는가"를 그린 곡선이다. EQ 화면이 이 곡선을 사용자에게 그대로 보여주는 식이다.

이 책의 모든 기술적 내용은 결국 다음 한 가지 질문에 답하기 위한 것이다.

> 이 곡선이 나오게 만드는 시간 영역 코드는 어떻게 짜는가?

## 차분 방정식 한 줄 — biquad

답은 이거다.

```text
y[n] = b0·x[n] + b1·x[n-1] + b2·x[n-2]
                 - a1·y[n-1] - a2·y[n-2]
```

`x`는 입력, `y`는 출력, `[n-k]`는 k 샘플 전 값이다. 즉 biquad는 **현재와 직전 두 입력, 그리고 직전 두 출력**으로 이번 출력을 만든다.

이게 전부다. 이 식 위에 다음 변형이 얹힐 뿐이다.

```text
1. 5개 계수 (b0, b1, b2, a1, a2)를 어떻게 정하느냐 → cookbook
2. 위 식을 코드로 어떻게 정확히 옮기느냐 → Direct Form II Transposed
3. 사용자가 freq/gain/Q를 돌릴 때 계수를 어떻게 갱신하느냐 → smoothing
```

## 매 챕터에서 답해야 할 질문

```text
1. 이 필터의 입력/출력은 한 채널의 샘플 한 개씩인가?
2. 이 필터의 상태는 무엇인가? (몇 개의 z⁻¹ 지연 변수)
3. 이 필터의 계수는 어떻게 freq/gain/Q에서 만들어지는가?
4. 이 필터의 frequency response는 어떻게 생겼는가? (그림)
```

## 추천 독서 순서

1. `Chapter 2 - EQ와 Filter의 관계` — frequency response부터 잡는다.
2. `Chapter 3 - Biquad 구조` — 차분 방정식, 상태 변수, Direct Form II Transposed.
3. `Chapter 4 - RBJ Cookbook` — 표준 필터 6종의 계수 공식.
4. `Chapter 5 - Parametric EQ와 Smoothing` — 사용자 손잡이와 클릭 방지.
5. `Chapter 6 - 예제` — biquad LPF, peaking EQ, sweep 검증.
6. `Chapter 7 - 복습`.

## 참고

이 책의 cookbook 계수는 RBJ(Robert Bristow-Johnson)의 *Audio EQ Cookbook*을 따른다. 이게 실질적 표준이다.

```text
https://www.w3.org/TR/audio-eq-cookbook/
```

이 책에서는 그 계수를 **그대로 쓰는 단계**까지를 다룬다. 직접 도출하지 않는다.

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- biquad의 "2차"가 의미하는 것은 무엇인가?
- 같은 차분 방정식을 Direct Form I과 II Transposed로 짜면 무엇이 다른가?
- LPF, HPF, peaking EQ의 frequency response가 어떻게 다른지 그림 가능한가?
- Q는 무엇을 결정하는가?
- 사용자가 freq 손잡이를 빠르게 돌리면 무슨 일이 생길 수 있는가?
- biquad 두 개를 직렬로 걸면 어떤 일이 생기는가?
