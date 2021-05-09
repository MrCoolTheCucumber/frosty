use std::{fs::File, io::Read};

use super::Cartridge;

pub struct MBC1 {
    is_ram_enabled: bool,
    num_rom_banks: u16,
    num_ram_banks: u16,
    current_rom_bank: usize,
    mode: u8, // 0 = ROM 1 = RAM

    rom_banks: Vec<[u8; 0x4000]>
}

impl MBC1 {
    pub fn new(
        mut file: File,
        rom_bank_0: [u8; 0x4000],
        cartridge_type_code: u8, 
        num_rom_banks: u16, 
        num_ram_banks: u16
    ) -> Self {
        let mut rom_banks = Vec::new();
        rom_banks.push(rom_bank_0);

        for _ in 0..num_rom_banks - 1 {
            let mut bank = [0; 0x4000];
            file.read_exact(&mut bank).ok();
            rom_banks.push(bank);
        }

        Self {
            is_ram_enabled: false,
            num_rom_banks,
            num_ram_banks,
            current_rom_bank: 1,
            mode: 0,

            rom_banks
        }
    }
}

impl Cartridge for MBC1 {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr & 0xF000 {
            0x0000 | 0x1000 | 0x2000 | 0x3000 => {
                self.rom_banks[0][addr as usize]
            }

            0x4000 | 0x5000 | 0x6000 | 0x7000 => {
                self.rom_banks[self.current_rom_bank][(addr - 0x4000) as usize]
            }

            _ => panic!()
        }
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0x0000 | 0x1000 => {
                self.is_ram_enabled = (value & 0x0F) == 0x0A;
            }

            0x2000 | 0x3000 => {
                self.current_rom_bank = (self.current_rom_bank & 0b0110_0000) + value as usize;
                if self.current_rom_bank == 0 { self.current_rom_bank = 1 }

                println!("Switched to bank {}!", self.current_rom_bank);
            }

            0x4000 | 0x5000 => {
                if self.mode == 0 {
                    self.current_rom_bank =
                        (self.current_rom_bank & 0x0b0001_1111) +
                        ((value & 0b0000_0011) << 5) as usize;
                } else {
                    panic!("mode 1 unimpl");
                }
            }

            0x6000 | 0x7000 => {
                self.mode = value & 1;
            }

            _ => panic!()
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        todo!()
    }
}