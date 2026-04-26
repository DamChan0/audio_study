# Chapter 5 - 샘플레이트 변환 (Sample Rate Conversion)

## 1. 왜 단순 인덱스 스킵으로 안 되나

가장 직관적인 방법은 이거다.

```text
48k → 24k :  매 2 샘플 중 1개만 골라 쓰기 (downsampling by 2)
24k → 48k :  각 샘플을 두 번 쓰기 (= upsampling by 2, zero-stuffing이 아닌 hold)
```

이것도 "sample rate 변환"이긴 하지만 결과 음질이 거칠다. 두 가지 문제 때문이다.

```text
1. aliasing
   원래 신호의 고주파가 다운샘플 후 낮은 주파수로 "접혀서" 들어옴.
   → 사람이 "지지직" 또는 mosquito 노이즈로 인지

2. imaging
   업샘플 시 zero/hold가 만든 인공 고주파 성분이 남음.
   → 신호 위에 거칠게 깔린 잡음
```

이 둘을 막으려면 **anti-aliasing 필터**(다운 시) / **anti-imaging 필터**(업 시)가 필요하다. 단순 인덱스 조작이 아니라 신호 처리 단계 한 개가 늘어난다.

## 2. SRC = filter + 인덱스 매핑

표준 SRC의 골격은 이렇다.

```text
업샘플 (예: 1 → 2)
  zero-stuffing (샘플 사이에 0 삽입)
  → low-pass filter (anti-imaging)
  → 결과 sample rate가 두 배

다운샘플 (예: 2 → 1)
  low-pass filter (anti-aliasing)
  → 매 N번째 샘플만 남김
  → 결과 sample rate가 절반
```

비-정수 비율(48 ↔ 44.1)에서는 두 단계를 결합한 polyphase filter나 FFT-based 방식이 표준이다.

이걸 직접 구현하면 까다롭다. 그래서 검증된 crate를 쓴다.

## 3. rubato 사용

`rubato`는 Rust의 표준 SRC 라이브러리다. 두 가지 알고리즘 그룹.

```text
SincFixed*       : sinc 보간, 매우 고품질
FftFixed*        : FFT-based, 고품질이고 큰 비율 변환에 유리
PolynomialFixed* : 빠르지만 음질 다소 낮음
```

기본 선택은 `SincFixedIn` 또는 `FftFixedInOut`이다.

```toml
[dependencies]
rubato = "0.15"
```

```rust
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters};

let params = SincInterpolationParameters {
    sinc_len: 256,
    f_cutoff: 0.95,
    interpolation: rubato::SincInterpolationType::Linear,
    oversampling_factor: 256,
    window: rubato::WindowFunction::BlackmanHarris2,
};

let mut resampler = SincFixedIn::<f32>::new(
    /*resample_ratio*/ 48000.0 / 44100.0,
    /*max_resample_ratio_relative*/ 1.0,
    params,
    /*chunk_size*/ 1024,
    /*channels*/ 2,
)?;
```

input/output 모두 **planar** Vec<Vec<f32>>다.

```rust
let input: Vec<Vec<f32>> = vec![left_channel, right_channel];
let output = resampler.process(&input, None)?;
// output: Vec<Vec<f32>> 새로운 sample rate
```

## 4. 어디에 SRC를 끼우나

원칙은 **DSP 체인의 양 끝**이다.

```text
파일 input → SRC (파일 → cpal SR) → DSP chain → SRC (cpal SR → 파일 SR) → encoder
                ▲                                      ▲
            여기에 한 번만                          저장할 때만
```

DSP 체인 내부에서 SR을 바꾸는 일은 거의 없다. 한 번 cpal SR로 정렬한 뒤로는 끝까지 그대로.

## 5. 실시간 SRC

실시간(콜백 동기) 환경에서 SRC가 필요한 경우.

```text
파일 SR ≠ cpal SR
  → decoder thread에서 SRC를 미리 적용하고 ring buffer로 cpal SR 샘플만 push
  → 콜백은 평소처럼 cpal SR로 처리
```

콜백 안에서 SRC를 직접 부르는 건 NG. SRC는 chunk 단위에서 가장 효율적이고, latency 특성도 콜백 단위와 다르다.

## 6. 비-정수 비율 SRC의 미묘함

48000 / 44100 = 1.088435... 같은 비율은 정수가 아니다. 입력 N 샘플 → 출력 N × 비율 샘플인데, 정확히 정수가 아니라 chunk마다 한 샘플 정도가 들쭉날쭉.

```text
chunk 1: 입력 1024 → 출력 1115
chunk 2: 입력 1024 → 출력 1116
chunk 3: 입력 1024 → 출력 1115
...
```

`rubato`의 `SincFixedIn`은 입력 chunk 크기를 고정한다. 출력 길이는 chunk마다 살짝 다를 수 있어서 ring buffer로 받는 게 가장 안전하다.

## 7. 품질과 비용

빠른 비교.

```text
SincFixedIn (sinc_len=256, oversampling=256)
  품질: 매우 높음 (오디오 마스터링/제출용)
  비용: 비싸지만 (latency 256 샘플) 고정

FftFixedInOut
  품질: 매우 높음
  비용: FFT 비용. 큰 비율 변환에 유리.

Polynomial (cubic)
  품질: 보통
  비용: 매우 낮음
  → 실시간 모니터, 게임 사운드 같은 데 적합
```

학습 단계에서는 sinc로 시작하고, 부하가 보이면 polynomial로 바꾼다.

## 8. SRC 결과 길이를 신뢰하지 말 것

SRC를 반복 호출하면, 입력 N 샘플에서 출력 N×ratio 샘플이 나오지 않을 수 있다. 첫 호출에서 latency만큼 출력이 0개일 수도 있고, 마지막 호출에서 남은 샘플이 한 번에 나올 수도 있다.

```text
정확한 비율 출력을 보장하지 않음
→ ring buffer / 가변 길이 vector로 받기
→ "지금까지 입력 누적 / 출력 누적"으로 검증
```

## 9. RuStudio 관점

```text
mod_player의 파일 재생:
  decoder thread:
    파일 SR로 디코딩 → 필요시 rubato로 cpal SR로 변환 → ring buffer

mastering 오프라인:
  곡 전체를 cpal/내부 SR로 처리 → 결과를 사용자가 원하는 SR로 변환 → encoder

streaming 발송:
  보통 stream platform이 정한 SR(48k)로 마스터링 마지막에 변환
```

## 자주 하는 실수

- 단순 인덱스 스킵으로 SR 변환 → aliasing/imaging 노이즈.
- DSP 체인 중간에 SRC를 끼움 → 양 끝 SR을 일관 안 맞추는 코드 복잡도.
- 콜백 안에서 SRC 직접 호출 → chunk 비용 큼.
- 출력 길이를 입력 × 비율로 가정 → 들쭉날쭉.
- planar/interleaved 변환을 SRC 안밖에서 일관 안 함 → 채널 섞임.

## 반드시 이해해야 할 것

- 단순 인덱스 조작은 SRC가 아니다. anti-aliasing/anti-imaging 필터가 함께 와야 한다.
- 비-정수 비율(48 ↔ 44.1)에서는 polyphase 또는 FFT-based가 표준. `rubato`가 두 가지 모두 제공.
- SRC는 DSP 체인의 양 끝에만 끼운다. 내부 SR을 한 번 정해 두면 흔들리지 않는다.
- 실시간에서는 별 thread + ring buffer로 격리. 콜백 안에서 직접 호출 금지.
- 출력 길이는 들쭉날쭉할 수 있으니 가변 vector / ring buffer로 받는다.
