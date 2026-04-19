use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use examples::AudioProcess;
use examples::chain;
use examples::effects::volume::Gain;
use examples::sources;
use examples::sources::sin_sound::SinSound;

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no device for output");
    let config = device.default_output_config()?;

    let sample_ratate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;
    let volume = 1.0;
    let frequency = 1000.0;

    let mut chain = chain::Chain::new();
    chain.add(Box::new(SinSound::new(
        sample_ratate,
        channels,
        frequency,
        volume,
    )));
    chain.add(Box::new(Gain::new(1000.5)));

    println!("sample format: {:?}", config.sample_format());
    println!("channels: {:?}", config.channels());
    println!("format: {:?}", config.buffer_size());

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            chain.run(data);
        },
        |err| eprint!("Error: {}", err),
        None,
    )?;

    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    Ok(())
}
