# 들어가며

이 책은 RuStudio 학습 시리즈의 아홉 번째 책이다.

지금까지 graph의 source는 두 종류였다.

```text
1. cpal 입력 (마이크/라인입력)
2. 파일 디코더 (07 책)
```

이 책에서 세 번째 종류가 등장한다 — **MIDI**.

```text
키보드/시퀀서 → MIDI event → synth/sampler 노드 → audio buffer
```

핵심은 한 가지 사실이다.

> MIDI는 오디오가 아니다. 소리가 아니라 "이벤트"다.

이 한 줄을 단단히 잡지 않으면 09 책 전체가 흐리게 보인다.

## 이 책이 답하려는 질문

```text
1. MIDI는 무엇이고 왜 audio buffer와 분리해서 다뤄야 하나?
2. note number를 frequency로 어떻게 바꾸나?
3. velocity, CC, pitch bend가 audio 파라미터에 어떻게 연결되나?
4. 키보드 → 콜백 → 소리 사이의 timing은 어떻게 정렬하나?
5. MIDI 시퀀스를 시간축에서 정확히 재생하려면 무엇이 필요하나?
```

## 이 책이 다루는 것

```text
1. MIDI 메시지 종류 — note on/off, velocity, CC, pitch bend, program change
2. note number ↔ frequency 변환
3. event를 audio thread로 안전하게 전달하는 패턴 (MIDI thread → SPSC → audio thread)
4. MIDI sample-accurate timing — 콜백 안 frame 인덱스 단위까지 정확하게
5. MIDI 파일 (SMF) 파싱과 시퀀싱
6. tempo / BPM / tick / SMPTE timing의 관계
```

## 이 책이 다루지 않는 것

```text
✗ 폴리포니 음성 관리의 깊은 알고리즘 (voice stealing 정책 등 일부만)
✗ MPE (MIDI Polyphonic Expression)
✗ MIDI 2.0 / Universal MIDI Packet
✗ SysEx 활용
✗ HUI / Mackie 컨트롤러 protocol
```

## 핵심 crate

```text
midir : OS의 MIDI 디바이스 열거 / 입출력 / 콜백 등록
midly : SMF (.mid 파일) 파싱과 인코딩
```

## 이 책을 다 읽고 나면

- MIDI가 sample buffer가 아닌 이유를 한 줄로 설명할 수 있다.
- MIDI note 69가 왜 440 Hz인지 식으로 보여줄 수 있다.
- velocity 0~127을 amplitude / volume에 매핑하는 두 가지 방식의 차이를 안다.
- 키보드를 누른 시점 → 소리가 나는 시점 사이에 어떤 latency 단계가 있는지 그릴 수 있다.
- BPM 120일 때 1 beat이 몇 ms / 몇 sample인지 계산할 수 있다.
- 콜백 안에서 sample-accurate한 MIDI event 처리 패턴을 안다.
