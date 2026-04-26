# 들어가며

이 책은 RuStudio 학습 시리즈의 열 번째 책이다.

지금까지의 모든 처리는 RuStudio 안에 있는 코드였다. 이 책은 처음으로 **경계를 넘는다**.

```text
RuStudio 내부 ←─────────→ 외부 호스트 (Reaper, Ardour, Bitwig, ...)
                  │
                  ▼
              Plugin 인터페이스
```

플러그인은 두 방향에서 등장한다.

```text
1. RuStudio가 host인 경우    : 외부 .vst3 / .clap 파일을 RuStudio에 로딩
2. RuStudio가 plugin인 경우  : RuStudio의 EQ/limiter/synth를 플러그인으로 빌드해서 외부 DAW에서 사용
```

이 책은 주로 **2번** — 플러그인을 만드는 쪽 — 에 집중한다. 1번(호스팅)은 별 책 분량이 필요한 큰 주제다.

## 이 책이 답하려는 질문

```text
1. 플러그인이 일반 라이브러리/실행파일과 무엇이 다른가?
2. host와 plugin 사이에는 어떤 종류의 인터페이스가 있는가? (audio, MIDI, parameters, state)
3. parameter가 왜 일반 변수가 아닌가?
4. plugin의 process callback은 cpal callback과 무엇이 같고 다른가?
5. 같은 처리 코드를 RuStudio 내부 모듈과 외부 플러그인으로 모두 빌드하려면 어떻게 분리하나?
```

## 이 책이 다루는 것

```text
1. host와 plugin의 경계 — 책임 분담
2. plugin format 개요 (VST3 / CLAP / AU / LV2)
3. parameter 모델 — 정의, automation, state save/restore
4. process callback의 실시간 제약 (cpal과 같은 규칙)
5. nih-plug crate로 Rust에서 플러그인 만들기
6. 같은 DSP 코드를 RuStudio와 플러그인 둘 다에 사용하는 분리 패턴
```

## 이 책이 다루지 않는 것

```text
✗ VST3 host 직접 구현 (외부 .vst3를 RuStudio가 로딩)
✗ CLAP host 직접 구현
✗ AU/Audio Unit (macOS-only) 깊이
✗ LV2 (주로 Linux) 깊이
✗ 플러그인 GUI 프레임워크 비교 깊이
✗ 사용자 라이선스 / 코드사이닝 / notarization
```

호스팅은 별도 큰 주제다. 이 책은 "플러그인을 만드는 쪽"의 시각을 단단히 한다.

## 핵심 crate

```text
nih-plug : Rust로 VST3/CLAP을 만드는 표준 framework
nih_plug_egui : 플러그인 GUI를 egui로
```

## 이 책을 다 읽고 나면

- 플러그인이 단독 실행 파일이 아닌 이유를 설명할 수 있다.
- VST3/CLAP/AU 셋이 어떤 차이가 있는지 표 한 줄씩으로 정리할 수 있다.
- parameter가 일반 변수와 다른 점 세 가지를 들 수 있다.
- plugin의 process()가 cpal 콜백과 같은 실시간 규칙을 따라야 하는 이유를 설명할 수 있다.
- nih-plug로 minimum gain plugin의 골격을 작성할 수 있다.
- RuStudio의 EqNode를 plugin으로도 빌드할 수 있도록 코드를 분리하는 방향을 안다.
