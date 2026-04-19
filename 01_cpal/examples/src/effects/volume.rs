use crate::AudioProcess;

pub struct Gain {
    gain: f32,
}

impl Gain {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }
}

// lib.rs의 trait을 구현
impl AudioProcess for Gain {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (out, &input) in output.iter_mut().zip(input.iter()) {
            *out = input * self.gain;
        }
    }
}
