# Chapter 8 - Realtime-safe DSP Chain 구조

이 장은 mod_player가 콜백 안에 끼워 넣을 **DSP chain**의 구조를 본다. 알고리즘 자체(EQ, comp, limiter)는 04~05 책의 영역이고, 여기서는 **wiring**만 다룬다.

## 한 그림

```text
콜백 안 (한 frame 단위로 반복):

[ source ]
    │
    ▼
[ DspChain (Vec<Box<dyn AudioModule>>) ]
    │
    ▼
[ master gain ]
    │
    ▼
output buffer (cpal data)
```

이 모양이 mod_player의 콜백 closure 안의 본체다.

## AudioModule trait

이 trait의 정확한 모양은 08 책(audio graph)의 영역이지만, mod_player가 보는 최소 trait은 이렇다.

```rust
pub trait AudioModule: Send {
    fn process(&mut self, buf: &mut [f32], channels: usize);
    fn reset(&mut self) {}
}
```

`Send`인 이유는 audio thread로 옮길 수 있어야 하기 때문이다. `Sync`는 굳이 요구하지 않는다 (audio thread 단독 사용).

## Chain은 Vec이다

```rust
pub struct DspChain {
    modules: Vec<Box<dyn AudioModule>>,
}

impl DspChain {
    pub fn process(&mut self, buf: &mut [f32], channels: usize) {
        for m in &mut self.modules {
            m.process(buf, channels);
        }
    }
}
```

이 Vec 자체는 콜백 *밖*에서 만들어진다. 콜백 안에서는 `process()`만 부른다.

## Chain 교체 — 가장 까다로운 부분

사용자가 EQ를 추가하거나, 모듈 순서를 바꾸거나, 하나를 제거하면 chain이 통째로 바뀐다. 이걸 audio thread가 보고 있는 동안 어떻게 바꿀까?

세 가지 옵션.

### 옵션 A: ArcSwap

```rust
let chain = Arc::new(ArcSwap::from_pointee(DspChain::new()));

// 콜백:
let snap = chain.load();
snap.process(buf, channels);   // 내부에서 lock-free pointer

// 외부:
chain.store(Arc::new(new_chain));
```

장점: 가장 단순.
단점: `process`가 `&mut self`인데 `ArcSwap`은 `&`만 줌 → 내부 가변성(`Mutex`/`RefCell`)이 필요해서 결국 lock 또는 cell 비용이 생긴다. 또는 chain을 `&mut`이 아닌 형태로 다시 설계해야 한다 (내부에 가변성을 모듈마다 둠).

### 옵션 B: SPSC 명령 + double-buffered chain

```rust
enum ChainCmd {
    Replace(Box<DspChain>),
    InsertAt { idx: usize, module: Box<dyn AudioModule> },
    Remove(usize),
}

// 콜백 안:
while let Ok(cmd) = chain_cmd_rx.pop() {
    apply_to(&mut self.chain, cmd);
}
self.chain.process(buf, channels);
```

옛 chain을 audio thread가 drop하지 않도록, swap된 옛 chain은 별도 회수 채널로 다시 외부 thread에 보낸다.

장점: chain이 audio thread 단독 소유라서 `&mut`이 자연스러움.
단점: 회수 채널이 추가됨.

이 옵션이 **mod_player 학습 단계의 권장 디자인**이다.

### 옵션 C: lock-free RCU 라이브러리

`crossbeam-epoch` 같은 라이브러리로 직접 RCU. 학습 단계에선 과한 복잡도.

## Source의 자리

source(예: oscillator, sample player)도 같은 chain 모델 안에 넣을 수 있다.

```rust
pub trait AudioSource: Send {
    fn render(&mut self, buf: &mut [f32], channels: usize);
}

pub struct ChainContainer {
    source: Box<dyn AudioSource>,
    chain: DspChain,
}

impl ChainContainer {
    fn process(&mut self, buf: &mut [f32], channels: usize) {
        self.source.render(buf, channels);     // 버퍼 채우기
        self.chain.process(buf, channels);     // 처리
    }
}
```

source 교체도 chain 교체와 같은 SPSC 명령 패턴으로.

## Master gain은 chain의 일부인가?

답: **chain 마지막 단의 모듈**로 두는 게 깔끔하다.

```rust
self.chain.modules.push(Box::new(MasterGain::new()));
```

이러면 master gain의 변경도 다른 모듈과 동일한 명령 채널을 쓴다. mod_player 입장에서는 특별한 케이스가 줄어든다.

다만 UI에 항상 보이는 마스터라서, 명령 채널과 무관하게 atomic 한 값으로 노출하는 것도 흔한 패턴이다 (둘 다 가능).

## Bypass와 모듈 enable/disable

각 모듈에 `bypass` 플래그를 두면 atomic 한 비트로 제어할 수 있다.

```rust
pub struct EqBand {
    biquad: Biquad,
    bypass: Arc<AtomicBool>,
}

impl AudioModule for EqBand {
    fn process(&mut self, buf: &mut [f32], channels: usize) {
        if self.bypass.load(Ordering::Relaxed) { return; }
        // 정상 처리
    }
}
```

장점: 사용자가 모듈 on/off를 빠르게 토글해도 chain 재구성 없이 처리.
주의: bypass 토글 시 짧은 cross-fade를 두지 않으면 클릭이 날 수 있음 (보통 5~30 ms cross-fade).

## Buffer 단위 처리

각 모듈이 **buffer 단위**로 처리한다 (한 콜백에 들어온 N 프레임을 한 번에).

```rust
fn process(&mut self, buf: &mut [f32], channels: usize) {
    for frame in buf.chunks_mut(channels) {
        // 한 frame 처리
    }
}
```

또는 sample 단위로 처리해도 되지만, buffer 단위가 일반적이다 (chain의 각 단이 입력 buffer를 받아 출력 buffer로 변환).

in-place vs out-of-place는 또 다른 결정. 가장 단순한 모델은 **모든 모듈이 in-place**로 같은 buffer를 변형한다 (위 코드처럼).

## 자주 하는 실수

- 콜백 안에서 `self.chain.modules.push(...)` → Vec 재할당. 실시간 위반.
- chain 교체 시 옛 chain을 콜백에서 drop → drop 비용을 audio thread가 떠안음. 회수 채널.
- bypass를 토글하는 순간 즉시 신호가 끊김 → 클릭. cross-fade 필요.
- chain 모듈마다 별도 atomic 통신 → 통제 흐름이 분산. 가능한 한 cmd 채널 하나로 묶는다.
- chain이 source를 모름 → "source가 끝났다" 정보를 chain이 받지 못함. AudioSource trait에 done 플래그 두기.

## 반드시 이해해야 할 것

- chain은 `Vec<Box<dyn AudioModule>>`이고, audio thread가 단독 소유한다.
- chain 교체는 SPSC 명령 + 회수 채널 패턴이 학습 단계에 가장 단순하다.
- source도 같은 모델에 들어간다 — `AudioSource` 후 `DspChain`.
- master gain은 chain의 마지막 단 모듈로 두는 게 깔끔하다.
- 콜백 안에서는 절대 모듈을 `push`/`remove` 하지 않는다. 명령 채널로만.
