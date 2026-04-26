# Chapter 7 - 자주 하는 실수와 복습

## 일반

- 디코딩을 cpal 콜백 안에서 호출 → underrun.
- 파일 SR을 cpal SR로 가정 → 음정 어긋남.
- 채널 수가 다른 경우를 처리 안 함 → 모노 파일이 한쪽 채널에만.
- planar/interleaved 변환을 일관 안 함 → 좌우 섞임 또는 노이즈.

## WAV / sample format

- f32 → i16 변환 clamp 누락 → wrap-around 디지털 노이즈.
- ÷ 32768 vs ÷ 32767 혼용 (정확도 영향).
- `WavWriter::finalize()` 누락 → 헤더 손상.
- bits_per_sample과 sample_format 불일치 spec.

## symphonia

- AudioBufferRef variant를 일부만 처리 후 panic.
- planar 디코더 출력을 interleaved cpal에 그대로 → 채널 섞임.
- seek 후 `decoder.reset()` 누락 → 노이즈.
- 매 packet마다 interleaved Vec 새로 할당 → 메모리 할당 폭증.

## rubato / SRC

- 단순 인덱스 스킵으로 SRC 시도 → aliasing.
- DSP 체인 중간에 SRC를 끼움 → 일관성 망가짐.
- 콜백 안에서 직접 SRC 호출 → chunk 단위 처리 부담.
- 출력 길이가 입력 × 비율과 정확히 같다고 가정 → 들쭉날쭉.

## 처리 단계 입출력 상태표

```text
hound WAV reader      input: 파일 경로   output: i16 또는 f32 stream  state: 파일 핸들
hound WAV writer      input: f32→i16     output: 파일                state: 파일 핸들
symphonia decoder     input: packet      output: planar PCM AudioBuffer state: codec 내부
planar → interleaved  input: Vec<Vec<f32>> output: Vec<f32>         state: 없음
rubato resampler      input: planar chunk output: planar chunk      state: filter 메모리
i16 → f32             input: i16         output: f32 (÷ 32768)       state: 없음
f32 → i16 (clamp)     input: f32         output: i16 (clamp + ×32767) state: 없음
```

## Phase 6 체크리스트

```text
□ WAV 파일을 hound로 읽고 다시 쓰는 round trip 가능
□ i16 ↔ f32 변환식을 외움 (÷ 32768, clamp + ×32767)
□ interleaved ↔ planar 변환 함수 직접 작성 가능 + 단위 테스트
□ symphonia로 mp3/flac 한 곡 디코딩 가능
□ AudioBufferRef variant를 f32 planar로 정규화하는 helper 작성 가능
□ rubato로 44.1k → 48k SRC 직접 동작 확인
□ 단순 인덱스 스킵 SRC가 왜 안 되는지 설명 가능 (aliasing/imaging)
□ 오프라인 마스터링 파이프라인 (decode→SRC→DSP→SRC→encode) 그림 가능
□ 같은 처리 코드를 실시간(콜백)과 오프라인(파일)에서 어떻게 분기/공유할지 설계 가능
```

## 03~06 책 도구의 재사용 지도

```text
03 phase accumulator        : 검증용 sweep tone 생성
03 envelope / gain          : 오프라인 fade-in/out, normalize
04 compressor / limiter     : 06번 예제의 마스터링 단계
05 EQ                       : 마스터링/파일별 EQ 프리셋
06 FFT                      : mp3와 wav의 spectrum 비교 (압축 손실 확인)
06 ring buffer (SPSC)       : 디코더 thread → audio thread 다리
```

## 다음 책으로 넘어가는 다리

다음 책은 `08_audio_graph_architecture`다.

지금까지의 처리는 "직선 사슬"이었다. source → DSP → output, 또는 decoder → DSP → encoder.

08 책은 그 사슬을 더 일반화한 **노드 그래프**로 옮긴다. 여러 source와 여러 destination, 그리고 send/return bus 같은 분기/합산 구조를 표현하기 위해서다. 07까지 본 파일 source는 그 그래프의 한 종류 노드가 된다.

## 한 줄 요약

> 파일 I/O는 디코더 → 변환(format / channel / rate) → DSP → 변환 → 인코더 4단계의 합성이다. 핵심은 sample format / interleave / sample rate를 일관 유지하고, 디코딩과 SRC를 cpal 콜백 밖으로 빼는 것이다.
