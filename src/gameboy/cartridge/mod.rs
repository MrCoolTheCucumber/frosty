use std::{fs::File, io::Read};

use crate::gameboy::cartridge::{mbc1::MBC1, rom::ROM};


// https://gbdev.io/pandocs/#the-cartridge-header
// http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf Section 2.6 (page 13)

// pub struct Cartridge {
//     rom_bank_n: Vec<[u8; 0x4000]>,
//     rom_bank_index: usize
// }

pub mod mbc1;
pub mod rom;

pub trait Cartridge {
    fn read_rom(&self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, value: u8);

    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, value: u8);
}

pub fn create(rom_path: &str) -> Box<dyn Cartridge> {
    let mut rom_bank_0 = [0u8; 0x4000];

    let file = File::open(rom_path);
    let mut file = match file {
        Ok(f) => f,
        Err(err) => panic!("Something went wrong reading the ROM: {}", err)
    };

    file.read_exact(&mut rom_bank_0).ok();

    // parse cart header
    // CGB flag
    if rom_bank_0[0x143] == 0xC0 {
        panic!("This rom is only supported for game boy color");
    }

    let cartridge_type_code = rom_bank_0[0x147];
    let rom_size_code = rom_bank_0[0x148];
    let ram_size_code = rom_bank_0[0x149];

    // This includes rom bank 0
    let num_rom_banks: u16 = match rom_size_code {
        0x00 => 2,   // 32KB
        0x01 => 4,   // 64KB
        0x02 => 8,   // 128KB
        0x03 => 16,  // 256KB
        0x04 => 32,  // 512KB
        0x05 => 64,  // 1MB
        0x06 => 128, // 2MB
        0x07 => 256, // 4MB
        0x08 => 512, // 8MB

        // pandocs says there are some other special codes
        // but is not sure if they are legit
        // lets define them anyway
        0x52 => 72,  // 1.1MB
        0x53 => 80,  // 1.2MB
        0x54 => 96,  // 1.5MB
        
        _ => panic!("Cartridge has invalid ROM size code? Code: {:#04X}", rom_size_code)
    };

    let num_ram_banks: u16 = match ram_size_code {
        0x00 => 0,
        0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,

        _ => panic!("Cartridge has invalid RAM size code? Code: {:#04X}", ram_size_code)
    };

    match cartridge_type_code {
        0x00 => Box::new(ROM::new(file, rom_bank_0)),
        0x01 | 0x02 | 0x03 => {
            println!("MBC1 cart created!");
            Box::new(MBC1::new(
                file, 
                rom_bank_0,
                cartridge_type_code, 
                num_rom_banks, 
                num_ram_banks
            ))
        }

        _ => unimplemented!("Unable to handle cartridge type: {:#04X}", cartridge_type_code)
    }
}
