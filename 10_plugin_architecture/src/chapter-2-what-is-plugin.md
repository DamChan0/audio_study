# Chapter 2 - Plugin이란 무엇인가

## 1. 한 문장 정의

> 플러그인은 host(=DAW)가 자기 audio engine 안에서 호출하는 처리 단위. 별 프로세스도 아니고, 별 GUI 앱도 아니다.

다음의 모든 일을 host가 한다.

```text
host의 일:
  - audio device 열기 (cpal에 해당)
  - transport (play/stop/record/loop)
  - 트랙 구성, 라우팅, 그래프 (08 책)
  - audio buffer를 plugin에 전달
  - plugin parameter UI를 화면에 표시 (또는 plugin의 자체 GUI를 embed)
  - plugin state 저장/복원 (project file에)
```

다음을 plugin이 한다.

```text
plugin의 일:
  - audio buffer 처리 (process 함수)
  - parameter 정의 (이름, 범위, 단위, default)
  - 자기 state 직렬화/복원
  - (선택) 자체 GUI 그리기
```

명확한 분리다. 이 분리가 plugin format이 강제하는 모든 인터페이스의 형태를 만든다.

## 2. 왜 별 프로세스가 아닌가

원래 plugin format이 만들어진 의도가 "host의 audio engine 안에 그냥 함수처럼 들어가서 빠르게 호출되기"였다.

```text
별 프로세스 모델 (RPC):
  host process ─[IPC]─► plugin process
  
  - process 경계에서 latency 발생
  - audio buffer 복사
  - 동기화 비용
  - 실시간 제약 만족 어려움
```

```text
같은 프로세스, 동적 로딩 (현재 표준):
  host loads .vst3 / .clap as shared library
  plugin functions are called directly
  
  - 거의 함수 호출 비용
  - 메모리 공유
  - 실시간 제약 만족 가능
```

그래서 plugin은 항상 **동적 라이브러리** (`.dll`, `.dylib`, `.so`)다. 그걸 OS-별 약속에 따라 묶은 것이 .vst3 / .clap 등의 포맷.

```text
.vst3   : Steinberg가 정의. macOS는 bundle, Windows는 폴더 구조 + .vst3 파일.
.clap   : 비교적 새 포맷, 단순한 단일 .clap 파일. 오픈 표준.
.component (.au): macOS Audio Unit, .component bundle.
.lv2    : 주로 Linux, 폴더 + .so + manifest.
```

## 3. 같은 plugin이 여러 포맷으로 빌드되는 이유

각 포맷이 정의한 ABI/메타데이터/feature가 다르다. 한 plugin이 여러 host에서 동작하려면 여러 포맷으로 빌드해야 한다.

`nih-plug`의 강점은 같은 Rust 코드 한 벌로 VST3와 CLAP 둘 다 출력해 준다는 점이다.

```text
nih-plug:
  Rust source one set
    │
    ├─► .vst3
    └─► .clap
```

이 책에서는 이 둘을 주 타겟으로 한다.

## 4. plugin 종류 — Effect / Instrument / Analyzer

처리 성격에 따라 분류.

```text
Effect (audio in → audio out)
  EQ, compressor, limiter, reverb, delay, chorus, ...
  
Instrument (MIDI in → audio out)
  Synth, sampler, drum machine, ...
  
Analyzer (audio in → audio out + visual)
  Spectrum analyzer, LUFS meter, ...
  대개 audio는 그대로 통과시키고 측정만 함
```

각 종류가 host에 자기를 어떻게 소개하느냐가 다르다 (input/output port 수, MIDI input 여부, latency 등).

## 5. plugin lifecycle

host가 plugin을 다루는 한 사이클.

```text
1. host가 plugin DLL을 로드
2. host가 plugin의 entry point를 호출 → plugin instance 생성
3. host가 plugin의 sample_rate, max_buffer_size 같은 설정 알림
4. host가 plugin.activate() 호출 (이 시점부터 process 가능)
5. host가 매 audio cycle 마다 plugin.process(buf, events, params) 호출
6. 사용자가 노브 만지면 host가 parameter 변경 통지
7. host가 plugin.deactivate() (재설정 또는 종료 직전)
8. host가 plugin instance drop
9. host가 DLL 언로드
```

plugin은 이 lifecycle 안에서 자기 메모리/상태를 관리해야 한다.

## 6. 자체 GUI vs Generic UI

plugin이 자체 GUI를 안 만들면 host가 일반 UI로 parameter들을 표시한다 (slider 목록).

```text
generic UI:
  host가 parameter list를 받아서 자동으로 slider/meter 그림.
  플러그인 시각적 정체성은 없지만, 작은 utility 플러그인엔 충분.

custom UI:
  plugin이 자체 GUI 코드 가짐 (egui, vizia 등)
  host가 그 GUI를 자기 윈도우 안에 embed
  플러그인 시각적 정체성을 만들 수 있음
```

이 책은 custom UI를 깊이 다루지 않는다 (11 책의 영역과 겹친다). nih-plug + egui 조합 정도까지만.

## 7. RuStudio 관점

```text
RuStudio가 plugin을 빌드해서 사용하는 시나리오:

A. 같은 모듈을 외부 DAW용으로 배포
   사용자가 다른 DAW에서 RuStudio EQ / mod_mastering을 쓸 수 있음

B. RuStudio 자체가 외부 플러그인을 호스팅 (별 책 분량의 작업)
   사용자가 자기 VST3/CLAP을 RuStudio 트랙에 끼움

이 책은 A 위주.
```

A를 위해 코드를 어떻게 분리할지가 6장 마지막에서 다룬다.

## 자주 하는 실수

- plugin을 별 프로세스/별 앱이라고 생각 → 실제는 host process 안의 함수.
- plugin이 자기 audio device를 연다고 가정 → audio I/O는 host의 일.
- 한 포맷만 빌드 → 사용자 DAW가 그 포맷을 지원 안 하면 못 씀.
- plugin lifecycle을 무시하고 한 번에 모든 자원 할당 → activate/deactivate 사이의 reset 누락.

## 반드시 이해해야 할 것

- 플러그인은 host가 dynamic-load하는 라이브러리다. 별 프로세스가 아니다.
- audio I/O / transport / 트랙 라우팅은 host의 일. 플러그인은 buffer 처리와 parameter만.
- nih-plug 같은 framework는 같은 Rust 코드로 여러 포맷을 출력해 준다.
- plugin lifecycle (load → instantiate → activate → process → deactivate → unload)을 고려해 자원을 관리한다.
