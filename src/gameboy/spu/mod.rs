use std::{cell::RefCell, rc::Rc};

use sdl2::audio::AudioQueue;

use self::{envelope::Envelope, sampled_wave::SampledWave, square_wave::{Duty, SquareWave, Sweep}, white_noise_wave::{WhiteNoiseGenerator, WhiteNoiseWave}};

mod white_noise_wave;
mod sampled_wave;
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
    buffer: [f32; SAMPLES_PER_BUFFER],
    buffer_pos: usize,

    enabled: bool,

    channel_1: SquareWave,
    channel_2: SquareWave,
    channel_3: SampledWave,
    channel_4: WhiteNoiseWave,

    mixer: Mixer,

    device: Option<Rc<RefCell<AudioQueue<f32>>>>
}

impl Spu {
    pub fn new(device: Option<Rc<RefCell<AudioQueue<f32>>>>) -> Self {
        Spu {
            sample_clock: CLOCKS_PER_SAMPLE,
            buffer: [0.0; SAMPLES_PER_BUFFER],
            buffer_pos: 0,

            enabled: false,

            channel_1: SquareWave::new(),
            channel_2: SquareWave::new(),
            channel_3: SampledWave::new(None),
            channel_4: WhiteNoiseWave::new(),

            mixer: Mixer::new(),

            device
        }
    }

    pub fn tick(&mut self) {
        if !self.enabled { return }

        self.channel_1.tick();
        self.channel_2.tick();
        self.channel_3.tick();
        self.channel_4.tick();

        if self.sample_clock == 0 {
            self.sample_clock = CLOCKS_PER_SAMPLE;
            self.sample();
        }

        self.sample_clock -= 1;
    }

    pub fn sample(&mut self) {
        let (left_sample, right_sample) = self.mixer.sample_and_mix(
            &self.channel_1, 
            &self.channel_2, 
            &self.channel_3, 
            &self.channel_4
        );

        self.buffer[self.buffer_pos] = left_sample;
        self.buffer_pos += 1;
        self.buffer[self.buffer_pos] = right_sample;
        self.buffer_pos += 1;

        if self.buffer_pos == self.buffer.len() {
            self.send_sample();
            self.buffer_pos = 0;
        }
    }

    pub fn send_sample(&mut self) {
        let mut buffer = [0.0; SAMPLES_PER_BUFFER];

        for i in 0..SAMPLES_PER_BUFFER {
            buffer[i] = self.buffer[i] / 10.0;
        }

        if self.device.is_some() {
            (*self.device.as_ref().unwrap()).borrow().queue(&buffer);
        }
    }

    pub fn get_nr50(&self) -> u8 {
        self.mixer.channel_vol_flags
    }

    pub fn set_nr50(&mut self, val: u8) {
        if !self.enabled { return }

        self.mixer.channel_vol_flags = val;
    }

    pub fn get_nr51(&self) -> u8 {
        self.mixer.channel_output_flags
    }

    pub fn set_nr51(&mut self, val: u8) {
        if !self.enabled { return }

        self.mixer.channel_output_flags = val;
    }

    pub fn get_nr52(&self) -> u8 {
        let enabled = if self.enabled {1} else {0};
        
        let mut channels: u8 = 0;
        channels |= if self.channel_1.enabled {1} else {0};
        channels |= (if self.channel_2.enabled {1} else {0}) << 1;
        channels |= (if self.channel_3.enabled {1} else {0}) << 2;
        channels |= (if self.channel_4.enabled {1} else {0}) << 3;

        (enabled << 7) | channels | 0b0111_0000
    }

