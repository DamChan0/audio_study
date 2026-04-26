# Chapter 11 - 예제 가이드: Tiny mod_player 만들기

이 장은 **너가 직접 만들면서 익히는 단계**다.

이 책의 chapter 4 ~ 8에서 본 개념들을 한 번씩 코드로 옮긴다. 단, 한 번에 다 만들지 않는다. 5개 작은 binary로 쪼개서 **각 예제가 한 가지 개념만 만지게** 한다.

## 목표 — Tiny mod_player

```text
01의 SinSound를 source로 받아서
mod_player가 그 stream을 들고
사용자 명령에 따라 transport 상태를 바꾸고
필요하면 config을 협상해서 다시 빌드하고
끝에 DSP chain slot까지 끼운다
```

5개 예제가 끝나면 위 한 줄이 그대로 동작한다. 매 예제는 **이전 예제의 Player 모양에서 한 가지를 더한다**.

## 01의 코드를 그대로 import해서 쓴다

이 책의 핵심 원칙. 우리는 새 source/effect를 짜지 않는다. `01_cpal/examples`에 이미 다음이 있다.

```text
cpal_examples (= 01_cpal/examples)
  ├── AudioProcess trait      ← lib.rs
  ├── sources::sin_sound::SinSound
  ├── effects::volume::Gain
  └── chain::Chain            ← Vec<Box<dyn AudioProcess>>
```

02 예제는 위 4가지를 import해서 쓴다. 다시 짜지 않는다.

---

## 디렉토리 / Cargo.toml 셋업

### 디렉토리 구조

```text
study/
  01_cpal/
    examples/
      Cargo.toml              ← package = "examples", edition = "2024"
      src/
        lib.rs
        sources.rs / sources/sin_sound.rs
        effects.rs / effects/volume.rs
        chain.rs
        bin/
          silence.rs
          sin_sound.rs
          dsp_chain.rs
  02_mod_player_architecture/
    examples/
      Cargo.toml              ← 이 장에서 갱신할 파일
      src/
        bin/                  ← 5개 binary가 들어갈 곳
          01_player_owns_stream.rs
          02_transport_state.rs
          03_control_channel.rs
          04_config_negotiate.rs
          05_dsp_chain_slot.rs
```

`02/examples`에는 `lib.rs`가 필요 없다. 모든 공용 타입은 `cpal_examples`에서 가져온다. binary끼리 공유할 헬퍼가 생기면 그 때 `lib.rs`를 추가하면 된다.

### `02/examples/Cargo.toml` 갱신

01의 crate를 path 의존성으로 추가한다. 두 crate 모두 `package = "examples"`라서 충돌을 막기 위해 `package` 별칭을 쓴다.

```toml
[package]
name = "mod_player_examples"
version = "0.1.0"
edition = "2024"

[dependencies]
cpal = "0.15"
anyhow = "1"
crossbeam-channel = "0.5"

# 01의 examples crate를 cpal_examples 라는 이름으로 import
cpal_examples = { path = "../../01_cpal/examples", package = "examples" }
```

### 01의 코드를 import하는 모양

```rust
use cpal_examples::AudioProcess;
use cpal_examples::sources::sin_sound::SinSound;
use cpal_examples::effects::volume::Gain;
use cpal_examples::chain::Chain;
```

이 4줄이 거의 모든 예제에서 등장한다.

### 한 binary 실행

```bash
cd 02_mod_player_architecture/examples
cargo run --bin 01_player_owns_stream
```

---

## 5개 예제의 누적 그림

각 예제가 무엇을 더 다루는지 한 표로 본다.

```text
        Player가 하는 일                              새로 등장하는 도구
─────   ────────────────────────────────────────     ────────────────────────────
01      Stream 한 개 소유 + drop                       Option<Stream>
02      + transport 상태 (Stopped/Playing/Paused)     AtomicBool / TransportState enum
03      + 명령 채널 (UI → audio thread)                crossbeam-channel SPSC
04      + config 협상 (SR/포맷/채널/buffer fallback)    NegotiatedConfig 추상
05      + DSP chain slot (master gain + bypass)       Chain (01) + 명령으로 gain 변경
```

이 5개를 끝내면 chapter 4 ~ 8의 핵심 5가지가 손에 들어온다.

---

## 예제 01 — Stream 소유

### 다루는 챕터

`Chapter 4 - Stream 소유권`.

### 만드는 것

`ModPlayer`라는 struct 하나. 필드는 `stream: Option<cpal::Stream>`과 `config: cpal::StreamConfig` 정도. 메서드는 `new`, `play`, `stop` 세 개.

`main()`에서 `Player`를 만들고 3초 재생한 뒤 `Player`를 drop한다.

