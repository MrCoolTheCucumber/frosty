use std::{fs::File, io::Read};

use super::Cartridge;


pub struct MBC5 {
    is_ram_enabled: bool,
    num_rom_banks: u16,
    num_ram_banks: u16,

    current_rom_bank: usize,
    current_ram_bank: usize,

    mode: u8, // 0 = ROM 1 = RAM

    rom_banks: Vec<[u8; 0x4000]>,
    ram_banks: Vec<[u8; 0x2000]>
}

impl MBC5 {
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

        let mut ram_banks = Vec::new();

        for _ in 0..num_ram_banks {
            let bank = [0; 0x2000];
            ram_banks.push(bank);
        }

        Self {
            is_ram_enabled: false,
            num_rom_banks,
            num_ram_banks,

            current_rom_bank: 1,
            current_ram_bank: 0,
            mode: 0,

            rom_banks,
            ram_banks
        }
    }
}

impl Cartridge for MBC5 {
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

            0x2000 => {
                self.current_rom_bank = (self.current_rom_bank & 0b1_0000_0000) + (value & 0xFF) as usize;
            }

            0x3000 => {
                self.current_rom_bank = (self.current_rom_bank & 0b0_1111_1111) + 
                    (((value & 1) as usize) << 8);
            }

            0x4000 | 0x5000 => {
                if value <= 0x0F {
                    self.current_ram_bank = value as usize;
                }
            }

            0x6000 | 0x7000 => {
                // self.mode = value & 1;
            }

            _ => panic!()
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if !self.is_ram_enabled { return 0xFF; }

        self.ram_banks[self.current_ram_bank][addr as usize]
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.is_ram_enabled { return }

        self.ram_banks[self.current_ram_bank][addr as usize] = value;
    }
}