use crate::AudioProcess;

pub struct Chain {
    processes: Vec<Box<dyn AudioProcess>>,
    temp_input: Vec<f32>,
    temp_output: Vec<f32>,
}

impl Chain {
    pub fn new(buffer_len: usize) -> Self {
        Self {
            processes: Vec::new(),
            temp_input: vec![0.0; buffer_len],
            temp_output: vec![0.0; buffer_len],
        }
    }

    pub fn run(&mut self, output: &mut [f32]) {
        debug_assert_eq!(self.temp_input.len(), output.len());
        debug_assert_eq!(self.temp_output.len(), output.len());

        self.temp_input.fill(0.0);
        self.temp_output.fill(0.0);

        for process in &mut self.processes {
            process.process(&self.temp_input, &mut self.temp_output);
            self.temp_input.copy_from_slice(&self.temp_output);
        }

        output.copy_from_slice(&self.temp_output);
    }

    pub fn add(&mut self, process: Box<dyn AudioProcess>) {
        self.processes.push(process);
    }
}

impl AudioProcess for Chain {
    fn process(&mut self, _input: &[f32], output: &mut [f32]) {
        self.run(output);
    }
}
