use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;

const VOLUME: f32 = 9.0; // volume of the sine wave

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

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            for frame in data.chunks_mut(channels) {
                let sample = phase.sin() * VOLUME; // volume is the volume
                for channel in frame {
                    *channel = sample;
                }
                phase += phase_increment;
                if phase >= 2.0 * PI {
                    phase -= 2.0 * PI;
                }
            }
        },
        |err| eprint!("Error: {}", err),
        None,
    )?;

    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    Ok(())
}
