# 들어가며

이 문서는 `RuStudio` 학습 시리즈의 세 번째 책이다.

`01_cpal`에서 "소리가 어떻게 하드웨어로 빠져나가는가"를 봤고, `02_mod_player_architecture`에서 "재생 구조를 누가 들고 있는가"를 봤다.

이 책은 그 다음 질문을 다룬다.

> 콜백 안에서 버퍼에 들어가는 그 `f32` 샘플 값은 도대체 어떻게 만들어지는가?

즉 이 책의 주제는 **샘플 단위 계산**이다. 신디사이저, EQ, 컴프, 리버브 같은 모든 DSP 모듈이 결국 한 줄로 환원되는 그 한 줄이다.

```text
output[i] = some_function(input[i], internal_state)
```

이 한 줄을 이해하면, 이후 mastering / EQ / FFT / graph 책은 같은 패턴의 변주가 된다.

## 이 책이 다루는 빌딩 블록

```text
oscillator   : 입력 없이 시간만으로 샘플을 만들어내는 소스
gain / dB    : amplitude를 곱셈으로 다루는 가장 기본 연산
pan          : 채널 간 amplitude 분배
mixer        : 여러 소스를 더해서 한 버퍼로 합치기
envelope     : 시간에 따른 amplitude 변화를 만드는 상태기계
delay buffer : 과거 샘플을 기억했다가 다시 꺼내는 stateful DSP
```

이 6개는 RuStudio Phase 2의 최소 단위 부품이다.

## 이 책을 다 읽고 나면

- "DSP가 뭐냐"는 질문에 한 문장으로 답할 수 있다.
- 위상 누산기(phase accumulator)를 종이 위에 그려서 설명할 수 있다.
- gain과 dB가 같은 것을 다른 스케일로 본다는 사실을 설명할 수 있다.
- envelope가 "현재 시간"이 아니라 "현재 상태"를 들고 있는 이유를 설명할 수 있다.
- delay buffer가 왜 stateful DSP의 가장 좋은 입문 예제인지 설명할 수 있다.
- 이 6개 블록이 mastering / EQ / FFT 책에서 어떻게 재사용되는지 예측할 수 있다.

## 이 책에서 약속하는 것 / 약속하지 않는 것

```text
✓ 각 블록의 동작을 그림과 식으로 설명
✓ Rust 코드 스니펫으로 핵심 패턴만 보여주기
✓ RuStudio mod_player 콜백 안에서 어떻게 쓰일지 연결

✗ 모든 합성/이펙트 알고리즘 망라
✗ 학술적인 신호처리 교재 수준의 증명
✗ 완성된 신디사이저 구현
```

깊이 들어가는 알고리즘(IIR/FIR 설계, 스펙트럼 해석)은 다음 책들의 몫이다.
