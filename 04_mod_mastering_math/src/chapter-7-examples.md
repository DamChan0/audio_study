# Chapter 7 - 예제와 검증 방향

이 장은 04 책의 처리 블록을 코드로 만질 때 어떤 단위로 쪼개고 어떻게 검증할지 정리한다.

## 권장 예제 목록

```text
01_db_meter         : peak / RMS 미터를 콜백 안에서 atomic으로 갱신, 콜백 밖 인쇄
02_envelope_follower: 입력 amplitude를 따라가는 envelope를 시각화
03_simple_compressor: threshold/ratio/attack/release 4파라미터 단일 밴드 컴프
04_makeup_gain      : 컴프 뒤 makeup gain의 효과 비교
05_brickwall_limit  : 단순 sample-peak 기준 brickwall limiter (look-ahead 없음)
06_lookahead_limit  : look-ahead 추가 후 distortion 차이 비교
07_kweighted_meter  : K-weighting 두 단 + 짧은 RMS의 결과 (간이 LUFS)
```

이 7개는 04 책의 모든 핵심 블록을 한 번씩 만진다. 각각이 한 binary 면 학습에 가장 좋다.

## 디렉토리 구성

03과 동일한 정책이다.

```text
04_mod_mastering_math/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml
    src/
      lib.rs            ← EnvFollower, Compressor, Limiter, KWeighting 등 공용
      bin/
        01_db_meter.rs
        02_envelope_follower.rs
        03_simple_compressor.rs
        04_makeup_gain.rs
        05_brickwall_limit.rs
        06_lookahead_limit.rs
        07_kweighted_meter.rs
```

## 모든 예제의 골격

이 책 예제는 입력 신호가 필요하다. 두 가지 방식 중 하나로 만든다.

```text
A. 03 책의 oscillator + envelope + delay로 합성한 테스트 신호
B. WAV 파일을 미리 메모리에 로드해 두고 재생 (07_audio_file_io까지 가지 않은 단계라면 단순 재현 코드면 충분)
```

A를 권장한다. 03 책 결과물을 그대로 쓸 수 있고, 신호 특성을 통제할 수 있다.

```rust
let stream = device.build_output_stream(
    &config,
    move |data: &mut [f32], _| {
        for frame in data.chunks_mut(channels) {
            let s = test_signal.next();        // 03 책 결과물

            // (이 책의 처리)
            let env = follower.next(s);
            let g   = compressor.gain_for(env);
            let out = s * g * makeup;

            for ch in frame.iter_mut() { *ch = out; }
        }
    },
    err_cb,
    None,
)?;
```

## 검증 방향

이 책 처리는 귀로만 검증하면 미세한 오류를 놓치기 쉽다. 아래 방식으로 같이 본다.

### 1. 정량 검증 — 정해진 입력 → 정해진 출력

```text
01_db_meter
  - 1.0 amplitude 사인파 입력 → peak ≈ 0 dBFS, RMS ≈ -3 dBFS

02_envelope_follower
  - 사각형 amplitude 변화(0 → 1 → 0) 입력 → envelope이 attack/release 시간 상수만큼 곡선

03_simple_compressor
  - threshold = -20 dB, ratio = 4:1, 입력 -10 dB 사인파
    → 출력 약 -12.5 dB (정적 곡선 식대로)
  - 입력 -25 dB → 출력 -25 dB (안 깎임)

04_makeup_gain
  - 컴프 적용 전/후 RMS 비교 → makeup gain만큼 차이가 나는가

05_brickwall_limit
  - ceiling = -1 dBFS, 입력에 spike 추가 → 출력 sample-peak이 -1 dBFS 이하인가

06_lookahead_limit
  - 같은 입력을 05 vs 06으로 처리한 출력의 transient 보존 비교

07_kweighted_meter
  - 1 kHz 사인파 vs 100 Hz 사인파 같은 amplitude 입력
    → K-weighting 후 RMS 차이가 표준에 명시된 수치(약 +1 ~ +2 dB)와 비슷한가
```

### 2. 시각 검증 — WAV로 저장해서 그래프로 보기

각 예제 출력을 1~2초 WAV로 떨어뜨려 Audacity 같은 툴에서 본다.

```text
컴프  : 입력 vs 출력 amplitude envelope이 어떻게 다른지 한눈에 확인
limiter: ceiling 선이 칼처럼 평평한지 확인
look-ahead: transient의 앞쪽 모양 차이 확인
```

WAV 저장은 07 책에서 본격 다루지만, 지금 단계에서는 `hound` crate를 잠깐 빌려와도 된다.

### 3. 단위 테스트로 정적 곡선 확인

`compute_gain` 같은 순수 함수는 단위 테스트가 쉽다.

```rust
#[test]
fn ratio_4_at_minus_10() {
    let g = compute_gain_db(-10.0, /*threshold*/ -20.0, /*ratio*/ 4.0);
    // 입력 -10, threshold -20, ratio 4
    // 위로 10 dB 넘었음 → 깎인 후 -20 + 10/4 = -17.5
    // gain_db = -17.5 - (-10) = -7.5
    assert!((g - (-7.5)).abs() < 0.01);
}
```

dynamics는 시간상수가 들어가서 통째로 단위 테스트하기 어렵지만, 정적 곡선은 단위 테스트하기 좋다.

## 콜백 안에서 지켜야 할 규칙 (다시)

이 책 예제도 03 책 규칙을 그대로 따른다.

```text
✗ Mutex / Vec::push / println! / 파일 IO / sleep
✓ atomic 파라미터 읽기
✓ 콜백 밖에서 dB→linear 변환된 결과만 받기
✓ 미리 할당된 ring buffer / delay line만 사용
```

특히 컴프/리미터의 attack/release coefficient는 콜백 안에서 매번 계산하면 안 된다. UI에서 ms 값이 바뀔 때 콜백 밖에서 한 번 변환 후 atomic으로 전달.

## 자주 하는 실수

- 컴프 출력이 envelope 모양으로 변형되어 나옴 → envelope 자체를 신호에 곱한 것이다 (gain 계수가 아니라).
- limiter의 ceiling이 0 dBFS → 짧은 inter-sample 클립이 사라지지 않음.
- WAV로 떨어뜨린 출력이 normal-int에서 클립 → `f32` → `i16` 변환 시 클램핑 빠짐.
- look-ahead 길이만큼 다른 트랙과 시간이 어긋남 → 음악적으로 매우 불편함.
- LUFS 미터 검증을 사인파 한 톤으로만 함 → K-weighting의 효과를 보려면 1 kHz vs 100 Hz 비교 필요.

## 반드시 이해해야 할 것

- 이 책 예제는 "정량 검증"이 핵심이다. dynamics 처리는 귀로만 보면 의도와 반대로 가기 쉽다.
- 모든 예제는 03 책의 콜백 골격을 그대로 쓰고, 그 안의 처리 블록만 바꾼다.
- 정적 곡선(컴프 식)은 단위 테스트가 쉽다. 시간 동작(envelope follower)은 WAV 그래프로 검증한다.
