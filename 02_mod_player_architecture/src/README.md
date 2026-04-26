# 들어가며

이 문서는 `RuStudio Phase 1`에서 `cpal` 다음 단계로 넘어가 **`mod_player` 아키텍처**를 학습하기 위한 mdBook이다.

이 책의 목적은 코드를 바로 많이 쓰는 것이 아니라, 아래 흐름을 먼저 머릿속에서 구조로 정리하는 것이다.

```text
cpal callback 이해
-> stream ownership 결정
-> transport state 설계
-> UI -> audio thread 제어 경계 설계
-> source -> DSP chain -> output 구조 확정
-> failure state / rebuild 전략 정리
```

## 이 책에서 말하는 `mod_player`

이 책의 `mod_player`는 **RuStudio 내부의 재생 담당 모듈**이다.

- `MOD Player` 같은 tracker 포맷 재생기가 아니다.
- `cpal` 위에 올라가는 앱 구조 계층이다.
- DSP 자체를 구현하는 책도 아니다.

## 이 책을 다 읽고 나면

- `cpal`과 `mod_player`의 경계를 설명할 수 있다.
- `Stream`을 누가 소유해야 하는지 설계 관점에서 설명할 수 있다.
- play / pause / stop / rebuild 상태를 상태기계처럼 정리할 수 있다.
- atomic / channel / ring buffer를 어떤 용도로 나눌지 판단할 수 있다.
- `mod_mastering` 같은 DSP 모듈이 어느 경계 뒤에 붙어야 하는지 설명할 수 있다.

## 추천 읽기 흐름

1. `이 책을 읽는 방법`
2. `mod_player는 무엇인가`
3. `책임 경계와 계층 분리`
4. `Stream 소유권`
5. `Transport 상태 모델`
6. `UI와 Audio Thread 제어 경로`
7. `Config 협상과 Fallback`
8. `Realtime-safe DSP Chain 구조`
9. `Device Lifecycle과 실패 상태`
10. `설계 체크리스트`
