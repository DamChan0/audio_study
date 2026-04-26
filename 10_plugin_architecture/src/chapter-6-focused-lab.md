# Chapter 6 - Focused Lab — 한 plugin으로 모든 단계 만지기

이 책은 broad example suite 정책이 아니라 **focused lab** 정책을 따른다. 한 작은 plugin을 끝까지 만들면서 4장의 모든 개념을 만지는 게 목적이다.

## 목표 plugin — Minimum Gain Plugin

```text
입력  : audio (stereo)
출력  : audio × gain
parameter: gain (-30 dB ~ +30 dB)
GUI    : nih_plug_egui로 단순 fader 한 개
빌드   : .vst3 + .clap
검증   : Reaper / Bitwig / Ardour 등에서 로드해서 동작 확인
```

이 한 가지가 동작하면 나머지 plugin은 변형일 뿐이다.

## 디렉토리

```text
10_plugin_architecture/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml          ← nih-plug, nih_plug_egui
    src/
      lib.rs            ← Plugin 구현체
      bin/              ← (binary는 없음. plugin은 cdylib)
```

`Cargo.toml`의 핵심.

```toml
[package]
name = "rustudio_gain_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
nih-plug = { git = "https://github.com/robbert-vdh/nih-plug" }
nih_plug_egui = { git = "https://github.com/robbert-vdh/nih-plug" }
```

## 단계 1 — Plugin struct와 Params

```rust
use nih_plug::prelude::*;
use std::sync::Arc;

struct GainPlugin {
    params: Arc<GainParams>,
}

#[derive(Params)]
struct GainParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for GainPlugin {
    fn default() -> Self {
        Self { params: Arc::new(GainParams::default()) }
    }
}

impl Default for GainParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}
```

이 코드의 의미.

```text
gain parameter:
  - 이름 "Gain"
  - default 1.0 (= 0 dB)
  - 범위 -30 dB ~ +30 dB, gain-skewed
  - 50 ms smoothing → click 자동 방지
  - 단위 " dB"
  - "0.5" → "-6 dB" 같은 string 변환 자동
```

## 단계 2 — Plugin trait 구현

```rust
impl Plugin for GainPlugin {
    const NAME: &'static str = "Rustudio Gain";
    const VENDOR: &'static str = "RuStudio";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels:  NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample *= gain;
            }
        }
        ProcessStatus::Normal
    }
}
```

```text
- 메타데이터 (NAME, VENDOR, ...)
- I/O layout: stereo in / stereo out
- MIDI 안 받음
- sample-accurate automation 사용
- params() — host에게 parameter list 노출
- process() — 핵심 처리
```

## 단계 3 — VST3 / CLAP 등록

```rust
impl ClapPlugin for GainPlugin {
    const CLAP_ID: &'static str = "studio.rustudio.gain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple gain plugin");
    const CLAP_MANUAL_URL:  Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for GainPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"RustudioGain1234";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(GainPlugin);
nih_export_vst3!(GainPlugin);
```

이 두 매크로 호출이 dynamic library의 entry point를 만든다.

## 단계 4 — 빌드

```bash
cargo xtask bundle rustudio_gain_plugin --release
```

(또는 nih-plug의 권장 방법). 결과물은 `target/bundled/Rustudio Gain.vst3` 같은 파일.

## 단계 5 — Host에서 로드 검증

```text
Linux : ~/.vst3, /usr/lib/vst3, /usr/local/lib/vst3
macOS : ~/Library/Audio/Plug-Ins/VST3, /Library/Audio/Plug-Ins/VST3
Windows: %CommonProgramFiles%\VST3
```

위치에 복사. DAW (Reaper / Bitwig / Ardour) 재시작. plugin scan에서 발견되면 동작.

검증.

```text
1. 트랙에 plugin 끼우기
2. parameter 자동 UI 또는 generic UI에 "Gain" slider 보이는가
3. slider 움직이면 amplitude가 변하는가 (귀와 meter)
4. -30 dB → 거의 무음
5. +30 dB → 디지털 클립 (host limiter가 잡거나 들리는 distortion)
6. 자동화 곡선 그리고 곡 진행 → smoothing 동작 확인
7. project 저장 → 재시작 → parameter 값이 보존
```

## 단계 6 — GUI 추가 (선택)

```rust
fn editor(&mut self, _: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    let params = self.params.clone();
    create_egui_editor(
        self.editor_state.clone(),
        (),
        |_, _| {},
        move |egui_ctx, setter, _state| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.label("Gain");
                ui.add(widgets::ParamSlider::for_param(&params.gain, setter));
            });
        },
    )
}
```

이 정도면 nih-plug가 자동 generic UI 대신 우리 egui 윈도우를 host에 보여 준다. UI 작업은 11 책에서 본격적으로.

## 7 — 다음 plugin들 (변형)

같은 구조 위에 처리만 바꾸면 된다.

```text
1. Gain Plugin                     (이 lab)
2. Stereo Width                    (M/S 처리)
3. Simple EQ (4-band parametric)   (05 책 EqChain 그대로)
4. Compressor (single-band)        (04 책 그대로)
5. LUFS Meter (audio in → audio out 그대로 + 측정 표시)
```

각 plugin은 처리 코드가 03 ~ 06 책의 모듈이고, plugin 외피만 새로 둔다.

## 8 — RuStudio 내부와 plugin의 코드 분리

권장 패턴.

```text
core crate    : 03 ~ 06 책의 모든 처리 블록 (SineOsc, Biquad, Compressor, Limiter, ...)
              : "no_std-ish" — 외부 IO 없음. struct + impl만.

internal crate: RuStudio 자체 audio engine (cpal + 03~07 + graph)
              : core를 dependency로

plugin crate  : nih-plug 기반 plugin
              : core를 dependency로
              : RuStudio internal에는 의존 안 함
```

이렇게 하면 같은 처리 알고리즘이 두 곳에서 그대로 동작하고, plugin 빌드가 RuStudio 전체 코드를 dragging하지 않는다.

## 자주 하는 실수

- crate-type이 `cdylib`이 아닌 default → DLL이 안 만들어짐.
- VST3_CLASS_ID 16 byte 안 채움 → 빌드 실패 또는 host scan 실패.
- nih_export_clap! / nih_export_vst3! 누락 → entry point 없음.
- AUDIO_IO_LAYOUTS에 mono/stereo 둘 다 등록 안 함 → mono 트랙에서 안 보임.
- smoothing 누락 → automation 시 click.
- core 처리 코드를 plugin crate에 직접 작성 → 다른 plugin이 못 빌리고 RuStudio도 중복 코드.

## 반드시 이해해야 할 것

- 한 작은 gain plugin이 plugin format의 모든 개념을 한 번에 만진다.
- core crate / internal crate / plugin crate 분리가 같은 코드를 여러 모드로 빌드 가능하게 한다.
- nih-plug는 framework이지 처리 코드를 작성해 주진 않는다. 처리는 우리 몫.
- 첫 plugin 검증은 host에서 직접 로드해서 한다 — 그게 plugin format 만족 여부의 진짜 테스트.
