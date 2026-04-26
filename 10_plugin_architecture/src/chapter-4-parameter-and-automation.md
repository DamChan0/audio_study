# Chapter 4 - Parameter Model과 Automation

## 1. Parameter는 일반 변수가 아니다

이 책에서 가장 중요한 한 줄.

> Plugin parameter는 host가 자동화/저장/UI 표시할 수 있는 **계약**이다. 단순 `f32` 변수가 아니다.

같은 처리를 RuStudio 내부 코드로 짜면 그냥 atomic이나 mutex 위에 얹어도 된다. plugin parameter는 다음을 추가로 보장해야 한다.

```text
1. host가 매 sample 다른 값을 줄 수 있어야 한다 (automation)
2. host가 임의 시점에 값을 강제 설정할 수 있어야 한다 (preset, project load)
3. host가 사용자에게 값을 표시할 때 plugin이 변환식을 제공한다 (-20 dB → "-20 dB" 같은 string)
4. plugin이 사용자 입력을 받았을 때 정해진 식으로 normalize한다
5. value 변경이 thread-safe해야 한다
6. 값 자체에 단위/범위/default가 명시되어 있어야 한다
```

이게 단순 `f32`보다 훨씬 강한 계약인 이유.

## 2. Parameter의 표준 메타데이터

```text
name              : "Threshold"
short_name        : "Thr"             (좁은 UI에서)
unit              : "dB"
range             : -60.0 ~ 0.0
default           : -20.0
step_size         : (선택) 정수일 때
automatable       : bool
smoothing         : (선택) 시간 상수 또는 곡선
value_to_string   : f32 → "-20 dB"
string_to_value   : "-20 dB" → f32
```

이 메타데이터를 host가 받아서 자동 UI에 사용하고, project file에 저장한다.

## 3. nih-plug의 parameter 정의

nih-plug에서 parameter는 declarative하게 정의된다.

```rust
use nih_plug::prelude::*;

#[derive(Params)]
struct GainPluginParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for GainPluginParams {
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
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}
```

이 한 곳에 정의하면 host UI에 자동으로 노출되고, project 저장/복원이 자동으로 처리된다.

## 4. Range 종류

parameter 값은 host에서 0.0 ~ 1.0 normalized로 다뤄진다. 그 사이에 어떤 곡선으로 매핑되느냐가 중요.

```text
Linear    : 0.0~1.0 → min~max 직선 매핑
Skewed    : 비선형 (gain dB처럼)
Reversed  : max~min (단순)
Logarithmic: log 곡선 (frequency cutoff 등)
```

dB-기반 parameter는 거의 항상 skewed range. 그래야 사용자가 노브를 돌릴 때 청각적 변화가 균등하게 느껴진다.

## 5. 자동화 (automation)

host가 parameter 값을 시간에 따라 변경할 수 있다.

```text
사용자가 자동화 곡선을 그림 (DAW 안)
   ↓
project 진행 시 host가 매 sample 또는 매 N sample마다 새 값을 plugin에 전달
```

plugin은 매 sample마다 변하는 값을 자연스럽게 처리해야 한다. 이게 곧 05 책의 parameter smoothing이다.

```rust
fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, ctx: &mut impl ProcessContext) -> ProcessStatus {
    for channel_samples in buffer.iter_samples() {
        let gain = self.params.gain.smoothed.next();
        for sample in channel_samples {
            *sample *= gain;
        }
    }
    ProcessStatus::Normal
}
```

`self.params.gain.smoothed.next()`가 매 sample 부드럽게 갱신된 값을 준다. 이게 nih-plug의 자동 smoothing이다 — 우리가 직접 1차 IIR 짤 필요 없음.

## 6. Sample-accurate vs block-accurate

```text
sample-accurate automation:
  매 sample마다 새 값. 부드러운 표현 가능. 비용 큼.

block-accurate (기본):
  buffer 시작 시 한 번 갱신. block 사이는 같은 값.
  대부분의 host가 기본 모드.
```

plugin이 sample-accurate를 명시적으로 지원한다고 host에 알려야 host가 그 모드로 줄지 결정한다. 학습 단계에서는 block-accurate로 시작.

## 7. Parameter Gesture — 사용자 입력 흐름

사용자가 노브를 돌릴 때 일어나는 일.

```text
1. begin_set_parameter(id)        : "노브 잡기 시작"
2. set_parameter(id, value) ...   : 매 mouse move마다 (또는 빠르게)
3. end_set_parameter(id)          : "노브 놓기"
```

이 begin/end가 중요하다 — host의 자동화 record 모드가 이걸 보고 한 번의 "사용자 동작"으로 묶는다. nih-plug에선 GUI 코드에서 이 사이클을 자동으로 다룬다.

## 8. State save / restore

parameter 값들은 plugin의 state의 일부다.

```text
host save:
  plugin이 모든 parameter의 현재 normalized value를 직렬화
  + custom internal state (만약 있으면)
  → bytes

host load:
  bytes → 모든 parameter를 그 값으로 설정
  → custom internal state 복원
```

nih-plug는 parameter 직렬화를 자동 처리. custom internal state(non-parameter)는 plugin 작성자가 별도 hook으로 처리.

## 9. Preset

preset은 그냥 미리 저장된 parameter 값들의 집합. 같은 save/load 메커니즘을 쓴다.

```text
preset.fxp / .vstpreset: 표준 binary 형식
또는
plugin이 자체 preset format을 둠
```

기본은 host 표준 형식을 따르는 것. plugin은 거의 일이 없다.

## 10. Modulation vs Automation 구분

비슷하지만 다른 두 개념.

```text
automation: host가 시간에 따라 parameter 값을 바꿈 (project file 안에 그려진 곡선)
modulation: 다른 신호가 parameter를 변조 (LFO, envelope follower, MIDI CC, sidechain)
```

modulation도 결국 plugin 안에서 이루어지지만, host가 보는 "표면 parameter" 값은 사용자가 설정한 값에 modulation depth를 더한 결과.

학습 단계에선 단순 automation까지만 다룬다.

## 자주 하는 실수

- parameter를 일반 mutex/atomic 위에 직접 만듦 → host와 통신하는 hook이 빠짐.
- range를 linear로만 두고 dB parameter를 노출 → 노브 동작이 청각적으로 어색.
- value_to_string / string_to_value 누락 → host UI에서 의미 없는 숫자만.
- smoothing 없이 자동화 곡선 적용 → click noise.
- begin/end_set_parameter 누락 → 자동화 record 시 불연속 점들이 생김.
- state save에서 parameter 외 internal state(IIR 상태 등) 빠뜨림.

## 반드시 이해해야 할 것

- parameter는 메타데이터 + 변환 + smoothing + automation + persistence가 묶인 강한 계약.
- nih-plug에서 declarative하게 정의하면 host UI / save / load가 자동으로 처리됨.
- automation 처리는 05 책 smoothing의 plugin 버전. nih-plug는 자동.
- modulation은 plugin 내부에서, automation은 host에서. 둘은 다른 레이어.
