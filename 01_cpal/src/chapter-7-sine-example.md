# Chapter 8 - 실습 예제: 440Hz 사인파 출력

이 예제는 `Phase 1`의 최소 성공 기준이다.

## 예제를 돌리기 전에 준비할 `Cargo.toml`

이 장의 예제는 아래 의존성을 기준으로 설명한다.

```toml
[package]
name = "cpal-practice"
version = "0.1.0"
edition = "2021"

[dependencies]
cpal = "0.15"
anyhow = "1"
```

- 패키지 이름은 자유롭게 정해도 된다.
- 의존성 이름은 반드시 `cpal`이다.
- 예제가 `anyhow::Result<()>`를 사용하므로 `anyhow`도 함께 넣는다.

## 예제

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("출력 장치 없음");
    let config = device.default_output_config()?;

    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;
    let mut phase = 0.0_f32;
    let frequency = 440.0_f32;
    let phase_increment = 2.0 * PI * frequency / sample_rate;

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            for frame in data.chunks_mut(channels) {
                let sample = phase.sin() * 0.3;
                for ch in frame.iter_mut() {
                    *ch = sample;
                }
                phase += phase_increment;
                if phase >= 2.0 * PI {
                    phase -= 2.0 * PI;
                }
            }
        },
        // 이건 데이터 콜백이 아니라 에러 콜백이다.
        // Chapter 5의 "콜백 안에서 하지 말아야 할 것"은 주로 위의 데이터 콜백을 가리킨다.
        |err| eprintln!("오디오 에러: {err}"),
        None,
    )?;

    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(())
}
```

## 이 예제가 검증하는 것

- 기본 출력 장치 선택
- 기본 출력 설정 조회
- 스트림 생성
- 콜백 실행
- 채널별 버퍼 채우기
- 샘플레이트 기반 위상 증가 계산
- 콜백 간 상태 유지

## 반드시 이해해야 할 것

### `phase`는 왜 콜백 밖에 있나

콜백 안에서 매번 `phase = 0.0`으로 시작하면 파형이 연속되지 않는다.

### `phase_increment` 공식은 왜 저 형태인가

```text
phase_increment = 2pi * freq / sample_rate
```

샘플 하나가 생성될 때마다 위상을 얼마나 진행시킬지 계산하는 식이다.

### 왜 `* 0.3`을 곱하나

- 귀 보호
- 클리핑 방지
- 학습용 예제에서 과도한 출력 회피

## 예제를 읽고 답할 질문

- 데이터 콜백 기준으로 보면, 이 예제에서 실시간 오디오 규칙 위반 요소는 무엇인가?
- `Stream`이 함수 끝에서 drop되면 무슨 일이 생기나?
- 샘플레이트를 44100으로 하드코딩하면 어떤 문제가 생기나?
