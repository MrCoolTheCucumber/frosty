use std::{fs::File, io::{Read, Write}, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

use super::{Cartridge, get_save_file_path_from_rom_path, try_read_save_file};

pub struct MBC3 {
    is_ram_rtc_enabled: bool,
    current_rom_bank: usize,
    current_ram_bank: usize,

    rom_banks: Vec<[u8; 0x4000]>,
    ram_banks: Vec<[u8; 0x2000]>,

    rtc_regs: [u8; 5],
    rtc_banked: bool,

    prev_latch_val: u8,
    save_file_path: PathBuf
}

impl MBC3 {
    pub fn new(
        mut file: File,
        path: &Path,
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
        let save_file_path = get_save_file_path_from_rom_path(path);

        // try to open save file
        let sav_file = File::open(&save_file_path);
        try_read_save_file(sav_file, num_ram_banks, &mut ram_banks);
        
        Self {
            is_ram_rtc_enabled: false,
            current_rom_bank: 1,
            current_ram_bank: 0,

            rtc_regs: [0; 5],
            rtc_banked: false,

            rom_banks,
            ram_banks,
            prev_latch_val: 204, // random val
            save_file_path
        }
    }
}

impl Drop for MBC3 {
    fn drop(&mut self) {
        // create save file
        let mut sav_file = File::create(&self.save_file_path).unwrap();
        for bank in &self.ram_banks {
            sav_file.write_all(bank).unwrap();
        }
        println!("Save file written!");

    }
}

impl Cartridge for MBC3 {
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
                self.is_ram_rtc_enabled = (value & 0x0F) == 0x0A;
            }

            0x2000 | 0x3000 => {
                self.current_rom_bank = (value & 0b0111_1111) as usize;
                if self.current_rom_bank == 0 { 
                    self.current_rom_bank = 1
                }
            }

            0x4000 | 0x5000 => {
                if value <= 0x03 {
                    self.current_ram_bank = value as usize;
                    self.rtc_banked = false;
                }
                else if value >= 0x08 && value <= 0x0C {
                    self.rtc_banked = true;
                } 
            }

            0x6000 | 0x7000 => {
                if self.prev_latch_val == 0x00 && value == 0x01 {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    
                    self.rtc_regs[0] = (now.as_secs() % 60) as u8;
                    self.rtc_regs[1] = ((now.as_secs() / 60) % 60) as u8;
                    self.rtc_regs[2] = (((now.as_secs() / 60) / 60) % 24) as u8;
                }
                
                self.prev_latch_val = value;
            }

            _ => panic!()
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if !self.is_ram_rtc_enabled { return 0xFF; }

        if self.rtc_banked {
            return self.rtc_regs[(addr - 0x08) as usize];
        }

        self.ram_banks[self.current_ram_bank][addr as usize]
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.is_ram_rtc_enabled { return }

        // what to do if rtc is banked?

        self.ram_banks[self.current_ram_bank][addr as usize] = value;
    }
}