# Chapter 6 - 예제와 시각화 컴포넌트

이 장은 4가지 핵심 시각화 컴포넌트를 단계별로 만져 볼 권장 예제다. framework는 egui로 가정 (다른 framework도 동일 패턴).

## 권장 예제 목록

```text
01_atomic_meter      : audio thread peak → atomic → egui bar
02_double_buf_spectrum: analyzer thread spectrum → double buffer → egui line plot
03_live_waveform     : audio ring buffer → egui line plot (오실로스코프)
04_eq_curve_overlay  : EQ params atomic + spectrum double buffer → 겹쳐 그리기
05_transport_ctrl    : UI button → atomic → audio thread transport
06_full_demo         : 위 5개를 한 윈도우에 합침 (간소 mod_player UI)
```

## 디렉토리

```text
11_ui_audio_integration/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml          ← cpal, egui, eframe, ringbuf, triple_buffer, rustfft
    src/
      lib.rs            ← AudioEngineState, atomic helper, double buffer
      bin/
        01_atomic_meter.rs
        02_double_buf_spectrum.rs
        03_live_waveform.rs
        04_eq_curve_overlay.rs
        05_transport_ctrl.rs
        06_full_demo.rs
```

## 01 — Atomic Meter

```rust
// audio engine state
struct EngineState {
    peak: AtomicU32,
}

// audio thread
fn audio_callback(data: &mut [f32], state: &EngineState) {
    let mut p: f32 = 0.0;
    for &s in data.iter() {
        let a = s.abs();
        if a > p { p = a; }
    }
    state.peak.store(p.to_bits(), Ordering::Relaxed);
    // ...
}

// UI thread (egui)
fn ui(ctx: &egui::Context, state: &EngineState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let peak = f32::from_bits(state.peak.load(Ordering::Relaxed));
        let db = 20.0 * peak.max(1e-9).log10();
        let bar = egui::ProgressBar::new(((db + 60.0) / 60.0).clamp(0.0, 1.0));
        ui.label(format!("Peak: {:.1} dB", db));
        ui.add(bar);
    });
}
```

검증.

```text
- 입력 음량에 따라 막대가 움직임
- 60 fps에서 부드러움 (atomic load는 거의 무비용)
- 큰 신호에서 막대가 끝까지 + UI thread 안 멈춤
```

## 02 — Double-buffered Spectrum

```rust
// analyzer thread
fn analyzer_loop(rb: Consumer, spec: &triple_buffer::Input<Vec<f32>>) {
    let mut window = vec![0.0f32; N];
    let win_func = hann_window(N);
    
    loop {
        if rb.len() >= N {
            // N 샘플 채움 + window + FFT + dB → spec.write()
            ...
            *spec.input_buffer() = result;
            spec.publish();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

// UI thread (egui)
fn draw_spectrum(ui: &mut egui::Ui, spec: &mut triple_buffer::Output<Vec<f32>>) {
    let bins = spec.read();
    let plot = egui::plot::Plot::new("spectrum")
        .view_aspect(2.0);
    plot.show(ui, |plot_ui| {
        let pts: egui::plot::PlotPoints = bins.iter().enumerate()
            .map(|(i, &db)| [i as f64, db as f64])
            .collect();
        plot_ui.line(egui::plot::Line::new(pts));
    });
}
```

검증.

```text
- spectrum 그래프가 60 fps에서 부드럽게 갱신
- 입력 신호 주파수가 바뀌면 spectrum 피크 위치가 따라감
- analyzer thread가 멈춰도 UI가 안 멈춤 (frozen frame 표시는 가능)
```

## 03 — Live waveform

```rust
// audio thread → ring buffer (SPSC)
let last_n_samples = ring.read_latest_n(WAVE_LEN);

// UI thread
fn draw_waveform(ui: &mut egui::Ui, samples: &[f32]) {
    let plot = egui::plot::Plot::new("wave");
    plot.show(ui, |plot_ui| {
        let pts: egui::plot::PlotPoints = samples.iter().enumerate()
            .map(|(i, &s)| [i as f64, s as f64])
            .collect();
        plot_ui.line(egui::plot::Line::new(pts));
    });
}
```

검증.

```text
- 사인파 입력 → 매끈한 곡선
- 트리거 없이 매번 다른 위상이라 약간 흐를 수 있음 (oscilloscope처럼 freeze 옵션 추가 가능)
```

