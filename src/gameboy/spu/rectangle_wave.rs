use crate::audio::Sample;

use super::{Mode, envelope::Envelope};


pub(super) struct RectangleWave {
    pub duty: Duty,
    envelope: Envelope,
    pub start_envelope: Envelope,
    pub freq: u16,
    pub mode: Mode,

    enabled: bool,
    remaining: u32,
    counter: u16,
    phase: u8
}

impl RectangleWave {
    pub fn new() -> Self {
        Self {
            duty: Duty::from(0),
            envelope: Envelope::new(0),
            start_envelope: Envelope::new(0),
            freq: 0,
            mode: Mode::Consecutive,

            enabled: false,
            remaining: 64 * 0x4000,
            counter: 0,
            phase: 0
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

        // TODO: freq sweeps

        if self.counter == 0 {
            self.counter = 4 * (0x800 - self.freq);
            self.phase = (self.phase + 1) % 8;
        }

        self.counter -= 1;
    }

    pub fn sample(&self) -> Sample {
        if !self.enabled { return 0 }
        
        if self.phase < self.duty.pulses_per_8() {
            self.envelope.volume
        } else {
            0
        }
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

    pub fn start(&mut self) {
        self.envelope = self.start_envelope;
        self.enabled = self.envelope.enabled();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Duty {
    D12 = 0,
    D25 = 1,
    D50 = 2,
    D75 = 3
}

impl From<u8> for Duty {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::D12,
            1 => Self::D25,
            2 => Self::D50,
            3 => Self::D75,
            4..=u8::MAX => panic!("Invalid Duty!")
        }
    }
}

impl Duty {
    fn pulses_per_8(self) -> u8 {
        match self {
            Duty::D12 => 1,
            Duty::D25 => 2,
            Duty::D50 => 4,
            Duty::D75 => 6
        }
    }
}
