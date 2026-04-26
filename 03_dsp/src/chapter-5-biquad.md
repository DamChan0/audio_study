# Chapter 6 - Biquad 필터 (Direct Form II Transposed)

이 챕터의 목표: **Biquad 한 개를 직접 구현하고, 계수 다섯 개만으로 저역/고역/피킹이 모두 표현된다는 것을 이해한다.**

## 왜 Biquad 인가

모든 IIR 필터를 2차 섹션의 **직렬 연결**로 쪼갤 수 있다. 그 2차 섹션이 Biquad 다.

- 수치 안정성이 좋다 (8차 필터 하나보다 2차 4개 직렬이 훨씬 안정).
- 계수가 5 개 (`b0, b1, b2, a1, a2`) 로 고정이라 구현 골격이 **필터 종류와 무관**하다.
- 필터 종류는 "계수 계산 공식"만 바뀔 뿐이다. (→ Chapter 7 RBJ)

## 전달함수

```text
        b0 + b1·z^-1 + b2·z^-2
H(z) = -----------------------
        1  + a1·z^-1 + a2·z^-2
```

`z^-1` 은 "1 샘플 지연". 위 식을 시간 도메인으로 풀면:

```text
y[n] = b0·x[n] + b1·x[n-1] + b2·x[n-2] - a1·y[n-1] - a2·y[n-2]
```

`x` 는 입력, `y` 는 출력. 과거 2 샘플씩 **상태로 들고 있어야 한다**.

## Direct Form 여러 가지

같은 전달함수도 **구현 형태가 여러 개**다. 그 중 실무 표준은:

- **Direct Form I**: 입력 지연 2 개 + 출력 지연 2 개 = state 4 개.
- **Direct Form II**: state 2 개로 줄인 형태. 대신 중간 노드가 오버플로 위험.
- **Direct Form II Transposed**: state 2 개 + 수치 안정성 좋음. **→ 이걸 쓴다.**

### Transposed Direct Form II

```rust
pub struct Biquad {
    b0: f64, b1: f64, b2: f64,
    a1: f64, a2: f64,
    z1: f64, z2: f64,   // 2 개의 지연 레지스터
}

impl Biquad {
    pub fn process_sample(&mut self, x: f64) -> f64 {
        let y  = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}
```

- state 는 `z1, z2` 두 개.
- 한 샘플 처리에 **곱셈 5 / 덧셈 4**. 가볍다.
- 계수는 외부에서 주어지는 것으로 본다. 필터 종류별 계수 공식은 Chapter 7.

## 왜 계수 변경은 위험한가

`b0, b1, b2, a1, a2` 를 한 샘플에 **전부 교체**해버리면:

- 이전 `z1, z2` 는 **옛 계수에 맞춰 쌓인 상태**.
- 새 계수와 섞이면서 출력이 튄다. → **클릭 노이즈**.

대응 방법 (Chapter 7 에서 다시):

1. 계수를 **프레임 경계에서만** 바꾼다 (간단).
2. 계수를 **시간에 따라 스무딩** 한다 (선형 보간). → 파라미터 자동화에 필요.

## 이 챕터에서 구현할 것

- `examples/src/filter/biquad.rs`: Transposed DF-II Biquad 골격 + `DspNode` 구현
- 계수는 **수동으로 주입** (Low-pass 공식을 다음 챕터 보기 전에 먼저 만들지 않는다)
- `examples/src/bin/eq_peak.rs` 는 Chapter 7 에서 작성

## 답할 질문

- 왜 `z1, z2` 는 `f64` 인가? 입력이 `f32` 여도?
- Biquad 여러 개를 직렬 연결할 때, 한 블록의 `z1/z2` 가 다음 블록에 영향을 주는 경로는 무엇인가?
- `reset()` 은 어떤 상태를 지워야 하는가? 계수인가, state 인가?
- 계수가 유효하지 않은 값 (분모 0 등) 이면 `process_sample` 안에서 어떻게 되는가?

## 완료 기준

- 단위 테스트로 **항등 필터** (`b0=1, 나머지=0`) 를 구현 → 입력이 그대로 출력되는지 확인.
- state 를 다른 인스턴스로 복사했을 때 필터가 동일하게 동작하는지 확인 (Clone 없이 `reset` + 같은 입력).
