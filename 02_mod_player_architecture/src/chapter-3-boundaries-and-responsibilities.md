# Chapter 3 - 책임 경계와 계층 분리

이 장은 RuStudio 안에서 `mod_player` 위/아래에 어떤 모듈이 있는지, 어떤 일을 누가 하는지를 결정한다.

이 결정이 흐려지면 EQ 코드가 cpal API를 직접 부르기 시작하고, UI 코드가 콜백 안에 들어가는 사고가 난다.

## 4개 계층 그림

```text
┌──────────────────── UI ───────────────────────┐
│  사용자 인터랙션, 시각화, 메터, 슬라이더       │
│  외부 입력 → mod_player에 명령으로 변환        │
└──────────────────┬────────────────────────────┘
                   │ commands / state
┌──────────────────▼────────────────────────────┐
│              mod_player                       │
│  Stream 소유 / transport / device lifecycle   │
│  source registry / DSP chain wiring           │
└──────────────────┬────────────────────────────┘
                   │ owns + drives
┌──────────────────▼────────────────────────────┐
│       dsp-core / mod_mastering / mod_eq       │
│  buffer 단위 처리 (gain / EQ / dynamics)       │
│  realtime-safe 인터페이스                       │
└──────────────────┬────────────────────────────┘
                   │ buffer
┌──────────────────▼────────────────────────────┐
│                   cpal                        │
│  Host / Device / Stream / 콜백                │
│  하드웨어 ↔ 샘플 버퍼 경계                      │
└───────────────────────────────────────────────┘
```

각 계층의 일은 분명히 다르다.

```text
UI            : "사람의 의도"를 다룬다
mod_player    : "재생 시스템의 의도"를 다룬다
dsp 모듈군    : "샘플의 계산"을 다룬다
cpal          : "하드웨어와의 약속"을 다룬다
```

## 책임을 누가 갖는가 — 구체적 사례

추상적인 그림은 위로 충분하다. 실제 결정으로 내려가 보자.

### "사용자가 play를 눌렀다"

```text
UI            : 버튼 클릭 → mod_player::play() 호출
mod_player    : 현재 transport 상태 확인 → Playing으로 전이 → Stream.play()
dsp 모듈      : 변동 없음 (이미 만들어진 chain 그대로 사용)
cpal          : Stream이 콜백을 시작
```

### "사용자가 EQ band 1의 freq를 1000 Hz로 바꿨다"

```text
UI            : 슬라이더 onChange → mod_player에 "EQ.band1.freq = 1000" 명령
mod_player    : 명령을 적절한 채널/atomic으로 audio thread에 전달
dsp 모듈      : (콜백 안에서) 새 freq를 받아 cookbook 계수 계산 → biquad에 적용
cpal          : 상관 없음 (콜백은 그대로 호출됨)
```

여기서 핵심은 UI가 **biquad를 직접 만지지 않는다**는 점이다. UI는 mod_player에게 의도만 던진다.

### "출력 장치가 USB가 빠지면서 사라졌다"

```text
cpal          : 에러 콜백 호출 (장치 분리 통보)
mod_player    : 그 신호를 받아 transport를 NoDevice로 전이 → Stream drop
dsp 모듈      : 변동 없음 (다음 build 때 다시 사용됨)
UI            : mod_player의 새 상태(NoDevice)를 polling/구독으로 보고 사용자에게 알림
```

### "곡이 끝나서 다음 곡으로 넘어가려면 source를 바꿔야 한다"

```text
UI            : "다음 곡" 명령 → mod_player::set_source(...)
mod_player    : 새 source를 채널/atomic으로 전달, 기존 source는 적절한 시점에 drop
dsp 모듈      : 변동 없음
cpal          : 변동 없음
```

source 교체는 콜백 *밖*에서 준비되고, 콜백 *안*에서 한 샘플 시점에 swap된다.

## 자주 깨지는 경계

학습 중에 가장 자주 무너지는 경계 4가지를 미리 짚는다.

### 1. UI가 cpal을 직접 부른다

```rust
// 안 됨
let stream = device.build_output_stream(/* ... */)?;
ui_state.stream = Some(stream);
```

이러면 사용자 화면이 닫히면 Stream이 drop되어 소리도 끝난다. Stream 소유는 mod_player에 둔다.

### 2. UI가 DSP 인스턴스를 직접 만진다

```rust
// 안 됨
ui_button.on_click(|| eq.bands[0].set_freq(1000.0));
```

`eq`가 audio thread에서 매 샘플 process()를 부르는 동안 UI thread에서 freq를 바꾸면 race가 난다. 가운데 mod_player가 atomic/channel로 한 번 받아야 한다.

### 3. DSP가 cpal을 직접 만진다

```rust
// 안 됨
impl Compressor {
    fn process(&mut self, ...) {
        let device = cpal::default_host().default_output_device();
        // ...
    }
}
```

DSP는 buffer만 본다. 장치/스트림은 모른다.

### 4. mod_player가 DSP 알고리즘을 구현한다

```rust
// 안 됨
impl ModPlayer {
    fn callback(&mut self, data: &mut [f32]) {
        for s in data.iter_mut() {
            *s = (self.phase * TAU).sin();   // 여기서 oscillator 만들지 말 것
        }
    }
}
```

mod_player는 source / chain을 **호출**한다. 직접 sine을 그리지 않는다.

## "어디에 둘지 모르겠을 때" 판단표

```text
"이 일은 사람의 의도와 관계 있나?"          → YES면 UI
"하드웨어/Stream 생애와 관계 있나?"          → YES면 mod_player
"버퍼 안 샘플 계산이 맞나?"                  → YES면 dsp 모듈
"OS/플랫폼 오디오 API와 직접 관계 있나?"     → YES면 cpal
```

대부분 결정이 이 4개로 풀린다.

## RuStudio 디렉토리 의미

이 분리는 단순한 추상이 아니라 워크스페이스 crate 분할로도 표현된다.

```text
crates/
  cpal-glue/      ← cpal 위의 얇은 helper (선택)
  mod_player/     ← 이 책의 주제
  dsp-core/       ← AudioNode trait, buffer 정의 (08 책)
  mod_mastering/  ← 04 책 영역
  mod_eq/         ← 05 책 영역
  ui/             ← 11 책 영역
```

상위 crate는 하위 crate에 의존할 수 있지만, 그 반대는 안 된다. mod_player는 dsp-core를 알지만, dsp-core는 mod_player를 모른다.

## 자주 하는 실수

- "지금은 작으니까 한 파일에 다 넣자" → 한 파일 안이라도 함수 단위로 책임을 분리하지 않으면 동일 문제.
- mod_player에 EQ 코드 넣음 → 다음에 multi-band가 생기면 mod_player가 폭발.
- UI에 cpal 코드 넣음 → 다음에 다른 UI 프레임워크로 바꿀 때 cpal까지 같이 옮겨야 함.
- "이번만" 콜백 안에서 lock → 한 번 들어가면 사라지지 않는다.

## 반드시 이해해야 할 것

- 4개 계층(UI / mod_player / DSP / cpal)이 책임 경계다. 위/아래로만 의존한다.
- 의심스러우면 "이 일은 사람의 의도냐 / 스트림 생애냐 / 샘플 계산이냐 / 하드웨어 약속이냐"로 분류한다.
- 경계를 깨면 가장 먼저 무너지는 것이 콜백 안 race다.
- 디렉토리 분할(crate 분할)이 의존 방향을 강제하는 가장 좋은 도구다.
