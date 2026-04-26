# Chapter 7 - 자주 하는 실수와 복습

## Plugin이란

- plugin이 별 프로세스 / 별 앱이라고 생각.
- audio device를 plugin이 연다고 가정 (host의 일).
- 한 포맷만 빌드 → 사용자 DAW가 미지원 시 못 씀.

## Host / Plugin 경계

- transport info 없이 BPM-sync 처리 시도.
- latency_samples 보고 누락 → PDC 깨짐.
- state save에 internal state(IIR z, ring buffer 채워진 데이터) 누락.
- audio thread / UI thread / loader thread 혼동.

## Parameter / Automation

- parameter를 일반 mutex/atomic으로 직접 만듦.
- range를 linear로만 → dB parameter UI가 어색.
- value_to_string / string_to_value 누락.
- smoothing 없이 automation → click.
- begin/end_set_parameter 누락 → record 깨짐.

## Process callback

- process() 안에서 Vec::push, Box::new.
- parameter를 매 sample atomic load (smoothing 없음) → click.
- MIDI timestamp 무시 → 모든 event를 buffer 시작에 적용.
- reset() 누락 → seek 후 노이즈.
- ProcessStatus::Tail 처리 누락 → reverb 끝 갑자기 잘림.
- planar/interleaved 가정 혼동.

## 빌드 / 배포

- crate-type cdylib 빠뜨림 → DLL 안 만들어짐.
- VST3_CLASS_ID 16 byte 미달 → host scan 실패.
- nih_export_clap! / nih_export_vst3! 누락.
- 한 plugin crate에 처리 코드와 plugin shim을 섞음 → 재사용 불가.

## 처리 단계 입출력 상태표

```text
plugin entry point   input: host load    output: instance      state: nih_export_*! 매크로
Plugin trait         input: lifecycle    output: 메타데이터    state: -
Params (declarative) input: parameter id output: value/string  state: nih-plug 자동 직렬화
process()            input: buffer + ctx output: in-place buf  state: 노드별 (IIR, env, etc.)
context.next_event() input: -            output: timed MIDI    state: nih-plug 큐
state save/load      input: bytes        output: bytes / setup state: parameters + custom
```

## Phase 9 체크리스트

```text
□ plugin이 동적 라이브러리이고 host process 안에서 로딩되는 이유 설명 가능
□ host와 plugin 책임 각각 3가지씩 들 수 있음
□ VST3 / CLAP / AU / LV2의 큰 차이를 한 줄씩 정리 가능
□ parameter 메타데이터 (name, range, default, value_to_string 등)을 외움
□ nih-plug에서 declarative parameter를 정의할 수 있음
□ smoothing이 자동 처리됨을 알고, 별도 코드 없이 click이 안 나는 이유 설명 가능
□ plugin process()가 cpal 콜백과 같은 실시간 규칙을 따른다는 사실을 외움
□ MIDI sample-accurate 처리 패턴을 plugin context로도 적용 가능
□ Reaper/Bitwig/Ardour 중 하나에서 minimum gain plugin 빌드/로드/동작 확인
□ core / internal / plugin 3-crate 분리의 의미와 이점을 설명 가능
```

## 03 ~ 09 책 도구의 재사용 지도

```text
03 SineOsc / Adsr / DelayLine     → synth plugin의 voice
04 Compressor / Limiter / EnvFollower → compressor plugin / limiter plugin
05 Biquad / EqChain / Smoothed    → EQ plugin (4 ~ 8 밴드 parametric)
06 FFT / window / dB              → spectrum analyzer plugin (audio passthrough + 측정)
07 file I/O                       → 보통 plugin에서는 안 씀 (host 책임)
08 graph                          → plugin 내부엔 sub-graph 가능, 외부 graph는 host
09 MIDI                           → instrument plugin (nih-plug context.next_event 사용)
```

## 다음 책으로 넘어가는 다리

다음 책은 `11_ui_audio_integration`이다.

10 책에서 plugin이 `editor()` hook으로 자체 GUI를 가질 수 있다는 점을 봤다. 11 책은 그 GUI 쪽 — RuStudio 본체나 plugin 양쪽에서 audio engine과 UI 사이의 통신을 어떻게 안전하게 다루느냐 — 를 다룬다.

GUI는 60 fps 정도로 도는 별 thread고, audio engine은 매 ms 단위 콜백을 도는 별 thread다. 둘 사이의 데이터 통신(meter / spectrum / waveform 등)이 11 책의 주제다.

## 한 줄 요약

> plugin은 host 안의 동적 라이브러리. 인터페이스는 audio + MIDI + parameter + transport + state. process()는 cpal 콜백과 동급의 실시간 함수. nih-plug가 boilerplate를 standardize해 주고, 우리가 짤 부분은 처리 자체. 같은 처리 코드를 RuStudio 내부와 plugin 양쪽에서 쓰려면 core crate 분리.
