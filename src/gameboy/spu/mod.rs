use std::{cell::RefCell, rc::Rc};

use sdl2::audio::AudioQueue;

use self::{envelope::Envelope, square_wave::{Duty, SquareWave, Sweep}};



mod square_wave;
mod envelope;

pub const MAX_VOLUME: Sample = (1 << 4) - 1;
pub const MAX_SAMPLE: Sample = MAX_VOLUME * 4 * 2; // 4 PCM streams, 2 channels 
pub const SAMPLES_PER_BUFFER: usize = 1024;
pub const SAMPLE_RATE: u32 = 48000;
pub const CLOCKS_PER_SAMPLE: u64 = 87;

pub type Sample = u8;
pub type SampleBuffer = [Sample; SAMPLES_PER_BUFFER];

pub struct Spu {
    sample_clock: u64,
    buffer: SampleBuffer,
    buffer_pos: usize,

    enabled: bool,

    channel_1: SquareWave,
    channel_2: SquareWave,

    device: Option<Rc<RefCell<AudioQueue<f32>>>>
}

impl Spu {
    pub fn new(device: Option<Rc<RefCell<AudioQueue<f32>>>>) -> Self {
        Spu {
            sample_clock: CLOCKS_PER_SAMPLE,
            buffer: [0; SAMPLES_PER_BUFFER],
            buffer_pos: 0,

            enabled: false,

            channel_1: SquareWave::new(),
            channel_2: SquareWave::new(),

            device
        }
    }

    pub fn tick(&mut self) {
        if !self.enabled { return }

        self.channel_1.tick();
        self.channel_2.tick();

        if self.sample_clock == 0 {
            self.sample_clock = CLOCKS_PER_SAMPLE;
            self.sample();
        }

        self.sample_clock -= 1;
    }

    pub fn sample(&mut self) {
        let mut sample = 
            self.channel_1.sample() +
            self.channel_2.sample();

        sample = sample * (u8::MAX / MAX_SAMPLE);

        self.buffer[self.buffer_pos] = sample;
        self.buffer_pos += 1;

        if self.buffer_pos == self.buffer.len() {
            self.send_sample();
            self.buffer_pos = 0;
        }
    }

    pub fn send_sample(&mut self) {
        let mut buffer = [0.0; SAMPLES_PER_BUFFER];
        for i in 0..SAMPLES_PER_BUFFER {
            let mut f: f32 = (self.buffer[i] - 128) as i8 as f32 / 128.0;
            if f > 1.0 { f = 1.0 }
            if f < -1.0 { f = -1.0 }

            buffer[i] = f * 0.1;
        }

        if self.device.is_some() {
            (*self.device.as_ref().unwrap()).borrow().queue(&buffer);
        }
    }

    pub fn get_nr52(&self) -> u8 {
        let enabled = if self.enabled {1} else {0};
        
        enabled << 7
    }

    pub fn set_nr52(&mut self, val: u8) {
        self.enabled = val & 0b1000_0000 != 0;

        if !self.enabled {
            self.reset();
        }
    }

    // CHANNEL 1

    pub fn get_nr10(&self) -> u8 {
        self.channel_1.sweep.into_u8()
    }

    pub fn set_nr10(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_1.sweep = Sweep::new(val);
    }

    pub fn get_nr11(&self) -> u8 {
        let duty = self.channel_1.duty as u8;
        duty << 6 | 0x3F
    }

    pub fn set_nr11(&mut self, val: u8) {
        if !self.enabled { return }

        let duty = Duty::from(val >> 6);
        let length = val & 0x3F;

        self.channel_1.duty = duty;
        self.channel_1.set_length(length);
    }

    pub fn get_nr12(&self) -> u8 {
        self.channel_1.start_envelope.into_u8()
    }

    pub fn set_nr12(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_1.set_envelope(Envelope::new(val));
    }

    pub fn get_nr13(&self) -> u8 {
        // write only
        0xFF 
    }

    pub fn set_nr13(&mut self, val: u8) {
        if !self.enabled { return }

        let mut freq = self.channel_1.freq;
        freq &= 0b1111_1111_0000_0000;
        freq |= val as u16;

        self.channel_1.freq = freq;
    }

    pub fn get_nr14(&self) -> u8 {
        // only mode is readable from this reg
        let mode = self.channel_1.mode as u8;

        mode << 6 | 0xbf
    }

    pub fn set_nr14(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_1.freq &= 0xFF;
        self.channel_1.freq |= ((val & 7) as u16) << 8;

        self.channel_1.mode = if val & 0x40 != 0 {
            Mode::Counter
        } else {
            Mode::Consecutive
        };

        if val & 0x80 != 0 {
            self.channel_1.start();
        }
    }

    // CHANNEL 2

    pub fn get_nr21(&self) -> u8 {
        let duty = self.channel_2.duty as u8;
        duty << 6 | 0x3F
    }

    pub fn set_nr21(&mut self, val: u8) {
        if !self.enabled { return }

        let duty = Duty::from(val >> 6);
        let length = val & 0x3F;

        self.channel_2.duty = duty;
        self.channel_2.set_length(length);
    }

    pub fn get_nr22(&self) -> u8 {
        self.channel_2.start_envelope.into_u8()
    }

    pub fn set_nr22(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_2.set_envelope(Envelope::new(val));
    }

    pub fn get_nr23(&self) -> u8 {
        // write only
        0xFF 
    }

    pub fn set_nr23(&mut self, val: u8) {
        if !self.enabled { return }

        let mut freq = self.channel_2.freq;
        freq &= 0b1111_1111_0000_0000;
        freq |= val as u16;

        self.channel_2.freq = freq;
    }

    pub fn get_nr24(&self) -> u8 {
        // only mode is readable from this reg
        let mode = self.channel_2.mode as u8;

        mode << 6 | 0xbf
    }

    pub fn set_nr24(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_2.freq &= 0xFF;
        self.channel_2.freq |= ((val & 7) as u16) << 8;

        self.channel_2.mode = if val & 0x40 != 0 {
            Mode::Counter
        } else {
            Mode::Consecutive
        };

        if val & 0x80 != 0 {
            self.channel_2.start();
        }
    }

    fn reset(&mut self) {
        self.channel_2 = SquareWave::new();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Consecutive = 0,
    Counter = 1
}
