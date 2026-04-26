# Chapter 8 - 자주 하는 실수와 복습

## 단위 / 측정

- dBFS 0 dB와 dB의 0을 혼용한다.
- "RMS는 -16 dB였다"고 윈도우 길이 없이 말한다.
- peak를 음량 척도로 사용 → transient에 휘둘림.
- log10(0)에서 NaN 발생 → `.max(1e-6)` 같은 floor 누락.

## Compressor

- threshold를 linear로 두고 dB와 비교.
- ratio = 0 → 0으로 나누기. ratio = 1이 "안 깎음"이다.
- envelope follower 출력을 신호에 그대로 곱함 → gain 계수가 아니라 envelope을 곱한 결과.
- attack/release 시간을 콜백 안에서 매 샘플 `exp` 호출.

## Limiter / True Peak

- ceiling을 0 dBFS로 두고 끝 → inter-sample 클립.
- look-ahead 없이 무한 ratio + 0 attack → 거친 saturator.
- look-ahead 길이만큼 신호 지연을 잊고 다른 트랙과 어긋남.
- sample-peak로 만족하고 true-peak 측정을 안 함.

## LUFS

- 단순 RMS와 같다고 생각.
- K-weighting biquad 계수를 새로 도출 (표준 값이 있다).
- Integrated를 곡 전체 단순 평균으로 계산 → 게이트 누락.
- M/S/I 중 하나만 보고 끝.

## 처리 블록 입출력 상태표

03 책의 분류를 그대로 적용한다.

```text
peak meter           input: 신호       output: 현재 peak       state: peak hold/release
RMS meter            input: 신호       output: 현재 RMS        state: 1차 IIR mean square
envelope follower    input: 신호       output: amp 추정값      state: state value
compressor           input: 신호       output: 신호 × gain     state: env follower 상태
limiter              input: 신호       output: 신호 × gain     state: env + delay buffer
LUFS meter           input: 신호       output: M/S/I LUFS      state: K-weighting 두 단 + 블록 버퍼
```

이 표를 그대로 머리에 넣으면 다음 책에서 EQ/biquad가 등장해도 같은 칸에 채워 넣을 수 있다.

## Phase 3 체크리스트

```text
□ peak / RMS / LUFS / true-peak를 한 줄씩 구분 가능
□ 0 dBFS의 정확한 의미를 설명 가능
□ compressor의 4파라미터가 신호를 어떻게 변형하는지 종이로 그림
□ compressor와 limiter의 차이를 한 문장으로 설명
□ inter-sample peak가 sample peak보다 높을 수 있다는 사실을 설명
□ K-weighting이 biquad 2개라는 사실을 알고 있음
□ Integrated LUFS의 게이트 절차를 설명 가능
□ mod_mastering 사슬 (compressor → makeup → limiter → meter) 그림 가능
```

## 03 책 블록의 재사용 지도

이 책의 처리들이 03 책에서 어떤 부품을 빌려쓰는지 다시 정리한다.

```text
03 oscillator     → 04에서는 등장 안 함 (input 신호 합성에만 쓰임)
03 gain/dB        → makeup gain, threshold 계산
03 envelope       → compressor / limiter envelope follower
03 delay buffer   → look-ahead limiter, true-peak 업샘플 지연
03 mixer          → 직접 등장 안 하지만, 사슬 끝에서 master bus가 곧 mixer 역할
```

새 알고리즘이 거의 없다는 사실을 다시 의식한다.

## 다음 책으로 넘어가는 다리

다음 책은 `05_eq_biquad`다.

이 책에서 만난 K-weighting의 두 단계 IIR 필터가 거기서 본격적으로 등장한다. RBJ cookbook 계수, biquad의 Direct Form II Transposed 구조, 클릭 노이즈 없는 파라미터 보간 — 전부 위 K-weighting의 배경을 더 깊게 본다.

또한 multi-band compressor를 만들려면 신호를 EQ로 N개 대역으로 쪼개는 일이 먼저 필요하다. 그 도구가 05 책이다.

## 한 줄 요약

> mastering은 "측정 + 동적 처리 + 안전망" 모듈이다. 모든 처리는 03 책의 envelope/gain/delay의 다른 조합이다. K-weighting은 biquad 두 개고, 그 biquad가 다음 책의 주제다.
