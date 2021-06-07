use super::{Mode, Sample, envelope::Envelope};


pub(super) struct WhiteNoiseWave {
    pub enabled: bool,
    pub white_noise_generator: WhiteNoiseGenerator,
    pub start_envelope: Envelope,
    pub envelope: Envelope,
    pub mode: Mode,
    remaining: u32
}

impl WhiteNoiseWave {
    pub fn new() -> Self {
        Self {
            enabled: false,
            white_noise_generator: WhiteNoiseGenerator::new(0),
            start_envelope: Envelope::new(0),
            envelope: Envelope::new(0),
            mode: Mode::Consecutive,
            remaining: 64 * 0x4000
        }
    }

    pub fn tick(&mut self) {
        if self.mode == Mode::Counter {
            if self.remaining == 0 {
                self.enabled = false;
                self.remaining = 64 * 0x4000;
                return;
            } 
                
            self.remaining -= 1;
        }

        if !self.enabled { return }

        self.envelope.tick();
        self.white_noise_generator.tick();
    }

    pub fn sample(&self) -> Sample {
        if !self.enabled { return 0 }

        match self.white_noise_generator.is_output_high() {
            true => self.envelope.volume,
            false => 0,
        }
    }

    pub fn start(&mut self) {
        self.envelope = self.start_envelope;
        self.enabled = self.envelope.enabled();
    }

    pub fn set_length(&mut self, length: u8) {
        assert!(length < 64);

        self.remaining = (64 - (length as u32)) * 0x4000;
    }

    pub fn set_envelope(&mut self, envelope: Envelope) {
        self.start_envelope = envelope;

        if !self.envelope.enabled() {
            self.enabled = false;
        }
    }

    pub fn is_dac_enabled(&self) -> bool {
        self.envelope.enabled()
    }
}

#[derive(Clone, Copy)]
enum CounterWidth {
    Width15 = 0,
    Width7  = 1
}

pub(super) struct WhiteNoiseGenerator {
    dividing_ratio: u8,
    shift_clock: u8,
    counter_width: CounterWidth,
    pub val: u8,
    noise: u16,

    cycles: u32
}

impl WhiteNoiseGenerator {
    pub fn new(val: u8) -> Self {
        let dividing_ratio = val & 0b0000_0111;
        let shift_clock = (val & 0b1111_0000) >> 4;
        let counter_width = match val & 0b0000_1000 == 0 {
            true => CounterWidth::Width15,
            false => CounterWidth::Width7
        };

        let noise = match counter_width {
            CounterWidth::Width15 => (1 << 15) - 1,
            CounterWidth::Width7 => (1 << 7) - 1,
        };

        Self {
            dividing_ratio,
            shift_clock,
            counter_width,
            val,
            noise,
            
            cycles: 0
        }
    }

    pub fn tick(&mut self) {
        let tot_cycles = match self.dividing_ratio {
            0 => 8 / 2,
            x => 8 * (x as u32)
        } * 1 << (self.shift_clock + 1);

        self.cycles = (self.cycles + 1) % tot_cycles;

        if self.cycles == 0 {
            let shift = self.noise >> 1;
            let carry = (self.noise ^ shift) & 1;

            self.noise = match self.counter_width {
                CounterWidth::Width15 => shift | (carry << 14),
                CounterWidth::Width7 => shift | (carry << 6),
            };
        }
    }

    pub fn is_output_high(&self) -> bool {
        !self.noise & 1 == 1
    }
}

