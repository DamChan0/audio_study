# Chapter 11 - 빠른 복습 요약

## 핵심 흐름

```text
1. host 얻기
2. output device 얻기
3. output config 읽기
4. output stream 만들기
5. play 호출
6. callback에서 버퍼 채우기
```

## 핵심 원칙

```text
콜백은 실시간 스레드다.
블로킹, 할당, IO는 피한다.
채널 수와 샘플레이트를 하드코딩하지 않는다.
stream 생명주기를 명확히 소유한다.
```

## Phase 1 체크리스트

```text
□ 440Hz 사인파 출력 성공
□ channels 동적 처리
□ sample_rate 동적 처리
□ play/pause 제어 경로 설계
□ DSP chain 삽입 포인트 설계
□ 오디오 스레드 규칙 위반 없음
```

## 다음 단계

이제 `cpal` 자체 학습은 1차 완료다.

다음으로 넘어가면 좋은 주제는 아래 둘 중 하나다.

- `mod_player` 아키텍처 스케치
- `mod_mastering`을 위한 dB / RMS / compressor 기초 수식
