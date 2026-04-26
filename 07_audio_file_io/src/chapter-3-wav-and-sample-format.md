# Chapter 3 - WAV와 샘플 포맷 변환

## 1. WAV는 학습용으로 가장 좋다

WAV는 사실상 무압축 + 단순 헤더 포맷이다.

```text
RIFF header (8 bytes)
  "RIFF" + file size + "WAVE"
fmt  chunk (16~40 bytes)
  sample rate, channels, bits per sample, format code (PCM/float)
data chunk
  실제 샘플 데이터 (raw bytes, interleaved)
```

압축이 없으니 디코딩이 단순하다. byte를 정해진 sample format으로 해석하면 끝. 학습 단계에서는 거의 항상 WAV로 시작/끝낸다.

## 2. PCM의 두 종류 — integer vs float

WAV의 sample 데이터는 두 가지 인코딩이 있다.

```text
integer PCM (가장 흔함):
  i16 : -32768 ~ 32767
  i24 : -8388608 ~ 8388607
  i32 : -2147483648 ~ 2147483647

float PCM:
  f32 : -1.0 ~ +1.0 (관례)
```

DAW/오디오 인터페이스에서는 32-bit float가 많고, 일반 음악 파일에서는 16-bit integer가 많다.

## 3. integer ↔ float 변환

DSP 코드는 보통 f32를 입력으로 받는다. 그래서 i16 → f32 변환이 거의 항상 필요하다.

```rust
fn i16_to_f32(s: i16) -> f32 {
    s as f32 / 32768.0          // 보통 32768로 나눔
}
```

여기 32768로 나눌까 32767로 나눌까 — 사소해 보이지만 의도는 다르다.

```text
÷ 32768 : i16의 음수 한계(-32768)가 정확히 -1.0이 됨, +32767은 +0.99997
÷ 32767 : i16의 양수 한계(+32767)가 정확히 +1.0이 됨, -32768은 -1.00003 (범위 초과)
```

대부분 표준은 ÷ 32768다. 비대칭이지만 -1.0 / +1.0 안에 안전하게 들어간다.

f32 → i16 (저장 시).

```rust
fn f32_to_i16(x: f32) -> i16 {
    let clamped = x.clamp(-1.0, 1.0);
    (clamped * 32767.0).round() as i16
}
```

여기서 **clamp가 핵심**이다. f32 신호가 -1.0~1.0을 넘으면 i16에서 wrap-around가 일어나 거친 디지털 노이즈가 된다. 04 책 limiter의 ceiling이 0 dBFS 근처여야 했던 이유 중 하나가 이거다.

## 4. interleaved 가정

WAV의 data chunk는 항상 **interleaved**다.

```text
data 안의 바이트 순서 (stereo i16):
  L0_lo L0_hi  R0_lo R0_hi  L1_lo L1_hi  R1_lo R1_hi  ...
  └─sample 0─┘ └─sample 0─┘ └─sample 1─┘ └─sample 1─┘
       L            R            L            R
```

그래서 WAV 데이터를 그대로 cpal interleaved 콜백에 흘려 보내는 일은 가장 자연스럽다 (sample format 변환만 하면).

## 5. hound로 WAV 다루기

`hound`는 학습용으로 가장 단순하고 정확한 WAV crate다.

```toml
[dependencies]
hound = "3"
```

읽기.

```rust
let mut reader = hound::WavReader::open("input.wav")?;
let spec = reader.spec();
println!("{} Hz, {} ch, {} bps, {:?}", spec.sample_rate, spec.channels,
    spec.bits_per_sample, spec.sample_format);

let samples: Vec<f32> = match spec.sample_format {
    hound::SampleFormat::Int => reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / 32768.0)
        .collect(),
    hound::SampleFormat::Float => reader.samples::<f32>()
        .map(|s| s.unwrap())
        .collect(),
};
```

