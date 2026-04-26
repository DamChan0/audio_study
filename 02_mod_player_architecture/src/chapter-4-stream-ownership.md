# Chapter 4 - Stream 소유권

01 책에서 본 핵심 사실 하나.

> `cpal::Stream`이 drop되면 오디오도 즉시 멈춘다.

즉 "누가 Stream을 들고 있는가"는 단순 변수 위치 문제가 아니라 **재생이 살아 있는가**의 문제다.

이 장은 그 소유권을 어떻게 설계할지 정한다.

## 한 줄 결론

> `Stream`은 `mod_player`가 **단 하나의 long-lived 필드**로 들고 있는다.

UI는 Stream을 알지 못하고, dsp 모듈도 모른다. 오직 mod_player만이 build / drop 권한을 가진다.

## 왜 mod_player인가

다른 후보들을 빠르게 탈락시켜 보자.

### "UI가 Stream을 들고 있으면?"

```text
- UI 화면이 닫히면 Stream이 drop → 소리가 멈춤
- UI 프레임워크 교체 시 Stream도 다시 짜야 함
- UI thread에서 build_output_stream을 부르는 게 어색함
```

### "static 전역에 두면?"

```text
- 테스트가 어렵다 (전역 상태)
- 두 개의 mod_player를 동시에 만드는 시나리오를 막아 버림
- drop 시점이 모호해짐
```

### "각 source가 자기 stream을 만들면?"

```text
- 다중 source = 다중 stream → 같은 장치를 여러 stream이 동시에 열려고 함
- mixer가 어디로 흘러가는지 따라가기 어려움
- 장치 lifecycle을 누가 관리하는지 분산
```

### "mod_player가 들고 있으면?"

```text
✓ Stream의 생애와 transport 상태가 한 곳에 있음
✓ 장치 변경 시 한 곳에서 rebuild
✓ UI / DSP는 영향 없음
✓ 두 개의 mod_player가 동시 존재 가능 (테스트 / 멀티 트랙 등)
```

mod_player가 답이다.

## 소유 모양

대략 이런 모양이 된다.

```rust
pub struct ModPlayer {
    stream: Option<cpal::Stream>,        // 현재 살아 있는 출력 스트림
    state: TransportState,               // Stopped / Playing / Paused / ...
    config: cpal::StreamConfig,          // 현재 협상된 설정
    cmd_tx: rtrb::Producer<AudioCommand>, // 콜백에 보낼 명령 채널
    state_tx: AtomicState,               // 콜백이 UI로 보낼 상태
    // ...
}
```

요점은 두 가지.

1. `stream`이 `Option<...>`이다. **없을 수 있는 상태**가 정상 경로다 (Rebuilding, NoDevice).
2. `cmd_tx`/`state_tx`가 콜백과의 다리. 이걸로만 콜백과 통신한다.

## Stream의 생애 5단계

mod_player가 다루는 Stream의 생애는 이렇게 본다.

```text
1. None              ← 앱 시작 직후 / 장치 없음
2. Building          ← supported_configs 협상 중
3. Idle (built)      ← 만들어졌지만 play() 호출 전
4. Playing           ← play() 호출 후, 콜백 활발히 돔
5. Drop              ← 다른 장치 선택 / 종료 / 에러 → 다음 단계로 회귀
```

이 5단계가 다음 장의 transport state machine 으로 자연스럽게 대응된다.

## "Stream을 담은 필드"의 함정

`Option<Stream>`을 써도 다음 두 가지를 조심한다.

### 1. drop 순서

```rust
self.stream = None;       // 새 stream 만들기 전에 명시적으로 drop
let new = build_stream(...)?;
self.stream = Some(new);
```

같은 장치를 두 stream이 동시에 열려고 하면 OS 레벨에서 거부될 수 있다. 새로 만들기 전에 옛것을 명시적으로 drop.

### 2. mutex로 감싸지 않기

```rust
// 안 좋음
stream: Mutex<Option<Stream>>
```

Stream은 audio thread가 보지 않는다 (Stream 자체는 mod_player가 보고, audio thread는 그 안의 콜백 closure만 본다). 그래서 mutex로 감쌀 필요가 사실상 없다.

만약 multi-threaded UI가 동시에 mod_player API를 부른다면, 그건 mod_player API 차원에서 `&mut self`로 강제하는 게 깔끔하다.

## 콜백 안과 밖의 데이터

Stream 소유 설계를 할 때 가장 헷갈리는 부분이다.

```text
콜백 밖 (mod_player가 들고 있는 것):
  - Stream 자체
  - TransportState
  - StreamConfig
  - cmd 송신 측 (Producer)
  - 상태 수신 측 (Consumer / Atomic 읽기)
  - source factory, dsp factory 등 "재료"

콜백 안 (Stream의 closure가 캡처해야 하는 것):
  - cmd 수신 측 (Consumer)
  - 상태 송신 측 (Producer / Atomic 쓰기)
  - source 인스턴스 (또는 source slot)
  - DSP chain 인스턴스 (또는 ABA-safe slot)
```

같은 SPSC 큐의 두 끝이 콜백 안과 밖에 각각 들어간다. mod_player를 만들 때 이 두 끝을 한 번에 만들고 closure에 한 끝을 옮긴다.

```rust
let (cmd_tx, cmd_rx) = rtrb::RingBuffer::<AudioCommand>::new(64);

let stream = device.build_output_stream(
    &config,
    move |data: &mut [f32], _| {
        // cmd_rx, source, chain 등을 캡처
        while let Ok(cmd) = cmd_rx.pop() { /* state 반영 */ }
        for frame in data.chunks_mut(channels) { /* ... */ }
    },
    err_cb,
    None,
)?;

let player = ModPlayer { stream: Some(stream), cmd_tx, /* ... */ };
```

이게 Stream 소유 설계의 표준 모양이다.

## Send / Sync 사정

`cpal::Stream`은 `!Send`다 (플랫폼에 따라). 즉 만들어진 thread를 떠날 수 없는 경우가 있다.

대처법.

```text
- mod_player를 한 thread에서만 다룸 (예: 메인 thread). 보통 OK.
- mod_player API를 그 thread로 라우팅하는 actor 형태로 감싸기.
```

대부분의 데스크탑 앱에서는 mod_player를 메인 thread에서만 다루고, 다른 thread는 명령/상태 채널로만 접근하는 형태가 깔끔하다.

## 자주 하는 실수

- 함수 안에서 `let stream = build_output_stream(...)?;` 하고 함수가 끝나면 stream drop → 1초 못 가서 멈춤. 01 책에서 본 실수.
- `Arc<Mutex<Stream>>`로 감싸고 audio thread에서 lock → 콜백 안 mutex 위반 + Stream을 콜백이 보지 않으니 의미도 없음.
- 두 mod_player가 같은 장치에 동시에 stream을 만듦 → OS 거부.
- 새 stream 만들기 전에 옛것을 drop하지 않음 → 같은 장치 두 번 열기.
- Stream을 send해서 다른 thread에서 play() → `!Send` 에러.

## 반드시 이해해야 할 것

- `cpal::Stream`의 소유자는 `mod_player`다. 그 어느 곳도 아니다.
- Stream은 `Option<...>`으로 둔다. "없는 상태"가 정상 경로다 (rebuild / NoDevice).
- 콜백 closure가 캡처하는 것과 mod_player가 들고 있는 것을 항상 두 묶음으로 구분한다.
- Stream을 mutex로 감쌀 필요는 거의 없다. 콜백은 Stream 자체를 안 본다.
- 새 stream 만들기 전에 옛것을 명시적으로 drop한다.
