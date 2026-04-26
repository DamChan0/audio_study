# Chapter 4 - ADSR 엔벨로프 상태머신

이 챕터의 목표: **Note On / Note Off 이벤트에 따라 진폭이 자연스럽게 변하는 엔벨로프를 구현한다.**

## ADSR 이란

```text
amplitude
   ^
 1 |   /\
   |  /  \
   | /    \________
   |/              \
   +---------------->  time
   | A | D |  S  | R |
```

- **A**ttack: 0 → 1 로 올라가는 시간
- **D**ecay: 1 → sustain 레벨로 떨어지는 시간
- **S**ustain: Note Off 가 올 때까지 유지할 레벨 (0~1)
- **R**elease: sustain → 0 으로 떨어지는 시간

Attack / Decay / Release 는 **시간(초)**, Sustain 은 **레벨(0~1)** 이다. 단위가 섞여 있다는 걸 기억.

## 상태머신으로 보는 법

```text
Idle --NoteOn--> Attack --done--> Decay --done--> Sustain
                                                     |
                                                     NoteOff
                                                     |
                                                     v
                                                   Release --done--> Idle
```

어느 상태든 **NoteOn 은 Attack 으로 전이**, **NoteOff 는 Release 로 전이**. (예외: 이미 Idle 이면 NoteOff 는 무시)

## 왜 상태머신으로 써야 하나

"경과 시간"만 들고 계산하면 안 되나? 안 된다.

- Release 중에 NoteOn 이 다시 오면 **현재 레벨에서 Attack 을 이어받아야** 클릭이 안 생긴다.
- Attack 중에 NoteOff 가 오면 **Attack 을 끊고 바로 Release 로**.
- Sustain 레벨이 실시간으로 바뀔 수 있다.

이 모든 경우는 "현재 상태 + 현재 레벨" 두 개로만 다뤄진다. 그래서 상태머신이 맞다.

## 구현 전략: 선형 vs 지수

단순하게는 `level` 을 매 샘플마다 고정 step 으로 증감시킨다 (선형).

```rust
fn tick(&mut self) -> f32 {
    match self.state {
        State::Attack => {
            self.level += self.attack_step;
            if self.level >= 1.0 {
                self.level = 1.0;
                self.state = State::Decay;
            }
        }
        // ...
    }
    self.level
}
```

실제 아날로그 엔벨로프는 지수(exponential) 곡선이다. 선형 → 지수로 바꾸는 건 나중 과제로 남겨둔다.

## 이 챕터에서 구현할 것

- `examples/src/envelope/adsr.rs`: 상태머신 + 선형 램프
- `examples/src/bin/adsr_blip.rs`: 1 초마다 NoteOn/NoteOff 반복하며 sine × envelope 재생

## 답할 질문

- `State::Release` 에서 `level` 이 정확히 0 이 되는 시점을 어떻게 감지하는가? 부동소수점 비교의 위험은?
- NoteOn → NoteOff 가 같은 샘플에 들어오면 어떻게 처리할 것인가?
- Attack 시간이 0 이면 step 계산에서 무슨 일이 생기는가? 방어 코드는 어디에 두어야 하는가?
- Sustain 레벨을 실시간으로 바꾸면 Sustain 중인 샘플은 어떻게 따라가야 자연스러운가? (스무딩)

## 완료 기준

- NoteOn/NoteOff 가 번갈아 들어올 때 클릭 없이 자연스럽게 소리가 난다.
- Release 중 NoteOn 이 다시 들어와도 이상한 점프가 없다.
- 엔벨로프 파라미터 (A/D/S/R) 를 바꿔 들었을 때 체감 변화가 명확하다.
