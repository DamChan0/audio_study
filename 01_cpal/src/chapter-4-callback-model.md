# Chapter 4 - 오디오 스트림과 콜백 모델

## 왜 콜백이 핵심인가

`build_output_stream()`에는 보통 두 종류의 콜백이 들어간다.

- 데이터 콜백: 실제 오디오 버퍼를 채우는 실시간 경로
- 에러 콜백: 스트림 에러를 알리는 별도 경로

이 중에서 실시간 제약이 가장 강하게 걸리는 것은 **데이터 콜백**이다.

```rust
let stream = device.build_output_stream(
    &config,
    move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
        for sample in data.iter_mut() {
            *sample = 0.0;
        }
    },
    move |err| {
        eprintln!("stream error: {err}");
    },
    None,
)?;
```

위 예제의 `eprintln!`은 **에러 콜백** 안에 있다. 이건 오디오 버퍼를 직접 채우는 데이터 콜백과는 구분해서 봐야 한다.

## `cpal`은 pull 모델에 가깝다

```text
오디오 하드웨어 -> "버퍼 필요" -> 콜백 호출
우리 코드 -> 버퍼 채움 -> 반환
하드웨어 -> 그 버퍼를 재생
```

즉 애플리케이션이 임의 타이밍에 밀어넣는 구조가 아니라, 장치가 필요할 때마다 콜백을 호출하는 구조라고 이해하는 편이 맞다.

## 데이터 콜백이 느리면 무슨 일이 생기나

- glitch
- crackle
- underrun
- 재생 타이밍 붕괴

이건 단순 성능 저하가 아니라 바로 "들리는 오류"로 나타난다.

## `data: &mut [f32]`는 무엇인가

장치가 이번 호출에서 채워달라고 요청한 출력 버퍼다.

예를 들어 스테레오라면 대체로 이런 식으로 본다.

```text
data = [L0, R0, L1, R1, L2, R2, ...]
```

그래서 채널 단위로 보려면 보통 이렇게 순회한다.

```rust
let channels = config.channels() as usize;

for frame in data.chunks_mut(channels) {
    for ch in frame.iter_mut() {
        *ch = 0.0;
    }
}
```

## 반드시 이해해야 할 것

- 데이터 콜백이 곧 오디오 엔진의 실시간 실행 지점이다.
- `data`는 프레임 버퍼가 아니라 인터리브된 샘플 슬라이스일 수 있다.
- `channels`를 하드코딩하면 나중에 바로 깨진다.
- 에러 콜백과 데이터 콜백을 같은 성격으로 보면 규칙을 잘못 적용하게 된다.
