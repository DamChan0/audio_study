# Chapter 6 - UI ↔ Audio Thread 제어 경로

이 장은 두 thread 사이에서 어떻게 안전하게 데이터를 주고받을지 정한다.

```text
UI thread       (블로킹 OK, 프레임 16ms)
mod_player API  (mostly UI thread에서 호출)
audio thread    (실시간, 블로킹 절대 안 됨)
```

이 경계 사이에는 일반 객체 공유, mutex, 채널, atomic 등 옵션이 많다. 이 장은 어떤 도구를 어디에 쓸지 결정한다.

## 한 줄 결론

```text
audio thread → mod_player : SPSC ring buffer (lock-free 채널)
mod_player → audio thread : SPSC ring buffer 또는 atomic
큰 객체 교체              : Arc swap 또는 channel
간단한 on/off / 한 정수    : atomic
```

mutex는 audio thread 경로에 절대 들어가지 않는다.

## 도구 4종 비교

### atomic

```rust
let active = Arc::new(AtomicBool::new(false));
let active_audio = active.clone();
move |data, _| { if !active_audio.load(Ordering::Relaxed) { /* ... */ } }
```

- 사용처: on/off, 한 정수 파라미터, 작은 enum
- 장점: 가장 빠르다, 단순하다
- 한계: 여러 값을 일관되게 묶어 보낼 수 없음 (한 atomic당 한 값)

### SPSC ring buffer (lock-free 채널)

```rust
use rtrb::{Producer, Consumer};

let (mut tx, mut rx) = rtrb::RingBuffer::<AudioCommand>::new(64);

// 콜백 안:
move |data, _| {
    while let Ok(cmd) = rx.pop() {
        match cmd { /* 명령 반영 */ }
    }
    // ...
}

// mod_player API:
tx.push(AudioCommand::SetGain(0.5)).ok();
```

- 사용처: 명령 흐름, source 교체 통보, 파라미터 변경 묶음
- 장점: 순서 보존, 묶인 명령 안전 전달
- 한계: 큐 가득 차면 push 실패 (정책 필요), unbounded는 audio thread에서 위험

`ringbuf`와 `rtrb` 두 가지가 흔하다.

```text
ringbuf : single-producer single-consumer + bounded. 안전, 검증 많음.
rtrb    : 같은 SPSC. API가 더 간결. 둘 다 적합.
```

### Arc swap (큰 객체 교체)

```rust
use arc_swap::ArcSwap;

let chain = Arc::new(ArcSwap::from_pointee(DspChain::default()));
let chain_audio = chain.clone();

move |data, _| {
    let snap = chain_audio.load();    // lock-free pointer load
    // snap을 매 콜백 또는 매 frame마다 사용
}

// 외부:
chain.store(Arc::new(new_chain));     // pointer swap만
```

- 사용처: DSP chain 통째로 교체, source 통째로 교체
- 장점: 큰 객체도 lock-free로 안전 교체 (RCU 패턴)
- 한계: 옛 객체의 drop 시점이 audio thread 마지막 사용 후로 미뤄짐

### channel (crossbeam / tokio 등)

대부분 audio thread 경로로는 부적절. UI ↔ background thread (파일 로드, 분석 등)에는 OK.

## 통신 카탈로그 (RuStudio 예시)

실제 mod_player에서 어떤 도구를 어디에 쓸지 표로 정리하면 다음과 같다.

```text
What                                Direction               Tool
─────────────────────────────────   ─────────────────       ──────────────
play / pause flag                   mp → audio thread       AtomicBool
master gain (linear)                mp → audio thread       AtomicU32 (f32 bits)
EQ band freq/gain/Q                 mp → audio thread       SPSC cmd queue + smoothing
source switch                       mp → audio thread       SPSC cmd or ArcSwap
DSP chain swap                      mp → audio thread       ArcSwap
underrun / xrun event               audio thread → mp       SPSC small event queue
peak / RMS meter (per buffer)       audio thread → mp       AtomicU32 또는 SPSC
current sample position             audio thread → mp       AtomicU64
```

