# Chapter 6 - Envelope와 시간축 amplitude 제어

envelope는 **amplitude가 시간에 따라 어떻게 변하는지를 만드는 상태기계**다.

```text
envelope
  input  : note_on / note_off 같은 트리거
  output : 매 샘플마다 0.0 ~ 1.0 사이의 amplitude 계수
  state  : 현재 단계(stage)와 그 단계 안에서 진행된 시간/레벨
```

전체 신호가 아니라 **신호에 곱해지는 amplitude 곡선**을 생각하면 된다.

```text
final[i] = source[i] · envelope_value[i]
```

## ADSR — 가장 흔한 envelope

ADSR은 4단계다.

```text
amplitude
   ▲
1.0│      /\
   │     /  \________
   │    /            \
   │   /              \
   │  /                \
   └─────────────────────► time
      A    D    S       R

A: attack   - 0  → 1     (note_on 직후)
D: decay    - 1  → sustain level
S: sustain  - level 유지 (note_off 까지)
R: release  - level → 0  (note_off 후)
```

이 그림이 envelope의 전부다. 4개 시간 파라미터(`attack`, `decay`, `release`)와 한 개 레벨 파라미터(`sustain`)로 곡선이 결정된다.

## envelope의 "상태"는 무엇인가

이 질문이 핵심이다.

```text
state =
  현재 단계 (Idle / Attack / Decay / Sustain / Release)
  현재 amplitude 값
  현재 단계에서 매 샘플당 변화량 (또는 단계 시작 시각)
```

샘플 인덱스 t를 그대로 식에 넣을 수도 있지만, 그러면 "지금 몇 단계인지" 분기가 매 샘플 복잡해진다. 보통은 단계 enum + 현재 amplitude 한 줄로 들고 다닌다.

```rust
enum Stage { Idle, Attack, Decay, Sustain, Release }

struct Adsr {
    stage: Stage,
    level: f32,          // 현재 amplitude 출력값
    sr: f32,
    attack: f32,         // seconds
    decay: f32,
    sustain: f32,        // 0.0 ~ 1.0
    release: f32,
}
```

## 한 샘플당 한 번 호출되는 함수

ADSR의 핵심 루프는 이런 모양이다.

```rust
impl Adsr {
    fn next(&mut self) -> f32 {
        match self.stage {
            Stage::Attack => {
                let step = 1.0 / (self.attack * self.sr);
                self.level += step;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                let step = (1.0 - self.sustain) / (self.decay * self.sr);
                self.level -= step;
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => { /* note_off까지 유지 */ }
            Stage::Release => {
                let step = self.sustain / (self.release * self.sr);
                self.level -= step;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
            }
            Stage::Idle => { self.level = 0.0; }
        }
        self.level
    }

    fn note_on(&mut self)  { self.stage = Stage::Attack; }
    fn note_off(&mut self) { self.stage = Stage::Release; }
}
```

여기서 보아야 할 점.

- 매 샘플 `step` 만큼만 더하거나 뺀다. 단순한 선형 보간이다.
- 실제 음악 사용에는 지수 곡선이 더 자연스럽지만, 선형 envelope만 이해해도 8할은 잡힌다.
- `sr`이 들어 있는 이유는 "초 단위 시간"을 "샘플 단위 step"으로 환산하기 위해서다.

## envelope를 신호에 적용하는 위치

```text
oscillator → × envelope → × gain → mixer → ...
              (이 곱셈)
```

각 voice마다 envelope를 하나씩 들고 있고, 그 voice의 신호에 곱하는 식이다.

```rust
for frame in data.chunks_mut(channels) {
    let s = osc.next(freq);
    let e = env.next();
    let v = s * e;
    for ch in frame.iter_mut() { *ch = v; }
}
```

## envelope는 어디에 또 등장하는가

같은 패턴이 여러 곳에 다시 나온다.

```text
ADSR              : 신디사이저 voice의 amplitude 윤곽   ← 이 장
parameter smoother: gain/cutoff 같은 파라미터 변화 완화 (zipper noise 제거)
compressor envelope: 신호 amplitude를 추적하는 follower (04_mod_mastering_math)
LFO envelope      : LFO의 깊이를 시간에 따라 변화시키기
```

전부 "값이 시간에 따라 부드럽게 변한다"는 동일한 골격이다. 그래서 envelope를 한 번 잘 짜면 RuStudio 거의 모든 곳에서 재사용된다.

## 자주 하는 실수

- attack/decay/release 시간을 ms 단위로만 받고 sample 단위로 환산을 잊는다 → step 계산이 깨진다.
- `Sustain`에서 release로 넘어갈 때 level을 1.0으로 reset → 끊긴 소리가 다시 튀어 오른다.
- `Idle` 상태가 없어서 voice가 끝나도 mixer가 계속 0을 더한다 → CPU 낭비. 끝난 voice는 명시적으로 비활성화한다.
- `attack = 0.0` 같은 0초 처리에 대해 0으로 나누기 → step 계산 시 분모가 0이 안 되도록 최소값(예: 1샘플)을 두어야 한다.

## 반드시 이해해야 할 것

- envelope는 "신호"가 아니라 "신호에 곱해지는 amplitude 곡선"이다.
- envelope의 상태는 (단계, 현재 level) 두 개로 충분하다.
- ADSR은 4단계 선형 보간이 골격이다. 곡선 종류(linear vs exponential)는 그 위에 얹는다.
- 같은 패턴이 parameter smoother, compressor envelope follower 등에서 다시 등장한다. 이 한 개를 단단히 이해하는 게 다음 책들의 토대다.
