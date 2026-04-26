# Chapter 5 - Limiter와 True Peak

## 1. Limiter는 컴프의 극단이다

limiter는 새로운 알고리즘이 아니다. compressor의 다음 두 조건을 함께 적용한 것이다.

```text
ratio       = ∞       (=threshold 위로는 절대 못 올라감)
attack      = 매우 짧음 (수 샘플 ~ 1 ms)
release     = 짧음~중간
```

목적도 분명하다.

> 출력이 절대로 어떤 ceiling을 넘지 못하게 한다.

mastering 사슬의 끝에 항상 limiter가 있는 이유는 단순하다. 이 단계 뒤에 cpal/파일 출력이 있고, 거기서 1.0(=0 dBFS)을 넘는 값은 그냥 잘려 나간다 — 디지털 클립.

## 2. brickwall limiter의 원리

```text
input ──► [ envelope follower (peak, 빠른 attack) ]
   │              │
   ▼              ▼
delay (look-ahead) ────────► gain 계산 (ceiling - peak_db)
   │                                  │
   └──────────────────────────────► (×) ──► output
```

컴프와 차이는 두 가지다.

1. **look-ahead**: 신호를 짧게 지연시켜 envelope follower가 그 지연 동안 미리 측정. peak가 닥치기 *전에* gain을 깎아둘 수 있다.
2. **soft envelope**: ratio가 무한이라 어택이 너무 거칠면 distortion이 생긴다. 짧은 시간상수로 부드러운 envelope을 그려서 gain을 곱한다.

look-ahead는 03 책의 delay buffer를 거의 그대로 쓴다.

```rust
let delayed = delay_line.process(input, look_ahead_samples, /* mix */ 1.0);
let env     = peak_follower.next(input);     // 미래 신호로 envelope
let g       = compute_brickwall_gain(env, ceiling_db);
*output     = delayed * g;
```

지연된 신호와 미래에서 측정한 envelope을 시간 정렬해서 곱한다. 이게 look-ahead의 핵심이다.

## 3. ceiling은 보통 0 dBFS보다 약간 아래다

이론상 ceiling을 0 dBFS로 두면 클립이 안 날 것 같지만, 실제 마스터링에서는 보통 -0.3 ~ -1.0 dBFS로 둔다.

```text
ceiling = -0.3 dBFS    (디지털 ↔ 아날로그 변환 마진)
ceiling = -1.0 dBFS    (방송 표준 등 보수적인 케이스)
```

이유는 다음 절에서 본다.

## 4. True Peak — sample-peak이 0 dBFS여도 클립이 날 수 있다

여기가 mastering의 함정 중 하나다.

```text
디지털 신호는 샘플 사이의 값을 모른다.
하지만 D/A 변환기는 샘플 사이를 보간해서 연속 신호로 만든다.
그래서 샘플 사이가 더 높은 amplitude를 가질 수 있다.
```

그림으로 보면 명확하다.

```text
  amplitude
   ▲
1.0│         ╳ ← 보간된 정점이 0 dBFS를 넘음
   │      ╳     ╳
   │   ●           ●     ← 실제 샘플들 (전부 ≤ 0 dBFS)
   │              ●
   │   ●
   └────────────────────► time
```

이 보간된 정점을 **inter-sample peak**, 또는 **true peak (TP)**라고 한다. 이걸 측정하지 않으면 sample-peak 0 dBFS인데 D/A 후 클립인 상황이 생긴다.

## 5. true peak를 어떻게 측정하나

표준은 ITU-R BS.1770의 부속서로 정의되어 있고, 핵심 아이디어는 단순하다.

> 신호를 4× 또는 8×로 업샘플링한 뒤, 그 결과의 sample-peak를 측정한다.

업샘플링은 보통 polyphase FIR로 한다. 구현은 약간의 분량이 있어서 여기선 큰 그림만 짚는다.

```text
input  ─► [ 4× polyphase FIR upsampler ] ─► [ sample-peak max ] ─► true_peak
```

이 책에서는 직접 구현하지 않고 "왜 sample peak로는 부족한가" + "측정에는 업샘플링이 필요하다" 두 가지만 단단히 한다. 실제 구현은 RuStudio Phase 단계에서 별도 모듈로 다룬다.

## 6. ISP-safe limiter

true-peak이 측정 가능하다면, limiter도 그 측정값을 기준으로 ceiling을 적용할 수 있다. 이걸 보통 **inter-sample-peak safe limiter** 또는 **true-peak limiter**라고 부른다.

```text
일반 limiter      : sample-peak 기준
TP/ISP limiter    : 업샘플링 후 peak 기준
```

streaming/방송 표준이 "true peak ≤ -1 dBTP" 같은 형태로 명시되어 있어서, 발송용 마스터에서는 ISP limiter가 사실상 필수다.

## 7. limiter가 깎으면 어떻게 되나

limiter가 강하게 작동하기 시작하면 신호의 dynamics가 많이 잘린다. 이걸 사용자에게 보여주는 게 **gain reduction meter (GR)**다.

```text
GR (dB) = 0 dB ── compressor/limiter가 깎아준 양 (음수)
```

GR meter는 03 envelope의 출력을 dB로 변환해 거꾸로 표시하는 형태다. 보통 limiter는 0 ~ -10 dB 영역에서 일하면 정상, -10 dB 이상 깎이면 음악적으로 무리가 시작된다.

## 자주 하는 실수

- ceiling을 0 dBFS로 두고 끝 → D/A 후 inter-sample 클립 가능.
- look-ahead 없이 무한 ratio + 0 attack → 거친 distortion (saturator처럼 들림).
- look-ahead 길이만큼 신호 지연이 발생하는 걸 잊고 다른 트랙과 시간 정렬을 깨뜨림.
- "limiter만 걸면 곡이 평평해짐" 식의 잘못된 사용. limiter는 안전망이지 음량 끌어올리기의 정답이 아니다.
- true peak 측정을 sample peak로 대체 → 그래픽 표시는 안전한데 실제는 클립.

## 반드시 이해해야 할 것

- limiter는 컴프의 ratio = ∞ + 빠른 attack 변형이다. 새 알고리즘이 아니다.
- look-ahead는 03 책의 delay buffer를 그대로 쓴다. 신호와 envelope의 시간 정렬용.
- 디지털 sample peak가 0 dBFS여도 D/A 후 클립이 날 수 있다 — true peak.
- 마스터링 ceiling은 보통 0 dBFS보다 살짝 아래에 둔다. 그게 inter-sample 마진이다.
