// 예제 02 — Transport 상태기계
//
// ── 핵심 멘탈 모델 ─────────────────────────────────────────────
// Stream은 "한 번"만 만든다.        ← build_output_stream() 호출 1회
// 그 안의 클로저는 OS가 "여러 번" 부른다. ← 콜백, 1초에 수백 회
//
// 식당 비유:
//   - Stream 생성       = 식당 영업 시작 (한 번)
//   - 클로저            = 등록된 레시피
//   - 콜백 호출         = 손님이 들어올 때마다 그 레시피로 요리 (계속)
//   - drop(Stream)      = 영업 종료
//
// 우리는 "stream을 멈췄다 켰다" 하지 않는다. 영업은 계속 중.
// 단지 콜백 안에서 active 플래그를 보고 "이번 손님에게 사인파를
// 줄지 / 빈 접시(0.0)를 줄지" 만 결정한다.
// ────────────────────────────────────────────────────────────

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal_examples::AudioProcess;
use cpal_examples::sources::sin_sound::SinSound;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportState {
    Stopped,
    Playing,
    Paused,
}

struct ModPlayer {
    // stream을 self에 보관하는 유일한 이유: drop을 막아서 "영업 유지".
    // 읽지는 않아서 컴파일러가 dead_code 경고를 낼 수 있다 (의도된 것).
    #[allow(dead_code)]
    stream: cpal::Stream,

    state: TransportState,

    // 콜백과 main이 공유하는 "지금 출력해야 하는가" 1비트 플래그.
    // 콜백이 매번 이걸 load해서 출력 데이터를 결정한다.
    active: Arc<AtomicBool>,

    // 콜백이 자기가 몇 번 불렸는지 기록. main이 가끔 들여다본다.
    // 이게 학습 목적: "stream은 1번 만들었는데 콜백은 N번 불린다"를 눈으로 본다.
    callback_count: Arc<AtomicU64>,
}

impl ModPlayer {
    pub fn new() -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("no output device available"))?;
        let supported = device.default_output_config()?;

        let sample_rate = supported.sample_rate().0 as f32;
        let channels = supported.channels() as usize;
        let mut stream_config: cpal::StreamConfig = supported.into();
        stream_config.buffer_size = cpal::BufferSize::Fixed(2048); // OS가 알아서 버퍼 사이즈 결정하게 한다.

        println!(
            "[init] sample_rate={} Hz, channels={}, buffer={:?}",
            sample_rate as u32, channels, stream_config.buffer_size
        );

        let mut sin_source = SinSound::new(sample_rate, channels, 488.0, 0.5);

        let active = Arc::new(AtomicBool::new(false));
        let callback_count = Arc::new(AtomicU64::new(0));

        let active_cb = Arc::clone(&active);
        let count_cb = Arc::clone(&callback_count);

        // ── 여기서 stream 1번 만든다. 이 줄은 평생 단 한 번만 실행된다. ──
        let stream = device.build_output_stream(
            &stream_config,
            // ── 이 클로저는 OS 오디오 스레드가 ~5.3ms마다 부른다 ──
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // 1) 호출 횟수 증가. 학습용이라 콜백 안에서 atomic 한 번 더 쓴다.
                //    실전에선 이런 카운터도 빼는 게 깔끔.
                count_cb.fetch_add(1, Ordering::Relaxed);

                // 2) 02 예제의 핵심: active 플래그 보고 출력 결정.
                //    여기서 stream을 만들거나 조작하지 않는다. data 버퍼만 채울 뿐.
                if active_cb.load(Ordering::Relaxed) {
                    sin_source.process(&[], data);
                } else {
                    data.fill(0.0);
                }
            },
            |err| eprintln!("stream error: {}", err),
            None,
        )?;

        // 영업 시작. 이후로 stream에 대한 직접 호출은 없다 (drop 제외).
        stream.play()?;

        Ok(Self {
            stream,
            state: TransportState::Stopped,
            active,
            callback_count,
        })
    }

    pub fn play(&mut self) -> anyhow::Result<()> {
        // Stopped/Paused 모두에서 Playing으로 진입 허용.
        // 이 메서드는 stream을 건드리지 않는다. 그냥 active=true 만 알린다.
        self.state = TransportState::Playing;
        self.active.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn pause(&mut self) -> anyhow::Result<()> {
        // 잘못된 전이는 panic 아닌 무시.
        if matches!(self.state, TransportState::Stopped) {
            return Ok(());
        }
        self.state = TransportState::Paused;
        self.active.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.state = TransportState::Stopped;
        self.active.store(false, Ordering::Relaxed);
        // 위치 reset hook 자리.
        Ok(())
    }

    pub fn state(&self) -> TransportState {
        self.state
    }

    pub fn callback_count(&self) -> u64 {
        self.callback_count.load(Ordering::Relaxed)
    }
}

fn main() -> anyhow::Result<()> {
    let mut player = ModPlayer::new()?;

    println!();
    println!("commands: p=play, a=pause, s=stop, c=show callback count, q=quit");
    println!("[player] state={:?}", player.state());

    let stdin = io::stdin();
    let mut buf = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        buf.clear();
        if stdin.lock().read_line(&mut buf)? == 0 {
            break; // EOF (Ctrl+D)
        }

        match buf.trim() {
            "p" | "play" => player.play()?,
            "a" | "pause" => player.pause()?,
            "s" | "stop" => player.stop()?,
            // 'c' 입력 시 콜백 누적 호출 횟수를 보여준다.
            // 처음 'c' 누른 시각 이후 얼마나 자주 콜백이 불렸는지 비교 가능.
            "c" | "count" => {
                println!(
                    "[player] callback called {} times so far",
                    player.callback_count()
                );
                continue;
            }
            "q" | "quit" => break,
            "" => continue,
            other => {
                println!("unknown command: {}", other);
                continue;
            }
        }

        println!(
            "[player] state={:?}, callback_count={}",
            player.state(),
            player.callback_count()
        );
    }

    println!("[player] dropping stream.");
    // main 끝 → player drop → stream drop → "영업 종료" → 콜백 호출 멈춤.
    Ok(())
}
