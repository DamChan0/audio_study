# Chapter 5 - Process Callback 사고방식

## 1. Plugin process()는 cpal 콜백의 동급 함수

이 책 전체에서 가장 중요한 한 줄.

```text
plugin.process(buffer)
  ≅
cpal callback (data: &mut [f32], _info)
```

같은 종류의 함수다. 같은 종류의 규칙이 적용된다.

```text
✗ Mutex::lock()
✗ Vec::push, Box::new (할당)
✗ println!, file IO, network
✗ async runtime, sleep
✗ panic / unwrap (가능한 한)
✗ syscall (timer 외)
```

```text
✓ atomic 읽기/쓰기
✓ lock-free queue (try_pop)
✓ 미리 할당된 buffer/state 사용
✓ 매 sample 단위 처리 (cache-friendly)
```

plugin format은 다르지만, 실시간 audio 코드의 규칙은 같다.

## 2. nih-plug의 process() 시그니처

```rust
fn process(
    &mut self,
    buffer:  &mut Buffer,
    aux:     &mut AuxiliaryBuffers,
    context: &mut impl ProcessContext,
) -> ProcessStatus {
    // 처리 ...
    ProcessStatus::Normal
}
```

각 인자.

```text
buffer  : main audio in/out (in-place, mut 슬라이스 포함)
aux     : sidechain input, etc.
context : transport info, MIDI events, parameter info
```

## 3. Buffer 다루기

`Buffer`는 channel-major (planar) 슬라이스 모음으로 노출된다.

```rust
// 모든 sample 순회 (channel별로 같은 처리)
for channel_samples in buffer.iter_samples() {
    let gain = self.params.gain.smoothed.next();
    for sample in channel_samples {
        *sample *= gain;
    }
}
```

또는 channel별로 별도 처리.

```rust
for (ch_idx, channel) in buffer.as_slice().iter_mut().enumerate() {
    let biquad = &mut self.biquads[ch_idx];
    for sample in channel.iter_mut() {
        *sample = biquad.process(*sample);
    }
}
```

cpal은 interleaved, plugin host는 보통 planar. 둘이 일관 안 되는 점에 주의.

## 4. Sample-accurate MIDI 처리

context로부터 MIDI events를 timestamp와 함께 받는다.

```rust
let mut next_event = context.next_event();
for (sample_idx, channel_samples) in buffer.iter_samples().enumerate() {
    while let Some(event) = next_event {
        if event.timing() != sample_idx as u32 { break; }
        synth.handle_midi(&event);
        next_event = context.next_event();
    }
    // 이 sample 처리
    let s = synth.next_sample();
    for ch in channel_samples {
        *ch = s;
    }
}
```

09 책에서 본 frame-offset 패턴 그대로다. nih-plug가 timestamp 정렬과 큐잉을 미리 처리해 준다.

## 5. Transport info 사용

```rust
fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, context: &mut impl ProcessContext) -> ProcessStatus {
    let transport = context.transport();
    let bpm = transport.tempo.unwrap_or(120.0);
    let position = transport.pos_samples().unwrap_or(0);
    let playing = transport.playing;
    
    // ...
}
```

BPM-synced delay, sequencer-aware effect 등에 사용.

## 6. Latency 보고

```rust
fn initialize(&mut self, audio_io: &AudioIOLayout, buffer_config: &BufferConfig, context: &mut impl InitContext) -> bool {
    context.set_latency_samples(self.lookahead_samples);
    true
}
```

시간이 가는 동안 latency가 변할 수 있는 경우 (look-ahead 켜기/끄기) `context.set_latency_samples()`를 process() 안에서도 호출 가능.

## 7. ProcessStatus

process()의 반환값은 host에게 plugin 상태를 알린다.

```text
Normal               : 정상 처리됨
Tail(samples)        : 이번 buffer 후 reverb tail이 N sample 더 남음 (host가 process 계속 호출)
KeepAlive            : 입력 없어도 계속 호출 (synth에서 release 처리)
```

reverb나 delay 같은 처리는 input이 끝나도 tail이 있어서 여러 buffer 동안 더 출력이 나온다. host에게 이 상태를 알리면 host가 적절히 알아서 처리한다.

## 8. CPAL 콜백과의 차이점

같은 실시간 규칙 안에서도 약간씩 다른 점.

```text
                  cpal callback              plugin process()
buffer 형태       interleaved                planar
buffer 길이       device-determined          host-determined (보통 가변)
sample rate       device-determined          host-determined
parameter 통신    SPSC + atomic (자체)       framework가 제공 (nih-plug)
MIDI input        별도 SPSC                  context.next_event()
transport info    없음 (자체 관리)           context.transport()
latency report    없음                       set_latency_samples()
state save        없음                       framework가 처리
```

다른 부분은 다 framework(nih-plug)가 처리해 준다. 우리가 짜는 건 처리 자체다.

## 9. 같은 DSP 코드 재사용

03 ~ 09 책에서 만든 DSP 블록을 plugin process()에 그대로 넣을 수 있다.

```rust
// 03 책의 SineOsc, Adsr
// 04 책의 EnvFollower, Compressor
// 05 책의 Biquad, EqChain
// 06 책의 fft 분석은 보통 별 thread (cpal 같음)
// 07 책의 file I/O는 plugin에선 보통 안 씀

// plugin의 process() 안:
for channel_samples in buffer.iter_samples() {
    let cutoff = self.params.cutoff.smoothed.next();
    self.eq.set_cutoff(cutoff);
    for s in channel_samples {
        *s = self.eq.process(*s);
    }
}
```

핵심: DSP 블록 자체는 plugin/host 무관. 둘 다 동일한 실시간 규칙을 따르므로 같은 코드가 양쪽에서 동작한다.

## 10. 활성화 / 비활성화

```rust
fn initialize(&mut self, ...) -> bool { /* 시작 시 한 번 */ }
fn reset(&mut self) { /* state 0으로 */ }
```

`reset()`이 매우 중요하다. host가 transport stop 후 재생, 또는 sample rate가 바뀐 후 process()가 다시 시작될 때 호출된다. IIR z 상태, ring buffer, envelope state 등을 0으로.

## 자주 하는 실수

- process() 안에서 Vec::push, Box::new → plugin이 host audio glitch.
- parameter 값을 매 sample 직접 atomic load → smoothing 없으니 click.
- MIDI event timestamp 무시 → 모든 event를 buffer 시작에 적용 (5~10 ms timing 오차).
- reset() 누락 → seek 후 노이즈.
- ProcessStatus::Tail 처리 누락 → reverb 끝이 갑자기 잘림.
- planar/interleaved 가정 혼동 → cpal과 plugin host에서 처리 차이를 일관성 없이 다룸.

## 반드시 이해해야 할 것

- plugin.process()는 cpal 콜백과 동급. 모든 실시간 규칙이 동일.
- nih-plug가 buffer / parameter / MIDI / transport / state 모두 standardize. 우리는 처리만.
- DSP 블록은 plugin과 RuStudio 내부에서 그대로 재사용 가능.
- reset()은 transport 변화 / sample rate 변화 시 모든 state를 0으로 돌리는 hook.
- ProcessStatus::Tail은 reverb 같은 처리의 끝을 host에게 알리는 신호.
