# Chapter 4 - Thread 경계와 상태 전달

이 장은 thread 사이 통신의 표준 패턴 세 가지를 정리한다.

```text
패턴 1: Atomic 변수    — 단일 값 (스칼라)
패턴 2: SPSC lock-free queue — sequence (이벤트)
패턴 3: Double / triple buffer — 큰 배열 (spectrum, waveform)
```

## 1. Atomic — 단일 스칼라

가장 단순. f32를 u32로 bit-cast해서 `AtomicU32`로 다룬다.

```rust
struct PeakState {
    value: AtomicU32,
}

impl PeakState {
    fn store(&self, x: f32) {
        self.value.store(x.to_bits(), Ordering::Relaxed);
    }
    fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }
}
```

사용 예.

```text
peak meter / RMS / LUFS / true peak / GR
fader 값 / pan 값 / EQ band freq/gain/Q
transport play/stop bool / sample position
underrun counter / CPU load percentage
```

`Ordering::Relaxed`가 거의 항상 충분 (단일 값을 다른 변수와 무관하게 다룰 때).

다른 atomic과 함께 ordering이 필요한 경우 (예: "이 값을 쓴 후에 그 다음 값을 읽기로 약속") `Acquire/Release`. double buffer swap이 그 예시.

## 2. SPSC Lock-free Queue — 이벤트 시퀀스

producer 1 / consumer 1 가정의 큐. 가장 빠른 lock-free 자료구조.

```toml
[dependencies]
ringbuf = "0.4"
# 또는
rtrb = "0.3"
```

`ringbuf`의 사용.

```rust
use ringbuf::HeapRb;

let rb = HeapRb::<MidiEvent>::new(256);
let (mut producer, mut consumer) = rb.split();

// producer (예: MIDI thread)
producer.push(event).ok();

// consumer (audio thread)
while let Some(event) = consumer.pop() {
    synth.handle_midi(&event);
}
```

용도.

```text
MIDI events: MIDI thread → audio thread
parameter gestures: UI thread → audio thread (사용자 fader 빠른 움직임)
log/diagnostic: audio thread → 별 thread → 화면 출력
```

queue 용량은 적당히 크게 (256~4096). 가득 차면 push가 실패하고 메시지가 떨어진다.

## 3. Double Buffer — 큰 배열

spectrum, waveform 같이 한 frame이 큰 배열일 때.

```rust
struct DoubleBuf<T: Copy> {
    a: Box<[T]>,
    b: Box<[T]>,
    which: AtomicUsize,        // 0 = a 최신, 1 = b 최신
}

impl<T: Copy + Default> DoubleBuf<T> {
    fn new(size: usize) -> Self {
        Self {
            a: vec![T::default(); size].into_boxed_slice(),
            b: vec![T::default(); size].into_boxed_slice(),
            which: AtomicUsize::new(0),
        }
    }

    // producer
    fn write(&self, src: &[T]) {
        // 현재 stale한 buffer에 쓰기
        let cur = self.which.load(Ordering::Acquire);
        let target = if cur == 0 { &self.b } else { &self.a };
        // SAFETY: producer가 단 하나라는 가정. 동일 버퍼 동시 쓰기는 없음.
        let target_mut = unsafe {
            std::slice::from_raw_parts_mut(
                target.as_ptr() as *mut T,
                target.len(),
            )
        };
        target_mut.copy_from_slice(src);
        self.which.store(1 - cur, Ordering::Release);
    }

    // consumer
    fn read(&self) -> &[T] {
        let cur = self.which.load(Ordering::Acquire);
        if cur == 0 { &self.a } else { &self.b }
    }
}
```

(이 코드는 단순화 버전이다. 실제 production은 `triple_buffer` crate 같은 검증된 구현을 권장.)

triple buffer가 더 안전한 선택이지만, 개념은 같다.

```toml
[dependencies]
triple_buffer = "7"
```

## 4. 데이터 흐름의 표준 모양

이 셋 패턴 위에 올린 RuStudio의 표준 데이터 흐름.

```text
audio thread → analyzer thread:
  ring buffer (SPSC, audio sample stream)

analyzer thread → UI thread:
  double buffer (spectrum)
  atomic (peak hold per bin은 옵션)

audio thread → UI thread:
  atomic (meter, transport, underrun)

UI thread → audio thread:
  atomic (fader values, transport commands as enum)
  SPSC (gesture begin/end, MIDI learn events)

MIDI thread → audio thread:
  SPSC (MIDI events with timestamp)
```

