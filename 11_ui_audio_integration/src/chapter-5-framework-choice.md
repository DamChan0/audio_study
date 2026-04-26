# Chapter 5 - Framework 선택 기준

## 1. Framework는 바뀐다 — 그 사실을 인정하기

UI framework 선택은 중요하지만 **항상 임시적**이다. 프로젝트 진행 중 또는 끝난 후 다른 framework로 옮겨야 할 일은 드물지 않다.

```text
egui   → custom GPU renderer
iced   → tauri (web 기반)
vizia  → 다른 audio-specific framework
tauri  → web view 기반
```

audio engine 코드가 framework에 종속되어 있으면 framework 교체 = engine 재작성. 이걸 피하는 게 이 장의 핵심 메시지다.

## 2. UI는 audio engine 본다, 반대 아니다

설계 원칙.

```text
✓ audio engine: framework 모름. 표준 인터페이스만 노출 (atomic / SPSC / double buffer).
✓ UI framework: audio engine을 본다. 그 인터페이스로 데이터를 꺼냄.

✗ audio engine이 framework type 사용 (egui::Context, iced::Element 등).
✗ audio engine이 UI framework 함수 호출.
```

이 분리가 깨지지 않으면 framework 교체는 UI 코드만 바꾸면 끝난다.

## 3. 후보 — egui

```text
모드     : immediate mode
강점     : 빠른 시작, 단순 코드, hot-reload, plugin GUI에 표준
약점     : 대규모 retained 구조 표현은 약간 어색 (복잡한 계층 위젯)
적합     : plugin GUI, tool window, debugger, 작은~중간 앱
```

immediate mode란 매 frame UI를 새로 그리는 방식. 상태는 별도 자료구조에 들고, 화면은 그 상태의 함수로 매번 재그리기.

```rust
egui::CentralPanel::default().show(ctx, |ui| {
    // 매 frame 호출됨
    let peak = audio_state.peak_l.load(Ordering::Relaxed);
    ui.add(meter_bar(peak));
    
    if ui.button("Play").clicked() {
        audio_state.transport_cmd.store(PLAY, Ordering::Relaxed);
    }
});
```

audio engine과 통합이 매우 자연스럽다 — 매 frame에 atomic을 읽고 그리고, 사용자 입력은 atomic을 쓴다.

## 4. 후보 — iced

```text
모드     : Elm-architecture (state + message + update + view)
강점     : 명확한 architecture, 큰 앱 구조에 좋음, 함수형 스타일
약점     : 약간 더 ceremonial, audio 통합이 immediate mode보다 한 단계 더
적합     : DAW 본체 같은 큰 앱, 명확한 state machine 원하는 경우
```

audio engine과 통합 시.

```rust
// audio thread → engine atomic
// iced subscription을 timer로 둠 (예: 매 16 ms tick)
// tick에서 atomic 읽어 message로 변환 → update → view에 반영
```

iced의 단방향 데이터 흐름이 audio engine의 자료구조와 잘 맞는다. 다만 boilerplate가 약간 더 많다.

## 5. 후보 — vizia

```text
모드     : reactive (lens-based)
강점     : audio app 전용 설계, plugin-friendly, 깊은 의존성 추적
약점     : 작은 커뮤니티, API 변동 잦을 수 있음
적합     : audio plugin, audio 전용 앱
```

audio app 전용으로 만들어졌기 때문에 meter / EQ curve 같은 위젯이 더 자연스럽게 표현된다. 단점은 framework 자체가 비교적 새롭고 변할 가능성이 있다는 점.

## 6. 다른 후보 — Tauri

```text
모드     : web view (HTML/CSS/JS) 기반
강점     : 디자이너 친화적, 풍부한 UI ecosystem
약점     : audio engine이 Rust인데 UI는 JS → IPC 비용
적합     : DAW 자체보다는 보조 도구, 또는 Tauri의 native side를 audio engine으로
```

매 frame audio data를 IPC로 보내는 비용이 부담일 수 있다. 60 fps에 spectrum 1024 floats는 약 60 kB/s — 실제로는 무리 없음.

## 7. 표준 인터페이스 패턴

framework가 무엇이든 audio engine은 다음 인터페이스만 노출한다.

```rust
pub struct AudioEngineState {
    // metrics (audio → UI)
    pub peak_l: AtomicU32,
    pub peak_r: AtomicU32,
    pub rms_l:  AtomicU32,
    pub transport_position: AtomicU64,
    pub underrun_count: AtomicU32,
    
    // commands (UI → audio)
    pub transport_cmd: AtomicU8,
    pub master_gain:   AtomicU32,
    
    // 큰 배열은 별도 모듈
    pub spectrum: Arc<DoubleBuf<f32>>,
}
```

framework 코드는 위 struct만 본다. framework 교체 시 위 struct는 그대로다.

## 8. 사용자 입력의 정밀도

framework마다 mouse move 빈도, button 인지 정확도, 키보드 latency 같은 차이가 있다.

```text
egui   : ~60 fps에서 mouse 이벤트 수집
iced   : event loop 기반, 정밀도 좋음
vizia  : audio plugin 환경 최적화
```

학습 단계에서는 큰 차이 없다. RuStudio가 실시간 입력을 매우 정밀히 다뤄야 하는 경우 (예: piano roll의 노트 dragging) framework 평가 시 이 점도 본다.

## 9. Hot reload / 반복 개발

```text
egui   : 빠른 build, immediate mode이라 변경 즉시 반영. 학습 빠름.
iced   : 빌드 비용 약간 더, structure 변경 시 update/view 둘 다 봐야 함.
vizia  : 비슷.
```

학습 단계에서 빠른 반복은 큰 가치다. 그래서 첫 UI는 보통 egui로 시작.

## 10. RuStudio 권장 시나리오

```text
plugin GUI (10 책):
  → nih_plug_egui (== egui) 거의 표준.

mod_player demo UI:
  → egui로 시작. 빠른 학습용.

본격 RuStudio 본체 UI:
  → egui로도 가능. 큰 앱이면 iced나 vizia 평가.
  → 핵심은 audio engine을 framework-agnostic으로 두는 것.
```

framework 결정에 너무 시간 안 쓴다. 첫 시도는 가장 빠른 것 (egui)으로, 한계가 보이면 그 때 다시 평가.

## 자주 하는 실수

- audio engine struct에 framework type 임포트 → framework 교체 시 cascade 변경.
- framework가 audio engine 자료구조를 lock으로 접근 → audio underrun.
- "이 framework가 다른 것보다 빠르다" 식의 micro-benchmark에 시간 낭비 → audio engine 안 정성이 훨씬 큰 문제.
- framework 결정에 1주 이상 소비 → 그동안 audio engine 진척 0.
- 한 번 정한 framework를 영원히 바꿀 수 없다고 가정.

## 반드시 이해해야 할 것

- audio engine은 framework를 모르고, framework가 audio engine을 본다 — 의존 방향 일관.
- atomic / SPSC / double buffer로만 노출된 인터페이스가 framework 교체를 가능하게 한다.
- 첫 UI는 빠른 반복이 중요. egui로 시작하는 게 학습/학습-RuStudio 단계에서 합리적.
- framework 선택에 너무 많은 시간 쓰지 않는다. 잘못된 첫 선택도 큰 비용이 아니다 — interface가 framework-independent하면.
