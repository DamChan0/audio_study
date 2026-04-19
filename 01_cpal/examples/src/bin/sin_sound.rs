use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use examples::AudioProcess;
use std::f32::consts::PI;

const VOLUME: f32 = 9.0; // volume of the sine wave

struct SinSound {
    phase: f32,
    frequency: f32,
    channels: usize,
    phase_increment: f32,
    sample_rate: f32,
    default_volume: f32,
}

impl SinSound {
    pub fn new(sample_rate: f32, channels: usize, frequency: f32, default_volume: f32) -> Self {
        let phase_increment = frequency * 2.0 * PI / sample_rate;
        Self {
            phase: 0.0,
            frequency,
            channels,
            phase_increment,
            sample_rate,
            default_volume,
        }
    }
}
impl AudioProcess for SinSound {
    fn process(&mut self, _input: &[f32], output: &mut [f32]) {
        for frame in output.chunks_mut(self.channels) {
            let sample = self.phase.sin() * self.default_volume; // volume is the volume
            for channel in frame {
                *channel = sample;
            }
            self.phase += self.phase_increment;
            if self.phase >= 2.0 * PI {
                self.phase -= 2.0 * PI;
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no device for output");
    let config = device.default_output_config()?;

    // parameters of the default output stream
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;
    let mut phase = 0.0_f32;
    let frequency = 440.0_f32; // A4 note

    // sin wave phase increment per sample
    let phase_increment = frequency * 2.0 * PI / sample_rate;

    println!("sample format: {:?}", config.sample_format());
    println!("channels: {:?}", config.channels());
    println!("format: {:?}", config.buffer_size());

    let mut sin_sound = SinSound::new(sample_rate, channels, frequency, VOLUME);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            sin_sound.process(&[], data);
        },
        |err| eprint!("Error: {}", err),
        None,
    )?;

    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    Ok(())
}
