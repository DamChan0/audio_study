# Chapter 7 - 예제용 crate 만들기와 Cargo.toml

이 장은 `cpal` 예제를 직접 만들고 실행하기 전에 반드시 정리해야 하는 부분이다.

## 1. 이름을 두 개로 나눠서 생각해야 한다

`Cargo.toml`에는 보통 아래 두 종류의 이름이 나온다.

```text
1. 내 패키지 이름
2. 의존성 crate 이름
```

### 내 패키지 이름

이건 자유롭게 정하면 된다.

예:

- `cpal-practice`
- `cpal-sine-lab`
- `rustudio-cpal-playground`

즉 `[package].name`은 네 연습용 프로젝트 이름이다.

### 의존성 crate 이름

이건 반드시 `cpal`이다.

```toml
[dependencies]
cpal = "0.15"
```

코드에서 `use cpal::traits::HostTrait;`처럼 쓰는 바로 그 이름이 의존성 이름이다.

## 2. 왜 여기서는 `cpal = "0.15"`를 쓰나

공식 docs.rs 최신 버전은 더 높을 수 있다.

하지만 현재 `RuStudio` 프로젝트 컨텍스트는 `cpal 0.15`를 기준으로 정리되어 있다. 그래서 학습 단계에서는 먼저 프로젝트 기준 버전에 맞추는 것이 좋다.

```toml
[dependencies]
cpal = "0.15"
```

이렇게 해야 문서, 예제, 프로젝트 맥락이 서로 덜 어긋난다.

## 3. 가장 단순한 학습용 `Cargo.toml`

440Hz 사인파 같은 최소 예제는 보통 아래 정도면 충분하다.

```toml
[package]
name = "cpal-practice"
version = "0.1.0"
edition = "2021"

[dependencies]
cpal = "0.15"
anyhow = "1"
```

의미는 이렇다.

- `cpal`: 장치, 스트림, 콜백 API
- `anyhow`: 학습용 예제에서 에러 전파를 단순하게 만들기 위한 도구

`anyhow`는 필수는 아니지만, 초반에는 에러 타입 설계보다 오디오 흐름 이해가 더 중요하므로 같이 쓰는 편이 편하다.

## 4. JACK을 실험하고 싶을 때

기본 학습은 feature 없이 시작하는 편이 좋다.

그래도 JACK 백엔드를 직접 써보고 싶다면 이렇게 둔다.

```toml
[dependencies]
cpal = { version = "0.15", features = ["jack"] }
anyhow = "1"
```

여기서 주의할 점:

- Rust feature를 켠다고 끝나지 않는다.
- Linux에서 JACK 개발 패키지와 실행 환경이 있어야 한다.
- 공식 README 기준으로 Linux에서는 ALSA 개발 파일도 필요한 경우가 있다.

즉 첫 번째 학습 예제는 기본 설정으로 시작하고, JACK은 두 번째 실험 주제로 넘기는 편이 안전하다.

## 5. 실제로 연습용 crate를 어떻게 시작하나

가장 단순한 흐름은 이렇다.

```bash
cargo new cpal-practice
cd cpal-practice
```

그리고 `Cargo.toml`을 아래처럼 잡는다.

```toml
[package]
name = "cpal-practice"
version = "0.1.0"
edition = "2021"

[dependencies]
cpal = "0.15"
anyhow = "1"
```

즉 다시 정리하면:

- 패키지 이름은 자유
- 의존성 이름은 `cpal`
- 학습용 최소 조합은 `cpal + anyhow`

## 6. RuStudio 관점에서 어떤 방식이 좋은가

학습 단계에서는 보통 두 방식 중 하나가 좋다.

### 방식 A - 완전히 분리된 연습용 crate

- 440Hz 사인파 같은 작은 실험에 좋다.
- 실제 workspace 구조를 더럽히지 않는다.
- 실패해도 부담이 적다.

### 방식 B - RuStudio 문서와 같이 가는 별도 연습용 crate

- 문서와 실험을 같이 보기 쉽다.
- 나중에 `mod_player` 설계로 연결하기 좋다.

현재 단계에서는 보통 방식 A가 가장 가볍다.

## 체크리스트

- `[package].name`과 `[dependencies].cpal`의 차이를 설명할 수 있는가?
- 왜 초반에는 feature 없이 시작하는 편이 좋은지 설명할 수 있는가?
- 왜 현재 학습 문서는 `cpal = "0.15"` 기준인지 설명할 수 있는가?
- JACK feature를 켰는데도 바로 안 될 수 있는 이유를 설명할 수 있는가?
