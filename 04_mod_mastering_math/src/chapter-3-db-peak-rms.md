# Chapter 3 - dB, Peak, RMS

이 장은 mastering 전체에서 가장 많이 등장하는 세 단어를 정리한다.

## 1. dBFS — 디지털 오디오의 0이 어디인가

dB는 비율을 로그로 표현한 단위지만, "기준이 무엇이냐"에 따라 종류가 갈린다.

```text
dBFS  : Full Scale 기준 (디지털 오디오에서 거의 표준)
        0 dBFS = 표현 가능한 최대 amplitude (= 1.0)
        -∞     = 0.0 (무음)

dBu / dBV : 아날로그 전압 기준
dBSPL    : 음압 기준 (귀로 들리는 절대 음량)
```

이 책에서 등장하는 모든 dB는 별다른 표시가 없으면 **dBFS**라고 보면 된다.

```text
linear   dBFS
1.000     0 dBFS    ← 디지털 한계
0.707    -3 dBFS
0.500    -6 dBFS
0.100   -20 dBFS
0.010   -40 dBFS
0.001   -60 dBFS
```

이 환산을 머리로 굴릴 수 있어야 컴프 threshold 같은 값을 빠르게 가늠할 수 있다.

## 2. Peak — 순간 최고 amplitude

**Peak**은 한 시간 구간 안에서 절대값이 가장 큰 샘플의 amplitude다.

```rust
fn peak_in_buffer(buf: &[f32]) -> f32 {
    let mut p = 0.0;
    for &s in buf {
        let a = s.abs();
        if a > p { p = a; }
    }
    p
}
```

특징.

- 매우 빠르게 변한다. 한 샘플만 튀어도 peak가 거기 따라간다.
- "0 dBFS 넘었음 = 클립 위험" 같은 안전 판정에 쓰인다.
- 음량의 척도로는 부적절하다. 짧고 큰 transient(드럼 어택 같은 것)에 휘둘려서 평균 음량을 반영하지 못한다.

### 디스플레이용 peak hold

UI에서 peak meter는 보통 "최근 N ms 동안의 최대값"을 잠깐 잡고 있다가 천천히 떨어뜨리는 식이다.

```text
순간 peak가 미터를 끌어올리고
hold 시간 동안 그 값을 유지하다가
release rate(예: 11.8 dB/s)로 내려간다
```

이 동작은 03 책의 envelope와 같은 패턴이다. 단계는 (Hold, Release) 두 개로 단순화된다.

## 3. RMS — 시간 평균 amplitude

**Root Mean Square**, 시간 구간 안 amplitude의 평균적 크기다.

```text
RMS = sqrt( mean( x[i]^2 ) )
```

코드로 보면 이렇다.

```rust
fn rms(buf: &[f32]) -> f32 {
    let sum_sq: f32 = buf.iter().map(|s| s * s).sum();
    (sum_sq / buf.len() as f32).sqrt()
}
```

특징.

- 짧은 transient에 잘 안 휘둘린다.
- 평균적인 "체감 음량"에 peak보다 가깝다 (완벽하진 않다 → LUFS 등장 이유).
- 윈도우 길이를 어떻게 잡느냐가 결과를 좌우한다 (보통 300 ms ~ 1 s).

### 실시간 RMS — 1차 IIR 평균

버퍼 끝나기 전에는 평균을 못 내냐? 그렇지 않다. 매 샘플 점진적으로 평균을 갱신하는 1차 IIR이 있다.

```rust
struct RmsFollower {
    state: f32,    // 현재 누적 mean square
    coeff: f32,    // 시간 상수 (0.0 ~ 1.0)
}

impl RmsFollower {
    fn next(&mut self, sample: f32) -> f32 {
        let sq = sample * sample;
        self.state = self.coeff * self.state + (1.0 - self.coeff) * sq;
        self.state.sqrt()
    }
}
```

`coeff`가 1에 가까우면 변화가 느리고, 0에 가까우면 빠르다. 이 구조는 03 envelope의 변형이며, compressor의 envelope follower와 사실상 같은 골격이다.

## 4. peak vs RMS — 직관

같은 신호를 두 방식으로 측정하면 일반적으로 RMS가 peak보다 낮은 값으로 나온다. 차이는 "신호가 얼마나 transient 스러운가"에 달려 있다.

```text
사인파 1.0 amplitude:
  peak = 1.0       (= 0 dBFS)
  RMS  = 0.707     (= -3 dBFS)
  → 차이 3 dB

드럼 hit (짧고 큰 transient):
  peak = 1.0
  RMS  = 0.1 ~ 0.3
  → 차이 10 dB 이상도 흔함
```

이 차이를 "**crest factor**"라고 한다.

```text
crest factor = peak / RMS  (또는 dB 단위로 peak_dB - RMS_dB)
```

mastering에서 crest factor는 "이 신호에 dynamics 처리가 얼마나 들어갔는지"의 거친 척도다.

## 5. compressor가 보는 amplitude는 peak일까 RMS일까

이 질문이 중요하다. **둘 다 가능하고, 어느 쪽을 보느냐가 컴프의 성격을 결정한다.**

```text
peak detection : transient에 즉각 반응. 어택이 빠른 컴프, drum 처리에 적합.
RMS detection  : 평균 음량에 반응. 부드러운 컴프, 보컬/마스터에 적합.
```

대부분의 디지털 컴프는 두 모드를 옵션으로 둔다. 이 책 단계에서는 peak detection을 기본으로 보고, 다음 단계에서 RMS detection 옵션을 추가하는 식으로 가면 된다.

## 자주 하는 실수

- "0 dBFS"와 "0 dB"를 혼용 → 기준이 다른 단위다.
- peak를 음량 척도로 사용 → transient 하나에 미터가 휘둘린다.
- RMS 윈도우 길이를 명시하지 않고 "RMS는 -16 dB였다"라고 말함 → 윈도우 길이에 따라 값이 달라진다.
- 콜백 안에서 매 샘플 `sqrt` 호출 비용을 무시 → 1차 IIR로 따라가는 것이 일반적이다.
- `f32`로 long-running RMS 계산 → 누적 오차. mastering 미터는 보통 `f64`.

## 반드시 이해해야 할 것

- dBFS의 0은 amplitude 1.0이다. 그 위로는 클립.
- peak는 "안전 판정", RMS는 "평균 음량 추정"에 쓰는 도구다. 같은 신호를 두 방식으로 본다.
- 1차 IIR로 RMS를 따라가는 패턴은 envelope follower / compressor / 1차 LPF 모두에서 반복된다.
- crest factor는 신호의 transient 성격을 보여주는 거친 척도다.
