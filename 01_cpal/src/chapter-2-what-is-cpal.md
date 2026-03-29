# Chapter 2 - cpal이 해결하는 문제

## `cpal`은 무엇인가

`cpal`은 `Cross-Platform Audio Library`다.

Rust에서 오디오 장치 I/O를 다룰 때, 플랫폼별 차이를 가능한 한 공통 API로 감싼 저수준 라이브러리라고 보면 된다.

비유하면 이런 역할이다.

```text
Linux   -> ALSA / JACK / PipeWire / PulseAudio
Windows -> WASAPI / ASIO
macOS   -> CoreAudio

cpal -> 위 백엔드 차이를 공통 Rust API로 감쌈
```

## `cpal`이 하는 일

```text
✓ 출력 장치 열기
✓ 입력 장치 열기
✓ 장치 설정 조회
✓ 스트림 생성
✓ 오디오 콜백 연결
✓ 샘플 버퍼를 하드웨어로 전달
```

## `cpal`이 하지 않는 일

```text
✗ 오디오 파일 디코딩
✗ EQ / compressor / limiter DSP
✗ 플레이리스트 관리
✗ 모듈 레지스트리 관리
✗ 프로젝트 저장/복원
```

즉 `cpal`은 오디오 워크스테이션 전체가 아니라, "하드웨어와 샘플 버퍼가 만나는 지점"을 담당한다.

## RuStudio 관점에서 왜 먼저 공부해야 하나

`Phase 1`의 `mod_player`는 아래 경계를 정확히 이해해야 한다.

- `cpal`은 장치와 스트림을 연다.
- `mod_player`는 재생 상태와 transport를 관리한다.
- `dsp-core`는 공통 DSP 인터페이스를 제공한다.
- `mod_mastering`은 버퍼를 실제로 처리한다.

이 경계를 모르고 시작하면 `cpal` 코드에 DSP, UI 상태, 모듈 교체 로직이 섞이기 쉽다.

## 여기서 반드시 이해해야 할 것

- `cpal`은 저수준 오디오 I/O 레이어다.
- `cpal`은 장치와 스트림을 다루지만, 플레이어 아키텍처 전체를 만들어주지 않는다.
- `Phase 1`에서 `cpal`을 공부하는 목적은 API 사용법 암기가 아니라 경계 설정이다.
