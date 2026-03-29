# Chapter 9 - 실습 예제: DSP 체인 삽입 구조

이 장은 `cpal` API 학습을 `RuStudio mod_player` 설계로 연결하는 구간이다.

## 핵심 아이디어

```text
오디오 장치 -> cpal callback -> source 생성 -> DSP chain -> output
```

## 축약 예제

```rust
pub trait AudioProcessor: Send {
    fn process(&mut self, buffer: &mut [f32], sample_rate: u32);
}

move |data: &mut [f32], _| {
    while let Ok(cmd) = cmd_rx.try_recv() {
        // play/pause 같은 제어 명령 반영
    }

    // 1. source 생성
    // 2. chain 순회
    for processor in chain.iter_mut() {
        processor.process(data, sample_rate);
    }
}
```

## 여기서 중요한 점

### 1. 콜백은 실행기다

콜백은 버퍼 처리 실행기 역할만 하는 편이 좋다.

- source 생성
- 명령 반영
- chain 처리

### 2. 구조 변경은 콜백 밖에서 준비한다

다음 같은 작업은 콜백 밖에서 생각해야 한다.

- 새 모듈 추가
- 모듈 제거
- 모듈 순서 재구성

왜냐하면 실시간 스레드에서 구조 변경은 비용이 크고 예측이 어렵기 때문이다.

### 3. `mod_player`와 `dsp-core` 경계

- `mod_player`: 스트림, transport, control 메시지
- `dsp-core`: 공통 trait
- `mod_mastering`: 구체 DSP 로직

## 설계 전에 적어볼 질문

- `Vec<Box<dyn AudioModule>>`는 누가 소유하는가?
- UI가 파라미터를 바꾸면 어떤 경로로 오디오 스레드에 전달되는가?
- 모듈 on/off는 atomic 값으로 충분한가, 메시지가 필요한가?
