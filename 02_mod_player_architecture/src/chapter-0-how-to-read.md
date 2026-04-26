# Chapter 1 - 이 책을 읽는 방법

이 책은 `cpal` API 레퍼런스 정리가 아니라 **아키텍처 사고 훈련**에 가깝다.

01 책에서는 "스트림이 어떻게 동작하는가"를 봤고, 이 책에서는 그 스트림을 둘러싼 **앱 구조**를 본다.

## 매 장에서 답할 4개 질문

각 장을 읽을 때 다음 4개를 매번 묻는다.

```text
1. 이 책임은 어느 계층에 있어야 하나?
2. 이 상태는 UI가 소유해야 하나, audio thread가 소유해야 하나?
3. 이 변경은 실시간 경로 안에서 해도 되나?
4. 실패하면 어느 상태로 표현해야 하나?
```

이 4개는 사실상 이 책 전체의 골격이다. 책을 다 읽고 나면 자기 코드를 보면서 자동으로 묻게 된다.

## 이 책에서 기대하는 산출물

이 책을 다 읽은 뒤에는 최소한 아래 결과물이 손에 있어야 한다.

```text
□ mod_player 책임 정의 1문단
□ stream ownership 메모 (누가 Stream을 들고 있나)
□ transport 상태 전이표 (Stopped / Playing / Paused / Rebuilding / NoDevice)
□ thread boundary 다이어그램 (UI ↔ audio thread 사이 데이터 흐름)
□ failure state 목록 (장치 없음 / 빌드 실패 / 콜백 에러 등)
```

이건 코드가 아니라 **종이 위 한 장짜리 설계도**다. 코드는 그 다음에 나온다.

## 추천 독서 순서

```text
Chapter 2  - mod_player는 무엇인가              ← 이 모듈의 정의부터
Chapter 3  - 책임 경계와 계층 분리              ← cpal/dsp/ui와 어떻게 나누나
Chapter 4  - Stream 소유권                     ← Stream을 누가 들고 있나
Chapter 5  - Transport 상태 모델                ← play / pause / stop의 상태기계
Chapter 6  - UI ↔ Audio thread 제어 경로        ← 두 스레드 사이 통신 도구
Chapter 7  - Config 협상과 Fallback             ← 장치별 SR / 채널 / 버퍼
Chapter 8  - Realtime-safe DSP Chain            ← DSP 체인을 어떻게 안전하게 끼우나
Chapter 9  - Device Lifecycle과 실패 상태       ← 장치 빠짐 / 변경에 어떻게 반응
Chapter 10 - 설계 체크리스트                    ← 닫는 장
```

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- `mod_player`의 책임을 한 문장으로 말할 수 있는가?
- `Stream`을 UI 스레드에서 만드는 게 왜 위험한가, 또는 왜 괜찮은가?
- play 누름 → 실제 첫 샘플이 출력될 때까지 무슨 일이 일어나는가?
- 사용자가 EQ 슬라이더를 돌렸을 때 그 값이 콜백까지 어떻게 전달되는가?
- 장치가 갑자기 사라졌을 때 UI에는 무엇이 보여야 하는가?
- Stream을 다시 만들어야 할 때 transport 상태는 어떻게 표현해야 하는가?

이 질문들에 답이 나오면 코드 작성에 들어가도 된다.
