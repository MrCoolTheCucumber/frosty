
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

            column_line: 0

        }
    }

    pub fn set_column_line(&mut self, val: u8) {
        self.column_line = val & 0b0011_0000;
    }

    pub fn read_joyp(&self) -> u8 {
        match self.column_line {
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

            _ => 0xFF
        }
    }
}