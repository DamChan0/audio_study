# Chapter 1 - 이 책을 읽는 방법

## MIDI를 "이벤트"로 보는 관점

지금까지의 모든 책은 **샘플 stream** 위에서 일했다.

```text
audio: 매 1/48000 초마다 샘플 한 개. 끊임없이 흐름.
```

MIDI는 다르다.

```text
midi: 일이 일어났을 때만 메시지 한 개. 보통 1 ms ~ 100 ms 단위로 드문드문.
```

이 둘을 같은 모델로 다루려고 하면 빠르게 무너진다. 매 챕터에서 의식할 분리.

```text
1. 이 단계가 다루는 것은 sample stream인가, event stream인가?
2. 이 단계의 timing은 sample 단위인가, ms 단위인가, beat 단위인가?
3. 이 데이터는 어느 thread에서 생기고 어느 thread에서 소비되나?
```

## 한 장 그림 — MIDI → Sound 흐름

이 책 전체가 결국 다음 흐름을 단단히 잡는 일이다.

```text
키보드 / 컨트롤러
      │
      ▼
[ OS MIDI driver ]
      │
      ▼
[ midir input thread ]   ← MIDI message 도착마다 콜백 호출
      │
      ▼
[ SPSC queue (timestamp 포함) ]
      │
      ▼
[ audio callback ]
      │  매 콜백 시작에 큐를 비움
      │  각 message에 대해:
      │    - 적절한 frame offset에 적용
      │    - synth/sampler 노드에 전달
      ▼
[ synth / sampler ]
      │
      ▼
[ audio buffer ]  ← graph 통해 master까지
```

## 추천 독서 순서

1. `Chapter 2 - MIDI는 오디오가 아니다` — 두 stream의 분리.
2. `Chapter 3 - Note, Velocity, CC, Pitch Bend` — 메시지 종류.
3. `Chapter 4 - Event에서 Sound로 연결하기` — thread 분리, timestamp, sample-accurate 처리.
4. `Chapter 5 - 시퀀싱과 타이밍` — BPM, beat, tick, SMF 파일.
5. `Chapter 6 - 예제` — 키보드 입력 → synth, MIDI 파일 → 시퀀서.
6. `Chapter 7 - 복습`.

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- MIDI 메시지의 평균 크기는? (약 3 byte) 1초 동안 100 메시지를 보내도 audio buffer에 비하면 무엇이 다른가?
- note 60은 어떤 음정에 어떤 주파수인가?
- velocity 100과 64의 amplitude 차이를 어떻게 만들 것인가?
- 키보드 → 소리까지의 latency는 어떤 항목들의 합인가?
- BPM 120, 4/4 박자에서 한 beat은 sample 몇 개인가? (48 kHz일 때)
- 콜백 안에서 MIDI event를 frame 정확도까지 적용하는 방법은?
