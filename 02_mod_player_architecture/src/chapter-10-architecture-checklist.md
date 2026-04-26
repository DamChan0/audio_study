# Chapter 10 - 설계 체크리스트

이 장은 02 책의 닫는 장이다. 책에서 다룬 내용을 종이 한 장짜리 체크리스트로 정리한다.

이 체크리스트가 채워지면 mod_player 모듈 코드를 본격적으로 짜기 시작해도 된다.

## A. 책임 정의 (Chapter 2, 3)

```text
□ mod_player의 책임을 한 문단으로 적었다
□ "mod_player가 하는 일" 6개 정도를 나열했다
□ "mod_player가 하지 않는 일" (DSP / 디코딩 / UI / MIDI / plugin host) 명시했다
□ 4개 계층(UI / mod_player / DSP / cpal) 그림을 그렸다
□ 책임이 모호한 항목은 "어디에 둘지 모르겠을 때 판단표"로 분류했다
```

## B. Stream 소유권 (Chapter 4)

```text
□ Stream을 mod_player의 Option<cpal::Stream> 필드로 두기로 결정
□ 새 Stream 만들기 전에 옛것을 명시적으로 drop하는 정책 명시
□ Stream을 mutex로 감싸지 않는다 (콜백은 Stream 자체를 안 본다)
□ 콜백 closure가 캡처할 것과 mod_player가 들고 있을 것을 두 묶음으로 분리
□ Send / Sync 사정을 확인 (Stream이 !Send인 플랫폼 고려)
```

## C. Transport 상태 (Chapter 5)

```text
□ TransportState enum: Stopped / Playing / Paused / Rebuilding / NoDevice
□ Rebuilding은 previous 상태를 들고 있는다
□ TransportCmd enum: Play / Pause / Resume / Stop / SetSource / DeviceChanged ...
□ 모든 전이를 한 match 함수로 모았다
□ 잘못된 전이는 panic이 아니라 Result::Err
□ 콜백 안에서는 transport를 직접 보지 않고 atomic 한 비트만 본다
```

## D. UI ↔ Audio thread 통신 (Chapter 6)

```text
□ atomic / SPSC ring / ArcSwap의 사용처를 결정했다
□ audio thread 경로에 mutex 없음을 확인
□ AudioCommand enum 설계: 작고 할당 없는 명령들
□ 옛 객체 drop을 audio thread가 떠안지 않도록 회수 채널 마련
□ UI ↔ mp 표 (peak/RMS/position/event 등)를 한 번 정리
□ SPSC 큐 가득 차는 정책 결정
```

## E. Config 협상 (Chapter 7)

```text
□ sample rate 우선순위 [48000, 44100, 96000, ...] 결정
□ sample format 우선순위 [F32, I16, U16] 결정
□ channels 우선순위 [2, 1] 결정
□ buffer size 우선순위 [Fixed(256), Fixed(128), Default] 결정
□ NegotiatedConfig 추상화로 cpal 타입을 외부로 흘리지 않음
□ SR / channels 변경 시 chain을 통째로 재생성하는 정책
□ 협상 실패 시 NegotiationLog로 사용자에게 보여줄 정보 보존
```

## F. DSP Chain (Chapter 8)

```text
□ AudioModule trait: Send + process(&mut self, buf, channels)
□ DspChain = Vec<Box<dyn AudioModule>>, audio thread 단독 소유
□ chain 교체는 SPSC 명령 + 회수 채널 패턴
□ master gain은 chain 마지막 단 모듈로
□ 모듈 bypass 토글 시 cross-fade
□ 콜백 안에서 chain.modules.push 절대 안 함
□ source는 별도 trait (AudioSource), chain 앞단에 위치
```

## G. Device Lifecycle (Chapter 9)

```text
□ 6가지 시나리오 각각의 transport 전이 정의
   (시작 시 장치 없음 / 변경 / 사라짐 / 다시 나타남 / build 실패 / stream 에러)
□ 에러 콜백은 SPSC 채널에 이벤트만 push, 상태 변경은 mod_player main flow
□ Rebuilding 재시도 횟수 제한 + backoff
□ LastError 구조체로 마지막 에러 정보 보존
□ NoDevice는 예외가 아니라 정상 경로
□ device hot-plug 정책 (자동 폴링 / 사용자 버튼) 결정
```

## H. 검증 산출물

```text
□ mod_player 책임 정의 1문단 (markdown 또는 코드 주석)
□ stream ownership 메모
□ transport 상태 전이표 (markdown table)
□ thread boundary 다이어그램 (ASCII art 또는 그림)
□ failure state 목록 + 사용자 메시지 초안
□ 콜백 안에서 절대 하지 않을 것 목록 (실시간 규칙)
□ AudioCommand enum 초안
□ NegotiatedConfig / DspChain / TransportState 초안 코드
```

## 작은 검증 시나리오

이 모듈이 제대로 설계됐는지 확인할 수 있는 빠른 사고 실험.

```text
1. 사용자가 play 누름 → 첫 샘플이 나가기까지 어느 thread가 무엇을 하나?
2. EQ band freq를 빠르게 돌릴 때 콜백 안에서는 무엇이 매 샘플 일어나는가?
3. USB 오디오 인터페이스를 뽑으면 1초 안에 어떤 상태가 표시되어야 하나?
4. 두 mod_player 인스턴스를 같은 장치에 만들면 어떻게 되나?
5. 콜백 안에서 println! 한 줄을 추가하면 청취 시 어떻게 들리나?
```

다섯 가지 시나리오에 막힘 없이 답이 나오면 설계가 견고하다.

## 다음 책으로 넘어가는 다리

다음 책은 `03_dsp_fundamentals`다.

이 책에서 만든 DSP chain의 자리에 들어갈 **모듈 본체**를 만든다. oscillator, gain, envelope, delay buffer — 그 6개 빌딩 블록이 03 책의 주제다.

```text
01 cpal              : 어떻게 소리가 나가는가
02 mod_player        : 누가 그 흐름을 관리하는가         ← 이 책
03 dsp_fundamentals  : 그 흐름 안 콜백에서 무엇을 계산하는가
```

## 한 줄 결론

> mod_player는 Stream + transport + lifecycle의 모듈이다. 이 책의 결과물은 코드가 아니라 **종이 한 장짜리 설계도 + 명확한 경계 + 안전한 thread 통신 표**다. 그 위에 03 책의 부품들이 얹힌다.