    pub fn set_nr52(&mut self, val: u8) {
        self.enabled = val & 0b1000_0000 != 0;

        if !self.enabled {
            self.reset();
        }

        if self.device.is_some() {
            let device = (*self.device.as_ref().unwrap()).borrow();
            if !self.enabled {
                device.pause();
            } else {
                device.clear();
                device.resume();
            }
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

    // CHANNEL 3

    pub fn get_nr30(&self) -> u8 {
        let enabled = if self.channel_3.enabled
            {1} else {0};
        
        (enabled << 7) | 0b0111_1111
    }

    pub fn set_nr30(&mut self, val: u8) {
        if !self.enabled { return }

        let enabled = val & 0x80 != 0;
        self.channel_3.set_enabled(enabled);
    }

    pub fn get_nr31(&self) -> u8 {
        0xFF
    }

    pub fn set_nr31(&mut self, val: u8) {
        if !self.enabled { return }
        
        self.channel_3.set_length(val);
    }

    pub fn get_nr32(&self) -> u8 {
        let output_level = self.channel_3.output_level;

        (output_level << 5) | 0b1001_1111
    }

    pub fn set_nr32(&mut self, val: u8) {
        if !self.enabled { return }
        
        self.channel_3.output_level = (val >> 5) & 3;
    }

    pub fn get_nr33(&self) -> u8 {
        0xFF
    }

    pub fn set_nr33(&mut self, val: u8) {
        if !self.enabled { return }
        
        let mut freq = self.channel_3.frequency;
        freq = (freq & 0x700) | (val as u16);

        self.channel_3.frequency = freq;
    }

    pub fn get_nr34(&self) -> u8 {
        // only mode is readable from this reg
        let mode = self.channel_3.mode as u8;

        mode << 6 | 0xBF
    }

    pub fn set_nr34(&mut self, val: u8) {
        if !self.enabled { return }

        let mut freq = self.channel_3.frequency;
        freq = (freq & 0xFF) | (((val & 7) as u16) << 8);

        self.channel_3.frequency = freq;

        self.channel_3.mode = if val & 0x40 != 0 {
            Mode::Counter
        } else {
            Mode::Consecutive
        };

        if val & 0x80 != 0 {
            self.channel_3.start();
        }
    }

    pub fn get_sample(&self, index: u8) -> u8 {
        let index = (index * 2) as usize;
        let sample_high = self.channel_3.samples[index];
        let sample_low = self.channel_3.samples[index + 1];

        (sample_high << 4) | sample_low
    }

    pub fn set_sample(&mut self, index: u8, sample: u8) {
        // uaffected by master enable/disable

        let index = (index * 2) as usize;
        let sample_high = (sample >> 4) as Sample;
        let sample_low = (sample & 0b0000_1111) as Sample;

        self.channel_3.samples[index] = sample_high;
        self.channel_3.samples[index + 1] = sample_low;
    }

    // CHANNEL 4

    pub fn get_nr41(&self) -> u8 {
        0xFF // read only?
    }

    pub fn set_nr41(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_4.set_length(val & 0b0011_1111)
    }

    pub fn get_nr42(&self) -> u8 {
        self.channel_4.start_envelope.into_u8()
    }

    pub fn set_nr42(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_4.set_envelope(Envelope::new(val));
    }

    pub fn get_nr43(&self) -> u8 {
        self.channel_4.white_noise_generator.val
    }

    pub fn set_nr43(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_4.white_noise_generator = WhiteNoiseGenerator::new(val);
    }

    pub fn get_nr44(&self) -> u8 {
        // only mode is readable from this reg
        let mode = self.channel_4.mode as u8;

        mode << 6 | 0xbf
    }

    pub fn set_nr44(&mut self, val: u8) {
        if !self.enabled { return }

        self.channel_4.mode = if val & 0x40 != 0 {
            Mode::Counter
        } else {
            Mode::Consecutive
        };

        if val & 0x80 != 0 {
            self.channel_4.start();
        }
    }

    fn reset(&mut self) {
        self.channel_1 = SquareWave::new();
        self.channel_2 = SquareWave::new();
        self.channel_3 = SampledWave::new(Some(self.channel_3.samples));
        self.channel_4 = WhiteNoiseWave::new();

        self.mixer = Mixer::new();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Consecutive = 0,
    Counter = 1
}

pub struct Mixer {
    channel_output_flags: u8,
    channel_vol_flags: u8,
    slope: f32
}

impl Mixer {
    const MAX_VOLTAGE: f32 = 1.0 * 4.0 * 8.0;

    pub fn new() -> Self {
        let output_start = -1.0;
        let output_end = 1.0;
        let input_start = 0f32;
        let input_end = MAX_VOLUME as f32;

        let slope: f32 = 1.0 * (output_end - output_start) / (input_end - input_start);

        Self {
            channel_output_flags: 0,
            channel_vol_flags: 0,
            slope
        }
    }

    fn so1_vol(&self) -> u8 {
        self.channel_vol_flags & 0b0000_0111
    }

    fn so2_vol(&self) -> u8 {
        (self.channel_vol_flags & 0b0111_0000) >> 4
    }

    #[inline]
    fn convert_sample_to_voltage(&self, sample: u8) -> f32 {
        // if sample > 15 {
        //     panic!("sample too large??!");
        // }

        // let input = sample as f32;
        
        //-1.0 + self.slope * input
        sample as f32
    }

    fn sample_and_mix(
        &self,
        channel_1: &SquareWave,
        channel_2: &SquareWave,
        channel_3: &SampledWave,
        channel_4: &WhiteNoiseWave
    ) -> (f32, f32) {
        let mut left_voltage = 0.0;
        let mut right_voltage = 0.0;

        let ch1_voltage = match channel_1.is_dac_enabled() {
            true => self.convert_sample_to_voltage(channel_1.sample()),
            false => 0.0,
        };

        let ch2_voltage = match channel_2.is_dac_enabled() {
            true => self.convert_sample_to_voltage(channel_2.sample()),
            false => 0.0,
        };

        let ch3_voltage = match channel_3.is_dac_enabled() {
            true => self.convert_sample_to_voltage(channel_3.sample()),
            false => 0.0,
        };

        let ch4_voltage = match channel_4.is_dac_enabled() {
            true => self.convert_sample_to_voltage(channel_4.sample()),
            false => 0.0,
        };

        // S01 is right, S02 is left

        // S01 
        if self.channel_output_flags & 0b0000_0001 != 0 {
            right_voltage += ch1_voltage;
        }

        if self.channel_output_flags & 0b0000_0010 != 0 {
            right_voltage += ch2_voltage;
        }

        if self.channel_output_flags & 0b0000_0100 != 0 {
            right_voltage += ch3_voltage;
        }

        if self.channel_output_flags & 0b0000_1000 != 0 {
            right_voltage += ch4_voltage;
        }

        // S02
        if self.channel_output_flags & 0b0001_0000 != 0 {
            left_voltage += ch1_voltage;
        }

        if self.channel_output_flags & 0b0010_0000 != 0 {
            left_voltage += ch2_voltage;
        }

        if self.channel_output_flags & 0b0100_0000 != 0 {
            left_voltage += ch3_voltage;
        }

        if self.channel_output_flags & 0b1000_0000 != 0 {
            left_voltage += ch4_voltage;
        }

        right_voltage = right_voltage * (self.so1_vol() as f32 + 1.0);
        left_voltage = left_voltage * (self.so2_vol() as f32 + 1.0);

        // println!("L: {}, R: {}", left_voltage / Self::MAX_VOLTAGE, right_voltage / Self::MAX_VOLTAGE);
        // (left_voltage / Self::MAX_VOLTAGE, right_voltage / Self::MAX_VOLTAGE)
        
        // why 480: (15 + 15 + 15 + 15) * max(0 + 1, 7 + 1) = 480

        (left_voltage / (480.0), right_voltage / (480.0))
    }
}
