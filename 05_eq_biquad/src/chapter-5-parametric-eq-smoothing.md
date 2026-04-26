# Chapter 5 - Parametric EQ와 Coefficient Smoothing

이 장은 EQ를 사용자가 실시간으로 만질 때 생기는 두 문제를 다룬다.

```text
1. parametric EQ를 어떻게 한 채널에 N개 직렬로 묶을 것인가
2. freq/gain/Q를 빠르게 돌리면 왜 클릭/지직 소리가 나는가, 어떻게 막는가
```

## 1. Parametric EQ란

parametric EQ는 사용자가 다음 셋을 직접 조절할 수 있는 EQ다.

```text
freq : 어느 주파수를 건드릴 것인가
gain : 얼마나 부스트/컷할 것인가 (dB)
Q    : 얼마나 좁게/넓게 건드릴 것인가
```

대부분의 DAW EQ는 4 ~ 8 밴드의 parametric을 직렬로 둔다.

```text
in → [Band1: peaking] → [Band2: peaking] → [Band3: peaking] → ... → out
       freq=80, +3dB    freq=600, -2dB    freq=4k, +5dB
```

각 밴드는 사용자가 모양도 바꿀 수 있다. 예를 들어 1번 밴드를 low-shelf로, 마지막 밴드를 high-shelf로 두는 식.

## 2. 한 밴드 = 한 Biquad

각 밴드는 Biquad 인스턴스 1개다.

```rust
struct EqBand {
    biquad: Biquad,
    f0: f32,
    gain_db: f32,
    q: f32,
    kind: BandKind,           // Peaking / LowShelf / HighShelf / LPF / HPF / ...
}

enum BandKind { Peaking, LowShelf, HighShelf, Lpf, Hpf, Bpf, Notch }
```

스테레오에선 채널마다 따로 인스턴스가 있어야 한다는 점은 03 장의 규칙 그대로다.

## 3. EQ chain 만들기

```rust
struct EqChain {
    bands: Vec<EqBand>,       // 한 채널에 대한 직렬 사슬
}

impl EqChain {
    fn process(&mut self, x: f32) -> f32 {
        let mut y = x;
        for band in &mut self.bands {
            y = band.biquad.process(y);
        }
        y
    }
}
```

이게 4-band parametric EQ의 본체다. 직렬은 단순히 한 단의 출력을 다음 단의 입력으로 넘긴다.

## 4. 클릭의 정체

이제 본 장의 핵심 문제다.

사용자가 freq 손잡이를 빠르게 1000 Hz → 1100 Hz로 돌리면, 콜백 안에서 한 샘플 사이에 모든 계수가 갑자기 바뀐다.

```text
샘플 n   : 계수 A로 process()
샘플 n+1 : 계수 B로 process()
```

biquad는 IIR이라 z1/z2가 *이전* 계수로 만들어진 값을 들고 있다. 그 상태에 갑자기 다른 계수가 적용되면 출력이 점프한다 — 그게 사람의 귀에는 짧은 "딸깍" 소리로 들린다.

```text
amplitude
   ▲
   │      /---\        /---\
   │     /     \  ╱│  /     \      ← 계수 변경 시점에 amplitude 점프
   │    /       \╱ │ /       \
   │   /         X─/          \
   │  /                        \
   └─────────────────────────────► time
                  ↑
              계수 변경
```

해결책 두 갈래.

```text
A. 파라미터(freq/gain/Q)를 천천히 움직여서 계수를 매 샘플 조금씩 갱신
B. 두 biquad를 두고 cross-fade
```

A가 단순하고 효과 좋아서 표준이다.

## 5. Parameter Smoothing — 1차 IIR 한 줄

03 책의 envelope follower와 같은 1차 IIR을 쓴다.

```rust
struct Smoothed {
    target: f32,
    current: f32,
    coeff: f32,        // 0.0 ~ 1.0, 1에 가까우면 천천히
}

impl Smoothed {
    fn next(&mut self) -> f32 {
        self.current = self.coeff * self.current + (1.0 - self.coeff) * self.target;
        self.current
    }
}
```

