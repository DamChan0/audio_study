# Chapter 2 - 파일 기반 오디오 흐름

## 1. 파일이 어떻게 신호가 되는가

파일은 디스크의 바이트 시퀀스다. 그 안에는 두 종류 정보가 들어있다.

```text
container 부분 : 어떤 코덱인가, sample rate, 채널 수, 길이, 메타데이터
codec data 부분: 실제 오디오 신호 (압축 또는 무압축 PCM)
```

이 둘을 분리해서 보는 게 첫걸음이다.

```text
.wav  : container = RIFF/WAVE, codec = 거의 항상 무압축 PCM
.mp3  : container = MP3 frame structure, codec = MPEG-1 Layer 3
.flac : container = FLAC stream, codec = FLAC (무손실 압축)
.ogg  : container = Ogg, codec = Vorbis 또는 Opus
```

## 2. 디코딩이 만들어내는 것

디코딩은 어느 포맷이든 결과적으로 다음을 만들어낸다.

```text
sample rate     : 예) 44_100, 48_000
channel count   : 예) 1, 2, 6
sample format   : 예) i16, i24, i32, f32
sample data     : 위 정보로 해석되는 raw 바이트
```

같은 음악이어도 디코딩 결과의 sample rate / format이 파일마다 다를 수 있다. 그래서 처리 코드는 반드시 이 정보를 디코더한테서 받아 와야 한다.

## 3. 실시간 vs 오프라인 흐름

같은 코드 흐름이지만 실행 모델이 다르다.

### 실시간 흐름 (모니터링, 미리듣기)

```text
worker thread:
  파일을 hop 단위로 디코딩
  → ring buffer에 push (lock-free)

audio thread (cpal 콜백):
  ring buffer에서 pop
  → DSP 적용
  → cpal 출력 buffer에 채움
```

핵심: 디코딩이 콜백 안에 들어가면 안 된다. mp3 디코딩은 한 packet에 수 ms가 걸리는 경우가 흔하다. 별 thread에서 미리 풀어 두고 ring buffer로 다리만 놓는다.

### 오프라인 흐름 (mastering 결과 저장)

```text
single thread (시간 제약 없음):
  파일 디코딩 (전부 또는 chunk 단위)
  → DSP 적용
  → 결과를 출력 인코더로 보냄
```

여기서 cpal은 등장하지 않는다. 시간 동기화가 필요 없으니 가장 빠른 속도로 돌면 된다.

## 4. 메모리 vs 스트리밍

파일을 한 번에 메모리에 다 올릴 수 있다면 가장 단순하다.

```text
30초 stereo 48kHz f32:
  30 · 48000 · 2 · 4 bytes ≈ 11 MB

3분 stereo 48kHz f32:
  180 · 48000 · 2 · 4 bytes ≈ 69 MB

1시간 stereo 48kHz f32:
  3600 · 48000 · 2 · 4 bytes ≈ 1.4 GB  ← 메모리에 통째로는 부담
```

곡 길이에 따라 두 모드가 나뉜다.

```text
짧은 파일      : 한 번에 Vec<f32>로 메모리에 풀어 둠
긴 파일/스트리밍: chunk 단위로 흘려 보냄 (디코더 → ring buffer → 처리)
```

학습 단계에서는 메모리에 다 올리는 모드부터 시작하는 게 단순하다.

## 5. 채널 배치 — interleaved vs planar

같은 샘플 데이터도 메모리 배치가 두 가지다.

```text
interleaved:
  [L0, R0, L1, R1, L2, R2, ...]

planar (channel-first):
  L = [L0, L1, L2, ...]
  R = [R0, R1, R2, ...]
```

cpal 콜백은 거의 항상 interleaved다. 일부 디코더(`symphonia`)는 planar `AudioBuffer`를 준다. 이 둘 사이의 변환이 자주 나온다.

```rust
// planar → interleaved (stereo)
fn planar_to_interleaved(l: &[f32], r: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(l.len() * 2);
    for i in 0..l.len() {
        out.push(l[i]);
        out.push(r[i]);
    }
    out
}
```

이 변환 자체는 단순하지만, 콜백 안에서 매 frame 새 Vec 할당하면 안 된다. 미리 할당된 buffer 재사용.

## 6. sample rate 정렬

cpal 출력의 sample rate, 파일의 sample rate, DSP 내부 sample rate가 다를 수 있다.

```text
cpal      : 48_000 Hz
파일      : 44_100 Hz (CD)
DSP 내부  : 48_000 Hz (cpal에 맞춤)

→ 파일을 읽은 직후 44.1k → 48k 변환 필요
→ 변환은 rubato (5장)
```

이걸 안 하면 음정이 약 9% 달라진다. CD 파일이 RuStudio에서 빠르게 들리는 식.

## 7. RuStudio 관점

```text
실시간 미리듣기 (mod_player):
  파일 → 디코더 thread → ring buffer → cpal 콜백 → DSP → out

오프라인 mastering (mod_mastering 곡 단위 처리):
  파일 → 메모리 → DSP chain → SRC → encoder → 새 파일

분석/시각화 (06 책 spectrum):
  파일 → 디코더 → analysis thread → STFT → UI
```

같은 디코딩/SRC 코드를 세 경로에서 재사용할 수 있게 설계하는 게 이 책의 실용적 목표다.

## 자주 하는 실수

- 디코딩을 cpal 콜백 안에서 처리 → underrun.
- 파일 sample rate를 cpal sample rate라고 가정 → 음정 어긋남.
- 채널 수가 다를 때 처리하지 않음 → 모노 파일을 stereo cpal에 그대로 → 한쪽 채널만 나옴.
- planar/interleaved 혼용 → 좌우 채널이 섞여 모노가 되거나 노이즈.
- 한 곡 전체를 메모리에 올리는 가정으로 코드 작성 후 큰 파일에서 OOM.

## 반드시 이해해야 할 것

- 파일은 container + codec data 두 부분으로 본다.
- 디코딩 결과는 sample rate / channels / sample format을 함께 들고 다닌다.
- 실시간 흐름은 디코딩과 콜백을 별 thread로 분리한다. ring buffer가 다리.
- 오프라인 흐름은 시간 제약 없이 파일 → 메모리 → 처리 → 파일.
- interleaved와 planar의 변환, 그리고 sample rate 정렬은 거의 항상 필요한 단계다.
