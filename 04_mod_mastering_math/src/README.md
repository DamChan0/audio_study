# 들어가며

이 책은 RuStudio 학습 시리즈의 네 번째 책이다.

`03_dsp_fundamentals`까지로 우리는 "샘플을 어떻게 만들고 합치는가"를 봤다. 이 책의 질문은 그 다음이다.

> 다 합쳐 놓은 결과물이 너무 시끄럽거나, 너무 들쭉날쭉하거나, 너무 조용할 때 어떻게 다스릴 것인가?

이게 **mastering**의 영역이다. RuStudio Phase 1의 메인 모듈인 `mod_mastering`이 다루는 주제다.

## mastering이 다루는 4가지 양

```text
peak     : 순간 최고 amplitude         "절대 이 선을 넘으면 안 된다"
RMS      : 시간 평균 amplitude         "평균 음량의 척도"
LUFS     : 사람 귀에 들리는 음량       "방송/스트리밍 표준 음량 단위"
true peak: 샘플 사이를 보간했을 때 peak  "디지털→아날로그 변환 후 실제 amplitude"
```

이 네 가지가 무엇이고 왜 다른지를 구분하는 것이 이 책의 절반이다.

나머지 절반은 **dynamics 처리** — compressor와 limiter — 다.

## 이 책이 다루는 처리 블록

```text
compressor : 큰 신호만 골라 줄이기 (=동적 범위 축소)
limiter    : 절대 어떤 임계도 못 넘게 강제 차단
true-peak  : 샘플 단위 peak가 아닌 보간 peak 측정
LUFS meter : ITU-R BS.1770 기준 음량 측정
```

이 네 개는 03 책에서 본 envelope, delay, gain의 응용이다. 새 알고리즘이 갑자기 등장하는 게 아니라, 같은 어휘로 만들어진 다른 단어다.

## 이 책의 범위와 한계

```text
✓ dB / peak / RMS / LUFS의 정의와 차이
✓ compressor / limiter의 블록 다이어그램과 코드 골격
✓ true-peak이 왜 sample-peak과 다른지
✓ ITU-R BS.1770 LUFS의 큰 그림 (정확한 표준 구현은 별도 작업)

✗ 완성된 mod_mastering 모듈 구현
✗ multi-band compressor
✗ 마스터링 워크플로우(=곡 단위 작업법)에 대한 음악적 조언
✗ 하드웨어 emulation
```

이 책을 마치면 mod_mastering의 **블록 다이어그램**을 종이에 그릴 수 있게 된다. 실제 모듈 구현은 그 다이어그램이 손에 들어온 다음 단계다.

## 이 책을 다 읽고 나면

- peak, RMS, LUFS, true peak 네 단어를 한 줄씩으로 구분해서 설명할 수 있다.
- dB의 0 dBFS가 무엇을 뜻하는지 정확히 말할 수 있다.
- compressor의 4개 파라미터(threshold, ratio, attack, release)가 무엇을 바꾸는지 설명할 수 있다.
- compressor와 limiter의 차이를 한 문장으로 설명할 수 있다.
- LUFS가 왜 단순 평균 amplitude가 아닌지 설명할 수 있다.
- mod_mastering의 처리 체인을 종이에 그릴 수 있다.
