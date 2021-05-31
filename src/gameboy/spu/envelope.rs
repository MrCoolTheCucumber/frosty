use super::MAX_VOLUME;


#[derive(Clone, Copy)]
pub(super) struct Envelope {
    pub volume: u8,
    direction: EnvelopeDirection,
    step_duration: u32,
    counter: u32
}

impl Envelope {
    pub fn new(val: u8) -> Self {
        let volume = val >> 4;
        let direction = if val & 0x8 != 0 {
            EnvelopeDirection::Increase 
        } else { 
            EnvelopeDirection::Decrease 
        };
        let length = (val & 7) as u32;

        Self {
            volume,
            direction,
            step_duration: length * 0x10000,
            counter: 0
        }
    }

    pub fn tick(&mut self) {
        if self.step_duration == 0 { return }

        self.counter += 1;
        self.counter %= self.step_duration;

        if self.counter == 0 {
            match self.direction {
                EnvelopeDirection::Decrease => self.dec_volume(),
                EnvelopeDirection::Increase => self.inc_volume()
            }
        }
    }

    fn inc_volume(&mut self) {
        if self.volume < MAX_VOLUME {
            self.volume += 1;
        }
    }

    fn dec_volume(&mut self) {
        if self.volume > 0 {
            self.volume -= 1;
        }
    }

    pub fn into_u8(&self) -> u8 {
        let length = (self.step_duration / 0x10000) as u8;

        (self.volume << 4) | ((self.direction as u8) << 3) | length
    }

    pub fn enabled(&self) -> bool {
        self.direction != EnvelopeDirection::Decrease || self.volume != 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeDirection {
    Decrease = 0,
    Increase = 1
}
