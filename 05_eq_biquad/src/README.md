# 들어가며

이 책은 RuStudio 학습 시리즈의 다섯 번째 책이다.

지금까지의 흐름을 다시 정리하자.

```text
01 cpal                : 소리가 어떻게 하드웨어로 빠져나가는가
02 mod_player          : 재생 구조를 누가 들고 있는가
03 dsp_fundamentals    : 샘플 단위 계산 빌딩 블록 (osc, gain, env, delay)
04 mod_mastering_math  : 동적 처리와 측정 (compressor, limiter, LUFS)
05 eq_biquad           ← 이 책. 주파수 축에서 amplitude를 바꾸는 도구
```

지금까지는 amplitude(크기)와 time(시간) 축을 다뤘다. 이 책에서 처음으로 **frequency(주파수) 축**에 손을 댄다.

## 이 책이 답하려는 질문

> "1 kHz 부근만 +6 dB 올리고 나머지는 그대로 두라"는 요청을, 도대체 어떻게 시간 영역의 단순 코드로 구현하는가?

답은 **biquad 필터**다. 이 책에서 보게 될 거의 모든 것은 다음 사실의 변주다.

```text
biquad 1개 = 2차 IIR 필터 = "이 모양의 frequency response를 만든다"
EQ = biquad 여러 개를 직렬 또는 병렬로 묶은 것
```

## 이 책이 다루는 것

```text
1. 필터의 frequency response가 무엇인가
2. biquad의 차분 방정식과 상태 변수
3. Direct Form II Transposed 구조 (수치 안정성)
4. RBJ Audio EQ Cookbook 계수 (peaking / low-shelf / high-shelf / LPF / HPF / BPF)
5. parametric EQ — freq, gain, Q 세 손잡이
6. coefficient smoothing — 실시간 파라미터 변경 시 클릭 제거
```

## 이 책이 다루지 않는 것

```text
✗ FIR 필터 설계
✗ linear-phase EQ
✗ 윈도우 메소드 / Parks-McClellan 같은 필터 설계 알고리즘
✗ 수학적 안정성 증명, 영점/극점 평면 분석의 깊은 이론
✗ 라떼-디저트급 minimum-phase / all-pass / Hilbert 변환
```

이 책은 **표준 cookbook 계수를 그대로 쓰는 단계**까지다. 직접 필터를 설계하는 일은 별도 영역이다.

## 04 책과의 연결

04 책 끝에서 LUFS의 K-weighting이 "biquad 두 개"라고만 말하고 넘어갔다. 이 책의 첫 번째 명시적 연결이 거기다.

또한 multi-band compressor는 신호를 LPF/HPF로 N개 대역으로 쪼갠 후 각 대역에 컴프를 걸고 합치는 구조다. 그 LPF/HPF가 이 책에서 등장한다.

## 이 책을 다 읽고 나면

- frequency response가 무엇인지 그래프로 그려서 설명할 수 있다.
- biquad 1개의 식을 보고 b/a 계수가 어디서 왔는지 짐작할 수 있다.
- Direct Form II Transposed가 왜 권장되는지 한 줄로 설명할 수 있다.
- Peaking EQ를 freq=1000, gain=+6dB, Q=1로 두면 어떤 곡선이 나오는지 그릴 수 있다.
- 실시간 EQ에서 사용자가 freq 손잡이를 돌렸을 때 왜 그 값을 그대로 콜백에 던지면 안 되는지 설명할 수 있다.
