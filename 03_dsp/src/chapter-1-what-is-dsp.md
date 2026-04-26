# Chapter 2 - DSP가 cpal 위에서 푸는 문제

`01_cpal` 에서 확인한 것:

```text
cpal -> Host -> Device -> Stream -> callback(&mut [f32])
```

여기까지 오면 우리는 **버퍼를 들고 있다**. 그런데 그 버퍼에 **무엇을 써야 할지**는 cpal 이 알려주지 않는다. 그걸 알려주는 계층이 DSP 다.

## 한 문장 정의

> DSP 는 "숫자 배열을 다른 숫자 배열로 바꾸는 규칙"이다.

이 책에서 다루는 DSP 블록은 모두 아래 모양이다.

```rust
pub trait DspNode: Send {
    fn process(&mut self, input: &[f32], output: &mut [f32]);
}
```

- `&mut self` → 블록은 상태를 갖는다. (phase, envelope state, filter delay line 등)
- `input: &[f32]` → 이전 블록의 출력 (source node는 빈 슬라이스)
- `output: &mut [f32]` → 이번 블록이 채워야 할 버퍼

이 모양은 `01_cpal` 의 `AudioProcess` 와 **동일**하다. 의도적이다. Phase 4 에서 그래프 노드로 승격시킬 수 있게 처음부터 같은 계약을 쓴다.

## 왜 직접 구현하나

이미 `fundsp`, `dasp` 같이 고품질 DSP 크레이트가 있다. 그런데도 직접 구현하는 이유는 세 가지다.

1. **수식 → 코드** 변환 경험이 있어야 크레이트를 읽을 수 있다.
2. `mod_mastering` 같은 DAW 플러그인은 결국 이 블록들의 조합이다.
3. 실시간 규칙 위반 (힙 할당, 락, panic) 을 블록 단위에서 감지하는 습관이 생긴다.

## 시간 도메인과 주파수 도메인

DSP 블록은 크게 두 계열이다.

| 계열 | 대표 블록 | 보는 축 |
|------|-----------|---------|
| 시간 도메인 | Oscillator, Envelope, Delay | 샘플 index |
| 주파수 도메인 | Filter (Biquad), EQ, FFT | 주파수 bin |

필터(Biquad)는 **시간 도메인에서 실행되지만 주파수 도메인을 목적으로 하는** 블록이다. 이 구분을 머릿속에 그려 두면 뒤 챕터가 훨씬 쉬워진다.

## 이 Phase 의 최종 그림

```text
[Oscillator] -> [Envelope] -> [Filter/EQ] -> [Output]
                                  |
                                  v
                              [FFT/LUFS]
                              (분석 경로)
```

- **생성**: Oscillator
- **제어**: Envelope
- **변형**: Filter / EQ
- **측정**: FFT / LUFS

이 그림이 곧 `mod_mastering` 크레이트의 초기 스케치다.

## 답할 질문

- 왜 `DspNode::process` 시그니처에 `sample_rate` 가 들어가지 않는가? 어디에 두어야 하는가?
- source 노드 (오실레이터) 의 `input` 은 왜 빈 슬라이스인가? 빈 슬라이스 대신 `Option` 으로 두면 무엇이 나빠지나?
- `&mut self` 대신 `&self` 로 하려면 상태를 어디로 빼야 하는가?
