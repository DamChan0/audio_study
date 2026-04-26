# Chapter 2 - DSP란 무엇인가

## 한 문장 정의

DSP는 **Digital Signal Processing**, 디지털 신호 처리다.

오디오 맥락에서는 더 좁게 말할 수 있다.

> 디지털 샘플 배열을 받아서 다른 샘플 배열로 바꾸는 계산.

전부다. 더 복잡하지 않다.

## 왜 cpal에서 끝내면 안 되는가

`01_cpal`에서 본 콜백을 다시 떠올려 보자.

```rust
move |data: &mut [f32], _info| {
    for sample in data.iter_mut() {
        *sample = 0.0;
    }
}
```

이 콜백이 채워야 하는 `data`는 결국 무음이다. `0.0`을 넣으면 아무 소리도 안 난다.

소리가 나려면 누군가는 그 자리에 **의미 있는 숫자**를 넣어야 한다. 그 "의미 있는 숫자를 만드는 일"이 DSP다.

```text
cpal       : 버퍼를 가져다 줌 / 채워진 버퍼를 받아 감
DSP        : 그 버퍼 안의 숫자를 결정함
mod_player : DSP를 어떻게 조합해서 콜백 안에 끼워 넣을지 결정함
```

이 셋을 섞기 시작하면 코드가 빠르게 무너진다. 그래서 셋을 분리해서 보는 감각을 먼저 갖는 것이 이 시리즈 전체의 기둥이다.

## DSP가 다루는 두 가지 축

DSP에서 어떤 처리든 결국 다음 두 축 중 하나(또는 둘 다)를 건드린다.

```text
amplitude (크기)  : 얼마나 큰 소리인가     → gain, envelope, compressor
time      (시간)  : 언제 소리가 나는가     → delay, reverb, oscillator phase
```

frequency(주파수)는 별도 축처럼 보이지만, 실제로는 "amplitude가 시간에 따라 어떻게 바뀌는가"의 결과다. 그래서 frequency를 다루는 EQ나 FFT도 결국은 amplitude와 time의 조합으로 환원된다.

이 책에서 다루는 6개 블록의 분류는 이렇다.

```text
amplitude 축 : gain, dB, pan, mixer, envelope
time 축      : oscillator (phase), delay buffer
```

## DSP에는 stateless와 stateful이 있다

이게 이 책에서 가장 중요한 분류다.

```text
stateless DSP : 출력이 현재 입력만으로 결정된다
                예) gain, dB 변환, pan, mixer

stateful DSP  : 출력이 현재 입력 + 과거 샘플 기억으로 결정된다
                예) oscillator (phase 기억), envelope (stage 기억),
                   delay (과거 N샘플 기억), 모든 IIR 필터
```

stateless는 단순하다. 그냥 함수다. 같은 입력이면 항상 같은 출력이 나온다.

stateful은 까다롭다. **상태가 어디에 살고, 누가 소유하고, 언제 reset되는가**가 핵심이다. 이 질문은 `02_mod_player_architecture`에서 본 transport 상태와 정확히 같은 종류의 질문이다.

## RuStudio 관점

RuStudio의 DSP는 다음 위치에 들어간다.

```text
mod_player 콜백
  └── source(들)            ← oscillator (이 책)
       └── DSP chain
            ├── gain         ← (이 책)
            ├── envelope     ← (이 책)
            ├── EQ           ← 05_eq_biquad
            ├── compressor   ← 04_mod_mastering_math
            └── delay/reverb ← (delay 자체는 이 책에서 시작)
       └── master gain
       └── output buffer (cpal)
```

즉 이 책의 6개 블록은 위 그림에서 가장 안쪽에 들어가는 부품들이다.

## 반드시 이해해야 할 것

- DSP는 cpal이 아니다. cpal은 버퍼를 옮기는 일이고, DSP는 그 버퍼 안 숫자를 만드는 일이다.
- 모든 DSP 블록은 amplitude / time 두 축 중 어느 쪽을 건드리는지로 먼저 분류해 보면 이해가 빠르다.
- DSP를 stateless인지 stateful인지로 먼저 묻는 습관을 들인다. 이 분류가 콜백 안에서 어디에 변수를 둘지 결정한다.
- "이 처리의 입력/출력/상태는 무엇인가?" — 이 질문 하나면 이 책의 거의 모든 챕터를 풀 수 있다.