매 샘플 `current`가 `target`을 향해 점진적으로 다가간다. 사용자가 손잡이를 돌려 `target`을 갑자기 1000 → 1100으로 바꿔도, `current`는 ms 단위로 부드럽게 따라간다.

```text
target  : ─────────────┐
                       │
                       └────────────  (1000 → 1100 step)

current : ─────╱──────╱───────────
              ╱  부드럽게 따라감
             ╱
            ╱
```

## 6. 전체 흐름

```text
콜백 밖 (UI/control thread):
  사용자가 손잡이를 돌림
  ↓
  freq.target = 1100;   gain.target = +5;   q.target = 0.8;
  (atomic 또는 SPSC 채널로 전달)

콜백 안 (audio thread, 매 샘플):
  let f0   = freq.next();
  let g    = gain.next();
  let q    = qq.next();
  let coef = peaking(f0, fs, g, q);   ← 계수 다시 만들기
  band.biquad.set_coeffs(coef);
  let y    = band.biquad.process(x);
```

여기서 한 가지 큰 비용 문제가 있다. **매 샘플 `peaking()`을 부르면 cos/sin/pow 호출이 많다.**

대처법.

```text
1. 매 샘플 호출 + JIT-friendly 코드: 모던 CPU에서 한 채널 4밴드면 보통 OK
2. 매 N 샘플(예: 16) 마다 갱신 + N샘플 사이는 그대로 사용
3. 두 계수 세트 사이를 선형 보간 (= 계수 자체를 smoothing)
```

학습 단계에서는 1번부터 시작하고, 부하가 보이면 2/3으로 옮긴다.

## 7. 손잡이 별로 다른 smoothing 시간이 적합한 경우

```text
freq, Q     : 50~100 ms 정도가 자연스러움
gain        : 매우 빠르게 움직여도 별 클릭이 없음 (10~30 ms)
on/off bypass: cross-fade 30 ms 정도
```

UI 차원에서 적절한 시간 상수를 미리 설계한다.

## 8. EQ chain reset 정책

곡 처음, 트랙 점프, 또는 큰 파라미터 점프 시 IIR 상태를 reset할지 말지 결정이 필요하다.

```text
정상 진행 중      : reset 절대 안 함 (state가 곡선의 일부)
파일 시작/seek    : 모든 biquad의 z1, z2 = 0
큰 파라미터 점프  : smoothing이 흡수 → reset 안 함
모드 전환(Peaking → LPF): cross-fade 한 단계 권장
```

## 9. RuStudio 관점

```text
mod_eq:
  EqChain × 채널 수
  각 EqBand마다 (Smoothed freq, gain, Q)
  사용자 UI → atomic으로 target 전달
  콜백마다 next() → cookbook → process()

mod_mastering의 K-weighting:
  계수 고정. smoothing 불필요. 부팅 때 한 번 설정.

multi-band split:
  cutoff smoothing 필요 (사용자가 cross-over 주파수를 만지는 경우)
```

## 자주 하는 실수

- 콜백 안에서 사용자 슬라이더 값을 직접 사용 → atomic으로 안 받으면 race condition.
- target이 갑자기 바뀌면 current도 같이 점프시키도록 잘못 짜기 → smoothing의 의미 무시.
- 매 샘플 cookbook 호출의 부하를 무시 → 16밴드 12채널 EQ에서 CPU 폭발.
- z1, z2를 손잡이 변경마다 0으로 reset → reset 자체가 더 큰 클릭을 만든다.
- gain만 smoothing하고 freq/Q를 그대로 → freq/Q 클릭만 그대로 남음.

## 반드시 이해해야 할 것

- parametric EQ는 EqBand(=Biquad + 메타) 여러 개의 직렬 사슬이다.
- IIR이라서 계수가 갑자기 바뀌면 z 상태 때문에 클릭이 난다.
- 해결책은 파라미터를 1차 IIR로 smoothing하는 것이다 (envelope과 같은 골격).
- 콜백 안 cookbook 호출 비용에 신경 쓴다. 매 샘플 / 매 N 샘플 / 계수 보간 — 단계적으로 올린다.
