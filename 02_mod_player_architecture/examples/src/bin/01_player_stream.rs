use cpal::SampleRate;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal_examples::effects::volume::{self, Gain};
use cpal_examples::sources::sin_sound::SinSound;
use cpal_examples::{AudioProcess, chain};
use mod_player_examples::input::{InputCommand, InputProcessor};
use std::any;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

struct ModPlayer {
    stream: Option<cpal::Stream>,
    volume: Arc<AtomicU32>,
}

impl ModPlayer {
    pub fn new() -> anyhow::Result<Self> {
        // initialize audio stream
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("no output device available"))?;
        let config = device.default_output_config()?;

        // values for stream
        let mut sample_rate = config.sample_rate().0 as f32;
        let mut channels = config.channels() as usize;
        let mut stream_confiog: cpal::StreamConfig = config.into();
        let mut frequency = 440.0; // A4 note frequency
        let mut volume = Arc::new(AtomicU32::new(0.5_f32.to_bits()));

        let cloned_volume = Arc::clone(&volume);

        // test data
        let mut sin_source = SinSound::new(sample_rate, channels, frequency, 1.0);

        // stream build
        let stream = device.build_output_stream(
            &stream_confiog,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                sin_source.process(&[], data);
                let volume_bits = cloned_volume.load(std::sync::atomic::Ordering::Relaxed);
                let volume_value = f32::from_bits(volume_bits);
                for sample in data.iter_mut() {
                    *sample *= volume_value;
                }
            },
            |err| eprint!("an error occurred on stream: {}", err),
            None,
        )?;

        stream.pause()?;

        Ok(Self {
            stream: Some(stream),
            volume,
        })
    }

    pub fn play(&mut self) -> anyhow::Result<(), anyhow::Error> {
        self.stream
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no stream"))?
            .play()?;
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<(), anyhow::Error> {
        self.stream
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no stream"))?
            .pause()?;
        Ok(())
    }

    pub fn volume_up(&mut self) -> anyhow::Result<(), anyhow::Error> {}

    pub fn volume_down(&mut self) -> anyhow::Result<(), anyhow::Error> {}
}

fn main() {
    let mut player = ModPlayer::new().expect("failed to create player");
    let input_processor = InputProcessor::new().expect("failed to create input processor");

    loop {
        let mut command = InputProcessor::key_board_input().expect("failed to get input");
        match command {
            InputCommand::Play => {
                player.play().expect("failed to play");
            }
            InputCommand::Stop => {
                player.stop().expect("failed to stop");
            }
        }
    }

    // keep the main thread alive while the audio is playing
    std::thread::sleep(std::time::Duration::from_secs(5));
}