여기서 `samples`는 interleaved Vec<f32>다. stereo이면 길이가 frame 수의 2배.

쓰기.

```rust
let spec = hound::WavSpec {
    channels: 2,
    sample_rate: 48000,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};
let mut writer = hound::WavWriter::create("out.wav", spec)?;
for &s in &samples {
    let i = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
    writer.write_sample(i)?;
}
writer.finalize()?;
```

`finalize()`를 잊으면 헤더의 길이 정보가 안 적힌다. 결과 파일이 깨진다.

## 6. interleaved ↔ planar 변환 패턴

DSP 코드에 따라 planar가 더 편한 경우가 있다 (channel별 별도 처리). 변환은 단순.

```rust
fn interleaved_to_planar(input: &[f32], channels: usize) -> Vec<Vec<f32>> {
    let frames = input.len() / channels;
    let mut out: Vec<Vec<f32>> = (0..channels)
        .map(|_| Vec::with_capacity(frames))
        .collect();
    for frame in input.chunks(channels) {
        for (ch, &s) in frame.iter().enumerate() {
            out[ch].push(s);
        }
    }
    out
}

fn planar_to_interleaved(input: &[Vec<f32>]) -> Vec<f32> {
    let frames = input[0].len();
    let channels = input.len();
    let mut out = Vec::with_capacity(frames * channels);
    for i in 0..frames {
        for ch in 0..channels {
            out.push(input[ch][i]);
        }
    }
    out
}
```

이 변환은 단순하지만 매 frame `Vec::push`는 콜백 안에서는 NG. 오프라인 처리에선 OK.

## 7. 채널 수 처리 — 모노/스테레오/멀티채널

문제는 입력 파일과 출력 장치의 채널 수가 다를 때다.

```text
mono 입력 → stereo 출력  : 같은 신호를 두 채널에 깔기
stereo 입력 → mono 출력  : 좌우를 합쳐서 평균 (또는 한쪽만)
stereo 입력 → 5.1 출력   : front L/R에만 두고 나머지는 0
```

가장 안전한 표준은 "두 채널 다운믹스 시 -3 dB(× 0.707) 곱하기"다. 이러면 양쪽 합쳐도 amplitude가 두 배로 안 된다.

```rust
let mono = (l + r) * 0.7071;
```

## 8. 24-bit 처리는 까다롭다

i24는 메모리상 자연스러운 타입이 없다. 보통 다음 중 하나로 다룬다.

```text
i32에 padded i24 (sign-extended)
  → ÷ 2^31 또는 ÷ 8388608

byte 단위 직접 파싱 (3 bytes per sample)
```

`hound`는 i24를 i32로 풀어 준다 (low 24 bits). DSP에서는 `(s as f32) / 8388608.0` 정도로 normalize한다. 학습 단계에서는 i16과 f32만 다뤄도 충분.

## 자주 하는 실수

- f32 → i16 변환 시 clamp 누락 → wrap-around로 거친 노이즈.
- ÷ 32768과 ÷ 32767을 혼용 → 작은 amplitude 차이지만 측정 시 발견됨.
- `WavWriter::finalize()` 누락 → 헤더 손상.
- planar 디코더 출력을 interleaved cpal에 그대로 → 채널 섞임.
- 모노/스테레오 변환에서 -3 dB 보정 누락 → 합산 후 클립.
- bits_per_sample = 16과 sample_format = Float를 같이 → 잘못된 spec.

## 반드시 이해해야 할 것

- WAV는 무압축 + 단순 헤더. 학습의 시작점이자 끝점.
- 입력 sample format은 거의 항상 i16 또는 f32. DSP는 f32로 정규화.
- f32 → i16 저장 시 반드시 clamp.
- WAV의 data는 interleaved. cpal 콜백도 interleaved. 둘은 자연스럽게 어울린다.
- 채널 수 변환은 별도 정책이 필요하다 (모노↔스테레오, 다운믹스 -3 dB).
