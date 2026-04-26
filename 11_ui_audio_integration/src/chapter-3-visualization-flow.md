# Chapter 3 - Meter, Spectrum, Waveform 데이터 흐름

이 장은 시각화 데이터의 종류와 그 흐름을 본다.

## 1. Meter — 가장 단순한 종류

peak meter / RMS meter / LUFS meter는 결국 **스칼라 값 몇 개**다.

```text
peak meter (per channel) : f32 한 개 + peak hold value 한 개
RMS meter                : f32 한 개
LUFS meter                : M/S/I 세 개 + true peak 한 개
```

audio thread가 매 callback에서 갱신하고, UI thread가 매 frame에서 읽는다.

```rust
struct AudioState {
    peak_l: AtomicU32,        // f32를 u32로 bit-cast해 atomic 저장
    peak_r: AtomicU32,
    rms_l:  AtomicU32,
    rms_r:  AtomicU32,
    lufs_m: AtomicU32,
    lufs_s: AtomicU32,
    lufs_i: AtomicU32,
}
```

읽기/쓰기.

```rust
// audio thread
state.peak_l.store(peak.to_bits(), Ordering::Relaxed);

// UI thread (매 frame)
let peak = f32::from_bits(state.peak_l.load(Ordering::Relaxed));
draw_meter_bar(peak);
```

`Relaxed`로 충분 (한 값만 다룸; 다른 변수와의 ordering 보장 불필요). 더 엄격한 ordering은 시각화에 의미 없다.

## 2. Peak hold는 UI thread가 관리해도 좋다

audio thread가 매 콜백 peak를 보내면, UI thread가 그것을 받아 hold + release를 적용해도 된다.

```text
audio thread → UI thread: 현재 peak (atomic)
UI thread (60fps):
  if 현재 peak > saved peak: saved = 현재 peak
  saved -= release_rate * dt
```

이렇게 하면 audio thread가 단순해진다 ("현재 peak만 알리면 끝"). UI 정책이 바뀌어도 audio는 안 바꾼다.

## 3. Spectrum — 큰 배열 데이터

spectrum은 N/2+1 개 bin × dB 값. N=2048이면 1025 floats.

이 정도 크기는 atomic으로 못 다룬다. **double buffer**가 표준.

```text
analyzer thread:
  buffer A를 채움
  완료 후 swap atomic으로 "최신 = A"

UI thread (매 frame):
  atomic 읽기 → "최신 = A"
  buffer A에서 데이터 복사 또는 직접 그리기
```

```rust
struct SpectrumDoubleBuf {
    buf_a: Box<[f32]>,
    buf_b: Box<[f32]>,
    latest: AtomicUsize,         // 0 = a, 1 = b
}

// analyzer
fn write(&mut self, new_data: &[f32]) {
    let writing = if self.latest.load(Ordering::Acquire) == 0 { &mut self.buf_b } else { &mut self.buf_a };
    writing.copy_from_slice(new_data);
    self.latest.store(if writing.as_ptr() == self.buf_a.as_ptr() { 0 } else { 1 }, Ordering::Release);
}

// UI
fn read(&self) -> &[f32] {
    if self.latest.load(Ordering::Acquire) == 0 { &self.buf_a } else { &self.buf_b }
}
```

작동 원리.

```text
- analyzer가 b를 채우는 동안 UI는 a를 읽음 (안전)
- analyzer가 b를 다 채우면 latest = 1로 swap
- 다음 UI frame부터는 b를 읽음
- analyzer는 다음 갱신을 a에 씀
- ...
```

이 단순한 swap이 lock 없이 cross-thread sharing을 가능하게 한다.

UI가 읽는 도중에 swap이 일어날 수 있는데, 그 frame은 한 frame 늙은 데이터를 끝까지 읽는다 (괜찮음). 다음 frame부터 새 데이터.

## 4. Triple buffer — UI rate가 오를 때

double buffer는 producer가 swap 직후 곧장 다시 쓰면 한 frame 동안 데이터가 갱신 안 될 수 있다.

```text
triple buffer:
  3개 buffer로 producer가 항상 새 buffer에 쓰고, consumer는 항상 최신 완료 buffer를 읽음
  → producer/consumer가 서로 안 막음
```

학습 단계에선 double로 시작, 부족하면 triple로. 또는 lock-free queue 한 슬롯 짜리.

## 5. Waveform — 시간축 amplitude

waveform 표시는 두 가지 모드.

### Live waveform (오실로스코프)

