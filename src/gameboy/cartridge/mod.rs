use std::{fs::File, io::{Error, Read}, path::{Path, PathBuf}};

use crate::gameboy::cartridge::{mbc1::MBC1, mbc3::MBC3, mbc5::MBC5, rom::ROM};

// https://gbdev.io/pandocs/#the-cartridge-header
// http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf Section 2.6 (page 13)


pub mod rom;
pub mod mbc1;
pub mod mbc3;
pub mod mbc5;

pub trait Cartridge {
    fn read_rom(&self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, value: u8);

    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, value: u8);
}

pub fn create(rom_path: &str) -> Box<dyn Cartridge> {
    let mut rom_bank_0 = [0u8; 0x4000];

    let path = Path::new(rom_path);
    let file = File::open(path);
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
        
        0x0F..=0x13 => {
            println!("MBC3 cart created!");
            Box::new(MBC3::new(
                file,
                path,
                rom_bank_0,
                cartridge_type_code,
                num_rom_banks,
                num_ram_banks
            ))
        }

        0x1A..=0x1E => {
            println!("MBC5 cart created!");
            Box::new(MBC5::new(
                file,
                path,
                rom_bank_0,
                cartridge_type_code,
                num_rom_banks,
                num_ram_banks
            ))
        }

        _ => unimplemented!("Unable to handle cartridge type: {:#04X}", cartridge_type_code)
    }
}

fn get_save_file_path_from_rom_path(path: &Path) -> PathBuf {
    let mut save_file_path = PathBuf::from(path);
    save_file_path.pop();
    let rom_name = path.file_name().unwrap().to_str().unwrap();
    let mut save_name = rom_name[..3].to_owned();
    save_name.push_str(".sav");
    save_file_path.push(save_name);
    save_file_path
}

fn try_read_save_file(sav_file: Result<File, Error>, num_ram_banks: u16, ram_banks: &mut Vec<[u8; 0x2000]>,) {
    match sav_file {
        Ok(mut file) => {
            let mut buf: Vec<u8> = Vec::new();
            let read_result = file.read_to_end(&mut buf);
            match read_result {
                Ok(_) => {
                    let bytes_read = buf.len();
                    if bytes_read != num_ram_banks as usize * 0x2000 {
                        println!(
                            "Save file was an unexpected length. Expected {}, actual: {}",
                            num_ram_banks as usize * 0x2000,
                            bytes_read
                        );
                    }
                    else {
                        // load save file
                        load_new_ram(ram_banks, num_ram_banks);
                        let mut index: usize = 0;
                        for bank in ram_banks {
                            for i in 0..0x2000 {
                                bank[i] = buf[index];
                                index += 1;
                            }
                        }
                        println!("Save file loaded!");
                    }
                }

                Err(_) => {
                    load_new_ram(ram_banks, num_ram_banks);
                }
            }
        }

        Err(_) => {
            load_new_ram(ram_banks, num_ram_banks);
        }
    }
}

fn load_new_ram(ram_banks: &mut Vec<[u8; 0x2000]>, num_ram_banks: u16) {
    // fill ram banks with blank memory
    for _ in 0..num_ram_banks {
        let bank = [0; 0x2000];
        ram_banks.push(bank);
    }
}
