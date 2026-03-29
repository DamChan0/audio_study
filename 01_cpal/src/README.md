# 들어가며

이 문서는 `RuStudio Phase 1`에 들어가기 전에 `cpal`을 순차적으로 학습하기 위한 mdBook이다.

목표는 단순 API 요약이 아니라, 아래 흐름이 자연스럽게 머리에 들어오게 만드는 것이다.

```text
cpal이 하는 일 이해
-> Host / Device / Stream 구조 이해
-> callback과 실시간 제약 이해
-> 예제용 crate와 Cargo.toml 준비
-> 440Hz 사인파 예제로 최소 성공 기준 이해
-> DSP 체인 삽입 구조를 mod_player 관점에서 사고
```

이 책은 `cpal_guide.md`의 학습 스타일을 mdBook 구조로 재정리한 버전이다.

## 이 책을 다 읽고 나면

- `cpal`의 `Host -> Device -> Stream` 모델을 설명할 수 있다.
- 오디오 콜백이 왜 일반 애플리케이션 코드와 다른 규칙을 따르는지 설명할 수 있다.
- `mod_player`를 만들 때 재생 제어와 DSP 체인을 어디까지 오디오 스레드에 넣어야 하는지 판단할 수 있다.
- `Phase 1`의 440Hz 사인파 출력이 왜 중요한 최소 성공 기준인지 설명할 수 있다.

## 추천 읽기 흐름

1. `이 책을 읽는 방법`
2. `cpal이 해결하는 문제`
3. `Host -> Device -> Stream`
4. `오디오 스트림과 콜백 모델`
5. `오디오 스레드 규칙`
6. `예제용 crate와 Cargo.toml`
7. `440Hz 사인파 예제`
8. `DSP 체인 삽입 구조`
9. `자주 하는 실수와 복습`
