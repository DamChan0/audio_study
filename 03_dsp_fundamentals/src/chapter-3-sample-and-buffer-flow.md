# Chapter 3 - 샘플 단위 사고와 버퍼 흐름

이 장의 목표는 단 하나다.

> "샘플", "프레임", "버퍼", "채널" — 이 네 단어를 절대 헷갈리지 않게 한다.

여기를 대충 넘기면 뒤에 등장하는 모든 코드가 흔들린다.

## 1. 샘플(sample)

샘플은 **한 채널의 한 시점 amplitude 값**이다.

`f32` 한 개 = 한 샘플. 보통 `-1.0 ~ +1.0` 범위로 정규화된 값을 쓴다.

```text
sample = 한 채널 × 한 시점의 숫자 하나
```

## 2. 프레임(frame)

프레임은 **모든 채널의 같은 시점 샘플 묶음**이다.

```text
mono   1채널: frame = [s]
stereo 2채널: frame = [L, R]
5.1    6채널: frame = [L, R, C, LFE, Ls, Rs]
```

즉 프레임 하나가 "스피커가 한 번 출력하는 단위"라고 생각하면 된다.

샘플레이트 48 kHz는 정확히는 "프레임을 1초에 48,000번 출력한다"는 뜻이다. 채널 수가 2면 그 뒤에 `f32`가 96,000개 들어간다.

## 3. 버퍼(buffer)

버퍼는 **프레임을 시간순으로 나열한 슬라이스**다.

`cpal`의 콜백이 받는 `data: &mut [f32]`가 바로 그 버퍼다.

문제는 그 버퍼가 어떻게 들어 있느냐다.

### 인터리브(interleaved)

대부분의 PC 오디오 API에서 기본 형태다.

```text
data = [L0, R0, L1, R1, L2, R2, L3, R3, ...]
        |--frame 0--|--frame 1--|--frame 2--|
```

프레임 기준으로 묶여 있다. cpal도 이 형태로 준다.

### 채널별(planar / non-interleaved)

각 채널이 따로 산다.

```text
left  = [L0, L1, L2, L3, ...]
right = [R0, R1, R2, R3, ...]
```

오프라인 처리, FFT, plugin host, 일부 DAW 내부에서 더 편한 형태다.

이 책에서 cpal과 직접 만나는 코드는 항상 인터리브다. 둘이 섞이면 아주 빠르게 채널이 어긋난다.

## 4. 인터리브 버퍼를 안전하게 순회하는 패턴

채널 수를 하드코딩하지 않는 것이 핵심이다.

```rust
let channels = config.channels() as usize;

for frame in data.chunks_mut(channels) {
    // frame.len() == channels
    let s = next_sample();           // 같은 시점 샘플 하나 생성
    for ch in frame.iter_mut() {
        *ch = s;                     // 모든 채널에 동일 신호
    }
}
```

`chunks_mut(channels)`는 인터리브 버퍼를 프레임 단위로 끊어 준다. 이게 안전한 가장 기본 패턴이다.

스테레오로 좌우 다른 신호를 보내려면 이렇게 한다.

```rust
for frame in data.chunks_mut(channels) {
    let (l, r) = next_stereo_sample();
    if let [left, right, ..] = frame {
        *left = l;
        *right = r;
    }
}
```

## 5. 한 콜백이 처리하는 양

cpal 콜백 한 번에 들어오는 `data` 길이는 정해져 있지 않다. 장치/드라이버/버퍼 크기에 따라 달라진다.

```text
한 번 콜백 호출 = N 프레임 처리
N = data.len() / channels
```

DSP 코드는 N에 의존하지 않게 짠다. "이번 호출이 64 프레임이든 1024 프레임이든 같은 결과가 나오는가?" — 이 질문에 항상 yes여야 한다.

상태가 있는 처리(envelope, delay, oscillator)에서 이 조건이 깨지기 가장 쉽다.

## 6. 시간 진행은 "샘플 인덱스"가 한다

DSP에서 "지금 시간"은 시계가 아니라 **지금까지 처리한 프레임 수**다.

```text
elapsed_seconds = total_frames_processed / sample_rate
```

이 사실이 다음 장 oscillator의 위상 누산기로 바로 이어진다.

## 자주 하는 실수

- 샘플과 프레임을 혼용 → 채널 수가 바뀌면 폭발
- `data.len()`을 프레임 수라고 가정 → 채널 수만큼 어긋남
- 채널 수를 2로 하드코딩 → 모노/멀티채널 장치에서 깨짐
- "이번 콜백에서 sine 한 주기를 만들자" 같은 콜백-크기 의존 코드 → 콜백 크기가 다음 호출에 달라지면 끊김

## 반드시 이해해야 할 것

- 샘플은 숫자 하나. 프레임은 채널 수만큼 묶인 한 시점. 버퍼는 프레임의 시간순 나열.
- cpal이 주는 버퍼는 인터리브다. `chunks_mut(channels)`로 프레임 단위 순회를 기본기로 익힌다.
- 콜백 한 번이 처리하는 프레임 수는 가변이다. DSP 코드는 그 수에 의존하지 않게 짠다.
- "지금 시간"을 묻는 모든 DSP는 결국 "처리한 프레임 수"를 센다.
