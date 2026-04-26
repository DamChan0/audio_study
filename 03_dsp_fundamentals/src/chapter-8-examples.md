# Chapter 8 - 예제 crate 구성

이 책의 6개 빌딩 블록을 한 곳에 모아 만지면서 감각을 잡는 단계다.

`02_mod_player_architecture`에서 이미 본 것처럼, 이 시리즈는 책마다 **dedicated example crate**를 두는 정책이다. 즉 본문 코드는 발췌이고, 동작은 별도 crate `examples/`에서 돌린다.

## 권장 예제 목록

각각 한 가지 개념만 보여주는 게 원칙이다.

```text
01_sine_osc       : SineOsc 구조체로 440 Hz 출력 (Chapter 4)
02_db_gain        : 사용자 dB 슬라이더를 linear로 변환해 곱하기 (Chapter 5)
03_pan_law        : equal-power pan 적용 후 모노 → 스테레오 (Chapter 5)
04_mixer_two_osc  : 220 Hz + 330 Hz 두 oscillator를 한 버퍼로 합산 (Chapter 5)
05_adsr_voice     : SineOsc × ADSR로 한 음 발음/감쇠 (Chapter 6)
06_simple_delay   : feedback 0.4 짜리 echo (Chapter 7)
```

이 6개를 한 crate에 binary `bin/` 6개로 두는 게 학습 흐름상 가장 좋다.

## 디렉토리 구성 권장

```text
03_dsp_fundamentals/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml          ← 이 crate가 dedicated example crate
    src/
      lib.rs            ← SineOsc, Adsr, DelayLine 같은 공용 코드
      bin/
        01_sine_osc.rs
        02_db_gain.rs
        03_pan_law.rs
        04_mixer_two_osc.rs
        05_adsr_voice.rs
        06_simple_delay.rs
```

이렇게 두면 본문에 등장한 작은 구조체들을 `examples/src/lib.rs`에 한 번 정의해 두고, 각 binary는 그 라이브러리를 import해서 cpal과 묶기만 하면 된다.

## 모든 예제가 따라야 하는 골격

cpal 부분은 `01_cpal`에서 본 그대로다. 이 책에서 새로 다루는 부분은 콜백 **안쪽**이다.

```rust
let stream = device.build_output_stream(
    &config,
    move |data: &mut [f32], _| {
        for frame in data.chunks_mut(channels) {
            // (1) 한 프레임에 들어갈 샘플을 만든다
            let s = generate_one_sample();

            // (2) 모든 채널에 같은 신호를 깐다 (또는 채널별로 다르게)
            for ch in frame.iter_mut() {
                *ch = s;
            }
        }
    },
    err_cb,
    None,
)?;
```

`generate_one_sample()` 자리에 oscillator, gain, ADSR, delay가 들어간다.

## 예제 작성 규칙

이 시리즈에서 예제를 만들 때 지킬 규칙은 셋이다.

1. **한 예제 = 한 개념.** mixer 예제에 ADSR을 끼우지 않는다. 두 개념을 합치고 싶으면 새 예제로.
2. **콜백 안에 실시간 규칙 위반이 없어야 한다.** `println!`/`Vec::push`/`Mutex::lock` 모두 금지. 디버깅 출력은 콜백 밖에서 atomic이나 channel로 빼낸 후 인쇄한다.
3. **예제 코드 옆에는 짧은 해설을 붙인다.** "이 줄은 왜 콜백 밖에 있는가" 식으로 한 줄씩 의도를 적는다.

## 검증 방법

각 예제는 귀로만 검증하지 않는다. 시각적으로도 확인한다.

```text
01_sine_osc      : 1초 녹음 → WAV 저장 → 파형이 깔끔한 사인인가? FFT에 440 Hz 단일 피크인가?
02_db_gain       : -6 dB 입력 시 출력 amplitude가 0.5인가?
03_pan_law       : pan = 0.5에서 좌/우 RMS가 거의 같고, 합 power가 끝과 비슷한가?
04_mixer_two_osc : FFT에 두 주파수 피크가 둘 다 보이는가?
05_adsr_voice    : envelope 곡선이 attack → decay → release로 매끈한가? 클릭 없음?
06_simple_delay  : 입력에 한 번 짧은 transient를 주면 그게 N ms 뒤에 반복되며 작아지는가?
```

여기서 "FFT 피크"는 `06_fft_and_spectrum`에서 본격적으로 다룬다. 지금은 Audacity 같은 도구로 시각 확인이면 충분하다.

## 다음 단계로 넘어가기 전 체크

이 6개 예제를 다 만들고 나면, 각 예제에 대해 다음 표가 머릿속에 자연스럽게 들어와야 한다.

```text
예제           input       output           state
sine_osc       (없음)      한 채널 신호     phase
db_gain        한 신호     같은 신호 ×lin   (없음)
pan_law        한 신호     스테레오 신호    (없음)
mixer          여러 신호   한 신호          (없음)
adsr_voice     트리거      신호×envelope    stage, level
simple_delay   한 신호     원본 + delayed   ring buffer + write idx
```

이 표가 머리에 들어왔으면 다음 책 `04_mod_mastering_math`에서 등장하는 compressor / limiter / LUFS도 같은 표로 분해해서 볼 수 있다.

## 반드시 이해해야 할 것

- 예제는 "구현 완성품"이 아니라 **개념을 한 가지씩 검증하는 도구**다.
- 모든 예제가 cpal 콜백 안에서 같은 골격을 공유한다. 다른 건 generate_one_sample 자리에 무엇이 들어가느냐다.
- 예제별로 input/output/state 표를 만들어 두는 습관이 다음 책들에서 그대로 통한다.
