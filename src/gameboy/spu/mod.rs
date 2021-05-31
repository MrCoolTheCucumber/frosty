use std::{sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel}};

use crate::audio::{SAMPLES_PER_BUFFER, Sample, SampleBuffer};

use self::{envelope::Envelope, rectangle_wave::{Duty, RectangleWave}};

const CLOCKS_PER_SAMPLE: u16 = 85;

mod rectangle_wave;
mod envelope;

const MAX_VOLUME: Sample = (1 << 4) - 1;
const MAX_SAMPLE: Sample = MAX_VOLUME * 8;

pub struct Spu {
    sender: SyncSender<SampleBuffer>,
    sample_clock: u16,
    buffer: SampleBuffer,
    buffer_pos: usize,

    enabled: bool,

    channel_2: RectangleWave,
}

impl Spu {
    pub fn new() -> (Self, Receiver<SampleBuffer>) {
        let (sender, receiver) = sync_channel(4);

        let spu = Spu {
            sender,
            sample_clock: CLOCKS_PER_SAMPLE,
            buffer: [0; SAMPLES_PER_BUFFER],
            buffer_pos: 0,

            enabled: false,

            channel_2: RectangleWave::new()
        };

        (spu, receiver)
    }

    pub fn tick(&mut self) {
        if !self.enabled { return }

        self.channel_2.tick();

        if self.sample_clock == 0 {
            self.sample_clock = CLOCKS_PER_SAMPLE;
            self.sample();
        }

        self.sample_clock -= 1;
    }

    pub fn sample(&mut self) {
        let sample = self.channel_2.sample();

        self.buffer[self.buffer_pos] = sample;
        self.buffer_pos += 1;

        if self.buffer_pos == self.buffer.len() {
            self.send_sample();
            self.buffer_pos = 0;
        }
    }

    pub fn send_sample(&mut self) {
        match self.sender.try_send(self.buffer) {
            Ok(_) => {}
            Err(e) => {
                match e {
                    TrySendError::Full(_) => println!("Tried to send sound sample, but channel is full?"),
                    _ => panic!("{:?}", e)
                }
            }
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
        self.channel_2 = RectangleWave::new();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Consecutive = 0,
    Counter = 1
}
