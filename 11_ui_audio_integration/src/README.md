# 들어가며

이 책은 RuStudio 학습 시리즈의 마지막 책이다.

지금까지 만든 audio engine은 다음을 다룰 수 있다.

```text
01 cpal              : 하드웨어 출력
02 mod_player        : 재생 운영
03 dsp_fundamentals  : 기본 처리 블록
04 mod_mastering     : 동적 처리와 측정
05 eq_biquad         : EQ
06 fft_and_spectrum  : 분석 데이터
07 audio_file_io     : 파일 입출력
08 audio_graph       : 노드 그래프
09 midi_integration  : MIDI 입력
10 plugin_architecture: 외부 플러그인
```

이제 마지막 한 가지 — 사용자에게 보이는 **UI** — 가 남았다.

## 이 책이 답하려는 질문

> 60 fps로 도는 GUI thread와 매 5 ms 콜백을 도는 audio thread가, 같은 데이터를 안전하고 부드럽게 주고받으려면 어떻게 해야 하는가?

이 한 질문이 이 책의 모든 chapter를 관통한다.

## 데이터가 흐르는 두 방향

```text
audio → UI 방향:
  meter (peak / RMS / LUFS)
  spectrum (FFT 결과)
  waveform (시간축 amplitude envelope)
  gain reduction meter
  현재 transport 위치
  underrun 카운트, CPU 부하 등 진단 정보

UI → audio 방향:
  fader / knob 변경
  transport 명령 (play / stop / seek / loop)
  graph 구조 변경 (트랙 추가/삭제, FX 삽입)
  parameter automation 그리기
  preset/project 로딩
```

이 두 방향은 다른 종류 데이터고, 다른 패턴으로 처리해야 한다.

## 이 책이 다루는 것

```text
1. UI thread와 audio thread를 왜 분리해야 하는가
2. 두 방향 데이터의 transfer 패턴 (atomic / SPSC / double buffer)
3. meter / spectrum / waveform / EQ curve의 데이터 모양과 갱신 주기
4. UI framework 선택 기준 (egui / iced / vizia)
5. immediate mode UI와 retained UI의 차이가 audio engine과 어떻게 만나는가
6. 화면 frame rate와 audio buffer rate의 정렬
```

## 이 책이 다루지 않는 것

```text
✗ 특정 UI framework의 깊은 사용법
✗ 디자인 (색, 레이아웃, 타이포그래피)
✗ accessibility / 접근성
✗ multi-window / docking 구조 (DAW의 깊은 UI 주제)
✗ project file format
✗ undo / redo 시스템
```

## 핵심 crate (옵션)

```text
egui   : immediate mode, 빠른 시작, plugin GUI에 자주 쓰임
iced   : Elm-architecture (retained mode 비슷), 보다 큰 앱에 적합
vizia  : audio app 전용 framework, lens-based reactive
```

세 가지 다 Rust에서 동작한다. 이 책은 어느 하나를 선택하지 않고 공통 패턴 위주로 다룬다.

## 이 책을 다 읽고 나면

- audio thread / UI thread / 분석 thread 셋의 분리 모델을 그림으로 그릴 수 있다.
- meter / spectrum / waveform 데이터를 audio thread에서 UI thread로 전달하는 표준 패턴 세 가지를 안다.
- 60 fps UI에서 audio thread에 lock을 거는 일이 왜 위험한지 설명할 수 있다.
- immediate mode와 retained mode의 차이를 audio engine 통합 관점에서 설명할 수 있다.
- 첫 UI를 만들 때 어떤 thread 구조로 시작하는 게 안전한지 안다.