### 01에서 가져올 것

- `cpal_examples::AudioProcess`
- `cpal_examples::sources::sin_sound::SinSound`

`SinSound`를 콜백 closure가 캡처해서 `process()`로 buffer를 채우게 한다.

### 사용자 흐름

```text
$ cargo run --bin 01_player_owns_stream
[player] built. playing 440 Hz sine for 3 seconds.
... (3초간 사인파)
[player] stopped, dropping stream.
```

키 입력은 받지 않는다. 단순히 "Player가 살아 있는 동안 소리가 난다"를 확인.

### 검증

- 3초 동안 사인파가 들리는가
- 3초 후 정확히 멈추는가 (`stream` field가 drop되어야 함)
- 함수 안에서 `let stream = ...;`만 하고 끝나면 안 들리는 것을 직접 비교해 본다 (잘못된 코드를 한 번 만들어 보고 정상 코드와 차이 인지)

### 흔한 함정 (먼저 피하기 위해 의식할 것)

- `play()` 메서드 안에서 `stream`을 만들고 그 함수 안에서 drop되게 두면 즉시 멈춘다 → `stream`을 self의 필드로 옮긴다.
- `stream`을 `Mutex`로 감싸지 않는다. Stream은 audio thread가 안 본다.
- 같은 Player가 두 번 build를 호출하면 OS가 거부할 수 있다. `play()`에서 `self.stream = None;` 로 옛것을 명시 drop 후 새로 만든다.

### 다음 예제로 넘어갈 때 더할 것

- transport 상태 (Stopped / Playing / Paused) — 02번이 시작점.

---

## 예제 02 — Transport 상태

### 다루는 챕터

`Chapter 5 - Transport 상태 모델`.

### 만드는 것

01의 Player에 두 가지를 더한다.

```text
1. TransportState enum (Stopped, Playing, Paused 정도면 충분; NoDevice/Rebuilding은 04 / 별도)
2. Arc<AtomicBool> active 플래그
   - 콜백 closure가 캡처
   - true면 SinSound가 buffer 채움, false면 buffer를 0.0으로 채움
```

stdin에서 한 줄씩 키 명령을 받는다.

```text
p   play
a   pause
s   stop
q   quit
```

`p`/`a`/`s`는 `TransportState`를 바꾸고 `active` 플래그를 갱신한다. `q`는 main loop를 끝내 Player가 drop된다.

### 사용자 흐름

```text
$ cargo run --bin 02_transport_state
[player] state=Stopped. p=play, a=pause, s=stop, q=quit
> p
[player] state=Playing
... (사인파)
> a
[player] state=Paused
... (무음, 콜백은 계속 호출됨)
> p
[player] state=Playing
... (사인파)
> s
[player] state=Stopped
> q
[player] dropping stream.
```

키 + Enter 형태가 가장 단순하다. raw mode는 안 쓴다.

### 검증

- `p` → 소리 시작, `a` → 즉시 무음, `p` 다시 → 즉시 재개
- `a`/`s` 모두 무음이지만 의미 차이를 코드로 표현했는가 (`s`는 위치 reset 등의 hook 자리)
- 잘못된 전이 (`Stopped` 상태에서 `pause`)를 시도하면 panic이 아니라 단순 무시 또는 에러 반환

### 흔한 함정

- `active`를 `Mutex<bool>`로 두면 콜백에서 lock → 실시간 위반. 반드시 `AtomicBool`.
- 상태를 `is_playing` / `is_paused` 같은 여러 bool로 표현 → 불가능한 조합이 생긴다. enum 한 개로.
- 잘못된 전이를 `panic!`으로 처리 → 사용자 키 한 번에 앱이 죽음. `Result` 또는 단순 무시.

### 다음 예제로 넘어갈 때 더할 것

- atomic 플래그 한 개로는 명령이 늘어날 수록 부족하다. 03에서 명령 채널로 일반화.

---

## 예제 03 — 제어 채널

### 다루는 챕터

`Chapter 6 - UI ↔ Audio Thread 제어 경로`.

### 만드는 것

02의 `AtomicBool`을 **`crossbeam-channel`의 SPSC 채널**로 대체한다.

```text
enum AudioCommand {
    Play,
    Pause,
    Stop,
    SetGain(f32),     // 새로 등장: master gain (linear)
}
```

`Player`는 producer 측 (`Sender<AudioCommand>`), 콜백 closure는 consumer 측 (`Receiver<AudioCommand>`)을 캡처. 매 콜백 시작에서 `try_recv` 루프로 큐를 비우고 내부 상태를 갱신한다.

추가 키 명령.

```text
g   master gain -3 dB
G   master gain +3 dB
```

