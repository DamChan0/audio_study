# Chapter 10 - 자주 하는 실수와 디버깅

## 실수 1. `Stream`을 바로 drop

```rust
fn play_sound() {
    let stream = device.build_output_stream(/* ... */).unwrap();
    stream.play().unwrap();
}
```

함수 종료와 함께 `stream`이 drop되면 소리도 즉시 멈춘다.

## 실수 2. 샘플레이트 하드코딩

```rust
let phase_increment = 2.0 * PI * 440.0 / 44100.0;
```

장치 샘플레이트가 48000Hz면 바로 잘못된 주파수가 된다.

## 실수 3. 채널 수를 2로 고정

모노 장치에서 인덱스 에러가 날 수 있다.

## 실수 4. 콜백 안에서 lock/할당/로그

작동은 할 수 있어도 glitch 원인이 된다.

## Linux 체크리스트

```bash
aplay -l
pactl list sinks
fuser /dev/snd/*
speaker-test -t sine -f 440
alsamixer
```

## 오디오가 안 날 때 확인 순서

1. 기본 출력 장치가 실제로 존재하는가?
2. 장치 설정 조회가 성공하는가?
3. 콜백이 실제로 호출되는가?
4. `Stream`이 살아 있는가?
5. 볼륨/뮤트/시스템 오디오 서버 상태가 정상인가?