```text
audio 콜백마다 마지막 N 샘플을 ring buffer에 저장
UI가 매 frame N개 가져다 그림 → 부드럽게 흐르는 파형
```

ring buffer 자체가 SPSC면 lock-free.

### File waveform (트랙 보기)

```text
파일 로딩 시 또는 background에서:
  N pixel-width의 min/max envelope을 미리 계산
  waveform.png 같은 텍스처로 캐싱

UI thread는 그 캐싱된 데이터를 그대로 그림
```

이 경우 audio thread는 거의 관련 없음. 오프라인 처리.

## 6. EQ Curve — 시각화의 두 갈래

EQ curve는 두 가지 방식으로 그릴 수 있다.

```text
방식 A: parameter 값 → UI에서 식 한 번 더 평가
  - audio thread는 audio 처리만
  - UI thread가 cookbook 식을 한 번 더 평가해 곡선 그림
  - 매 frame 같은 식을 호출하지만 cost 작음 (수십 점 evaluate)
  - audio와 UI가 별 코드, 같은 식 두 번 → 동기화 부담

방식 B: audio thread가 frequency response 배열 갱신
  - parameter 변경 시 audio thread가 cookbook → response 배열을 atomic으로 UI에
  - UI는 atomic 읽기만
  - audio thread에서 cookbook 호출은 (이미 5장에서 본) 큰 비용은 아님
```

대부분 A가 더 단순하다. parameter 정의 (계수가 아닌 freq/gain/Q)만 atomic으로 공유하면 UI가 알아서 식을 평가.

## 7. Spectrum + EQ Curve 겹쳐 그리기

EQ UI의 표준 모양: 실시간 spectrum 위에 EQ 곡선 overlay.

```text
화면:
  배경: 실시간 spectrum (반투명)
  전경: EQ 곡선 (불투명, 사용자가 만지는 노드 표시 포함)
```

각 layer가 독립적인 데이터 source.

```text
spectrum: analyzer thread → double buffer → UI
EQ curve: parameter atomic → UI에서 식 평가
```

UI thread는 둘을 한 frame 안에 차례로 그린다.

## 8. Transport / 진행 위치

```text
audio thread → UI: 현재 sample position (u64 atomic)
UI thread는 그것을 sample → tick → bar/beat로 변환해 표시

UI thread → audio: play/stop/seek 명령 (enum atomic 또는 SPSC)
```

진행 위치는 매 frame UI에서 그리지만 audio thread는 매 sample 갱신한다 (그 사이의 차이는 cosmetic). 정확한 wall clock 동기는 audio가 알고, UI는 그 snapshot을 매 frame 보는 식.

## 9. CPU 부하 / Underrun 카운트

```text
audio thread:
  콜백 시작 → 끝 시간 측정 → atomic 평균 갱신
  underrun 발생 시 atomic counter 증가

UI:
  매 frame atomic 읽기 → "Engine load: 45%" 등 표시
```

실시간 진단 정보는 사용자에게 신뢰감을 준다.

## 10. RuStudio 관점

```text
mod_player UI:
  meter (atomic), transport position (atomic)
  
EQ UI:
  parameters (atomic per parameter), spectrum (double buffer)
  
mastering UI:
  LUFS M/S/I (atomic), GR meter (atomic), spectrum (double buffer), true peak (atomic)
  
piano roll UI:
  곡 데이터 (UI 자체 보유), 진행 위치 (atomic), 녹음 메시지 (SPSC)
```

## 자주 하는 실수

- spectrum을 audio thread에서 직접 만듦 → FFT가 콜백 안에 들어감.
- meter 갱신을 audio rate로 → atomic store가 너무 자주.
- waveform live 모드를 ring buffer 없이 → 일관 안 된 데이터 보기.
- EQ 곡선 그릴 때 cookbook 식을 매 frame 100번씩 평가 → 점 수를 적당히 (60~200) 잡기.
- transport position을 매 frame audio thread가 갱신하지 않음 → UI가 부드럽지 않음.

## 반드시 이해해야 할 것

- 시각화 데이터의 종류 (스칼라 / 배열 / sequence)에 따라 다른 패턴 (atomic / double buffer / SPSC).
- spectrum은 analyzer thread에서, meter는 audio thread에서, waveform live는 ring buffer로.
- EQ 곡선은 parameter atomic + UI에서 식 평가가 대부분 가장 단순하다.
- audio thread는 시각화 데이터를 "쉽게 가져갈 수 있게 노출"만 한다. 그리는 일은 안 한다.
