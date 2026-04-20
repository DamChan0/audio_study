# Chapter 5 - 오디오 스레드 규칙

## 절대 원칙

여기서 말하는 "콜백"은 **오디오 데이터 콜백**을 뜻한다.

즉 `build_output_stream()`에 넘기는 `move |data: &mut [f32], ...| { ... }` 부분이다. 이 경로는 실시간 오디오 스레드로 취급해야 한다.

반면 에러 콜백은 별도 경로다. 에러 콜백도 가볍게 유지하는 것이 좋지만, 데이터 콜백과 동일한 실시간 규칙으로 단순 동일시하면 설명이 틀어질 수 있다.

### 데이터 콜백 내부 금지

```text
✗ Mutex::lock()
✗ Vec::push(), Box::new()
✗ println!, eprintln!
✗ 파일 읽기/쓰기
✗ async runtime 호출
✗ sleep
```

### 이유

```text
블로킹 / 할당 / 스케줄러 개입 / IO
-> 실행 시간 불확정
-> 버퍼 공급 지연
-> glitch 발생
```

## 허용 패턴

### 1. atomic 값 읽기

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let paused = Arc::new(AtomicBool::new(false));
let paused_in_audio = paused.clone();

move |data: &mut [f32], _| {
    if paused_in_audio.load(Ordering::Relaxed) {
        for sample in data.iter_mut() {
            *sample = 0.0;
        }
    }
}
```

### 2. non-blocking channel

```rust
use crossbeam_channel::bounded;

let (cmd_tx, cmd_rx) = bounded::<AudioCommand>(32);

move |data: &mut [f32], _| {
    while let Ok(cmd) = cmd_rx.try_recv() {
        // 상태 반영
    }
}
```

## RuStudio 연결 포인트

`mod_player`에서 `Play/Pause` 같은 transport 명령은 오디오 스레드에 안전하게 전달되어야 한다.

또한 `Vec<Box<dyn AudioModule>>` 같은 DSP 체인은 콜백 안에서 처리될 수 있어도, 그 구조 변경 자체는 콜백 밖에서 준비되어야 한다.

## 자주 하는 오해

- `Mutex`가 Rust에서 안전하니까 오디오에서도 안전하다 -> 아님
- 잠깐의 `println!` 정도는 괜찮다 -> 아님
- 테스트용 예제니까 콜백에서 뭐든 해도 된다 -> 나중에 구조가 망가짐
- 에러 콜백과 데이터 콜백은 어차피 둘 다 callback이니까 같은 규칙으로 보면 된다 -> 부정확함

## 정리

```text
강한 실시간 제약 대상 = 데이터 콜백
주의는 필요하지만 구분해서 봐야 하는 것 = 에러 콜백
```