이 표를 mod_player 설계 초기에 한 번 그려 두면, 코드를 짤 때 어느 채널이 필요한지 헷갈리지 않는다.

## 명령 enum 설계

audio thread로 보낼 명령은 보통 enum으로 한 큐에 묶는다.

```rust
pub enum AudioCommand {
    SetGain(f32),
    SetEqBand { idx: usize, freq: f32, gain_db: f32, q: f32 },
    SwapSource(SourceSlot),
    Bypass(bool),
    // ...
}
```

설계 원칙.

```text
1. 명령은 작아야 한다 (할당 없이 push 가능).
2. 명령에 Box / String / Vec를 넣지 않는다 (할당 + drop이 audio thread에서 일어남).
3. 큰 객체를 보낼 때는 ArcSwap 또는 미리 만들어진 슬롯 + index로 우회.
```

세 번째가 중요하다. 새 source를 보낼 때 `SwapSource(Box<dyn Source>)`로 보내면, 옛 source의 drop이 audio thread에서 일어나서 실시간 위반이 된다.

대처: source는 미리 외부에서 만들어 두고, "교체 명령"만 보낸다. 옛 source는 "버려야 할 객체 채널"로 다시 회수해 외부 thread에서 drop.

```rust
SwapSource { new: NonNull<dyn Source> }      // 또는 슬롯 인덱스
```

## 상태를 UI로 보내기

audio thread에서 UI로 보내는 데이터는 보통 두 종류다.

```text
- "현재 값" (peak, RMS, position) → atomic으로 충분
- "이벤트" (underrun 발생, source 끝) → 작은 SPSC 이벤트 큐
```

UI는 60 fps로 polling 한다. atomic은 매 프레임 load. 이벤트 큐는 매 프레임 drain.

```rust
// 콜백 안:
peak_atomic.store(peak.to_bits(), Ordering::Relaxed);

// UI thread (예: 16 ms마다):
let peak = f32::from_bits(peak_atomic.load(Ordering::Relaxed));
ui.draw_meter(peak);
```

## 명령 큐가 가득 찰 때

```rust
match tx.push(cmd) {
    Ok(()) => {}
    Err(_) => {
        // 큐 가득 → 명령 dropped
        log::warn!("audio command queue full");
    }
}
```

큐 크기는 보통 64 ~ 256이면 충분하다 (사용자가 1초에 보내는 명령 수가 그 안에 들어감). 큐 가득은 audio thread가 명령을 안 빼간다는 뜻이고, 그건 콜백 자체가 멈췄다는 신호다.

## 자주 하는 실수

- audio thread에 mutex 한 번이라도 들어감 → 글리치.
- `String` / `Vec` 명령을 audio thread로 보냄 → 옛 객체의 drop 비용을 audio thread가 떠안음.
- atomic의 ordering을 모두 SeqCst로 → 비싸다. 대부분 Relaxed 또는 Acquire/Release면 충분.
- Arc swap 후 옛 객체가 audio thread를 떠나는 순간을 가정 → 다음 콜백이 들어오기 전엔 안전 보장 안 됨. 보통 RCU 라이브러리가 처리해주지만 직접 구현하면 함정.
- 모든 채널을 unbounded로 → audio thread가 push할 때 push가 막힐 수 있음. SPSC는 항상 bounded.

## 반드시 이해해야 할 것

- audio thread 경계에서는 mutex 금지, atomic / SPSC ring / ArcSwap만 사용.
- 명령은 작고 할당 없는 enum으로 묶는다. 큰 객체는 슬롯/포인터/RCU로 우회.
- mod_player → audio thread는 "명령 + 파라미터", audio thread → mod_player는 "현재 값 + 이벤트"로 두 흐름을 분리한다.
- 옛 객체의 drop을 audio thread가 떠안지 않게 회수 채널로 빼낸다.
- 큐 크기와 가득 차는 정책을 미리 정한다.
