# Chapter 4 - 압축 포맷 디코딩

## 1. 왜 WAV가 끝이 아닌가

WAV는 단순하지만 용량이 크다.

```text
3분 stereo 16-bit 44.1 kHz WAV ≈ 30 MB
같은 곡 320 kbps mp3            ≈ 7 MB
같은 곡 lossless FLAC           ≈ 18 MB
```

음악 라이브러리에서 만나는 거의 모든 파일은 압축이다. RuStudio가 사용자 파일을 받으려면 mp3/flac/ogg 디코딩이 필요하다.

## 2. 압축 포맷의 공통 구조

압축 포맷들은 거의 다 packet/frame 기반이다.

```text
파일은 N개의 작은 packet으로 나뉘어 있음
각 packet을 디코더에 넣으면 → PCM 샘플 한 묶음이 나옴

packet 1 → samples for time 0 ~ T1
packet 2 → samples for time T1 ~ T2
packet 3 → samples for time T2 ~ T3
...
```

각 packet이 만들어내는 샘플 수는 코덱에 따라 정해져 있다.

```text
mp3 frame  : 약 1152 PCM 샘플 (Layer 3)
AAC frame  : 약 1024 PCM 샘플
FLAC block : 가변 (보통 4096 샘플 부근)
Vorbis     : 가변
Opus       : 2.5 / 5 / 10 / 20 / 40 / 60 ms
```

이 사실의 의미는 "디코딩이 균일한 비용이 아니다"다. 한 packet 디코딩은 수 µs ~ 수 ms. 콜백에 직접 넣으면 위험하다.

## 3. symphonia의 큰 그림

`symphonia`는 Rust로 짠 다포맷 디코더다.

```toml
[dependencies]
symphonia = { version = "0.5", features = ["mp3", "flac", "ogg", "wav"] }
```

핵심 컴포넌트.

```text
FormatReader  : container parsing (ogg, mp4, riff 등)
Decoder       : packet → PCM 샘플
AudioBuffer   : PCM 샘플 컨테이너 (planar, 다양한 sample format)
```

사용 흐름.

```rust
let src = std::fs::File::open(path)?;
let mss = symphonia::core::io::MediaSourceStream::new(Box::new(src), Default::default());

let probed = symphonia::default::get_probe()
    .format(&Hint::new(), mss, &Default::default(), &Default::default())?;

let mut format = probed.format;
let track = format.default_track().unwrap();
let track_id = track.id;
let codec_params = track.codec_params.clone();

let mut decoder = symphonia::default::get_codecs()
    .make(&codec_params, &Default::default())?;

loop {
    let packet = match format.next_packet() {
        Ok(p) => p,
        Err(_) => break,
    };
    if packet.track_id() != track_id { continue; }

    match decoder.decode(&packet) {
        Ok(decoded) => {
            // decoded는 AudioBufferRef (planar)
            // f32 planar로 복사 → 처리
        }
        Err(_) => break,
    }
}
```

이 골격이 모든 포맷에서 공통이다.

## 4. AudioBuffer는 보통 planar

`symphonia`의 디코딩 결과는 거의 planar다. interleaved cpal에 보내려면 변환이 필요하다.

```rust
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::sample::Sample;

if let AudioBufferRef::F32(buf) = decoded {
    let channels = buf.spec().channels.count();
    let frames = buf.frames();
    let mut interleaved = vec![0.0f32; frames * channels];
    for ch in 0..channels {
        let plane = buf.chan(ch);
        for i in 0..frames {
            interleaved[i * channels + ch] = plane[i];
        }
    }
    // interleaved를 ring buffer로 push
}
```

매 packet마다 interleaved Vec를 새로 할당하면 메모리 할당량이 크다. 미리 할당된 buffer 재사용 패턴이 정석.

## 5. sample format이 다양하다

`AudioBufferRef`는 여러 variant를 가진다.

```text
U8 / U16 / U24 / U32 / S8 / S16 / S24 / S32 / F32 / F64
```

코덱에 따라 다른 variant가 나온다. mp3/aac는 보통 f32, flac은 i16/i24/i32, opus는 f32.

전부 f32로 정규화하는 helper 한 개를 두는 게 정석.

```rust
fn convert_to_f32(decoded: AudioBufferRef, out: &mut Vec<f32>) {
    use symphonia::core::audio::AudioBufferRef::*;
    out.clear();
    match decoded {
        F32(b) => /* planar f32 → interleaved f32 */,
        S16(b) => /* planar i16 → interleaved f32 (÷ 32768) */,
        S24(b) => /* planar i24 → interleaved f32 (÷ 8388608) */,
        S32(b) => /* planar i32 → interleaved f32 (÷ 2^31) */,
        // ...
        _ => unimplemented!(),
    }
}
```

이 helper가 디코더와 DSP 사이의 standard interface다.

## 6. 메타데이터

압축 포맷은 보통 메타데이터를 별도 chunk/box에 들고 있다.

```text
mp3   : ID3v1 / ID3v2
flac  : Vorbis comment
ogg   : Vorbis comment
mp4/m4a: iTunes-style atom
```

`symphonia`는 이걸 통합 인터페이스로 노출한다.

```rust
if let Some(meta) = format.metadata().current() {
    for tag in meta.tags() {
        println!("{:?}: {}", tag.std_key, tag.value);
    }
}
```

DAW에서 트랙 이름/아티스트를 자동으로 채울 때 이 정보를 쓴다.

## 7. seeking

곡 중간으로 점프하려면 seek가 필요하다. 압축 포맷은 그냥 byte offset으로 seek하면 안 된다 (packet 경계, key frame, codec 상태).

```rust
format.seek(SeekMode::Coarse, SeekTo::Time { time, track_id: Some(track_id) })?;
decoder.reset();   // 디코더 내부 상태 reset
```

`decoder.reset()`을 까먹으면 seek 직후 노이즈가 들린다 — 디코더가 이전 packet 컨텍스트로 새 packet을 풀려고 한다.

## 8. RuStudio 관점

```text
mod_player의 파일 source:
  worker thread:
    symphonia decode 루프
    → f32 interleaved buffer 변환
    → ring buffer로 push
  audio thread:
    ring buffer pop → DSP → cpal out

mastering 오프라인 처리:
  symphonia로 파일 전체 디코딩 → Vec<f32> 메모리에 통째로
  → DSP chain → SRC → encoder
```

라이선스 주의: mp3/aac 디코딩은 일부 지역/조건에서 라이선스 이슈가 있을 수 있다. 상업 배포 전에 확인이 필요하지만, 학습/오픈소스 단계에서는 `symphonia`가 안전한 선택.

## 자주 하는 실수

- 디코딩을 cpal 콜백 안에서 → underrun.
- planar AudioBuffer를 그대로 cpal에 넣음 → 채널 섞임.
- sample format variant를 일일이 처리 안 하고 panic → 다른 파일에서 즉사.
- seek 후 `decoder.reset()` 누락 → 노이즈.
- packet 디코딩 실패를 그냥 무시 → 신호 끊김 후 silence.
- 매 packet마다 interleaved Vec를 새로 할당 → 메모리 할당 폭증.

## 반드시 이해해야 할 것

- 압축 포맷은 packet 단위로 풀린다. 디코딩 비용이 균일하지 않다.
- `symphonia`는 sample format / planar 형태를 다양하게 내보낸다. f32 interleaved로 정규화하는 helper가 표준.
- 디코딩은 항상 별 thread에서. ring buffer가 audio thread와의 다리.
- seek는 단순 byte offset이 아니다. `decoder.reset()`이 함께 와야 한다.
