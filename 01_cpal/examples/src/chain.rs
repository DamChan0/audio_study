use crate::AudioProcess;

pub struct Chain {
    processes: Vec<Box<dyn AudioProcess>>,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
        }
    }

    pub fn run(&mut self, output: &mut [f32]) {
        let mut temp_input = vec![0.0; output.len()];
        let mut temp_output = vec![0.0; output.len()];

        for process in &mut self.processes {
            process.process(&temp_input, &mut temp_output);
            temp_input.copy_from_slice(&temp_output);
        }

        output.copy_from_slice(&temp_output);
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