(`g`/`G` 하나는 +/- 한 단계라 약간 직관적이지 않을 수 있다. `+`/`-`로 바꿔도 됨.)

### 검증

- `p`/`a`/`s`가 02와 동일하게 작동
- `g`/`G`로 gain 변경 시 즉시 amplitude가 변함
- 채널 capacity를 작게 (예: 4) 두고 빠르게 키를 누르면 push 실패가 발생할 수 있음 — 그때 어떻게 처리하는지 코드로 표현 (보통 `log::warn!` 또는 단순 drop)

### 흔한 함정

- `mpsc::channel()` 표준 라이브러리 채널은 unbounded — audio thread에 push할 때 막히진 않지만 명령이 무한 누적될 수 있다. `crossbeam-channel::bounded(N)`이 정석.
- `String`이나 `Box<...>` 가 들어간 명령을 콜백에 보냄 → 콜백이 drop 비용을 떠안음. `AudioCommand`는 `Copy` 가능한 작은 enum으로.
- 콜백 안에서 `for cmd in rx.iter()` 처럼 blocking iterator를 쓰면 큐가 빌 때까지 콜백이 안 끝남. `try_recv` 루프.

### 다음 예제로 넘어갈 때 더할 것

- 지금은 `default_output_config()`로 한 번에 빌드. 04에서 우선순위 fallback을 더한다.

---

## 예제 04 — Config 협상

### 다루는 챕터

`Chapter 7 - Config 협상과 Fallback`.

### 만드는 것

03의 Player에 **빌드 전 단계**로 `negotiate()` 함수를 끼운다.

```text
fn negotiate(device: &cpal::Device) -> anyhow::Result<NegotiatedConfig>
```

내부 동작.

```text
1. device.supported_output_configs() 가져오기
2. PREFERRED_SAMPLE_RATES = [48_000, 44_100, 96_000, 88_200, 24_000]
3. PREFERRED_SAMPLE_FORMATS = [F32, I16, U16]
4. PREFERRED_CHANNELS = [2, 1]
5. PREFERRED_BUFFER_SIZES = [Fixed(256), Fixed(128), Default]
6. 우선순위 위에서부터 시도, 일치하는 첫 조합을 선택
7. 모두 실패하면 default_output_config()으로 fallback
```

`NegotiatedConfig`는 `cpal::StreamConfig`로 변환하기 전 단계의 우리 추상.

```rust
struct NegotiatedConfig {
    sample_rate: u32,
    channels: u16,
    sample_format: cpal::SampleFormat,
    buffer_size: cpal::BufferSize,
}
```

선택된 config를 stdout에 인쇄해서 어떤 fallback이 일어났는지 보여 준다.

```text
[negotiate] supported configs: 12
[negotiate] tried 48000 Hz / F32 / 2ch / Fixed(256) → OK
[player] using sample_rate=48000, channels=2, format=F32, buffer=Fixed(256)
```

### 검증

- 선호 config가 그대로 들어오는 일반 환경에서 우선순위 1번이 선택됨
- USB 장치 같은 데서는 다른 SR이 선택될 수 있음
- 일부러 우선순위 리스트의 1번을 장치가 지원하지 않는 값(예: 192_000)으로 바꿔 보면 2번으로 fallback되는지 확인
- 협상 결과로 `SinSound::new(sample_rate, channels, ...)`에 들어가는 인자가 일관되게 갱신됨

### 흔한 함정

- `default_output_config()`만 쓰고 fallback을 안 두면 USB 장치 변경 직후 빌드 실패 가능.
- `BufferSize::Fixed(N)`을 우선순위에 안 두면 latency가 들쭉날쭉 (학습용으론 큰 차이 없지만, 실제 RuStudio에서는 의미 있음).
- DSP 인스턴스(`SinSound`)를 협상 *전*에 만들면 sample rate가 어긋남. 협상 → 그 결과로 `SinSound::new()`.

### 다음 예제로 넘어갈 때 더할 것

- 지금은 source 한 개가 buffer를 직접 채움. 05에서 그 사이에 chain slot을 끼운다.

---

## 예제 05 — DSP Chain Slot

### 다루는 챕터

`Chapter 8 - Realtime-safe DSP Chain 구조`.

### 만드는 것

04의 player에 **DSP chain**을 끼운다. chain은 `cpal_examples::chain::Chain` (01 코드)을 그대로 쓴다.

```text
콜백 안 한 frame:

[ SinSound ]   ← buffer 채움 (Source 역할)
   │
   ▼
[ Chain (= Vec<Box<dyn AudioProcess>>) ]
   │  ├── Gain (master gain, 01의 Gain)
   │  └── (확장 자리: 04 책 limiter, 05 책 EQ 등이 여기 들어갈 것)
   ▼
output buffer
```

