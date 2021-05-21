use sdl2::keyboard::Keycode;


// http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-Input

pub struct Input {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8,
    pub start: u8,
    pub select: u8,
    pub a: u8,
    pub b: u8,

    column_line: u8
}

impl Input {
    pub fn new() -> Self {
        Self {
            up: 1,
            down: 1,
            left: 1,
            right: 1,
            start: 1,
            select: 1,
            a: 1,
            b: 1,

            column_line: 0x30

        }
    }

    pub fn set_column_line(&mut self, val: u8) {
        self.column_line = val & 0b0011_0000;
    }

    pub fn read_joyp(&self) -> u8 {
        let joyp = match self.column_line {
            // 4th bit 
            0x10 => {
                let mut result = 0 | self.a;
                result = result | (self.b << 1);
                result = result | (self.select << 2);
                result = result | (self.start << 3);
                result
            }

            // 5th bit
            0x20 => {
                let mut result = 0 | self.right;
                result = result | (self.left << 1);
                result = result | (self.up << 2);
                result = result | (self.down << 3);
                result
            }

            // 4th & 5th
            0x30 => {
                self.a | (self.b << 1) | (self.select << 2) | (self.start << 3) |
                (self.left << 1) | (self.up << 2) | (self.down << 3)
            }

            _ => 0
        };

        joyp | 0b1100_0000
    }

    pub fn key_down(&mut self, code: Keycode) -> bool {
        match code {
            Keycode::W => self.up = 0,
            Keycode::A => self.left = 0,
            Keycode::S => self.down = 0,
            Keycode::D => self.right = 0,
            Keycode::O => self.a = 0,
            Keycode::K => self.b = 0,
            Keycode::N => self.select = 0,
            Keycode::M => self.start = 0,
            _ => return false
        };

        true
    }

    pub fn key_up(&mut self, code: Keycode) {
        match code {
            Keycode::W => self.up = 1,
            Keycode::A => self.left = 1,
            Keycode::S => self.down = 1,
            Keycode::D => self.right = 1,
            Keycode::O => self.a = 1,
            Keycode::K => self.b = 1,
            Keycode::N => self.select = 1,
            Keycode::M => self.start = 1,
            _ => {} // do nothing
        }
    }
}