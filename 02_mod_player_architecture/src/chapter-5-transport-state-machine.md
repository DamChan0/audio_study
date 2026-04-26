# Chapter 5 - Transport 상태 모델

mod_player의 두 번째 핵심은 **transport 상태기계**다. Stream의 생애를 사용자/시스템 입장에서 본 추상이다.

## 5개 상태

이 책에서 권장하는 최소 상태 집합은 다섯 가지다.

```text
Stopped     : Stream은 만들어져 있으나 재생 안 함 (또는 만들어져 있지 않음)
Playing     : 콜백이 활발히 돌고 출력이 나감
Paused      : Stream은 살아 있으나 콜백이 0을 출력 중 (또는 일시 정지)
Rebuilding  : 장치/설정 변경 → Stream을 다시 만드는 중 (transient)
NoDevice    : 출력 장치가 없거나 사라짐
```

각 상태의 의미와 용도.

```text
Stopped    UI: "재생" 버튼이 활성. 메터는 0. 위치는 시작점.
Playing    UI: "정지" 버튼이 활성. 메터가 움직임.
Paused     UI: "재생" 또는 "재개". 메터는 0. 현재 위치 유지.
Rebuilding UI: 짧은 spinner / "재구성 중...". 사용자 입력 잠시 차단 가능.
NoDevice   UI: "오디오 장치를 선택하세요" 패널. 다른 모든 동작 차단.
```

## 상태 전이 그림

```text
                           ┌──────── NoDevice ─────────┐
                           │                            │
                           │  device                    │ device
                           │  appeared                  │ removed
                           ▼                            │
        ┌──────────► Stopped ◄──────────────────────────┤
        │                  │                            │
        │   stop / source  │ play()                     │
        │     change       │                            │
        │                  ▼                            │
        │            Playing ──── pause() ──► Paused    │
        │              │   ▲                    │       │
        │              │   │                    │       │
        │              │   └─── resume() ───────┘       │
        │              │                                │
        │              │ device / config changed        │
        │              ▼                                │
        └──── Rebuilding ◄──────────────────────────────┘
                  │
                  │ build success
                  ▼
              (Stopped 또는 Playing 으로 복귀)
```

## 상태 전이 사건 카탈로그

각 상태에서 어떤 사건으로 어디로 가는지를 표로 정리한다.

```text
From          Event                   To             Action
─────────     ────────────────────    ─────────      ─────────────────────────────
Stopped       play()                  Playing        Stream.play()
Stopped       set_source(...)         Stopped        source slot 교체
Stopped       device removed          NoDevice       Stream drop
Playing       pause()                 Paused         Stream.pause() 또는 콜백 0 출력
Playing       stop()                  Stopped        Stream.pause() + source rewind
Playing       device removed          NoDevice       Stream drop
Playing       config changed          Rebuilding    Stream drop, build start
Paused        resume()                Playing        Stream.play()
Paused        stop()                  Stopped        rewind
Paused        device removed          NoDevice       Stream drop
Rebuilding    build ok                Playing/Stop   resume 또는 stop, 이전 의도 복원
Rebuilding    build fail              NoDevice       에러 보고
NoDevice      device appeared         Stopped        새 장치로 build → Stopped
NoDevice      *                       NoDevice       다른 명령 차단
```

이 표가 곧 `match` 문 골격이 된다.

## Rust로 표현

```rust
pub enum TransportState {
    Stopped,
    Playing,
    Paused,
    Rebuilding { previous: Box<TransportState> },
    NoDevice,
}

pub enum TransportCmd {
    Play,
    Pause,
    Resume,
    Stop,
    SetSource(SourceHandle),
    DeviceChanged(DeviceId),
    DeviceRemoved,
    DeviceAppeared(DeviceId),
}
```

`Rebuilding`이 `previous`를 들고 있는 점에 주의. 빌드 성공 후 어디로 돌아갈지 기억해야 한다.

전이 함수의 모양은 이렇다.

