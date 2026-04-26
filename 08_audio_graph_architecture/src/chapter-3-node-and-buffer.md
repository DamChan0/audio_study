# Chapter 3 - AudioNode와 AudioBuffer

## 1. 한 장 정의

```text
AudioNode   : "buffer를 받아 buffer를 만드는" 처리 블록의 추상화
AudioBuffer : 그 buffer 자체. 채널 수, 길이, 데이터.
```

이 두 가지가 graph의 모든 노드와 모든 edge를 표현한다.

## 2. AudioNode의 최소 인터페이스

DAW 그래프 trait는 이 정도가 시작점이다.

```rust
pub trait AudioNode: Send {
    /// 한 사이클의 buffer 처리.
    fn process(&mut self, input: &AudioBuffer, output: &mut AudioBuffer);

    /// 이 노드의 latency (처리 결과가 출력에 나오는 데 걸리는 샘플 수).
    fn latency_samples(&self) -> usize { 0 }

    /// graph reset (transport stop 등에서 호출).
    fn reset(&mut self) {}
}
```

각 메소드 의미.

```text
process()        : 매 콜백 사이클(또는 하위 사이클) 마다 호출.
                   input을 읽고 output에 채운다.
latency_samples(): 0이 기본. delay/look-ahead가 있는 노드만 양수 반환.
reset()          : 내부 상태 초기화. 곡 처음/seek/사용자 reset 시 호출.
```

03 ~ 07 책의 모든 처리 블록은 이 trait의 한 구현이 된다.

```text
SineOscNode      : input 무시, output에 sine 채움
EqNode           : input → biquad chain → output
CompressorNode   : input → envelope → gain → output
DelayNode        : input → ring buffer → mix → output
MixerNode        : 여러 input → 합산 → output
```

## 3. 단일 입출력 vs 다중 입출력

위의 trait 시그니처는 단일 input, 단일 output이다. 그런데 mixer는 N입력 1출력이다. 어떻게 표현할까?

방법 두 가지.

```text
A. trait에 inputs/outputs 슬라이스를 받게 한다
   fn process(&mut self, inputs: &[AudioBuffer], outputs: &mut [AudioBuffer]);

B. trait는 단일 in/out, 합산은 graph가 노드 앞 단계에서 한다
   여러 source의 출력 buffer를 더해서 mixer 노드의 입력 buffer 한 개에 모아둔다
```

학습 단계에서는 A가 더 일반적이고, 모든 DAW 엔진(JUCE, nih-plug, web audio)이 A 모델이다.

```rust
pub trait AudioNode: Send {
    fn process(
        &mut self,
        inputs:  &[&AudioBuffer],
        outputs: &mut [AudioBuffer],
    );

    fn num_inputs(&self) -> usize  { 1 }
    fn num_outputs(&self) -> usize { 1 }

    fn latency_samples(&self) -> usize { 0 }
    fn reset(&mut self) {}
}
```

이게 이 책 기본 모델이다.

## 4. AudioBuffer 구조

```rust
pub struct AudioBuffer {
    channels: usize,
    frames:   usize,
    data:     Vec<f32>,    // length = channels * frames (interleaved)
}
```

또는 planar.

```rust
pub struct AudioBuffer {
    channels: Vec<Vec<f32>>,    // channels[c][frame]
}
```

대부분의 DAW 엔진은 **planar**를 쓴다. 채널별로 분리된 처리 (EQ는 채널 별 biquad, compressor는 stereo-linked detector 등) 가 자연스럽다.

이 책에서는 planar AudioBuffer를 가정한다. cpal 콜백 진입/탈출 시점에 interleaved ↔ planar 변환을 한 번씩 한다.

## 5. Buffer 소유권 — 누가 들고 있나

graph 모델에서 가장 까다로운 부분이다. 두 가지 모델.

### 모델 A — 노드가 자기 출력 buffer를 소유

```text
각 노드: AudioBuffer output_buffer

process() 호출 시 자기 buffer를 채움.
다음 노드는 이전 노드의 output_buffer를 빌려 input으로 본다.
```

