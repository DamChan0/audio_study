# Chapter 1 - 이 책을 읽는 방법

## 핵심 관점 — 두 thread는 다른 시간을 산다

```text
audio thread:
  매 5~10 ms마다 깨어남.
  실시간 deadline. 늦으면 즉시 underrun.
  
UI thread:
  매 16 ms (60 fps) 또는 33 ms (30 fps) 깨어남.
  체감상 지연이 보임 (5 ms 이상).
  block 가능, 할당 OK, 사용자가 그렇게 느린 건 못 받아들임.
```

같은 데이터를 두 곳에서 다루면 빠르게 무너진다. 매 챕터에서 의식할 질문.

```text
1. 이 데이터는 어느 thread에서 생산되나?
2. 어느 thread에서 소비되나?
3. 갱신 주기는 어느 정도면 충분한가? (audio rate / UI rate)
4. 이 데이터가 한 frame 늦으면 사용자가 느끼는가?
```

## 한 장 그림 — 세 thread

이 책 전체가 결국 다음 그림 위에서 도는 일이다.

```text
[ audio thread ]            [ analysis thread ]            [ UI thread ]
   매 ~5 ms                    매 ~20 ms                       매 ~16 ms
   처리 + meter 갱신           FFT + smoothing                   화면 그리기
        │                         │                                │
        │ atomic store             │ double-buffered                │ atomic load
        ▼                         ▼                                ▼
   peak/RMS/transport         spectrum frame                  사용자 입력 → audio thread로
        ▲                                                        ▲
        │ atomic load                                             │ atomic store
        └─────────────────────────────────────────────────────────┘
```

이 셋의 분리는 06/08/10 책에서 조각조각 나왔다. 11 책은 그걸 정리하고 표준화한다.

## 추천 독서 순서

1. `Chapter 2 - UI와 Audio는 왜 분리해야 하나` — 두 thread의 요구사항 차이.
2. `Chapter 3 - Meter, Spectrum, Waveform 데이터 흐름` — 시각화 데이터의 종류와 모양.
3. `Chapter 4 - Thread 경계와 상태 전달` — 표준 패턴 세 가지.
4. `Chapter 5 - Framework 선택 기준` — egui / iced / vizia, 그리고 framework가 영향 주면 안 되는 영역.
5. `Chapter 6 - 예제` — 4개의 시각화 컴포넌트를 실제 코드 골격으로.
6. `Chapter 7 - 복습`.

## 읽으면서 의식할 한 가지

> framework는 바뀐다. audio engine은 안 바뀐다.

UI framework는 시간에 따라 바뀔 수 있다 (egui 버전, 새 framework 등장 등). audio engine 코드가 framework에 종속되면 framework 바꿀 때 engine 전부 다시 짜야 한다.

이 책의 모든 패턴은 framework-independent하게 설계한다. UI가 audio engine에 매달리는 게 아니라, audio engine이 framework-agnostic interface를 노출하고 framework가 그걸 본다.

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- audio thread에서 GUI 라이브러리 함수를 직접 부르면 왜 안 되는가?
- meter 갱신 주기를 audio rate로 두면 왜 의미가 없는가?
- spectrum 데이터를 audio thread → UI thread로 보내는 패턴 세 가지는?
- 사용자가 fader를 빠르게 움직일 때 audio thread에 어떻게 값이 도달하나?
- transport stop을 UI thread가 audio thread에 어떻게 알리나?
- immediate mode UI는 retained mode UI와 비교해 audio 통합에 어떤 장단점이 있는가?
- framework 교체 시 audio engine 코드가 바뀌면 안 되는 이유는?
