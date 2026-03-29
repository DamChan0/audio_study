use cpal::Stream;
///
/// Host
///  └── Device
///       └── SupportedStreamConfig / StreamConfig
///            └── Stream
///                 └── callback
use cpal::traits::HostTrait;
use cpal::traits::{DeviceTrait, StreamTrait};

// fn play_stream(
//     device: cpal::Device,
//     config: cpal::StreamConfig,
//     data_callback: impl FnMut(&mut [f32]),
//     error_callback: impl FnMut(cpal::Error),
//     timeout: std::time::Duration,
// ) -> anyhow::Result<()> {
//     let Stream = device.build_output_stream(config, data_callback, error_callback, timeout)?;
//     Stream.play()?;
// }

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no device");
    let supported_config = device.default_output_config()?;
    println!("Default output config: {:?}", supported_config);
    let default_config: cpal::StreamConfig = supported_config.into();
    println!("Default output stream config: {:?}", default_config);

    let stream = device.build_output_stream(
        &default_config,
        // DATA example : data = [L0, R0, L1, R1, L2, R2, ...]
        move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
            for sample_data in data.iter_mut() {
                *sample_data = 0.0; // silence
            }
        },
        move |err| {
            eprintln!("an error occurred on stream: {}", err);
        },
        None,
    )?;

    Ok(())
}
