use super::{Mode, Sample};


pub(super) struct SampledWave {
    pub enabled: bool,
    running: bool,
    
    pub remaining: u32,
    pub output_level: u8,
    pub frequency: u16,
    cycle: u16,
    pub mode: Mode,

    pub samples: [Sample; 32],
    sample_index: usize
}

impl SampledWave {
    pub fn new(samples: Option<[Sample; 32]>) -> Self {
        let samples = match samples {
            Some(s) => s,
            None => [0; 32],
        };

        Self {
            enabled: false,
            running: false,

            remaining: 0x100 * 0x4000,
            output_level: 0,
            frequency: 0,
            cycle: 0,
            mode: Mode::Consecutive,
            samples,
            sample_index: 0
        }
    }

    pub fn tick(&mut self) {
        if self.mode == Mode::Counter {
            if self.remaining == 0 {
                self.enabled = false;
                self.remaining = 0x100 * 0x4000;
                return;
            } 
                
            self.remaining -= 1;
        }

        if !self.running { return }

        if self.cycle == 0 {
            self.cycle = 2 * (0x800 - self.frequency);

            self.sample_index = (self.sample_index + 1) % 32;
        }

        self.cycle -= 1;
    }

    pub fn sample(&self) -> Sample {
        if !self.running { return 0 }

        let sample = self.samples[self.sample_index];

        match self.output_level {
            0 => 0,
            1 => sample,
            2 => sample / 2,
            3 => sample / 4,
            _ => unreachable!()
        }
    }

    pub fn set_length(&mut self, length: u8) {
        self.remaining = (0x100 - (length as u32)) * 0x4000;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.running = false;
        }
    }

    pub fn start(&mut self) {
        self.running = self.enabled;
    }

    pub fn is_dac_enabled(&self) -> bool {
        // self.enabled or self.running?
        self.running
    }
}