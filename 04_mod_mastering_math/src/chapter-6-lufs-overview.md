# Chapter 6 - LUFS와 Loudness 개요

이 장은 mastering의 마지막 측정 단위, **LUFS**의 큰 그림을 본다.

이 책에서는 LUFS를 직접 표준대로 구현하지는 않는다. 그건 별도 모듈 작업이다. 여기서는 "LUFS가 무엇이고 왜 RMS와 다른가"를 잡는다.

## 1. 왜 RMS가 부족한가

03/04 책에서 본 RMS는 amplitude 기반 평균이다. 문제가 두 가지 있다.

```text
1. 사람의 청각은 모든 주파수에 평등하지 않다.
   같은 amplitude라도 1 kHz와 100 Hz는 다른 음량으로 들린다.

2. 짧은 큰 소리(가령 짧은 페이지) 평균은 들리는 음량과 안 맞는다.
   고요한 구간이 너무 많으면 평균이 너무 낮게 잡힌다.
```

이 두 문제를 풀기 위해 ITU-R(국제전기통신연합 라디오통신부문)이 정의한 게 LUFS다.

## 2. LUFS = Loudness Units relative to Full Scale

핵심 개념 두 가지로 RMS를 보정한다.

```text
1. K-weighting filter
   사람이 더 잘 듣는 주파수 대역(중-고역)을 강조하는 2단 IIR 필터.

2. Gating
   너무 조용한 부분(절대 게이트 -70 LUFS 이하, 상대 게이트 -10 LU 이하)을
   평균에서 제외해서 "소리가 있는 구간"만 평균에 반영.
```

이 두 보정을 적용한 평균 음량이 LUFS다. 단위는 "LUFS"고, 두 LUFS 값의 차이는 "LU"라고 부른다.

## 3. K-weighting — 두 단계 IIR 필터

K-weighting은 두 개의 biquad가 직렬로 걸린 필터다.

```text
input ──► [ shelving filter (~+4 dB at 2 kHz) ]
              │
              ▼
          [ high-pass filter (~38 Hz) ]
              │
              ▼
          weighted signal → 그 다음 RMS 측정
```

이 두 단의 계수는 표준에 명시돼 있어서 그대로 쓰면 된다. 03 책의 필터 골격(z⁻¹ z⁻² 상태)을 가진 biquad 두 개다 — 즉 K-weighting은 **biquad 2개**일 뿐이다. 새 알고리즘이 아니다.

## 4. 측정 시간 윈도우 3가지

LUFS는 측정 시간에 따라 세 가지 종류가 있다.

```text
Momentary  (M)  : 400 ms 평균. 미터의 빠른 바늘.
Short-term (S)  : 3 s 평균. 보통 큰 미터의 메인 표시.
Integrated (I)  : 곡 전체 평균 (게이트 적용). 마스터링/방송 기준.
```

UI에서 본 LUFS 미터는 보통 이 셋을 동시에 표시한다.

## 5. 게이트(gating) — 조용한 부분 제외

Integrated LUFS의 핵심은 게이트다.

```text
1. 신호를 400 ms 블록으로 자른다 (75% overlap)
2. 각 블록에 K-weighted RMS 측정
3. 절대 게이트: -70 LUFS 이하 블록은 버린다 (= 무음 구간)
4. 임시 평균을 구한 뒤
5. 상대 게이트: 임시 평균 - 10 LU 이하 블록도 버린다
6. 남은 블록들의 평균이 Integrated LUFS
```

이 절차를 지키지 않으면 "조용한 도입부 30초가 평균을 끌어내려서 곡이 실제보다 부드럽게 측정"되는 일이 벌어진다.

## 6. 표준 출처

```text
ITU-R BS.1770   : LUFS 측정 알고리즘 정의 (K-weighting + gating + integration)
EBU R128         : 방송용 -23 LUFS / true peak ≤ -1 dBTP 운영 규격
스트리밍 플랫폼  : 보통 -14 LUFS 근처 정규화 (Spotify ≈ -14, Apple ≈ -16, YouTube ≈ -14)
```

플랫폼별 수치는 시기에 따라 바뀐다. 마스터링 작업할 때는 그때그때 확인하는 게 안전하다.

## 7. RuStudio LUFS 모듈의 구조

이 책 범위에서 만들 LUFS 모듈의 블록 다이어그램은 이렇다.

```text
input ──► [ K-weighting 단 1 (shelving) ]
              │
              ▼
          [ K-weighting 단 2 (HPF) ]
              │
              ▼
          [ 400 ms block buffer ]
              │
              ▼
          [ block mean square ]
              │
              ▼
          [ gate + sliding integration ]
              │
              ▼
          M / S / I 출력
```

거의 모든 부품이 03 책에서 본 도구다.

```text
biquad           : 05_eq_biquad에서 본격적으로 다룸
block buffer     : 03 delay buffer의 변형
mean square      : 03 RMS의 제곱 부분
sliding average  : 03 envelope의 1차 IIR
```

## 8. 실시간 vs 오프라인 측정

```text
실시간   : 매 콜백 마다 K-weighting → 블록 누적 → M/S/I 갱신
오프라인 : 곡 전체 K-weighting → 블록 분할 → 게이트 → I 한 번 계산
```

UI 미터는 실시간, 곡 전체 발송용 측정은 오프라인 둘 다 필요할 가능성이 높다. RuStudio에서 두 모드를 모두 고려해 설계해 두면 좋다.

## 자주 하는 실수

- LUFS를 그냥 "RMS인데 단위만 다른 것"으로 이해 → K-weighting과 gating을 빼먹게 된다.
- Integrated LUFS를 "곡 전체 단순 평균"으로 계산 → 조용한 구간 때문에 값이 이상하게 낮아짐.
- M/S/I 셋 중 하나만 보고 다른 두 가지를 잊음 → 사용자에게 잘못된 값으로 보여줌.
- True peak와 LUFS를 같은 개념으로 묶음 → 둘은 측정하는 양이 다르다 (peak vs 음량).
- K-weighting biquad 계수를 직접 새로 도출 → 표준 값이 정해져 있다. 그대로 쓴다.

## 반드시 이해해야 할 것

- LUFS는 K-weighting + gating으로 RMS의 두 약점(주파수 가중, 무음 평균)을 보정한 음량이다.
- K-weighting은 biquad 2개다. 새 종류의 필터가 아니다.
- Integrated LUFS는 블록 평균과 두 단계 게이트를 거친 결과다. 단순 곡 평균이 아니다.
- 마스터링 사슬의 끝(또는 limiter 뒤)에 LUFS 측정이 붙는다. 사용자/플랫폼 기준 음량을 맞추는 도구다.
