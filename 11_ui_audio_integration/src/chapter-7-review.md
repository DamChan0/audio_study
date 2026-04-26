# Chapter 7 - 자주 하는 실수와 복습

## Thread 분리

- audio thread에서 GUI 함수 직접 호출.
- UI thread에서 audio state에 mutex 잡고 접근.
- 같은 자료구조에 mutex 없이 raw 공유 (race).
- "잠깐만 lock 잡으면 되겠지" → audio underrun.

## Atomic / SPSC / Double buffer

- 큰 spectrum 배열을 atomic 하나로 다루려 시도 → 불가능.
- SPSC가 가득 찼을 때 처리 안 함 → silent drop.
- double buffer ordering (Acquire/Release) 누락 → race.
- triple_buffer를 lock으로 wrap → 의미 없음.
- atomic ordering을 SeqCst로 → 불필요한 barrier.

## 시각화 데이터

- spectrum을 audio thread에서 직접 만듦 → FFT가 콜백에 들어감.
- meter 갱신을 audio rate로 → atomic store가 너무 자주.
- live waveform을 ring buffer 없이 → 데이터 일관성 없음.
- EQ 곡선을 매 frame 1000+ 점으로 evaluate → 60~200 점이면 충분.
- transport position을 audio thread가 안 갱신 → UI가 멈춰 보임.

## Framework

- audio engine struct에 framework type import → 교체 시 cascade 변경.
- framework 결정에 1주 이상 소비.
- "이 framework가 다른 것보다 빠르다" 식 micro-benchmark에 시간 낭비.
- 첫 framework 선택을 영원한 결정으로 봄.

## 데이터 흐름 매핑 표

| 종류 | 방향 | 패턴 | 빈도 |
|---|---|---|---|
| peak/RMS/LUFS | audio→UI | atomic | UI 60 fps load |
| transport state | audio↔UI | atomic | command 시 |
| fader/knob | UI→audio | atomic | UI mouse move |
| transport command | UI→audio | atomic swap | 사용자 클릭 |
| MIDI events (record) | audio→UI | SPSC | event 시 |
| parameter automation | UI→audio | SPSC + smoothing | 사용자 그리기 |
| spectrum bins | analyzer→UI | double/triple buffer | 30~60 fps |
| waveform live | audio→UI | ring buffer | 60 fps read |
| EQ curve | params atomic→UI | UI에서 식 평가 | 60 fps |
| underrun count | audio→UI | atomic counter | 발생 시 |

## Phase 10 체크리스트

```text
□ audio thread / UI thread / analyzer thread 셋의 분리를 그림으로 그릴 수 있다
□ 세 패턴 (atomic / SPSC / double buffer)을 데이터 종류에 맞게 적용 가능
□ atomic Ordering 선택의 이유를 단순히라도 설명 가능
□ peak meter, spectrum, waveform, EQ curve의 데이터 모양과 갱신 주기를 외움
□ framework가 바뀌어도 audio engine은 안 바뀐다는 원칙을 코드로 반영
□ egui로 minimum mod_player UI를 골격까지 만들 수 있다
□ "audio가 안 끊긴다"를 UI 통합의 첫 검증 기준으로 삼는다
```

## 03 ~ 10 책 도구의 재사용 지도

```text
03 ring buffer / SPSC 패턴       → audio thread → analyzer / UI 다리
04 peak / RMS                    → meter UI 데이터
05 EQ parameters                 → EQ curve UI에서 식 평가
06 FFT / spectrum                → analyzer thread 결과 → double buffer
07 file decoding (별 thread)     → 디코딩 thread → audio engine
08 graph + atomic params         → fader / knob의 audio 적용
09 MIDI events (SPSC)            → MIDI thread → audio thread
10 plugin parameter framework    → UI thread ↔ audio thread의 framework 표준화
```

## 시리즈 끝맺음

이로써 `01_cpal`부터 `11_ui_audio_integration`까지의 RuStudio 학습 시리즈가 끝난다.

지금까지 다룬 흐름을 한 그림으로 정리하면.

```text
[ MIDI input ]
    │ SPSC
    ▼
[ Audio source nodes (file / synth / cpal input) ]
    │
    ▼
[ DSP nodes (EQ, compressor, ...) ]    ← 03 ~ 05 책
    │
[ Audio graph ]                         ← 08 책
    │
[ master mastering chain ]              ← 04 책
    │
[ cpal output ]                         ← 01 책
    │
    ▼
실제 소리

병렬로:
[ audio thread → analyzer thread (ring buffer) → spectrum → UI ]   ← 06, 11 책
[ UI thread ↔ audio thread (atomic / SPSC) ]                        ← 11 책
[ external host ↔ plugin (audio + MIDI + parameters) ]              ← 10 책
[ file I/O (별 thread) ]                                             ← 07 책
```

이 그림이 머리에 들어오면 RuStudio Phase 1 ~ Phase 8까지의 거의 모든 부품이 머리에 있다.

## 한 줄 요약

> audio thread는 시간 deadline 안에 sample을 만든다. UI thread는 60 fps에서 그것을 사용자에게 보여준다. 둘 사이는 lock-free 인터페이스 (atomic / SPSC / double buffer)로만 만난다. framework는 바뀌어도 인터페이스는 안 바뀐다.
