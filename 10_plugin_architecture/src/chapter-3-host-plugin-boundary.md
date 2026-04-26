# Chapter 3 - Host와 Plugin 경계

## 1. 두 쪽이 주고받는 것

```text
host → plugin                     plugin → host
─────────────────────────         ─────────────────────────
audio buffer (input)              audio buffer (output)
MIDI events (timestamped)         MIDI events (rare; e.g. arpeggiator)
parameter 값 (set)                parameter 변경 알림 (knob 돌림)
transport info (BPM, position)    
sample rate, max buffer size      latency 보고
sidechain audio (optional)        meter / visualizer 데이터 (optional)
```

이 표가 host-plugin 인터페이스의 거의 전부다.

## 2. Audio I/O — port와 channel

plugin은 자기에게 입력 port가 몇 개, 출력 port가 몇 개인지 host에게 알려야 한다.

```text
mono effect      : in 1ch → out 1ch
stereo effect    : in 2ch → out 2ch
synth (instrument): in 0ch → out 2ch (MIDI in 별도)
sidechain comp   : in 2ch (main) + 2ch (sidechain) → out 2ch
M/S processor    : in 2ch → out 2ch (단 채널 의미가 M/S)
```

host는 이 정보를 보고 트랙 라우팅을 결정한다. 예를 들어 stereo 트랙에 mono effect를 끼우면 host가 자동으로 채널을 처리한다 (DAW마다 다름).

## 3. MIDI / event input

instrument plugin은 MIDI input을 갖는다. 09 책의 timestamped event 모델 그대로다.

```text
plugin.process()가 받는 events:
  [(frame_offset, midi_message), (frame_offset, midi_message), ...]
  
plugin이 frame_offset을 보고 sample-accurate timing 적용
```

effect plugin은 보통 MIDI를 받지 않지만, modulation effect (sidechain trigger, key tracking 등) 는 받을 수 있다.

## 4. Parameter 통신 — 가장 까다로운 부분

parameter는 단순 변수가 아니다. host가 다음 모든 일을 할 수 있어야 한다.

```text
1. 사용자가 노브를 돌리면 plugin이 알림
2. 사용자가 자동화 곡선을 그리면 plugin이 매 sample 다른 값을 받음
3. project를 저장하면 plugin이 현재 parameter 값을 직렬화
4. project를 불러오면 plugin이 그 값을 복원
5. preset을 적용하면 한 번에 여러 parameter가 바뀜
6. 노브를 더블클릭하면 default 값으로 돌아감
```

이 모든 일이 동작하게 하려면 plugin parameter는 다음 메타데이터를 가져야 한다.

```text
- name              : "Threshold"
- short_name        : "Thr"
- range             : -60 dB ~ 0 dB
- default           : -20 dB
- unit              : "dB"
- value_to_string   : 0.5 (normalized) → "-12 dB" 같은 표시 변환
- string_to_value   : "-12 dB" → 0.5 같은 사용자 입력 파싱
- step              : (선택) 정수 step
- automatable       : 가능한가
```

이게 4장의 주제다.

## 5. Transport info — 곡 위치 공유

host는 매 process()마다 현재 transport 상태를 plugin에 줄 수 있다.

```text
- BPM
- bar / beat / tick 위치
- playing / stopped / recording 상태
- loop 활성 여부
- sample rate
- frame count (이번 buffer)
```

tempo-synced delay (bpm-synced) 같은 처리는 이 정보가 필수다. 이 정보 없이는 plugin이 "지금 곡 시작 후 몇 ms"를 모른다.

## 6. Latency 보고

look-ahead limiter처럼 처리에 N 샘플 지연이 생기면 plugin이 host에게 보고해야 한다.

```text
plugin.latency_samples() → host
  → host가 다른 트랙들에 같은 양의 delay 추가 (PDC)
  → 트랙들 시간 정렬 유지
```

08 책의 PDC 그대로다.

## 7. Sidechain — 두 번째 audio input

특수 plugin은 두 번째 audio input port를 갖는다. compressor의 sidechain input이 가장 흔한 사례.

```text
plugin process(main_in, sidechain_in, main_out)
  envelope follower: sidechain_in 기반
  gain 적용:        main_in × gain → main_out
```

host UI에서 "어느 트랙을 sidechain으로 보낼지"를 선택하게 한다.

## 8. State save / restore

project를 저장/복원할 때, host는 plugin의 모든 state(parameter 값 + 자체 internal state)를 직렬화한다.

```text
host: plugin.save_state() → bytes
host: project file 안에 그 bytes 저장

나중에:
host: bytes를 plugin.load_state(bytes) 로 복원
```

plugin은 이 두 함수를 정확히 구현해야 한다. 안 그러면 사용자가 작업 다음에 project를 다시 열었을 때 plugin이 default로 돌아간다.

`nih-plug`는 parameter 직렬화를 자동으로 해 준다. plugin 작성자는 추가 internal state(예: delay buffer 채워진 데이터)만 별 처리하면 된다.

## 9. Threading — 어느 스레드에서 무엇이 일어나나

```text
audio thread:
  - process()
  - 절대 block / allocate / IO 금지

UI thread:
  - GUI 그리기
  - 사용자 입력 → parameter 변경 (atomic으로 audio thread에)

loader / setup thread:
  - lifecycle (initialize, activate, deactivate)
  - 비교적 자유로움 (할당 OK)
```

이 셋의 분리는 cpal 콜백에서 본 규칙과 동일하다. plugin도 똑같이 따른다.

## 10. RuStudio가 plugin이 될 때 잃는 것

같은 처리 코드를 RuStudio 내부와 plugin으로 빌드하면, plugin 빌드에서는 다음을 못 한다.

```text
- audio device 직접 열기 (host가 함)
- 다른 트랙의 audio 직접 접근 (host의 라우팅으로만)
- 자기 transport (host의 transport에 종속)
- file I/O (UI thread에서만, 그것도 host 정책 따라)
```

RuStudio 내부 코드에서 위 기능들을 자유롭게 쓰면, plugin 변환 시 그 부분을 포기하거나 재구성해야 한다. 코드 분리가 중요한 이유.

## 자주 하는 실수

- plugin이 자기 audio 출력을 만든다고 가정 → host로 buffer를 반환하는 일이다.
- parameter 변경을 plugin이 직접 host에 push → 사용자 noise. host의 자동화 시스템과 충돌.
- transport info 없이 BPM-sync 처리 시도 → host가 안 주면 추정 못 함.
- latency_samples 보고 누락 → 다른 트랙과 시간 어긋남.
- state save에 internal buffer 누락 → project 복원 후 첫 buffer가 silence/노이즈.

## 반드시 이해해야 할 것

- 인터페이스는 audio I/O + MIDI + parameter + transport + state. 이 다섯이 host-plugin 경계의 거의 전부.
- parameter는 단순 변수가 아니라 메타데이터를 가진 강한 계약.
- audio thread / UI thread / loader thread의 분리는 cpal과 동일.
- RuStudio 내부 코드와 plugin 빌드 사이에 잃는 것을 의식하고 코드를 분리한다.