이 매핑이 머리에 있으면 어떤 새 데이터가 등장해도 어디로 보낼지 결정이 쉽다.

## 5. UI → audio 명령의 표현

transport command 같은 것은 enum + atomic으로 충분하다.

```rust
#[repr(u32)]
enum TransportCmd { None, Play, Pause, Stop, Seek(u64) }

// UI: state.transport_cmd.store(TransportCmd::Play as u32, Ordering::Relaxed);
// audio: 매 콜백 시작에 검사 → 처리 → None으로 reset
```

복잡한 명령(파라미터 자동화 record 같은)은 SPSC.

## 6. Reverse — audio thread → UI 알림

audio thread가 UI에 비동기 알림을 보내고 싶을 때 (예: "파일 끝 도달", "underrun 발생").

```text
audio thread: counter atomic 증가
UI thread:    매 frame counter 읽고 직전 값과 비교 → 변화 감지
```

또는 SPSC로 실제 이벤트 객체를 보낼 수도. 단순 알림은 counter 패턴이 충분.

## 7. Lock 전혀 안 쓰나?

대부분 안 쓴다. 다만 audio thread를 *건드리지 않는* 다른 thread들 사이는 mutex 써도 OK.

```text
설정 thread (project file load) ↔ analyzer thread: mutex OK (둘 다 audio thread 아님)
UI thread ↔ audio thread: 절대 mutex 금지
audio thread ↔ analyzer thread: 절대 mutex 금지
```

mutex가 안 좋은 게 아니라, **audio thread만은** 절대 안 된다. 다른 thread 간에는 단순함이 더 중요할 수 있다.

## 8. Allocation 정책

audio thread는 절대 새로 할당하지 않는다.

```text
audio thread:
  미리 할당된 buffer / state만 사용
  Vec::push 같은 grow 금지

analyzer thread:
  주기적 할당 OK (큰 buffer 재할당은 안 좋음)
  hot path는 미리 할당 권장

UI thread:
  자유로움 (단 frame 안에 끝나야 함)
```

double buffer / SPSC queue 모두 시작 시 한 번 할당하고 그 뒤로는 재할당 안 함.

## 9. 시작 / 종료 순서

```text
시작:
  audio device 열기 → analyzer thread 시작 → UI thread 시작 → audio stream play

종료 (역순):
  UI 닫기 → audio stream pause → analyzer thread stop → audio device close
```

graceful 종료를 위해 thread들이 stop flag를 보고 알아서 종료할 수 있게 해야 한다.

## 10. RuStudio의 thread 구조 정리

```text
main thread:
  app entry, 사용자 메인 윈도우. 보통 UI thread와 같음.

audio thread (cpal 콜백):
  매 콜백마다 graph.run()
  meter, transport position 등을 atomic으로 노출
  parameter atomic / SPSC 읽기

analyzer thread (별 thread 또는 timer task):
  audio thread에서 받은 ring buffer를 hop 단위로 FFT
  결과를 double buffer로 UI에 노출

MIDI input thread (midir 콜백):
  메시지를 timestamped queue로 audio thread에 전달

worker thread (오프라인 처리, file load 등):
  decoder, 큰 작업
  결과를 audio engine state로 옮김 (audio thread는 atomic 또는 swap으로 받기)
```

이 5개 thread가 RuStudio 전체의 thread 모양이다.

## 자주 하는 실수

- 큰 spectrum 배열을 atomic 하나로 다루려고 시도 → 불가능.
- SPSC가 가득 찼을 때 처리 안 함 → 메시지 drop이 silent.
- double buffer의 swap이 race condition을 유발 → ordering (Acquire/Release) 누락.
- audio thread에서 SPSC try_pop()이 Vec::with_capacity 같은 함수를 부르는 type → 알게 모르게 할당.
- UI에서 audio state에 mutex로 접근 → 확실히 잘못됨.

## 반드시 이해해야 할 것

- 세 패턴: atomic (스칼라), SPSC (sequence), double buffer (배열).
- 데이터 종류와 방향에 맞는 패턴을 선택한다.
- audio thread는 어떤 경우에도 lock 잡지 않는다. 새 할당도 하지 않는다.
- 이 패턴들 위에 RuStudio의 5개 thread가 자연스럽게 올라간다.
