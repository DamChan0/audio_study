// 예제 03 — 제어 채널 (Control Channel)
//
// ── 핵심 멘탈 모델 ─────────────────────────────────────────────
// 02에선 명령이 "있다/없다" 1bit(active) 한 개라 AtomicBool로 충분.
// 03부턴 명령 종류가 늘어난다 (Play / Pause / Stop / SetGain(f32)).
//   → atomic 여러 개로 쪼개지 않고, "한 큐에 enum으로" 흘려보낸다.
//
// 편지함 비유:
//   - bounded(64)         = 64통 들어가는 편지함
//   - tx.send(cmd)        = main이 편지를 넣음 (main thread가 발신자)
//   - rx.try_recv()       = callback이 한 통 꺼냄 (없으면 즉시 포기)
//   - while let Ok(cmd)   = 콜백 시작마다 편지함 싹 비움
//
// callback은 stream도 큐도 만들지 않는다.
// 들어온 명령만 보고 자기 내부 상태(active, gain)를 갱신하고,
// 그 상태에 따라 data buffer를 채운다.
//
// 콜백 안 금지: send / recv(blocking) / heap alloc / lock.
// 콜백 안 허용: try_recv / atomic load·store / sample 곱셈.
// ────────────────────────────────────────────────────────────
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal_examples::AudioProcess;
use cpal_examples::sources::sin_sound::SinSound;
use crossbeam_channel::{Sender, bounded};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Gain 한 틱 변경 단위. 1.4125 ≈ +3dB, 0.7079 ≈ -3dB.
// 한 틱을 ±6dB로 바꾸고 싶으면 1.9953 / 0.5012, ±1dB는 1.1220 / 0.8913.
const GAIN_STEP_UP: f32 = 1.4125;
const GAIN_STEP_DOWN: f32 = 0.7079;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AudioCommand {
    Play,
    Pause,
    Stop,
    SetGain(f32),
}

struct ModPlayer {
    // stream을 self에 보관하는 유일한 이유: drop을 막아서 "영업 유지".
    // 읽지는 않아서 컴파일러가 dead_code 경고를 낼 수 있다 (의도된 것).
    #[allow(dead_code)]
    stream: cpal::Stream,

    state: TransportState,
    gain: f32,

    command_tx: Sender<AudioCommand>,

    // 콜백이 자기가 몇 번 불렸는지 기록. main이 가끔 들여다본다.
    // 이게 학습 목적: "stream은 1번 만들었는데 콜백은 N번 불린다"를 눈으로 본다.
    callback_count: Arc<AtomicU64>,
}

impl ModPlayer {
    pub fn new() -> anyhow::Result<Self> {
        let (command_tx, command_rx) = bounded::<AudioCommand>(64);

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

        let mut active = false;

        let mut sin_source = SinSound::new(sample_rate, channels, 488.0, 0.5);

        let callback_count = Arc::new(AtomicU64::new(0));

        let count_cb = Arc::clone(&callback_count);

        let mut gain = 1.0;

        // ── 여기서 stream 1번 만든다. 이 줄은 평생 단 한 번만 실행된다. ──
        let stream = device.build_output_stream(
            &stream_config,
            // ── 이 클로저는 OS 오디오 스레드가 ~5.3ms마다 부른다 ──
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // 1) 호출 횟수 증가. 학습용이라 콜백 안에서 atomic 한 번 더 쓴다.
                //    실전에선 이런 카운터도 빼는 게 깔끔.
                count_cb.fetch_add(1, Ordering::Relaxed);

                while let Ok(command) = command_rx.try_recv() {
                    match command {
                        AudioCommand::Play => {
                            // active = true
                            active = true;
                        }
                        AudioCommand::Pause => {
                            // active = false
                            active = false;
                        }
                        AudioCommand::Stop => {
                            // active = false + 위치 reset
                            active = false;
                        }
                        AudioCommand::SetGain(new_gain) => {
                            gain = new_gain;
                        }
                    }
                }

                if active {
                    sin_source.process(&[], data);
                    for sample in data.iter_mut() {
                        *sample *= gain;
                    }
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
            command_tx,
            callback_count,
            gain,
        })
    }

    pub fn play(&mut self) -> anyhow::Result<()> {
        self.command_tx.send(AudioCommand::Play)?;
        self.state = TransportState::Playing;
        Ok(())
    }

    pub fn pause(&mut self) -> anyhow::Result<()> {
        // 잘못된 전이는 panic 아닌 무시.
        if matches!(self.state, TransportState::Stopped) {
            return Ok(());
        }
        self.command_tx.send(AudioCommand::Pause)?;
        self.state = TransportState::Paused;
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.command_tx.send(AudioCommand::Stop)?;
        self.state = TransportState::Stopped;
        Ok(())
    }

    // factor를 그대로 받아 곱한다. 위/아래는 호출하는 쪽이 결정.
    // factor > 1.0 → 키움, factor < 1.0 → 줄임, factor == 1.0 → 변화 없음.
    pub fn change_gain(&mut self, factor: f32) -> anyhow::Result<()> {
        let new_gain = (self.gain * factor).clamp(0.0, 2.0);
        self.command_tx.send(AudioCommand::SetGain(new_gain))?;
        self.gain = new_gain;
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
    println!("commands: p=play, a=pause, s=stop, g/G=gain down/up, c=count, q=quit");
    println!(
        "[player] state={:?}, gain={:.3}",
        player.state(),
        player.gain
    );

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
            "g" => player.change_gain(GAIN_STEP_DOWN)?,
            "G" => player.change_gain(GAIN_STEP_UP)?,
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
            "[player] state={:?}, gain={:.3}, callback_count={}",
            player.state(),
            player.gain,
            player.callback_count()
        );
    }

    println!("[player] dropping stream.");
    // main 끝 → player drop → stream drop → "영업 종료" → 콜백 호출 멈춤.
    Ok(())
}
