# Chapter 2 - EQ와 Filter의 관계

## EQ는 결국 "주파수별 gain"이다

03 책에서 본 gain은 한 가지 숫자였다.

```rust
out = in * gain;
```

EQ는 그 gain이 **주파수에 따라 달라지는 버전**이다.

```text
out(at 100 Hz)  = in(at 100 Hz)  · gain_at_100
out(at 1 kHz)   = in(at 1 kHz)   · gain_at_1k
out(at 10 kHz)  = in(at 10 kHz)  · gain_at_10k
```

문제는 시간 영역 신호에서는 "이 샘플이 100 Hz의 일부다"라고 분해해 둔 게 아니다. 그래서 주파수별 gain을 시간 축에서 표현하는 도구가 필요하다. 그 도구가 **filter**다.

## Filter란

filter는 한 줄로 이렇게 정의된다.

> 입력 신호를 받아서 frequency response에 따라 amplitude/phase를 바꿔서 내보내는 시스템.

frequency response는 두 가지 곡선의 쌍이다.

```text
amplitude response : 각 주파수에서 amplitude를 얼마나 키우거나 줄이느냐
phase response     : 각 주파수에서 위상을 얼마나 어긋나게 하느냐
```

EQ에서 사용자가 보는 곡선은 amplitude response다. phase response는 보이진 않지만 항상 같이 따라 다닌다 (linear-phase EQ가 아닌 한).

## 몇 가지 표준 필터 모양

이 책에서 다룰 6가지 모양만 그림으로 잡아 두자.

### LPF (Low-Pass Filter)

```text
gain
0dB ──────────────╲
                   ╲
                    ╲
                     ╲___
                          freq →
        cutoff
```

특정 cutoff 주파수 위쪽을 깎는다. "고음만 제거"의 도구.

### HPF (High-Pass Filter)

```text
gain
0dB           ___╱──────────
              ╱
             ╱
            ╱
                          freq →
        cutoff
```

LPF의 반대. "저음만 제거"의 도구.

### BPF (Band-Pass Filter)

```text
gain
0dB         ╱──╲
           ╱    ╲
          ╱      ╲
         ╱        ╲
                          freq →
            center
```

특정 중심 주파수 부근만 통과시킨다.

### Notch

```text
gain
0dB ──────╲    ╱──────
           ╲  ╱
            ╲╱
                          freq →
         center
```

BPF의 반대. 특정 부근만 깎는다.

### Peaking EQ

```text
gain
+6dB         ╱╲
            ╱  ╲
0dB ───────╱    ╲────────
                          freq →
            center
```

특정 부근만 부스트 또는 컷. 사용자가 가장 자주 만지는 EQ 종류.

### Shelving (Low-shelf / High-shelf)

```text
low-shelf:
gain
+6dB ─────╲
           ╲___
0dB             ─────────
                          freq →
        corner

high-shelf:
gain
0dB ─────────___
                ╲
                 ╲─────  +6dB
                          freq →
              corner
```

cutoff 한쪽 영역을 통째로 부스트/컷.

이 6가지면 99%의 EQ 작업을 커버한다.

## 디지털 필터의 두 갈래 — IIR vs FIR

filter를 시간 영역에서 구현하는 방식은 크게 둘이다.

```text
FIR (Finite Impulse Response)
  - 출력 = 과거 N개 입력의 가중합
  - phase가 깔끔하다 (linear-phase 가능)
  - 동일 해상도를 내려면 계수가 많이 필요 (수십 ~ 수백 개)
  - 계산량이 크다

IIR (Infinite Impulse Response)
  - 출력 = 과거 입력 + 과거 출력의 가중합
  - 적은 계수로 가파른 곡선을 낸다 (효율적)
  - phase가 비-linear (시각화 EQ 화면이 그대로 phase는 아님)
  - 수치 안정성에 주의 (계수 변화 시 클릭, 자기-진동 가능)
```

EQ에서 가장 흔히 쓰이는 IIR 단위가 **biquad**(2차 IIR)다. 이 책의 메인 주제다.

## "biquad 1개"가 의미하는 것

biquad는 위 6가지 모양 각각을 표현할 수 있는 가장 작은 IIR 단위다.

```text
biquad 1개 = LPF 한 개 또는 HPF 한 개 또는 peaking 한 개...

EQ 한 채널 (예: 4-band parametric EQ)
  = biquad 4개를 직렬로 연결
  = 직렬: 한 단의 출력이 다음 단의 입력
```

이 직렬 연결의 중요한 사실 두 가지.

```text
1. 직렬 biquad들의 frequency response는 곱(=dB로 합)으로 합쳐진다.
   → "100 Hz +3 dB" + "1 kHz +6 dB"는 두 biquad를 직렬로.

2. 같은 biquad 두 개를 직렬로 걸면 곡선이 두 배로 가팔라진다.
   → 12 dB/oct LPF가 24 dB/oct가 됨.
```

## RuStudio 관점

이 책의 결과물은 다음 모듈에서 직접 쓰인다.

```text
mod_eq             : parametric EQ 본체 (biquad N개의 직렬 사슬)
mod_mastering      : K-weighting (biquad 2개), multi-band split (LPF/HPF)
analyzer 화면      : EQ 곡선 표시 → 06_fft_and_spectrum과 합쳐 실시간 그래프
```

즉 biquad는 RuStudio 안에서 가장 많이 등장하는 처리 블록이 된다.

## 자주 하는 실수

- "EQ는 곡선을 그리는 그래픽 도구"라고만 이해 → 그 곡선이 시간 영역 차분 방정식으로 환원된다는 사실을 놓침.
- LPF와 HPF를 단순히 "어떤 주파수 위/아래를 자르는 칼"로만 봄 → cutoff 근처는 부드럽게 변하는 곡선이지 직각이 아님.
- Q를 모든 필터에서 같은 의미로 봄 → LPF/HPF에서 Q는 cutoff 부근의 강조량이고, peaking에서 Q는 폭이다 (의미가 다르지만 같은 변수명).

## 반드시 이해해야 할 것

- EQ는 "주파수별로 다른 gain"이고, 그 분해를 시간 영역에서 가능하게 하는 도구가 filter다.
- 이 책에서 다룰 6가지 filter 모양(LPF/HPF/BPF/Notch/Peaking/Shelving)을 머릿속에 그림으로 들고 있어야 한다.
- IIR이 EQ에 흔히 쓰이는 이유는 적은 계수로 가파른 곡선을 만들기 때문이다. 대가는 phase 비선형과 수치 안정성 주의.
- biquad 1개 = 6가지 모양 중 한 개. EQ는 biquad 여러 개의 직렬이다.
