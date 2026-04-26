# RuStudio Study Series Plan

이 문서는 `study/01_cpal`, `study/02_mod_player_architecture` 다음에 이어질 학습 시리즈 계획을 정리한 handoff 문서다.

목적은 다음과 같다.

- Claude가 이후 `03`, `04`, `05`... mdBook을 만들 때 순서를 헷갈리지 않게 하기
- `mod_player`와 `DSP`, `MOD Player` 같은 용어를 섞지 않게 하기
- RuStudio의 실제 Phase 목표에 맞는 순서로 학습 자료를 쌓기

---

## 핵심 원칙

학습 순서는 아래 기준을 따른다.

```text
cpal 이해
-> mod_player architecture 설계
-> DSP fundamentals
-> mod_mastering math
-> EQ / FFT / file I/O / graph / MIDI / plugin / UI
```

즉 먼저 **소리를 내는 구조**를 이해하고, 그 다음 **샘플을 어떻게 계산하는가**로 들어간다.

---

## 용어 구분

### `mod_player`

RuStudio 내부의 **재생 담당 모듈**.

- stream ownership
- transport state
- control path
- device/config fallback
- source -> DSP chain -> output 구조

### `MOD Player`

`.mod`, `.xm`, `.s3m`, `.it` 같은 tracker format 재생기.

이건 `mod_player`와 완전히 다른 주제다.

**Claude는 이 둘을 절대 섞지 말 것.**

---

## 현재까지 확정된 순서

### `01_cpal`

목표:

- Host / Device / Stream 이해
- callback 모델 이해
- 실시간 오디오 규칙 이해
- 440Hz 사인파 예제
- DSP chain 삽입 포인트 감각 얻기

### `02_mod_player_architecture`

목표:

- `cpal`과 `mod_player` 경계 정리
- stream ownership
- transport state machine
- UI <-> audio thread control path
- config negotiation / fallback
- realtime-safe DSP chain 구조
- device lifecycle / failure state

---

## 다음 mdBook 시리즈 권장 순서

### `03_dsp_fundamentals`

이 책은 `mod_player` 다음에 오는 **기초 신호처리 빌딩 블록** 책이다.

핵심 범위:

- oscillator
- gain / dB
- pan
- mixer basics
- envelope
- delay buffer
- sample-by-sample processing 감각

이유:

- `02`가 구조라면, `03`은 그 구조 안에 들어갈 최소 DSP 블록이다.
- `mod_mastering`으로 바로 가기 전에 기초 샘플 처리 감각이 먼저 필요하다.

---

### `04_mod_mastering_math`

이 책은 `mod_mastering`을 위한 수학과 처리 흐름 책이다.

핵심 범위:

- dB <-> linear
- RMS / peak
- threshold / ratio / knee
- attack / release
- compressor block diagram
- limiter와 compressor 차이
- LUFS 개요
- true peak 개요

이유:

- RuStudio Phase 1 핵심 모듈이 `mod_mastering`이기 때문
- `03_dsp_fundamentals` 다음에 가야 이해가 자연스럽다

---

### `05_eq_biquad`

핵심 범위:

- biquad 구조
- Direct Form II Transposed
- RBJ cookbook
- peaking / shelf / HPF / LPF
- coefficient smoothing

---

### `06_fft_and_spectrum`

핵심 범위:

- FFT
- window function
- STFT
- dBFS 변환
- analyzer용 데이터 생성

---

### `07_audio_file_io`

핵심 범위:

- WAV 읽기/쓰기
- interleave / deinterleave
- int <-> float 변환
- symphonia 디코딩
- rubato resampling

---

### `08_audio_graph_architecture`

핵심 범위:

- AudioNode trait
- AudioBuffer
- node graph
- DAG
- mixer / bus / master chain

이유:

- graph는 중요하지만, `mod_player`와 기초 DSP가 먼저 깔려야 훨씬 잘 이해된다.

---

### `09_midi_integration`

핵심 범위:

- MIDI input
- note -> frequency
- velocity
- CC
- pitch bend
- MIDI sequencing

---

### `10_plugin_architecture`

핵심 범위:

- nih-plug
- parameter model
- plugin `process()` callback
- automation
- host/plugin boundary

---

### `11_ui_audio_integration`

핵심 범위:

- UI framework 선택
- meter / spectrum / waveform
- UI와 audio thread 데이터 교환

---

### `12_tracker_mod_player` (Optional)

이건 진짜 tracker format 재생기다.

핵심 범위:

- `.mod/.xm/.s3m/.it` 포맷
- pattern / row / tick
- sample voice mixer
- tracker sequencer

주의:

- 이 책은 `mod_player`와 관계없다.
- 선택 과목처럼 뒤로 미룬다.

---

## Claude에게 주는 직접 지시

Claude는 이후 학습 책을 만들 때 아래 규칙을 따라야 한다.

1. `02` 다음은 `03_dsp_fundamentals`로 간다.
2. `mod_player`와 `MOD Player`를 혼동하지 않는다.
3. 각 책은 mdBook 구조로 만든다.
4. 각 책은 순차 학습 구조를 따른다.
   - 정의
   - 구조
   - 동작 흐름
   - 예제
   - 흔한 실수
   - 복습
5. 핵심 주장에는 반드시 출처 또는 근거를 붙인다.
6. RuStudio 실제 목표와 연결되지 않는 주제 확장은 하지 않는다.
7. DSP를 `mod_player`보다 먼저 오게 재배치하지 않는다.

---

## 추천 실제 디렉토리 순서

```text
study/
  01_cpal/
  02_mod_player_architecture/
  03_dsp_fundamentals/
  04_mod_mastering_math/
  05_eq_biquad/
  06_fft_and_spectrum/
  07_audio_file_io/
  08_audio_graph_architecture/
  09_midi_integration/
  10_plugin_architecture/
  11_ui_audio_integration/
  12_tracker_mod_player/
```

---

## 한 줄 결론

`01`은 오디오 I/O, `02`는 재생 구조, `03`부터 DSP다.

즉 Claude는 다음 책을 만들 때 **`03_dsp_fundamentals`부터 시작하면 된다.**