```rust
impl ModPlayer {
    pub fn handle(&mut self, cmd: TransportCmd) -> Result<(), TransportError> {
        match (self.state.clone(), cmd) {
            (TransportState::Stopped, TransportCmd::Play) => {
                self.stream.as_ref().ok_or(TransportError::NoStream)?.play()?;
                self.state = TransportState::Playing;
                Ok(())
            }
            (TransportState::Playing, TransportCmd::Pause) => {
                self.stream.as_ref().unwrap().pause()?;
                self.state = TransportState::Paused;
                Ok(())
            }
            (TransportState::Playing, TransportCmd::DeviceRemoved) => {
                self.stream = None;
                self.state = TransportState::NoDevice;
                Ok(())
            }
            // ...
            (state, cmd) => Err(TransportError::InvalidTransition { state, cmd }),
        }
    }
}
```

요점.

- 모든 명령은 한 함수의 `match`로 들어간다. 분기가 분산되면 표와 어긋난다.
- 잘못된 전이는 에러로 반환한다 (panic 아니다).

## 콜백 안에서의 transport

콜백 안에서는 보통 atomic 한 비트만 본다 — "지금 0을 출력해야 하나?".

```rust
// 콜백 closure가 캡처
let active = Arc::new(AtomicBool::new(false));
let active_in = active.clone();

move |data: &mut [f32], _| {
    if !active_in.load(Ordering::Relaxed) {
        for s in data.iter_mut() { *s = 0.0; }
        return;
    }
    // 정상 처리
}
```

`active`가 false면 Paused 또는 Stopped다 (콜백은 두 가지를 구분할 필요가 없다 — UI 차원의 구분이다).

mod_player가 `Stream::pause()`/`Stream::play()`를 직접 부르는 방식과 atomic flag 방식 중 어느 쪽을 쓰느냐는 트레이드오프다.

```text
Stream::pause()  : OS가 콜백 호출을 멈춘다. 가장 깔끔.
                   다만 일부 백엔드에서는 latency가 있다.

atomic flag      : 콜백은 계속 호출되지만 0을 출력. 즉시 응답.
                   meter / DSP 시간 진행 등을 콜백이 계속 보고 싶을 때.
```

대부분 처음 단계에서는 `Stream::pause()` 만으로 충분하다.

## Rebuilding의 함정

Stream이 다시 만들어지는 동안에는 콜백이 없다. 그동안 들어온 명령은 어디로 가나?

선택 두 가지.

```text
1. cmd 큐를 드롭하고, 빌드 완료 후 마지막 상태만 적용
2. cmd 큐를 보존하고, 빌드 완료 후 큐의 명령들을 순서대로 적용
```

대부분의 경우 1이 더 안전하다. 사용자가 빌드 중 5번 "play"를 눌러도 결과는 한 번이면 된다.

## 자주 하는 실수

- 상태를 enum이 아니라 여러 bool로 표현 (`is_playing`, `is_paused`, ...) → 불가능한 조합이 생김.
- Rebuilding 중에 사용자 명령을 그대로 처리 → 빌드 끝나기 전에 stream이 없으니 panic.
- 잘못된 전이를 panic으로 처리 → 사용자 동작 한 번에 앱이 죽음. Result로 반환한다.
- 콜백 안에서 transport 상태를 mutex로 읽음 → 실시간 위반. atomic flag 1개로 좁혀야 한다.
- NoDevice 상태에 처리하지 않고 다른 명령을 그냥 받음 → "왜 안 들리지" 디버깅 끝없음.

## 반드시 이해해야 할 것

- transport는 5개 상태(Stopped / Playing / Paused / Rebuilding / NoDevice)로 충분하다.
- 모든 전이는 한 `match` 함수로 모은다. 표 형태로 그려서 코드와 1:1 대응시킨다.
- 콜백 안에서는 transport를 보지 않고 atomic 한 비트만 본다 (또는 OS pause 사용).
- Rebuilding 중에는 사용자 명령을 어떻게 처리할지 정책을 미리 정한다.
- 잘못된 전이는 에러다. panic이 아니다.