## 04 — EQ curve overlay

EQ 곡선은 parameter atomic만 읽고, UI thread가 식 한 번 더 평가해 그린다.

```rust
fn draw_eq_curve(ui: &mut egui::Ui, params: &EqParams) {
    let f0 = f32::from_bits(params.freq.load(Ordering::Relaxed));
    let g  = f32::from_bits(params.gain.load(Ordering::Relaxed));
    let q  = f32::from_bits(params.q.load(Ordering::Relaxed));
    
    let plot = egui::plot::Plot::new("eq");
    plot.show(ui, |plot_ui| {
        // 200 점 정도 evaluate
        let pts: egui::plot::PlotPoints = (0..200).map(|i| {
            let freq = log_spaced_freq(i, 200, 20.0, 20000.0);
            let mag_db = peaking_eq_response(f0, g, q, freq);
            [freq.log10() as f64, mag_db as f64]
        }).collect();
        plot_ui.line(egui::plot::Line::new(pts));
    });
}
```

`peaking_eq_response`는 cookbook 식을 freq 한 점에 evaluate. 200 점 × 60 fps = 12000 evals/sec. 여유.

배경에 02번 spectrum까지 함께 그리면 EQ UI가 완성된다.

## 05 — Transport command

```rust
// UI button
if ui.button("▶ Play").clicked() {
    state.transport_cmd.store(CMD_PLAY, Ordering::Relaxed);
}
if ui.button("■ Stop").clicked() {
    state.transport_cmd.store(CMD_STOP, Ordering::Relaxed);
}

// audio thread (콜백 시작)
let cmd = state.transport_cmd.swap(CMD_NONE, Ordering::Relaxed);
match cmd {
    CMD_PLAY => self.transport.play(),
    CMD_STOP => self.transport.stop(),
    _ => {}
}
```

`swap()`을 쓰면 명령이 정확히 한 번 처리된다 (load + reset이 atomic).

검증.

```text
- play/stop 버튼이 즉시 반응
- 빠른 연속 클릭에도 한 명령씩 정확히 처리
```

## 06 — Full demo (간소 mod_player UI)

위 5개를 한 윈도우에 모은다.

```text
+----------------------------------+
| ▶ ■ ⏹  | bar 1 beat 2.3       |    ← transport + position
+----------------------------------+
|                                  |
|  [waveform live]                 |    ← live waveform
|                                  |
|----------------------------------|
|                                  |
|  [spectrum + EQ curve overlay]   |    ← 02 + 04
|                                  |
|----------------------------------|
| L: ████░░░  -3.2 dB              |    ← peak meters
| R: █████░░  -1.8 dB              |
+----------------------------------+
```

이 demo가 mod_player UI의 minimum 모양이다. 한 윈도우 안에서 위 5개 패턴이 서로 안 충돌하고 동시에 동작한다.

## 검증 — 모든 데모에 공통

```text
- audio가 끊김 없이 들림 (UI 부담 줘도 audio 정상)
- UI가 60 fps 유지 (모든 위젯 합쳐도)
- 마우스 무거운 동작 (drag, scroll) 중에도 audio 정상
- 윈도우 최소화/복귀 시 audio는 계속 (UI는 멈춤 OK)
- 종료 시 클린하게 (audio thread → analyzer thread → UI 순서)
```

이 5개가 통과하면 thread 분리가 잘 됐다는 뜻이다.

## 자주 하는 실수

- audio thread가 spectrum 직접 만듦 → FFT가 콜백에 들어감.
- UI thread가 audio engine state에 mutex로 접근 → audio underrun.
- atomic store ordering을 SeqCst로 둠 → 불필요한 메모리 barrier.
- triple_buffer를 lock으로 wrap → 의미 없음.
- transport command를 atomic swap 안 하고 store만 → 같은 명령이 두 번 처리될 수 있음.

## 반드시 이해해야 할 것

- 4가지 시각화 컴포넌트(meter / spectrum / waveform / EQ curve) 각각이 어떤 패턴 위에 올라가는지 명확.
- UI는 60 fps에서 atomic 읽고 그리는 일만. 처리는 안 한다.
- 06번 full demo가 mod_player UI의 minimum 출발점.
- "audio가 안 끊긴다"가 통합의 첫 검증 기준.
