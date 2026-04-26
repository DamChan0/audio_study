# Chapter 1 - 이 책을 읽는 방법

이 책에서 가장 중요한 분리는 이 셋이다.

```text
1. file format     : 파일이 디스크에서 어떤 모양인가 (.wav, .mp3, ...)
2. sample format   : 메모리에서 한 샘플의 표현 (i16, i24, f32, ...)
3. channel layout  : 메모리에서 채널 배열 (interleaved vs planar, mono/stereo/5.1)
```

이 셋이 섞이면 빠르게 무너진다. 매번 "지금 어느 단계에 있는가"부터 묻는다.

## 매 챕터에서 답해야 할 질문

```text
1. 이 단계의 입력 포맷은 무엇인가?
2. 이 단계의 출력 포맷은 무엇인가?
3. 이 단계는 실시간에서 돌 수 있는가, 오프라인 전용인가?
4. 이 단계가 메모리에 얼마나 많은 데이터를 올려놓는가?
```

## 파일 → 신호 → 파일 한 장 그림

이 책의 모든 흐름을 한 장으로 그리면 이렇다.

```text
.wav / .mp3 / .flac
    │
    ▼
[ container parse ]  ← 헤더, 메타데이터, 패킷 경계
    │
    ▼
[ codec decode ]     ← PCM 또는 압축 → raw 샘플
    │
    ▼
[ sample format convert ]  ← i16/i24/i32 → f32, planar ↔ interleaved
    │
    ▼
[ resample if needed ]    ← 48k ↔ 44.1k
    │
    ▼
[ DSP / analysis ]        ← 03~06 책 도구들
    │
    ▼
[ resample if needed ]    ← 출력 sample rate에 맞춤
    │
    ▼
[ sample format convert ] ← f32 → i16 등 (clamp 주의)
    │
    ▼
[ encoder ]               ← WAV 등으로 패킹
    │
    ▼
.wav 출력 파일
```

이 책의 6개 chapter가 위 흐름의 각 칸에 대응한다.

## 추천 독서 순서

1. `Chapter 2 - 파일 기반 오디오 흐름` — 위 그림의 큰 그림.
2. `Chapter 3 - WAV와 샘플 포맷 변환` — 시작점이자 끝점인 WAV.
3. `Chapter 4 - 압축 포맷 디코딩` — packet 기반 디코딩의 흐름.
4. `Chapter 5 - 샘플레이트 변환` — 왜 단순 스킵으로 안 되는가.
5. `Chapter 6 - 예제` — 실제 파이프라인을 코드로.
6. `Chapter 7 - 복습`.

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- WAV는 압축인가, 무압축인가?
- i16 PCM에서 -32768과 32767은 amplitude 어디에 매핑되는가?
- interleaved와 planar 중 cpal 콜백은 어느 쪽을 받는가?
- mp3 디코더가 1 packet을 디코딩하면 보통 몇 샘플이 나오는가?
- 48 kHz → 44.1 kHz 변환을 단순히 매 N 번째 샘플 골라내기로 하면 어떤 문제가 생기는가?
- 같은 EQ 처리를 실시간과 오프라인에서 쓸 때 어디서 어떻게 분기되는가?
