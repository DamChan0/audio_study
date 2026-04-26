use cpal::SampleRate;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal_examples::chain::Chain;
use cpal_examples::effects::volume::Gain;
use cpal_examples::sources::sin_sound::SinSound;
use cpal_examples::{AudioProcess, chain};
use std::any;

struct ModPlayer {
    stream: Option<cpal::Stream>,
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

        // test data
        let mut sin_source = SinSound::new(sample_rate, channels, frequency, 0.5);

        // stream build
        let stream = device.build_output_stream(
            &stream_confiog,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                sin_source.process(&[], data);
            },
            |err| eprint!("an error occurred on stream: {}", err),
            None,
        )?;

        Ok(Self {
            stream: Some(stream),
        })
    }

    pub fn play(&mut self) -> anyhow::Result<()> {
        self.stream
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no stream"))?
            .play()?;
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {}
}

fn main() {}
