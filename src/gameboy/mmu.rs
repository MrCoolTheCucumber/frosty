use rand::Rng;

use super::{cartridge::Cartridge, input::Input, interupt::{InterruptFlag, Interupt}, ppu::PpuMode, spu::Spu, timer::Timer};

const PALETTE: [u8; 4] = [
    255, 192, 96, 0
];

pub struct Mmu {
    pub spu: Spu,
    pub interupts: Interupt,
    pub input: Input,
    pub timer: Timer,
    cartridge: Box<dyn Cartridge>,

    gpu_vram: [u8; 0x2000],
    working_ram: [u8; 0x2000],

    pub io: [u8; 0x100],
    zero_page: [u8; 0x80],

    pub sprite_table: [u8; 0xA0],
    pub sprite_palette: [[u8; 4]; 2],

    pub tileset: [[[u8; 8]; 8]; 384],
    pub bg_palette: [u8; 4],

    dma_transfer_index: u16,
    dma_transfer_base_addr: u16,

    dma_queue_counter: u8,
    dma_queue_val: u16,

    dma_active: bool,
    dma_active_clock: u8,

    stat_irq_state: bool
}

impl Mmu {
    pub fn new(cartridge: Box<dyn Cartridge>, spu: Spu) -> Self {
        let mut mmu = Self {
            spu,
            interupts: Interupt::new(),
            input: Input::new(),
            timer: Timer::new(),
            cartridge,

            gpu_vram: [0; 0x2000],
            working_ram: [0; 0x2000],
            io: [0; 0x100],
            zero_page: [0; 0x80],

            sprite_table: [0; 0xA0],
            sprite_palette: [
                [PALETTE[0], PALETTE[1], PALETTE[2], PALETTE[3]],
                [PALETTE[0], PALETTE[1], PALETTE[2], PALETTE[3]]
            ],

            // ppu
            tileset: [[[0; 8]; 8]; 384],
            bg_palette: [
                PALETTE[0], PALETTE[1], PALETTE[2], PALETTE[3]
            ],

            dma_transfer_base_addr: 0,
            dma_transfer_index: 0,

            dma_queue_counter: 0,
            dma_queue_val: 0,

            dma_active: false,
            dma_active_clock: 0,

            stat_irq_state: false
        };

        mmu.randomize_ram_values();
        mmu.setup_uninit_ram();

        // set up zero page mem
        mmu.write_byte(0xFF02, 0x7E);
        mmu.write_byte(0xFF10, 0x80);
        mmu.write_byte(0xFF11, 0xBF);
        mmu.write_byte(0xFF12, 0xF3);
        mmu.write_byte(0xFF14, 0xBF);
        mmu.write_byte(0xFF16, 0x3F);
        mmu.write_byte(0xFF19, 0xBF);
        mmu.write_byte(0xFF1A, 0x7A);
        mmu.write_byte(0xFF1B, 0xFF);
        mmu.write_byte(0xFF1C, 0x9F);
        mmu.write_byte(0xFF1E, 0xBF);
        mmu.write_byte(0xFF20, 0xFF);
        mmu.write_byte(0xFF23, 0xBF);
        mmu.write_byte(0xFF24, 0x77);
        mmu.write_byte(0xFF25, 0xF3);
        mmu.write_byte(0xFF26, 0xF1);
        mmu.write_byte(0xFF40, 0x91);
        mmu.write_byte(0xFF47, 0xFC);
        mmu.write_byte(0xFF48, 0xFF);
        mmu.write_byte(0xFF49, 0xFF);

        mmu
    }

    fn randomize_ram_values(&mut self) {
        let mut rng = rand::thread_rng();
        
        for val in &mut self.working_ram {
            *val = rng.gen_range(0..=u8::MAX);
        }
    }

