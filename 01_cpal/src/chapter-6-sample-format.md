# Chapter 6 - 샘플 포맷과 채널 구조

## 왜 이걸 초반에 봐야 하나

`cpal`에서 출력 버퍼를 채울 때는 장치가 요구하는 샘플 포맷과 채널 수를 따라야 한다.

## 대표 포맷

```text
F32 -> -1.0 .. 1.0
I16 -> -32768 .. 32767
U16 -> 0 .. 65535
```

DSP 관점에서는 내부 연산을 보통 `f32`로 통일하는 편이 가장 단순하다.

## 예제: 장치 포맷 확인

```rust
use cpal::traits::DeviceTrait;

let supported = device.default_output_config()?;

println!("sample rate: {}", supported.sample_rate().0);
println!("channels: {}", supported.channels());
println!("format: {:?}", supported.sample_format());
```

## 예제: 채널 인터리빙 이해

```text
channels = 2
data.len() = 1024

-> 실제 프레임 수는 512
-> 레이아웃은 [L0, R0, L1, R1, ...]
```

## 설계 포인트

- 내부 DSP 버퍼를 `f32`로 둘 것인가?
- 장치 출력 포맷 변환은 어느 계층에서 할 것인가?
- stereo/mono를 콜백 내부에서 어떻게 일반화할 것인가?

## 체크포인트

- `data.len()`과 프레임 수는 항상 같은가?
- `channels=1`일 때도 코드가 안전한가?
- `f32`를 내부 표준 포맷으로 쓰는 이유를 설명할 수 있는가?
