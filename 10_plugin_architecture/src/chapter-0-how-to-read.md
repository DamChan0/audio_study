# Chapter 1 - 이 책을 읽는 방법

## 핵심 관점 — "경계가 무엇을 강제하는가"

플러그인은 단순한 함수 호출이 아니다. host와 plugin은 **다른 실행 컨텍스트**에 있다.

```text
host (DAW): 자기 스레드, 자기 audio engine, 자기 transport
plugin    : host가 만들어준 호출 시점에만 깨어남
            host가 준 buffer만 만질 수 있음
            host가 준 parameter 값만 알 수 있음
```

이 분리가 plugin format이 강제하는 거의 모든 규칙의 근원이다. 매 챕터에서 의식할 질문.

```text
1. 이 데이터/제어는 host가 주나, plugin이 주나?
2. 이 호출은 어느 스레드에서 일어나나? (audio / UI / loader)
3. 이 동작이 host 입장에서 안전한가? (block, allocate, panic)
4. 이걸 어떻게 두 곳(RuStudio 내부 / 플러그인 빌드) 모두 동작하게 짤 것인가?
```

## 한 장 그림 — host ↔ plugin 인터페이스

이 책 전체가 결국 이 그림이다.

```text
[ Host (DAW) ]
   │  audio buffer in
   │  MIDI events
   │  parameter values
   │  transport info
   │
   ▼
[ Plugin.process(buf, events, params) ]
   │  audio buffer out
   │  parameter "gesture" output (UI knob 움직임)
   │
   ▼
[ Host: 다른 트랙들과 합성, 출력 ]
```

이 인터페이스의 양 끝에 두 종류 코드가 있다.

```text
host쪽 코드  : DAW(host)가 작성. 우리는 안 만짐.
plugin쪽 코드: 우리가 작성. 제약을 지켜야 한다.
```

## 추천 독서 순서

1. `Chapter 2 - Plugin이란 무엇인가` — 단독 실행 파일이 아닌 이유, 호스팅 모델.
2. `Chapter 3 - Host와 Plugin 경계` — 누가 무엇을 책임지나.
3. `Chapter 4 - Parameter Model과 Automation` — parameter의 강한 계약.
4. `Chapter 5 - Process Callback 사고방식` — 실시간 규칙 (cpal과의 비교).
5. `Chapter 6 - Focused Lab` — minimum gain plugin 한 개로 모든 단계를 만지기.
6. `Chapter 7 - 복습`.

## 핵심 사실 한 줄

```text
plugin의 process()는 cpal 콜백의 동급 함수다.
실시간 제약, lock-free, no-allocation이 그대로 적용된다.
```

01 ~ 09 책에서 익힌 모든 실시간 규칙이 여기서도 똑같이 작동한다. 다른 점은 buffer를 host가 준다는 것뿐이다.

## 학습 완료 기준

이 책을 다 읽고 나면 아래 질문에 답할 수 있어야 한다.

- 플러그인은 왜 .so / .dll / .vst3 같은 동적 라이브러리 형태인가?
- host의 책임 3가지와 plugin의 책임 3가지를 구분해 말할 수 있는가?
- parameter automation과 일반 변수의 차이를 설명할 수 있는가?
- plugin이 process() 안에서 절대 하지 말아야 할 일 다섯 가지를 들 수 있는가?
- nih-plug에서 parameter를 추가하는 단계를 골격으로 적을 수 있는가?
- 같은 EQ 처리를 RuStudio 내부와 플러그인 두 형태로 빌드하려면 코드를 어떻게 나눌 것인가?