장점: 단순. 메모리 할당이 노드 단위로 명확.
단점: 한 buffer가 여러 destination으로 가면 빌려주기가 복잡.

### 모델 B — graph가 buffer pool을 소유

```text
graph: Vec<AudioBuffer> pool
각 edge에 buffer 한 개 할당 (또는 재사용)
처리 순서대로 적절한 buffer를 input/output으로 노드에 넘김
```

장점: zero-copy 시도 가능. 분기/합산 자연스러움.
단점: 구현 복잡.

학습 단계에서는 A로 시작하는 게 일반적이다. RuStudio도 처음에는 A 모델이 자연스럽다.

## 6. Buffer 크기는 어떻게 정하나

cpal 콜백이 한 번에 N 프레임을 요청하면, graph도 그 N에 맞춰 처리하는 게 표준이다.

```text
cpal 콜백 받음 (frames = N)
  ↓
graph가 모든 internal buffer를 frames = N으로 채움
  ↓
output 노드의 buffer를 cpal로 다시 interleave해서 넘김
```

대안은 graph 내부 buffer 크기를 고정(예: 64 또는 128)하고, cpal 콜백 size와 다를 때 쪼개거나 합치는 식. 이건 multi-rate 그래프 같은 advanced 영역.

## 7. Latency reporting

같은 graph 안에 latency가 다른 노드들이 섞여 있을 수 있다.

```text
[Source A] → [direct master]                  latency 0
[Source A] → [look-ahead limiter] → [master]  latency 256
```

이 두 path가 master에서 합쳐질 때, 같은 시간의 신호가 다른 시간에 도착한다. delay 보상 (PDC: Plugin Delay Compensation)이 필요하다.

```text
모든 path의 latency를 측정 → 최대값을 기준으로
짧은 path에 그 차이만큼의 delay 노드를 추가
```

각 노드의 `latency_samples()`가 이 계산의 입력이다. 이 책은 단순히 "이런 메소드가 있다, 이런 일에 쓰인다"까지만 본다. 실제 PDC 구현은 별도 주제.

## 8. Stateful 노드의 reset 시점

```text
콜백 시작/매 사이클: reset 안 함 (state가 곡선의 일부)
transport stop      : 모든 노드 reset 권장
seek               : reset (decoder + DSP 모두)
사용자가 트랙 추가  : 그 트랙 노드만 새로 생성
사용자가 노드 교체  : 새 노드로 교체 + cross-fade (또는 그냥 리셋)
```

03 책 envelope, 04 책 envelope follower, 05 책 biquad의 z1/z2가 모두 reset 대상.

## 9. RuStudio 관점

```text
mod_player 단계 (단순 chain)
  → 노드 trait 없이 직선 sequence로 충분

mixer + send/return 단계 (graph 도입 시점)
  → AudioNode trait 도입
  → 각 처리 블록을 노드 구현체로 개별 packaging

플러그인 호스팅 단계 (10 책)
  → 외부 플러그인을 한 노드로 wrap
  → trait 구현으로 graph에 끼워 넣기
```

## 자주 하는 실수

- planar/interleaved 가정을 노드마다 다르게 → buffer 변환 누락으로 채널 섞임.
- buffer pool 모델을 처음부터 시도 → 복잡도 폭발. A 모델로 시작 권장.
- latency_samples()를 무시하고 PDC 적용 안 함 → look-ahead limiter 들어갔을 때 트랙들 시간 어긋남.
- reset()을 매 콜백마다 호출 → 신호가 매번 끊김.
- AudioBuffer 크기가 노드마다 다른데 일치 검증 없음 → 이상한 위치에서 panic.

## 반드시 이해해야 할 것

- AudioNode = process / latency / reset의 trait. 모든 처리 블록의 공통 인터페이스.
- AudioBuffer는 보통 planar. cpal과의 경계에서 interleave 변환.
- buffer 소유권 모델은 "노드가 자기 출력 buffer 소유" 모델로 시작이 단순.
- latency 다른 path들의 시간 정렬은 PDC. `latency_samples()`가 그 입력.
