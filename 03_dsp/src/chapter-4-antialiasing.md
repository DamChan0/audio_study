# Chapter 5 - 앨리어싱과 PolyBLEP

이 챕터의 목표: **단순 Saw/Square 가 왜 "지저분하게 들리는지"** 를 주파수 관점에서 설명하고, **PolyBLEP** 로 완화한다.

## 왜 문제가 생기나

Saw wave 를 `2 * phase - 1` 로 바로 만들면 **위상이 1 에서 0 으로 떨어지는 순간이 불연속** 이다. 수학적으로 이 불연속은 무한대 고주파 성분을 포함한다.

Nyquist 주파수 (`sample_rate / 2`) 를 넘는 성분은 샘플링 과정에서 **되접혀서 (aliasing)** 가청대역으로 내려온다. 그래서 고음이 이상하게 찍힌다.

```text
진짜 Saw 스펙트럼 : ... 만큼 하모닉이 무한히 있음
실제 샘플링된 Saw : Nyquist 위 하모닉이 되접혀 내려옴 → 불쾌한 고음
```

## 해결 아이디어: 불연속을 부드럽게

- **무겁게 해결**: 모든 하모닉을 Nyquist 아래로 제한 (band-limited). BLIT / BLEP 같은 이론적 방법.
- **가볍게 해결**: 불연속 지점 **근처 2~4 샘플만 보정**. → **PolyBLEP** (Polynomial BLEP).

PolyBLEP 은 수식이 짧고 실시간에 무리가 없다. 표준적인 선택이다.

## PolyBLEP 개요

Saw wave 기준으로, 불연속이 일어나는 phase=0 주변에서 보정항을 더한다.

```text
dt = phase_inc   (= freq / sr)

if phase < dt:
    t = phase / dt
    correction = t + t - t*t - 1
elif phase > 1 - dt:
    t = (phase - 1) / dt
    correction = t*t + t + t + 1
else:
    correction = 0

saw = (2*phase - 1) - correction
```

수식 자체는 위키피디아/DSP 교과서에 있다. **암기할 필요는 없다**. 중요한 건:

- 보정은 **불연속 지점 ±dt 범위에서만** 적용된다.
- `dt` 는 주파수가 높을수록 커진다. 고주파일수록 보정 범위가 넓어진다 (직관과 맞다).

## Square 로 확장

Square wave 는 불연속이 두 군데(0, 0.5) 있다. 각각에 PolyBLEP 을 더하면 된다.

## 이 챕터에서 할 일

- `examples/src/oscillator/saw.rs` 에 **PolyBLEP 버전**을 추가 (기존 단순 버전은 지우지 않고 비교용으로 남겨도 됨).
- `examples/src/oscillator/square.rs` 에 **PolyBLEP 버전** 추가.
- `examples/src/bin/play_osc.rs` 에서 단순/PolyBLEP 를 전환해 A/B 비교.

## 답할 질문

- PolyBLEP 을 켰을 때 저주파 (예: 50Hz) 에서 음색 차이가 거의 없는 이유는?
- 왜 고주파 (2kHz 이상) 에서 차이가 극적인가?
- Triangle wave 는 PolyBLEP 이 필요한가, 필요 없는가?
- FFT 로 단순 Saw 와 PolyBLEP Saw 의 스펙트럼을 비교하면 무엇이 보일까?

## 완료 기준

- 2kHz Saw 를 비교했을 때 PolyBLEP 버전이 더 "매끄럽게" 들린다.
- 저주파에서는 차이가 미미하다는 점도 확인.
- (선택) Phase 2-3 FFT 구현 후 다시 돌아와서 스펙트럼으로 차이를 눈으로 확인.