    fn setup_uninit_ram(&mut self) {
        let addresses: Vec<usize> = vec![
            // K = A, L = B, M = C, N = D, O = E, P = F
            0x03, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x15, 0x1F, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 
            0x2D, 0x2E, 0x2F, 0x4D, 0x4E, 0x4F, 0x57
        ];

        for addr in &addresses {
            self.io[*addr] = 0xFF;
        }

        for i in 0x50..=0x7F {
            self.io[i] = 0xFF;
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr & 0xF000 {
            // rom_bank_0
            0x0000 | 0x1000 | 0x2000 | 0x3000 |
            0x4000 | 0x5000 | 0x6000 | 0x7000 => {
                self.cartridge.read_rom(addr)
            }

            // vram
            0x8000 | 0x9000 => {
                self.gpu_vram[(addr - 0x8000) as usize]
            }
            
            // cart ram
            0xA000 | 0xB000 => {
                self.cartridge.read_ram(addr - 0xA000)
            }
            
            // internal ram
            0xC000 | 0xD000 => {
                self.working_ram[(addr - 0xC000) as usize]
            }

            // 0xE000 to 0xFFxx is a mirror of the internal ram

            0xE000 => {
                self.working_ram[(addr - 0xE000) as usize]
            }

            0xF000 => {
                match addr & 0x0F00 {
                    0x0000 | 0x0100 | 0x0200 | 0x0300 | 0x0400 |
                    0x0500 | 0x0600 | 0x0700 | 0x0800 | 0x0900 |
                    0x0A00 | 0x0B00 | 0x0C00 | 0x0D00 => {
                        return self.working_ram[(addr - 0xE000) as usize];
                    },

                    0x0E00 => {
                        if addr < 0xFEA0 {
                            // if we are in mode 2, return 0xFF
                            let mode = self.io[0x41] & 3;
                            // if mode == 3 || mode == 2 {
                            //     return 0xFF;
                            // }

                            if self.dma_active { return 0xFF }

                            return self.sprite_table[(addr - 0xFE00) as usize];
                        }

                        // FEAO -> FEFF
                        // "Empty but usable for io"?
                        // Some just return here
                        return 0;
                    },

                    0x0F00 => {
                        if addr == 0xFF00 {
                            return self.input.read_joyp();
                        }

                        else if addr == 0xFF0F {
                            return 0b1110_0000 | (self.interupts.flags & 0b0001_1111);
                        }

                        // LCD STAT
                        else if addr == 0xFF41 {
                            return 0b1000_0000 | self.io[0x41];
                        }
                      
                        else if addr == 0xFFFF {
                            return self.interupts.enable
                        }

                        else if addr >= 0xFF80 && addr <= 0xFFFE {
                            return self.zero_page[(addr - 0xFF80) as usize]
                        }

                        else if addr >= 0xFF04 && addr <= 0xFF07 {
                            return self.timer.read(addr)
                        }

                        // SOUND
                        else if addr == 0xFF10 {
                            return self.spu.get_nr10()
                        }

                        else if addr == 0xFF11 {
                            return self.spu.get_nr11()
                        }

                        else if addr == 0xFF12 {
                            return self.spu.get_nr12()
                        }

                        else if addr == 0xFF13 {
                            return self.spu.get_nr13()
                        }

                        else if addr == 0xFF14 {
                            return self.spu.get_nr14()
                        }

                        else if addr == 0xFF16 {
                            return self.spu.get_nr21()
                        }

                        else if addr == 0xFF17 {
                            return self.spu.get_nr22()
                        }

                        else if addr == 0xFF18 {
                            return self.spu.get_nr23()
                        }

                        else if addr == 0xFF19 {
                            return self.spu.get_nr24()
                        }

                        else if addr == 0xFF1A {
                            return self.spu.get_nr30();
                        }

                        else if addr == 0xFF1B {
                            return self.spu.get_nr31();
                        }

                        else if addr == 0xFF1C {
                            return self.spu.get_nr32();
                        }

                        else if addr == 0xFF1D {
                            return self.spu.get_nr33();
                        }

                        else if addr == 0xFF1E {
                            return self.spu.get_nr34();
                        }

                        else if addr >= 0xFF30 && addr <= 0xFF3F {
                            return self.spu.get_sample((addr - 0xFF30) as u8);
                        }

                        else if addr == 0xFF20 {
                            return self.spu.get_nr41()
                        }

                        else if addr == 0xFF21 {
                            return self.spu.get_nr42()
                        }

                        else if addr == 0xFF22 {
                            return self.spu.get_nr43()
                        }
                        

                        else if addr == 0xFF23 {
                            return self.spu.get_nr44()
                        }

                        else if addr == 0xFF24 {
                            return self.spu.get_nr50();
                        }

                        else if addr == 0xFF25 {
                            return self.spu.get_nr51();
                        }

                        else if addr == 0xFF26 {
                            return self.spu.get_nr52();
                        }

                        else if addr >= 0xFF00 && addr <= 0xFF7F {
                            return self.io[(addr - 0xFF00) as usize]
                        } 
                        
                        else {
                            panic!("unhandled byte read from memory! Addr: {:#X}", addr);
                        }
                    },

                    _ => {
                        println!("Unhandled branch in read request for mem (0xFxxx): {:#X}", addr);
                        std::process::exit(0);
                    }
                }
            }

            _ => {
                panic!("Unhandled read at addr {:#06X}", addr);
            }
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr & 0xF000 {
            0x0000 | 0x1000 | 0x2000 | 0x3000 | 0x4000 |
            0x5000 | 0x6000 | 0x7000 => {
                self.cartridge.write_rom(addr, val);
            }

            // vram
            0x8000 | 0x9000 => {
                self.gpu_vram[(addr - 0x8000) as usize] = val;
            }

            0xA000 | 0xB000 => {
                self.cartridge.write_ram(addr - 0xA000, val);
            }

            0xC000 | 0xD000 => {
                self.working_ram[(addr - 0xC000) as usize] = val;
            },

            0xE000 => {
                self.working_ram[(addr - 0xE000) as usize] = val;
            },

            0xF000 => {
                match addr & 0x0F00 {
                    0x0000 | 0x0100 | 0x0200 | 0x0300 | 0x0400 |
                    0x0500 | 0x0600 | 0x0700 | 0x0800 | 0x0900 |
                    0x0A00 | 0x0B00 | 0x0C00 | 0x0D00 => {
                        self.working_ram[(addr - 0xE000) as usize] = val;
                    },

                    0x0E00 => {
                        if self.dma_active { return; }

                        if addr < 0xFEA0 {
                            self.sprite_table[(addr - 0xFE00) as usize] = val;
                        }

                        // "Empty but usable for io"?
                        // Some just return here
                        return;
                    },

                    0x0F00 => {
                        if addr == 0xFF00 {
                            self.input.set_column_line(val);
                        }

                        else if addr >= 0xFF80 && addr <= 0xFFFE {
                            self.zero_page[(addr - 0xFF80) as usize] = val;
                        }

                        else if addr >= 0xFF03 && addr <= 0xFF07 {
                            self.timer.write(addr, val);
                        }

                        else if addr == 0xFF0F {
                            self.interupts.flags = val;
                        }

                        // LCD CONTROL
                        else if addr == 0xFF40 {
                            self.io[0x40] = val;
                            
                            // is the lcd set to off?
                            if val >> 7 == 0 {
                                // reset ly to 0
                                self.io[0x44] = 0;
                            }
                        }

                        else if addr == 0xFF41 {
                            let stat = self.io[0x41];
                            self.io[0x41] = (stat & 0b1000_0111) | (val & 0b0111_1000);

                            self.update_stat_irq_conditions(String::from("STAT WRITE"));
                        }

                        else if addr == 0xFF44 {
                            // Do nothing, this is read only ?
                        }

                        else if addr == 0xFF45 {
                            self.io[0x45] = val;

                            if val == self.io[0x44] {
                                self.io[0x41] = self.io[0x41] | 0b0000_0100;
                            } else {
                                self.io[0x41] = self.io[0x41] & 0b1111_1011;
                            }
                        }

                        else if addr == 0xFF46 {
                            // let source_addr: u16 = (val as u16) << 8;

                            // for i in 0..160 {
                            //     let src_val = self.read_byte(source_addr + i);
                            //     self.write_byte(0xFE00 + i, src_val);
                            // }
                            self.dma_queue(val);

                            self.io[0x46] = val;
                        }

                        else if addr == 0xFF47 {
                            for i in 0..4 {
                                self.bg_palette[i] = PALETTE[((val >> (i * 2)) & 3) as usize];
                            }

                            self.io[0x47] = val;
                        }

                        else if addr == 0xFF48 {
                            for i in 0..4 {
                                self.sprite_palette[0][i] = PALETTE[((val >> (i * 2)) & 3) as usize];
                            }

                            self.io[0x48] = val;
                        }

                        else if addr == 0xFF49 {
                            for i in 0..4 {
                                self.sprite_palette[1][i] = PALETTE[((val >> (i * 2)) & 3) as usize];
                            }

                            self.io[0x49] = val;
                        }

                        // SOUND
                        else if addr == 0xFF10 {
                            self.spu.set_nr10(val);
                        }

                        else if addr == 0xFF11 {
                            self.spu.set_nr11(val);
                        }

                        else if addr == 0xFF12 {
                            self.spu.set_nr12(val);
                        }

                        else if addr == 0xFF13 {
                            self.spu.set_nr13(val);
                        }

                        else if addr == 0xFF14 {
                            self.spu.set_nr14(val);
                        }

                        else if addr == 0xFF16 {
                            self.spu.set_nr21(val);
                        }

                        else if addr == 0xFF17 {
                            self.spu.set_nr22(val);
                        }

                        else if addr == 0xFF18 {
                            self.spu.set_nr23(val);
                        }

                        else if addr == 0xFF19 {
                            self.spu.set_nr24(val);
                        }

                        else if addr == 0xFF1A {
                            self.spu.set_nr30(val);
                        }

                        else if addr == 0xFF1B {
                            self.spu.set_nr31(val);
                        }

                        else if addr == 0xFF1C {
                            self.spu.set_nr32(val);
                        }

                        else if addr == 0xFF1D {
                            self.spu.set_nr33(val);
                        }

                        else if addr == 0xFF1E {
                            self.spu.set_nr34(val);
                        }

                        else if addr >= 0xFF30 && addr <= 0xFF3F {
                            self.spu.set_sample((addr - 0xFF30) as u8, val);
                        }

                        else if addr == 0xFF20 {
                            self.spu.set_nr41(val);
                        }

                        else if addr == 0xFF21 {
                            self.spu.set_nr42(val);
                        }

                        else if addr == 0xFF22 {
                            self.spu.set_nr43(val);
                        }

                        else if addr == 0xFF23 {
                            self.spu.set_nr44(val);
                        }

                        else if addr == 0xFF24 {
                            self.spu.set_nr50(val);
                        }
                        else if addr == 0xFF25 {
                            self.spu.set_nr51(val);
                        }

                        else if addr == 0xFF26 {
                            self.spu.set_nr52(val);
                        }

                        else if addr >= 0xFF00 && addr <= 0xFF7F {
                            self.io[(addr - 0xFF00) as usize] = val;
                        }
                        
                        else if addr == 0xFFFF {
                            self.interupts.enable = val;
                        } 
                        
                        else {
                            panic!("unhandled byte write to memory! Addr: {:#X} Val: {:#X}", addr, val);
                        }
                    },

                    _ => {
                        println!("Unhandled branch in write request for mem (0xFxxx): {:#X}, Val: {:#X}", addr, val);
                        std::process::exit(0);
                    }
                }
            },

            _ => {
                println!("Unhandled write request for mem address: {:#X}, Val: {:#X}", addr, val);
                std::process::exit(0);
            }
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        self.read_byte(addr) as u16 + ((self.read_byte(addr + 1) as u16) << 8)
    }

    pub fn write_word(&mut self, addr: u16, val: u16) {
        let lower_val: u8 = (val & 0x00FF) as u8;
        let higher_val: u8 = ((val & 0xFF00) >> 8) as u8;

        self.write_byte(addr, lower_val);
        self.write_byte(addr + 1, higher_val);
    }

    pub fn dma_queue(&mut self, val: u8) {
        self.dma_queue_val = val as u16;
        self.dma_queue_counter = 5;
    }
    
    pub fn dma_tick(&mut self) {
        // tick transfer, if active
        if self.dma_active {
            self.dma_active_clock += 1;

            if self.dma_active_clock == 4 { 
                let src_val = self.read_byte(self.dma_transfer_base_addr + self.dma_transfer_index);
                self.sprite_table[self.dma_transfer_index as usize] = src_val;
                self.dma_transfer_index += 1;
    
                if self.dma_transfer_index == 160 {
                    self.dma_active = false;
                }

                self.dma_active_clock = 0;
            }
        }

        // tick queue
        if self.dma_queue_counter > 0 {
            self.dma_queue_counter -= 1;
    
            if self.dma_queue_counter == 0 {
                self.dma_transfer_base_addr = self.dma_queue_val << 8;
                self.dma_transfer_index = 0;
                self.dma_active = true;
                self.dma_active_clock = 0;
            }
        }
    }

    pub fn update_stat_irq_conditions(&mut self, src: String) {
        let stat = self.io[0x41];
        let mut stat_irq_state = false;

        let mut debug = String::from("");

        let mode = PpuMode::from_u8(stat & 0b0000_0011);
        match mode {
            PpuMode::HBlank => {
                if stat & 0b0000_1000 != 0 {
                    stat_irq_state = true;
                    debug.push_str("Hblank ");
                }
            }

            PpuMode::VBlank => {
                if stat & 0b0001_0000 != 0 {
                    stat_irq_state = true;
                    debug.push_str("VBlank ");
                }
            }

            PpuMode::OAM => {
                if stat & 0b0010_0000 != 0 {
                    stat_irq_state = true;
                    debug.push_str("OAM ");
                }
            }

            _ => {}
        }

        if stat & 0b0100_0000 != 0 {
            let ly = self.io[0x44];
            let lyc = self.io[0x45];

            if ly == lyc {
                stat_irq_state = true;
                debug.push_str("LY=LYC ");
            }
        }

        // rising edge
        if !self.stat_irq_state && stat_irq_state {
            self.interupts.request_interupt(InterruptFlag::Stat);
        }

        self.stat_irq_state = stat_irq_state;
    }
}