# Chapter 2 - UI와 Audio는 왜 분리해야 하나

## 1. 시간 단위가 다르다

```text
audio thread: 콜백 사이에 ~5 ms 여유
UI thread   : 60 fps면 frame 사이에 ~16 ms 여유
```

이 둘이 같은 시간을 산다고 생각하면 빠르게 무너진다.

예시: 사용자가 fader를 천천히 움직이는 1초 동안.

```text
audio thread가 처리한 sample 수:  48,000개 (× 채널 수)
UI thread가 그린 frame 수      :  60개
fader가 움직인 위치 변화 수      :  ~50번 (mouse move 빈도)
```

audio thread가 매 sample마다 fader 변화를 보고 새로 곱한다 해도, UI thread는 1/300 비율의 정밀도로만 갱신된다. UI 입장에서 audio가 필요로 하는 정밀도는 의미 없다.

## 2. Block 가능성이 다르다

```text
audio thread:
  block 절대 금지. lock 잡으면 즉시 underrun 위험.
  syscall, 할당, IO 모두 위험.

UI thread:
  16 ms 안에만 끝나면 OK.
  텍스처 업로드, 메모리 할당, 일부 IO 모두 가능.
```

같은 자료구조에 두 thread가 동시에 접근하려면 lock이 필요하다. 하지만 lock은 audio thread에서 못 쓴다 → 다른 mechanism (atomic, lock-free queue, double buffer)이 필요.

## 3. 데이터 양이 다르다

```text
audio thread → UI thread (시각화 데이터):
  meter:    스칼라 1~3개 (peak, RMS, LUFS) per 채널
  spectrum: 1024~4096 bin × dB 값
  waveform: 화면 width 픽셀 × min/max 두 값

audio thread는 매 sample 처리하지만, UI에 보내는 양은 매 frame 한 번. 그 정도면 충분.
```

audio rate(48 kHz)로 spectrum을 만드는 건 무의미하다. 사람 눈은 60 fps 이상 잘 인지 못한다.

```text
spectrum 갱신 주기  : 30~60 fps
meter 갱신 주기     : 30~60 fps (peak hold 포함)
waveform 스크롤     : 60 fps (부드러운 움직임에)
EQ curve 그리기     : parameter 변경 시 + 매 frame (저비용)
```

## 4. 실패의 의미가 다르다

```text
audio thread 실패  : 들리는 결함. 사용자가 즉시 인지. 신뢰 손상.
UI thread 실패    : 1 frame 거름. 사용자가 거의 인지 못함. 한 번이면 무시 가능.
```

이 비대칭이 모든 설계 결정의 근간이다. UI 쪽에서 실패해도 괜찮지만, audio 쪽이 망가지면 안 된다 — 그래서 UI가 audio thread에 부담을 주는 일은 절대 안 된다.

## 5. 만약 분리 안 하면 — 흔한 실패 시나리오

```text
시나리오 1: audio thread에서 GUI 함수 직접 호출
  → GUI 함수가 메모리 할당, 일부는 lock
  → 갑자기 콜백 시간이 들쭉날쭉
  → underrun, glitch

시나리오 2: UI thread에서 audio state에 lock 잡고 접근
  → audio thread도 같은 lock을 기다리게 됨
  → UI 한 번 mouse move마다 audio가 5~10 ms 멈춤
  → "지직"하는 노이즈가 매번 발생

시나리오 3: 같은 자료구조를 mutex 없이 공유
  → race condition
  → 가끔 잘못된 값으로 audio 처리, 가끔 UI에 NaN 표시
  → 재현 불가능한 디버깅 지옥
```

## 6. 분리의 표준 패턴

이 셋이 11 책의 4장 주제다.

```text
1. atomic 변수
   - 단일 값 (peak meter, fader 값, transport state 등)
   - 가장 단순. 자료구조 한 슬롯이면 OK.

2. SPSC lock-free queue
   - 단방향 sequence (MIDI events, parameter gestures, log messages)
   - producer 1, consumer 1 가정. 가장 빠른 lock-free 큐.

3. double-buffered slice / triple buffer
   - 큰 배열 (spectrum, waveform 데이터)
   - producer가 한 buffer 채우는 동안 consumer가 다른 buffer 읽음
   - swap 시점에 atomic pointer
```

## 7. 무엇을 어디로 보낼지 매핑

| 종류 | 방향 | 패턴 |
|---|---|---|
| peak / RMS / LUFS | audio → UI | atomic |
| transport state | audio → UI / UI → audio | atomic |
| fader / knob 값 | UI → audio | atomic |
| transport command | UI → audio | atomic 또는 SPSC |
| MIDI events (record) | audio → UI | SPSC |
| parameter automation | UI → audio | SPSC + smoothing |
| spectrum bins | analyzer thread → UI | double buffer |
| waveform | analyzer thread → UI | double buffer 또는 SPSC |
| EQ curve params | audio (또는 UI 자체) → UI | atomic per param |
| underrun count | audio → UI | atomic |

## 8. RuStudio 관점

```text
mod_player UI:
  meter / 트랙 fader / transport / 진행 위치

EQ UI:
  EQ curve (parameter들 → UI 자체에서 식 한번 더 평가)
  + 실시간 spectrum (감지된 신호 위에 EQ 곡선 겹침)
  + 4 ~ 8 밴드 노브들

mastering UI:
  LUFS M/S/I 미터
  GR 미터 (compressor/limiter 깎인 양)
  K-weighted spectrum

piano roll UI:
  현재 transport 위치 (audio → UI)
  사용자가 그린 노트 (UI → audio sequencer)
  녹음된 MIDI (audio → UI, SPSC)
```

이 모두가 위 8장의 매핑 위에 올라간다.

## 자주 하는 실수

- "잠깐인데 lock 한 번 걸어도 되겠지" → 그 잠깐이 audio underrun을 만든다.
- audio thread에서 GUI 함수 호출 → 함수가 무엇을 하는지 control 불가.
- UI 갱신을 audio rate로 → CPU 낭비, UI 부드럽지도 않음.
- mutex 없이 raw 공유 → race condition.
- spectrum 데이터를 매 콜백마다 ms 단위 보냄 → UI가 따라가지 못해 backpressure.

## 반드시 이해해야 할 것

- audio thread와 UI thread는 시간 단위, block 가능성, 데이터 양, 실패 의미가 모두 다르다.
- 두 thread 사이의 통신은 항상 lock-free (atomic / SPSC / double buffer) 패턴.
- 데이터 종류에 맞는 패턴 선택이 중요하다 — 단일 값/sequence/큰 배열에 따라 다르다.
- audio thread의 deadline을 어떤 경우에도 침범하지 않는다.