이번에는 **chain의 master gain을 03의 명령 채널로 변경**한다. `AudioCommand::SetGain`이 chain 안의 `Gain`의 값을 갱신.

```text
주의: 01의 Gain 구현이 gain 필드를 들고 있으므로,
      그 필드를 외부에서 갱신할 수 있게 하거나,
      chain 안에서 atomic으로 공유하는 wrapper를 만든다.
      (둘 중 어느 방식을 쓸지가 너의 설계 결정이다)
```

추가 키 명령.

```text
b   bypass on/off (chain 전체)
```

bypass는 `AtomicBool`로 두고, 콜백에서 `if bypass { source 출력 그대로 } else { chain.run(...) }`.

### 검증

- 04와 동일한 사용자 흐름이 작동
- `g`/`G`로 master gain이 즉시 변함
- `b` 한 번 → chain 통과 안 함 (Gain이 0.5로 줄어 있어도 원본 amplitude로 들림). 다시 한 번 → 정상
- chain의 모듈을 늘려도 (예: 두 번째 Gain) 콜백 안에서 같은 `chain.run()` 한 줄로 처리됨을 확인
- 실행 중 `SetGain(0.0)`을 보내고 `SetGain(0.5)`을 빠르게 연속 보내면 click이 나는지 관찰 (smoothing이 없으면 짧게 click). 이게 05 책의 parameter smoothing이 필요한 이유의 시각/청각적 확인.

### 흔한 함정

- 콜백 안에서 `chain.modules.push(...)` 같은 호출 → Vec 재할당. 절대 안 됨. 모듈 추가/제거는 콜백 밖에서 미리 만들어 둔 chain을 swap하는 식 (이번 예제 범위는 swap까진 안 가도 됨).
- `Chain`이 `Sync`가 아니라서 `Arc<Chain>`을 두 thread에서 동시에 만지지 못함. chain은 **콜백 closure 안에서 단독 소유** + 외부와는 명령 채널로만 통신.
- bypass를 토글하는 순간 click이 남 → 정상이다. 이걸 어떻게 부드럽게 할지가 다음 책들의 주제 (cross-fade).
- 04의 `SetGain` 명령이 03 단계에서는 source의 amplitude를 직접 곱했을 수 있다. 05 단계에서는 그 곱셈이 chain 안의 Gain 모듈로 옮겨가야 한다. 두 곳에서 동시에 곱하면 결과가 두 배.

### 다음 단계 (다음 책으로)

이 chain slot이 있으면 다음 책들이 다음과 같이 자연스럽게 끼워진다.

```text
03_dsp_fundamentals : envelope, delay 같은 새 AudioProcess를 chain에 push
04_mod_mastering    : Compressor/Limiter를 chain의 master gain 앞에 push
05_eq_biquad        : EQ band 4개를 chain 앞쪽에 push
06_fft_and_spectrum : analyzer thread를 별도로 띄우고 Player가 ring buffer 다리만 제공
```

---

## 작업 순서 권장

```text
1. examples/Cargo.toml 갱신 (위의 모양으로)
2. examples/src/bin/01_player_owns_stream.rs 부터 차례로 작성
3. 각 binary를 끝낼 때마다 cargo run으로 동작 확인
4. 다음 binary는 직전 binary 코드를 복사해 시작점으로 두고, 한 가지만 더한다
   (모든 binary가 self-contained이므로 진화하는 단일 파일이 아님)
```

같은 코드가 binary 사이에 약간 중복되는 건 의도다. 각 단계의 차이를 한 파일에서 명확히 보기 위해서.

## 검증 체크리스트

```text
□ 01에서 cpal_examples::SinSound 가 정확히 import되고 사인파가 들린다
□ 02에서 transport 상태 전이가 잘못된 조합에 panic하지 않는다
□ 03에서 명령 채널이 bounded이고 가득 찰 때 처리가 있다
□ 04에서 negotiation 결과가 stdout에 정확히 표시된다
□ 05에서 chain.run()이 콜백 안에서 한 줄로 처리된다
□ 모든 binary가 콜백 안에서 lock / 할당 / println을 부르지 않는다
```

마지막 항목이 가장 중요하다. 의심되면 콜백 closure 안의 모든 호출을 위 책 chapter 6의 표와 대조한다.

## 한 줄 요약

> 01의 `SinSound` / `Chain` / `Gain`을 import해서, mod_player가 chapter 4 ~ 8의 5가지 책임 (소유 / 전이 / 제어 / 협상 / chain slot)을 한 단계씩 갖춰 가는 5개 binary를 만든다. 콜백 안 실시간 규칙 위반은 단 한 곳도 없어야 한다.
