# Chapter 9 - Device Lifecycle과 실패 상태

이 장은 장치 변경/사라짐/실패에 mod_player가 어떻게 반응할지 정한다.

01 책에서 본 사실 두 가지를 다시 떠올린다.

```text
- default_output_device()는 Option<Device>를 돌려준다.
- Stream의 에러 콜백은 데이터 콜백과 별도다.
```

즉 "장치 없음"과 "스트림 에러"는 mod_player가 다뤄야 할 정상 경로다.

## 6가지 실패 시나리오

이 장에서 다룰 시나리오는 다음과 같다.

```text
1. 앱 시작 시 default 장치가 없음
2. 사용자가 다른 장치를 선택함
3. 사용한 장치가 OS에서 사라짐 (USB 분리)
4. 장치가 다시 나타남
5. Stream build가 실패함
6. Stream 에러 콜백이 호출됨 (재생 중 에러)
```

각 시나리오를 mod_player의 transport state로 매핑한다.

```text
시나리오                          → transport state
─────────────────────────────     ──────────────────
1. default 장치 없음              → NoDevice
2. 사용자가 다른 장치 선택        → Rebuilding → Stopped/Playing
3. 사용한 장치가 사라짐           → NoDevice
4. 장치가 다시 나타남             → Stopped (사용자가 명시적으로 선택해야 Playing)
5. Stream build 실패              → NoDevice + 에러 정보 보존
6. Stream 에러 콜백 호출          → 종류에 따라 Rebuilding 또는 NoDevice
```

## 1. 앱 시작 시 default 장치가 없음

```rust
let host = cpal::default_host();
let device = host.default_output_device();

let mut player = ModPlayer::new();
match device {
    Some(d) => player.attach_device(d)?,
    None    => player.set_state(TransportState::NoDevice),
}
```

UI는 NoDevice 화면을 보여 준다. 사용자가 새 장치를 연결하거나 직접 선택할 때 attach가 일어난다.

## 2. 사용자가 다른 장치 선택

```text
UI: device dropdown 클릭 → 선택된 DeviceId 전달
mp::handle(SelectDevice(id)):
    1. transport를 Rebuilding(previous = current_state)으로 전이
    2. 현재 stream drop
    3. 선택된 device로 negotiate → build → play (필요시)
    4. 성공 시 previous로 복귀 (Playing → Playing, Stopped → Stopped)
    5. 실패 시 NoDevice로 전이
```

## 3. 사용한 장치가 사라짐

이건 mod_player가 직접 감지하지 못한다. 에러 콜백을 통해 신호가 온다.

```rust
let err_cb = move |err: cpal::StreamError| {
    let _ = err_tx.push(StreamErrorEvent::from(err));
};
```

에러 콜백 안에서는 lock 절대 금지. SPSC 채널에 이벤트만 push.

```rust
// mod_player가 매 tick에 처리
while let Ok(ev) = err_rx.pop() {
    match ev.kind {
        StreamErrorKind::DeviceNotAvailable => {
            self.stream = None;
            self.state = TransportState::NoDevice;
            self.notify_ui(StateChange::DeviceLost);
        }
        StreamErrorKind::BackendSpecific(_) => {
            // 보수적으로 NoDevice 처리 또는 rebuild 시도
        }
    }
}
```

여기서 핵심: 에러 콜백 자체가 transport state를 바꾸지 않는다. 이벤트를 보낼 뿐이다.

## 4. 장치가 다시 나타남

OS가 device hot-plug 알림을 주는 백엔드도 있고 아닌 백엔드도 있다. 학습 단계에서는 두 가지 정책이 충분하다.

```text
A. 폴링: mod_player가 일정 주기로 default device를 확인
B. 사용자 트리거: UI에 "장치 다시 검색" 버튼
```

장치가 다시 나타나도 mod_player는 자동으로 Playing으로 복귀하지 않는다. 사용자가 의도적으로 play를 누르거나, "마지막 장치 자동 복원" 옵션을 켰을 때만.

## 5. Stream build 실패

```rust
let res = device.build_output_stream(&config, data_cb, err_cb, None);
match res {
    Ok(s) => {
        self.stream = Some(s);
        self.state = restore_previous;
    }
    Err(e) => {
        self.stream = None;
        self.state = TransportState::NoDevice;
        self.last_error = Some(e.into());
        self.notify_ui(StateChange::BuildFailed(e.kind()));
    }
}
```

UI에 보여 줄 정보:

```text
- 시도한 device 이름
- 시도한 sample rate / channels / format
- 받은 에러
- "다른 장치 선택" 또는 "다시 시도" 액션
```

## 6. Stream 에러 콜백 (재생 중 에러)

`cpal::StreamError`는 두 변종이 있다.

```text
DeviceNotAvailable     : 장치 사라짐
BackendSpecific(...)   : 백엔드(드라이버) 고유 에러
```

전자는 NoDevice로. 후자는 보수적으로 처리.

```rust
StreamErrorKind::BackendSpecific(_) => {
    if self.error_retry_count < MAX_RETRIES {
        self.state = TransportState::Rebuilding {
            previous: Box::new(self.state.clone()),
        };
        // 재시도
        self.error_retry_count += 1;
    } else {
        self.state = TransportState::NoDevice;
    }
}
```

연속 재시도 횟수 제한을 두지 않으면 무한 rebuild에 빠질 수 있다.

## 에러 정보 모델

mod_player는 사용자에게 보여 줄 마지막 에러를 들고 있어야 한다.

```rust
pub struct LastError {
    pub when: Instant,
    pub kind: ErrorKind,
    pub message: String,
    pub recovery_hint: Option<String>,
}

pub enum ErrorKind {
    NoDevice,
    BuildFailed(BuildErrorDetail),
    StreamError(StreamErrorDetail),
    NegotiationFailed(NegotiationLog),
}
```

`recovery_hint`는 UI가 "다음에 무엇을 해야 하는지" 사용자에게 알려줄 때 쓴다.

## 자주 하는 실수

- 에러 콜백 안에서 mutex/lock 사용 → 에러 콜백도 가벼운 경로다 (단, 데이터 콜백만큼 엄격하지 않음). SPSC 채널이 안전.
- 장치가 사라졌을 때 그냥 panic → 사용자 경험 최악. NoDevice 상태로 우아하게 전이.
- rebuild 무한 재시도 → 횟수 제한 + backoff 필요.
- 같은 장치를 두 stream이 동시에 열려고 함 → drop을 명시적으로 먼저.
- 에러 정보를 로그에만 남김 → UI가 사용자에게 못 보여 줌. 마지막 에러를 mod_player 필드로 보존.

## 반드시 이해해야 할 것

- "장치 없음"은 예외가 아니라 정상 경로다. NoDevice 상태로 명시적으로 표현한다.
- 에러 콜백은 이벤트만 push, 상태 변경은 mod_player가 main flow에서 처리.
- Rebuilding은 transient 상태다. 빌드 결과에 따라 이전 상태로 복귀하거나 NoDevice로 간다.
- 무한 재시도를 막는 retry 카운터와 backoff가 필요하다.
- 마지막 에러 정보는 mod_player가 들고 있어야 UI가 사용자에게 보여 줄 수 있다.
