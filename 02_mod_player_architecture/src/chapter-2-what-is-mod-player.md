# Chapter 2 - mod_player는 무엇인가

## 한 문장 정의

> `mod_player`는 RuStudio 안에서 **재생 스트림의 생애와 transport 상태를 책임지는 모듈**이다.

여기서 한 단어씩 풀어보자.

```text
"재생 스트림"     : cpal의 출력 Stream. 콜백을 들고 있는 객체.
"생애"            : 만들기, 시작, 정지, 다시 만들기, 폐기.
"transport 상태"  : Stopped / Playing / Paused / Rebuilding / NoDevice.
"책임"            : 이 일을 다른 모듈로 흘려보내지 않고 여기서 처리한다.
```

## mod_player가 하는 일

```text
✓ cpal Stream을 만들고 들고 있는다
✓ play / pause / stop 같은 transport 명령을 받아 처리한다
✓ source(예: oscillator, sample player)와 출력 buffer 사이를 연결한다
✓ DSP chain을 콜백 안 흐름에 끼워 넣는다
✓ 장치가 사라지거나 설정이 바뀌면 Stream을 다시 만든다
✓ 현재 상태를 UI가 읽을 수 있게 노출한다
```

## mod_player가 하지 않는 일

```text
✗ DSP 알고리즘 그 자체 구현 (그건 dsp-core / mod_mastering / mod_eq의 일)
✗ 오디오 파일 디코딩 (07_audio_file_io 영역)
✗ 사용자 UI 레이아웃, 슬라이더 그리기 (11_ui_audio_integration)
✗ 플러그인 호스팅 (10_plugin_architecture)
✗ MIDI 입력 (09_midi_integration)
```

이 분리가 깨지면 가장 흔히 일어나는 일은 cpal 콜백 안에 EQ 슬라이더 mutex가 들어가는 사고다. 그래서 경계를 먼저 그어 둔다.

## "mod_player"와 "MOD Player"는 다르다

이 시리즈에서 가장 많이 헷갈리는 두 단어다.

```text
mod_player   : RuStudio 내부 모듈. 재생 담당. 이 책의 주제.
MOD Player   : .mod / .xm / .s3m 같은 tracker format 재생기.
               별도 프로그램. 12_tracker_mod_player 책의 주제.
```

코드 내 변수 이름, 디렉토리 이름, 문서 헤더에서 항상 소문자 `mod_player`로 통일한다. 이 책의 mod_player는 tracker와 무관하다.

## 단순화된 그림

```text
┌──────────── UI thread ────────────┐
│ play / pause / EQ slider / meter │
└──────────────┬───────────────────┘
               │ commands ↓        ↑ state
               │
┌──────────── mod_player ──────────┐
│ Stream 소유                       │
│ Transport state machine          │
│ Source registry                  │
│ DSP chain wiring                 │
│ Device lifecycle                 │
└──────────────┬───────────────────┘
               │ owns
               ▼
        cpal::Stream
               │ callback
               ▼
        audio thread (실시간)
```

UI는 mod_player에 명령을 보내고 상태를 읽는다. mod_player가 cpal Stream을 들고 있고, 그 Stream의 콜백 안에서 source/DSP가 돈다.

## 왜 이 모듈을 따로 두는가

`cpal::Stream`을 직접 UI 코드에서 만들고 들고 있어도 동작은 한다. 그런데 다음 시점에 빠르게 무너진다.

```text
- 사용자가 출력 장치를 바꾼다 → Stream을 새로 만들어야 함. UI에서?
- 장치가 갑자기 사라진다     → 어떤 화면을 보여줄지 결정 필요
- 샘플레이트가 바뀐다        → 모든 source/DSP의 sr를 업데이트
- play / pause를 빠르게 누른다 → 상태 일관성 필요
- DSP chain을 교체한다       → 콜백 안에서 안전하게 어떻게?
```

이 결정 5개를 한곳에 모은 모듈이 mod_player다. 그래서 이 모듈이 없으면 UI 코드와 cpal 코드 사이가 계속 뒤섞인다.

## 자주 하는 오해

- "mod_player가 큰 매니저 클래스라서 god object 아닌가?" → 책임이 좁다. transport와 stream lifecycle만이다. DSP는 안 한다.
- "Stream을 들고 있는 단순 wrapper 아닌가?" → wrapper는 오너십 + 상태기계 + 통신 채널을 함께 들지 않는다.
- "이거 그냥 audio engine 아닌가?" → audio engine이라는 단어는 너무 넓다. mod_player는 그 안의 한 계층이다.

## 반드시 이해해야 할 것

- `mod_player`는 "Stream + transport + lifecycle"의 모듈이다. DSP가 아니다.
- 소문자 `mod_player`와 대문자 `MOD Player`는 완전히 다른 주제다.
- 이 모듈이 없으면 cpal과 UI가 직접 마주쳐 빠르게 무너진다.
- 책임이 좁다는 점이 mod_player의 장점이다.
