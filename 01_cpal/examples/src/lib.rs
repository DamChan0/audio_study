pub mod chain;
pub mod effects;
pub mod sources;

pub trait AudioProcess: Send {
    fn process(&mut self, input: &[f32], output: &mut [f32]);
}
