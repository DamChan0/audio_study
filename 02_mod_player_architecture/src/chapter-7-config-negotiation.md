# Chapter 7 - Config 협상과 Fallback

cpal에서 `default_output_config()`를 그냥 쓰면 잘 된다. 하지만 실제 앱은 다음 시점에 빠르게 무너진다.

```text
- 사용자가 출력 장치를 바꿈 (USB → 내장 스피커)
- 새 장치가 우리가 원하는 SR / 채널을 지원하지 않음
- 우리가 fixed buffer size를 원하는데 장치가 거부함
```

이 장은 mod_player가 config 협상과 fallback을 어떻게 다룰지 정한다.

## 우선순위 정책 — 세 가지를 정한다

```text
1. Sample rate 우선순위
2. Sample format 우선순위
3. Buffer size 우선순위 (또는 Default)
```

RuStudio에서 권장하는 기본값.

```text
Sample rate    : [48_000, 44_100, 96_000, 88_200, 24_000]
Sample format  : [F32, I16, U16]
Buffer size    : Default (또는 Fixed 256 → Fixed 128 → Default)
Channels       : [2, 1] (원하는 만큼 모든 옵션 시도)
```

## 협상 알고리즘

```rust
fn negotiate(device: &cpal::Device) -> Result<NegotiatedConfig, ConfigError> {
    let supported: Vec<cpal::SupportedStreamConfigRange> =
        device.supported_output_configs()?.collect();

    for &sr in &PREFERRED_SAMPLE_RATES {
        for &fmt in &PREFERRED_SAMPLE_FORMATS {
            for &ch in &PREFERRED_CHANNELS {
                if let Some(cfg) = supported.iter().find(|c| {
                    c.sample_format() == fmt
                        && c.channels() == ch
                        && c.min_sample_rate().0 <= sr
                        && c.max_sample_rate().0 >= sr
                }) {
                    let chosen = cfg.clone().with_sample_rate(cpal::SampleRate(sr));
                    return Ok(NegotiatedConfig::from(chosen));
                }
            }
        }
    }

    // 어떤 우선순위도 안 맞으면 default로 fallback
    Ok(NegotiatedConfig::from(device.default_output_config()?))
}
```

핵심.

- 우선순위 리스트를 위에서 아래로 시도
- 모두 실패 시 `default_output_config()`로 fallback
- 그것마저 실패하면 에러 반환 → mod_player는 NoDevice 상태로 전이

## NegotiatedConfig 추상화

cpal `StreamConfig`를 그대로 다루는 대신 mod_player 내부 추상을 두는 게 깔끔하다.

```rust
pub struct NegotiatedConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: cpal::SampleFormat,
    pub buffer_size: cpal::BufferSize,
}

impl From<NegotiatedConfig> for cpal::StreamConfig {
    fn from(c: NegotiatedConfig) -> Self {
        cpal::StreamConfig {
            channels: c.channels,
            sample_rate: cpal::SampleRate(c.sample_rate),
            buffer_size: c.buffer_size,
        }
    }
}
```

이 추상이 있으면 다음 시점에 도움이 된다.

```text
- DSP 모듈이 sr/channels를 알 때 (cpal 의존성을 안 받아도 됨)
- 테스트에서 가짜 config를 만들 때
- UI에 현재 config를 표시할 때
```

## Buffer size 정책

cpal의 `BufferSize`는 두 종류.

```text
Default       : 장치가 알아서 정함 (보통 OS 기본값)
Fixed(N)      : 정확히 N 프레임 단위로 콜백
```

Fixed의 장점은 latency 예측 가능. 단점은 장치가 거부하면 build 자체가 실패한다.

권장 정책.

```text
1. Fixed(256) 시도
2. 실패하면 Fixed(128)
3. 실패하면 Default
```

이 fallback은 negotiate 안에 한 번 더 포함될 수 있다.

```rust
const PREFERRED_BUFFER_SIZES: &[cpal::BufferSize] = &[
    cpal::BufferSize::Fixed(256),
    cpal::BufferSize::Fixed(128),
    cpal::BufferSize::Default,
];
```

## DSP 모듈에 sr / channels 전달하기

협상 결과는 DSP 인스턴스 생성 시점에 한 번 전달된다.

```rust
let cfg = negotiate(&device)?;
let osc = SineOsc::new(cfg.sample_rate as f64);
let env = Adsr::new(cfg.sample_rate as f32);
let chain = DspChain::with_config(&cfg);
```

콜백 안에서는 sr / channels가 변하지 않는다 (그 콜백의 lifetime 동안 고정). 변하는 순간 → Stream rebuild.

## SR 변경 시 처리

sample rate가 바뀌면 DSP 인스턴스의 시간 상수가 모두 어긋난다 (osc phase increment, envelope step, biquad coeffs, delay 길이).

```text
SR 변경 발생 → mod_player가 Rebuilding 상태로 전이
            → 새 cfg로 DSP factory 재호출
            → 새 chain 인스턴스 만들기
            → 새 Stream build
            → 빌드 성공 시 Playing/Stopped 복귀
```

DSP 모듈이 "sr를 갱신할 수 있는 메서드"를 제공해도 좋지만, 가장 안전한 정책은 "sr 변경 = chain 통째로 재생성"이다.

## Channels 변경 시 처리

채널 수가 바뀌면 DSP 인스턴스의 channel-bound 상태(예: stereo biquad 두 개)도 다시 만든다.

같은 정책 — chain 재생성.

## 협상 실패의 사용자 경험

negotiate가 다 실패하면 NoDevice 상태로 전이한다. 그 상태에서 UI는 다음을 보여줘야 한다.

```text
- 어떤 장치를 시도했는가
- 어떤 SR / 채널 / 포맷을 시도했는가
- 어떤 단계에서 실패했는가
- 사용자가 다시 시도할 수 있는 액션 (예: "장치 다시 선택")
```

이 정보는 협상 알고리즘이 NegotiationLog 같은 구조체로 모아 두면 좋다.

## 자주 하는 실수

- `default_output_config()`만 쓰고 fallback 없음 → 장치 변경 직후 빌드 실패하면 끝.
- sr를 콜백 안에서 매번 묻기 (`config.sample_rate.0`) → cpal 의존성이 콜백 안까지 들어옴. 콜백 시작 전에 한 번 캡처.
- DSP 인스턴스를 재사용한 채 SR만 바꿈 → osc 위상 진행이 어긋나 음정 오류.
- Fixed buffer size를 우선순위에 두지 않음 → latency가 들쭉날쭉.
- 협상 결과를 로그에 안 남김 → 사용자/디버깅이 막막.

## 반드시 이해해야 할 것

- mod_player는 "원하는 SR / 포맷 / 채널 / 버퍼 크기"의 우선순위 리스트를 가진다. 위에서 아래로 시도, 안 되면 default.
- 협상 결과는 `NegotiatedConfig`로 추상화한다. cpal 타입을 외부로 새지 않게.
- SR / channels가 바뀌면 DSP 인스턴스를 통째로 재생성한다 (= Stream rebuild).
- Fixed buffer size는 latency 예측에 좋지만 장치가 거부할 수 있어 fallback이 필요하다.
- 협상 실패의 정보는 사용자에게 그대로 보여 줄 수 있어야 한다.
