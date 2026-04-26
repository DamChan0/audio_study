# Chapter 6 - 예제와 파이프라인

이 장은 07 책의 모든 단계를 실제로 만져 볼 권장 예제 목록이다.

## 권장 예제 목록

```text
01_wav_round_trip   : WAV 읽고 → 그대로 다시 쓰기 (sample format 변환만)
02_int_float_convert: i16 → f32 → i16 round trip의 amplitude 비교
03_interleave       : interleaved ↔ planar 변환을 단위 테스트
04_decode_mp3       : symphonia로 mp3 → f32 planar → interleaved
05_resample         : rubato로 44.1k WAV → 48k WAV
06_offline_master   : decode → DSP chain (compressor + limiter) → SRC → WAV
```

이 6개를 끝내면 파일 입출력의 거의 모든 단계가 손에 들어온다.

## 디렉토리

```text
07_audio_file_io/
  Cargo.toml
  src/
    (mdBook 본문)
  examples/
    Cargo.toml          ← hound, symphonia, rubato
    src/
      lib.rs            ← f32 interleave/planar helper, decoder helper, resampler helper
      bin/
        01_wav_round_trip.rs
        02_int_float_convert.rs
        03_interleave.rs
        04_decode_mp3.rs
        05_resample.rs
        06_offline_master.rs
```

## 01 — WAV round trip

```rust
fn main() -> anyhow::Result<()> {
    let mut reader = hound::WavReader::open("input.wav")?;
    let spec = reader.spec();
    let samples: Vec<f32> = reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / 32768.0)
        .collect();

    let mut writer = hound::WavWriter::create("output.wav", spec)?;
    for s in samples {
        let i = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        writer.write_sample(i)?;
    }
    writer.finalize()?;
    Ok(())
}
```

검증.

```text
- 입력과 출력의 byte 비교: i16 → f32 → i16 round trip은 1 LSB 정도 차이가 있을 수 있음
- 청취 비교: 차이 들리지 않아야 함
- spec 비교: sample rate, channels, bits 동일
```

## 02 — int ↔ float 정확도

```rust
fn main() {
    for s_in in [-32768i16, -16384, 0, 16384, 32767] {
        let f = s_in as f32 / 32768.0;
        let s_out = (f.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        println!("i16 {:6} → f32 {:.6} → i16 {:6}", s_in, f, s_out);
    }
}
```

검증 — 인접한 i16 값이 round trip 후 어떻게 보존/손실되는지 직접 본다.

## 03 — interleave 단위 테스트

```rust
#[test]
fn round_trip_stereo() {
    let interleaved: Vec<f32> = vec![1.0, -1.0, 0.5, -0.5, 0.25, -0.25];
    let planar = interleaved_to_planar(&interleaved, 2);
    assert_eq!(planar[0], vec![1.0, 0.5, 0.25]);
    assert_eq!(planar[1], vec![-1.0, -0.5, -0.25]);
    let back = planar_to_interleaved(&planar);
    assert_eq!(back, interleaved);
}
```

이 단위 테스트가 통과하면 채널 변환 로직은 신뢰할 수 있다.

## 04 — symphonia로 mp3 디코딩

이 예제는 코드가 길어서 골격만.

```text
1) MediaSourceStream 생성
2) probe → format / decoder
3) loop:
     packet 받기
     decoder.decode(packet)
     AudioBufferRef::F32 또는 S16/S24를 f32 planar로 변환
     planar → interleaved
     output Vec<f32>에 누적
4) 결과를 WAV로 저장 (검증용)
```

검증.

```text
- 같은 곡의 mp3와 wav 버전 RMS 비교 (mp3는 손실이라 약간 다름)
- spectrum 비교 (mp3는 ~16 kHz 위가 약함)
- 길이가 일치하는가
```

## 05 — rubato SRC

```rust
let mut resampler = SincFixedIn::<f32>::new(
    48000.0 / 44100.0, 1.0, params, 1024, 2)?;
```

44.1k stereo WAV 입력 → 48k stereo WAV 출력. 입력은 hound로 읽어서 planar로 분리하고, rubato에 chunk씩 넣고, 결과를 다시 interleaved로 합쳐 쓴다.

검증.

```text
- 출력 sample rate가 48000 인가
- 출력 길이가 입력 × (48/44.1)에 거의 일치하는가 (오차 ~ chunk 단위)
- 청취 비교: 음정이 같은가 (고주파 시험음 이용)
```

## 06 — offline mastering pipeline

이 책의 정점. 모든 도구를 한 번에 쓴다.

```text
1) input.wav 또는 input.mp3 디코딩
2) (필요시) SRC: 파일 SR → 내부 SR (예: 48000)
3) DSP: compressor + limiter (04 책)
4) (필요시) SRC: 내부 SR → 출력 SR (예: 48000 → 44100)
5) clip-safe f32 → i16 변환
6) output.wav 저장
```

이걸 한 binary로 짜는 게 mod_mastering 오프라인 모드의 prototype이다.

검증.

```text
- 입력 LUFS와 출력 LUFS 비교 (출력이 목표 음량에 도달했는가)
- 출력 sample peak이 -1 dBFS 이하인가
- 출력 길이가 입력과 (SRC 비율 안에서) 일치하는가
- 들리는 차이: dynamics 압축이 자연스러운가
```

## 콜백 골격이 등장하지 않는 점

이 책 예제는 cpal 콜백을 거의 쓰지 않는다. 오프라인 처리가 중심이기 때문이다.

실시간 미리듣기까지 합치고 싶으면, 06번 결과를 cpal 콜백으로 흘려 넣는 단순 추가만 해서 02 책 mod_player 흐름과 결합할 수 있다.

## 자주 하는 실수

- f32 → i16에서 clamp 누락 → wrap-around.
- mp3 디코딩에서 길이가 정확히 안 맞음 → priming/padding 정보(LAME header)를 무시. 학습 단계에선 오차 무시 OK.
- rubato 사용 시 chunk 크기를 다르게 호출 → 대부분 fixed-size 모드라 panic.
- decode-then-resample-then-process가 아니라 process-then-decode 순서를 헷갈림 → 처리가 잘못된 SR에서 적용.
- offline pipeline에서 매 단계에 새 Vec 할당 → 곡 길이 클 때 메모리 폭발. 적절한 chunk 처리.

## 반드시 이해해야 할 것

- 모든 예제는 디코더 + 변환 + 처리 + 인코더의 4단계 합성이다.
- 각 단계에서 sample format / channel layout / sample rate가 일관 유지되는지 항상 의식한다.
- 단위 테스트로 변환 로직(03번)을 단단히 잡으면, 큰 파이프라인에서 디버깅이 훨씬 쉽다.
- 06번이 mod_mastering 오프라인 모드의 prototype이다. 같은 코드를 cpal과 결합하면 실시간 미리듣기로 전환된다.
