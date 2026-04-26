# Chapter 1 - 이 책을 읽는 방법

이 책은 mastering을 **음악적 기교**가 아니라 **블록 다이어그램**으로 본다.

그래서 매 처리를 만날 때마다 다음 4개 질문에 답할 수 있게 읽는다.

```text
1. 이 블록이 측정(meter)인가, 처리(processor)인가?
2. 이 블록이 다루는 양은 무엇인가? (peak / RMS / LUFS / true-peak)
3. 이 블록이 amplitude를 줄이는가, 아니면 그냥 보고만 있는가?
4. 이 블록의 상태는 무엇인가?
```

## meter와 processor를 먼저 분리한다

이 책의 모든 블록은 둘 중 하나다.

```text
meter:    신호를 보고 숫자를 출력. 신호 자체는 안 건드림.
          예) peak meter, RMS meter, LUFS meter, true-peak meter

processor: 신호를 받아서 amplitude를 바꿔서 내보냄.
          예) compressor, limiter
```

학습 단계에서 이 둘을 섞으면 빠르게 무너진다. 책을 읽는 동안 매 블록마다 "이건 미터냐 프로세서냐"부터 먼저 묻는다.

## 추천 독서 순서

1. `Chapter 2 - mod_mastering의 목표` — 이 모듈이 RuStudio에서 무엇을 책임지는지부터 본다.
2. `Chapter 3 - dB, Peak, RMS` — 모든 미터의 출발점.
3. `Chapter 4 - Compressor 구조` — 처음 만나는 dynamics 프로세서.
4. `Chapter 5 - Limiter와 True Peak` — 마지막 안전망과 그것이 보호해야 할 양.
5. `Chapter 6 - LUFS` — 최신 음량 표준의 큰 그림.
6. `Chapter 7 - 예제와 검증 방향` — 위 블록을 코드로 만질 때 따라야 할 골격.
7. `Chapter 8 - 복습` — 자주 하는 실수와 체크리스트로 닫는다.

## 이 책에서 외울 거의 유일한 식

```text
dB = 20 · log10(linear)
linear = 10^(dB / 20)
```

이 식 하나면 90%다. 나머지는 이 식을 **언제** 변환하느냐의 문제다 (보통 콜백 밖).

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- peak amplitude와 RMS amplitude는 무엇이 다른가?
- "0 dBFS를 넘었다"는 말은 정확히 무슨 뜻인가?
- compressor의 threshold를 -20 dB로 두고 ratio를 4:1로 두면 -10 dB 신호가 어떻게 나오는가?
- compressor와 limiter는 본질적으로 어떻게 다른가?
- 왜 sample-peak이 0 dBFS여도 D/A 변환 후 클립이 날 수 있는가?
- LUFS가 단순 RMS와 다른 점 두 가지는 무엇인가?

이 질문에 답이 나오면 mod_mastering 모듈 설계로 들어가도 된다.
