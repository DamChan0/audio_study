# Chapter 6 - 자주 하는 실수와 복습

## FFT 일반

- bin 인덱스를 그대로 Hz로 사용 (실은 `k · fs / N`).
- N을 2의 거듭제곱이 아닌 값으로 두기 → 일부 구현에서 느림.
- 실수 신호인데 일반 FFT로 N개 bin 다 처리 → RealFft로 절반 비용.
- 복소수 결과를 그대로 시각화 → magnitude 계산 누락.

## Window / STFT

- window 미적용 → 한 톤이 broadband로 leakage.
- window coherent gain 보정 누락 → magnitude가 일관되게 절반.
- hop = N으로 overlap 없음 → frame 경계의 신호 누락.
- 매 frame마다 화면을 갱신 → 60 fps 모니터에선 의미 없는 부담.

## dBFS / 시각화

- log10(0) → NaN. floor 누락.
- 가로축을 선형 bin으로 매핑 → 저음역 사라짐. 로그 매핑이 표준.
- 세로축을 magnitude linear로 → 작은 값이 안 보임. dB가 표준.
- smoothing을 audio 콜백 안에서 처리 → 비싼 작업.
- spectrum 배열을 매 frame `new`로 할당 → 미리 할당된 두 버퍼 재사용.

## 처리 단계 입출력 상태표

```text
ring buffer SPSC      input: 샘플       output: 샘플          state: 인덱스 두 개
window 적용           input: N 샘플     output: N 샘플        state: 미리 만든 window 배열
FFT (rustfft)         input: N 샘플     output: N/2+1 복소수  state: rustfft plan (재사용)
magnitude → dB        input: 복소수     output: dB 배열        state: 없음
smoothing             input: dB 배열    output: dB 배열        state: bin별 1차 IIR
peak hold             input: dB 배열    output: peak 배열      state: bin별 max + 떨어지는 속도
UI mapping            input: dB 배열    output: 픽셀           state: 화면 width/height
```

## Phase 5 체크리스트

```text
□ N 샘플 FFT의 출력 개수와 bin width를 즉시 계산할 수 있다
□ 실수 신호일 때 bin이 절반만 의미 있다는 사실을 안다
□ Hann window를 안 곱하면 spectrum이 어떻게 보일지 예측할 수 있다
□ STFT의 hop과 overlap의 의미를 그림으로 설명할 수 있다
□ magnitude → dBFS 변환의 한 줄 식을 외운다 (`20 · log10(max(m, 1e-9))`)
□ 가로축을 로그 매핑하는 이유를 설명할 수 있다
□ smoothing과 peak hold가 시각화 단계의 도구라는 점을 안다
□ audio 콜백 / FFT 분석 / UI 렌더링 셋이 별 스레드라는 패턴을 그릴 수 있다
□ 1톤 예제로 N=2048 vs N=4096 차이를 직접 봤다
```

## 03 ~ 05 책 도구의 재사용 지도

```text
03 phase accumulator     : 테스트 톤 / sweep 생성
03 ring buffer (SPSC)    : audio thread → analysis thread 다리
03 envelope follower     : spectrum bin smoothing의 1차 IIR
04 RMS / peak meter      : analyzer 옆에 nominal level 표시용
05 K-weighting / EQ      : spectrum 표시 + EQ 곡선 비교
05 Smoothed parameter    : spectrum smoothing과 같은 1차 IIR (다만 bin별 배열)
```

## 다음 책으로 넘어가는 다리

다음 책은 `07_audio_file_io`다.

지금까지의 입력은 cpal 실시간 캡처 또는 합성 신호였다. 07에서는 WAV / MP3 / FLAC 같은 파일을 디코딩해 같은 콜백 골격에 흘려 넣는 방법을 다룬다.

또한 07 책의 `rubato`(샘플레이트 컨버전)는 spectrogram dump를 다른 sample rate에서 만들 때나, 24 kHz Nyquist를 넘는 측정용 업샘플 분석에 직접 쓰인다.

## 한 줄 요약

> FFT는 N 샘플을 N/2+1개 주파수 bin으로 옮기는 도구다. 시각화 파이프라인은 콜백 → ring buffer → window+FFT → magnitude→dB → smoothing/peak hold → 로그 매핑 → 화면 픽셀이다. 각 단계는 별 스레드에 분리한다.
