use crate::AudioProcess;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;

const VOLUME: f32 = 9.0; // volume of the sine wave

pub struct SinSound {
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
