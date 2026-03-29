# Chapter 3 - Host -> Device -> Stream

## 핵심 추상화 계층

`cpal`은 아래 흐름으로 이해하면 된다.

```text
Host
 └── Device
      └── SupportedStreamConfig / StreamConfig
           └── Stream
                └── callback
```

## 1. `Host`

`Host`는 플랫폼 오디오 시스템 진입점이다.

```rust
use cpal::traits::HostTrait;

let host = cpal::default_host();
let available_hosts = cpal::available_hosts();
```

`Host`를 이해할 때 핵심은 이거다.

- "오디오 장치 목록을 제공하는 루트"
- "기본 장치를 찾는 출발점"
- Linux에서는 ALSA/JACK 같은 백엔드 차이가 여기에 걸림

## 2. `Device`

`Device`는 실제 입출력 장치다.

```rust
use cpal::traits::{DeviceTrait, HostTrait};

let host = cpal::default_host();
let device: Option<cpal::Device> = host.default_output_device();
```

여기서 `Option<cpal::Device>`라는 점이 중요하다.

근거:

- 타입 시그니처 자체가 `Option<Device>`다.
- `cpal` 공식 문서 `How to use cpal` 섹션에는 `default_*_device()`가 `Option<Device>`를 반환하며, "no device is available for that stream type on the system"인 경우가 있다고 적혀 있다.
- 출처: <https://docs.rs/cpal/latest/cpal/> (`How to use cpal`)

- 기본 출력 장치가 없을 수 있음
- 오디오 서버 상태가 이상할 수 있음
- 장치가 분리됐을 수 있음

앞의 첫 번째 항목은 공식 문서에 직접 근거가 있다. 뒤의 두 항목은 그 `None` 가능성을 실무적으로 해석한 예시다.

즉 장치 없음은 "예외적 사고"가 아니라, API 레벨에서 미리 표현된 정상적인 실패 경로다.

## 3. `SupportedStreamConfig`와 `StreamConfig`

이 둘은 반드시 구분해야 한다.

```rust
use cpal::traits::DeviceTrait;

let supported = device.default_output_config()?;
let config: cpal::StreamConfig = supported.into();
```

- `SupportedStreamConfig`: 장치가 허용하는 실제 설정 정보
- `StreamConfig`: 스트림 생성에 넘기는 설정 값

처음 학습 단계에서는 `default_output_config()`를 기준으로 시작하는 것이 가장 안전하다.

## 4. `Stream`

`Stream`은 열린 오디오 데이터 흐름이다.

```rust
use cpal::traits::{DeviceTrait, StreamTrait};

let stream = device.build_output_stream(/* ... */)?;
stream.play()?;
```

스트림을 이해할 때 꼭 기억할 것:

- 스트림을 열었다고 끝이 아님
- `play()`를 호출해야 실제 시작될 수 있음
- `Stream`이 drop되면 오디오도 멈춤

## 설계 전에 답할 질문

- `Stream`을 누가 소유할 것인가?
- 장치 재선택이 필요할 때 누가 스트림을 다시 만들 것인가?
- `mod_player`는 장치 선택 실패를 어떤 상태로 표현할 것인가?
