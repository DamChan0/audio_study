# Chapter 3 - Biquad 구조

biquad는 "biquadratic"의 줄임말이다. 분자와 분모가 모두 2차 다항식인 IIR 필터를 뜻한다.

이 장에서는 다음 세 가지를 본다.

```text
1. biquad의 차분 방정식
2. 상태 변수가 무엇이고 왜 두 개인가
3. Direct Form II Transposed 구조 (이 책에서 권장하는 구현)
```

## 1. 차분 방정식

```text
y[n] = b0·x[n] + b1·x[n-1] + b2·x[n-2]
                - a1·y[n-1] - a2·y[n-2]
```

기호 의미.

```text
x[n]   : 현재 입력 샘플
y[n]   : 현재 출력 샘플
x[n-k] : k 샘플 전 입력
y[n-k] : k 샘플 전 출력
b0..b2 : feedforward 계수 (분자)
a1, a2 : feedback 계수 (분모; a0는 정규화로 1로 둠)
```

핵심 관찰.

- 5개 계수만 있으면 LPF/HPF/Peaking/Shelf 등 어느 모양이든 만들 수 있다.
- 과거 출력을 다시 쓰는 부분이 IIR의 "I"(infinite)다. 한 번 들어간 신호가 영원히 영향을 줄 수 있다 → 안정성에 주의.

## 2. 상태 변수가 두 개인 이유

biquad의 "2차"는 곧 "직전 2개의 입력과 출력을 기억한다"는 뜻이다.

```text
x[n-1], x[n-2]   : 입력 메모리 2개
y[n-1], y[n-2]   : 출력 메모리 2개
```

이 4개가 상태처럼 보이지만, 영리한 구현(Direct Form II 계열)에서는 **상태 변수 2개**로 충분하다.

이 사실은 03 책의 delay buffer에서 본 패턴과 같다 — "필요한 과거를 z⁻¹ 지연 단위로 표현"한다.

```text
x → [b0] → ─────────┐
                    ▼
                    y
   ┌───[z⁻¹]───[b1]─┴───[a1 (negative)]───[z⁻¹]───┐
   │                                              │
   └───[z⁻¹]───[b2]─────[a2 (negative)]───[z⁻¹]──┘

(Direct Form I — 메모리 4개, 단순하지만 비효율)
```

이걸 줄여서 메모리 2개로 만든 것이 Direct Form II.

## 3. Direct Form II Transposed — 이 책의 기준

여러 형태 중 디지털 오디오에서 권장되는 것이 **Direct Form II Transposed (DF2T)**다.

이유 두 가지.

```text
1. 메모리(상태)가 2개로 충분 — 캐시/효율
2. 부동소수 누적 오차에 강함 — 32-bit float에서도 안정적
```

코드로 보면 이렇다.

```rust
struct Biquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    z1: f32, z2: f32,           // 상태 변수 두 개
}

impl Biquad {
    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}
```

이 6줄짜리 코드가 이 책의 핵심이다. 거의 모든 디지털 EQ가 이 모양 위에서 돈다.

해설.

```text
y = b0·x + z1                ← 출력 계산
z1 = b1·x - a1·y + z2        ← 다음 호출의 z1 갱신
z2 = b2·x - a2·y             ← 다음 호출의 z2 갱신
```

`z1`, `z2`가 다음 샘플 호출 시 "과거"의 정보를 담은 채 살아 있다. 이게 IIR의 메모리다.

## 4. 다른 form들과의 비교 (왜 DF2T인가)

빠른 비교만.

```text
Direct Form I              메모리 4개  | 단순  | 변동 계수에 비교적 안전
Direct Form II             메모리 2개  | 효율  | 계수가 큰 경우 internal saturation 가능
Direct Form II Transposed  메모리 2개  | 효율  | 32-bit float에서 가장 안정 ← 권장
Lattice / SVF              구조 다름   | 빠른 변조용 (modulated filter)
```

이 책 단계에서는 DF2T 한 가지로 모든 cookbook 필터를 만든다. 다른 form은 modulated filter (가령 빠르게 cutoff가 흔들리는 신디 필터)에서 더 의미가 있다.

## 5. SISO와 채널 처리

biquad는 single-input single-output이다. 즉 한 채널씩 처리한다.

```text
stereo EQ:
  left  → Biquad(L) → out_left
  right → Biquad(R) → out_right
```

채널마다 별도의 Biquad 인스턴스(=별도의 z1/z2)가 필요하다. 한 인스턴스를 두 채널에 공유하면 채널이 서로 섞여 모노가 된다.

multi-band 같이 한 채널에 N개의 biquad를 쌓는 구조는 이렇다.

```text
in → Biquad(low) → Biquad(mid) → Biquad(high) → out
       z1, z2          z1, z2          z1, z2
```

## 6. 안정성 — 발산하지 않는 계수의 조건

IIR은 잘못된 계수에서 출력이 무한히 커질 수 있다.

```text
biquad가 안정 ⇔ 분모 다항식의 영점이 단위원 안에 있다
              ⇔ |a2| < 1  AND  |a1| < 1 + a2
```

cookbook 공식을 그대로 쓰면 이 조건이 자동으로 만족된다. 직접 계수를 손대는 경우에만 신경 쓰면 된다.

## 7. RuStudio 관점

```text
mod_eq:
  4-band parametric EQ = Biquad × 4 (채널 수 × 4)
  → stereo면 Biquad 인스턴스 8개

mod_mastering의 K-weighting:
  Biquad 2단 (shelving + HPF)

multi-band compressor의 split:
  LPF/HPF Biquad 쌍 → 두 대역 또는 그 이상
```

수십 개의 biquad가 한 콜백에서 동시에 돌 수 있다. 그래서 이 6줄짜리 process 함수가 빠르고 안정적이어야 한다.

## 자주 하는 실수

- 채널끼리 같은 Biquad 인스턴스를 공유 → 좌우가 섞임.
- z1, z2를 콜백마다 0으로 reset → 매 콜백 시작에 클릭.
- a0가 1이 아닌데 정규화하지 않고 사용 → 계수 정규화 빠뜨리면 amplitude가 어긋남.
- f32로 가파른 high-Q 필터 → 어떤 cookbook 계수에선 round-off 누적이 보일 수 있다. 이 경우 SVF 같은 다른 form이 더 안정적.

## 반드시 이해해야 할 것

- biquad의 상태는 z1, z2 두 개. 매 샘플 갱신.
- Direct Form II Transposed가 32-bit float 디지털 오디오에서 권장 구현이다.
- 위의 6줄 process 함수가 이 책의 핵심 코드다. cookbook 계수는 그 위에 얹힌다.
- 채널마다 별도의 Biquad 인스턴스가 필요하다. 인스턴스 = (계수 5개 + 상태 2개).
