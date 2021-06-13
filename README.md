# Frosty

Frosty is a GameBoy emulator written in rust.

![image](https://user-images.githubusercontent.com/16002713/119414812-330f3000-bce8-11eb-9eac-b12482dbc3f2.png)
![image](https://user-images.githubusercontent.com/16002713/119414937-78336200-bce8-11eb-96d3-62a601df39a1.png)
![image](https://user-images.githubusercontent.com/16002713/119415269-2b03c000-bce9-11eb-9185-022d400062cb.png)
![image](https://user-images.githubusercontent.com/16002713/119415428-833ac200-bce9-11eb-9253-2d06c72ff08a.png)


## TODO:
- Improve ppu timings
- Re-implement sound. Current sound is ok, but its missing a lot of the required quirks.
- Implement all cartridge types. Currently ROM and MBC1/3/5  (which is a lot to be fair)
- Pass the sub-instruction timing tests (completely uselses? but nice to have). This is very easy to do.

## Tests
All Blargg cpu_instrs and instr_timing tests passing, as well as the dmg-acid2 ppu test!


### Blargg Tests

- [x] cpu_instrs
    - [x] 01-special
    - [x] 02-interrupts
    - [x] 03-op sp,hl
    - [x] 04-op r,imm
    - [x] 05-op rp
    - [x] 06-ld r,r
    - [x] 07-jr,jp,call,ret,rst
    - [x] 08-misc instrs
    - [x] 09-op r,r
    - [x] 10-bit ops
    - [x] 11-op a,(hl)
- [x] instr_timing
- [ ] halt_bug

### Hacktix scribbltests
- [x] fairylake
- [x] lycscx
- [x] lycscy
- [x] palettely
- [x] scxly
- [x] statcount
- [x] winpos

### DMG-Acid2

![image](https://user-images.githubusercontent.com/16002713/117734032-83679780-b1ea-11eb-868f-7b937e2e6cd8.png)

### Mooneye Tests

- [ ] Bits
    - [x] mem_oam
    - [x] reg_f
    - [ ] unused_hwio
- [x] Instr
    - [x] daa
- [ ] Interrupts
    - [ ] ie_push
- [ ] oam dma
    - [x] basic
    - [x] reg_read
    - [ ] sources
- [ ] serial
    - [ ] boot_sclk_align 
- [x] Timer
    - [x] div_write
    - [x] rapid_toggle
    - [x] tim00
    - [x] tim00_div_trigger
    - [x] tim01
    - [x] tim01_div_trigger
    - [x] tim10
    - [x] tim10_div_trigger
    - [x] tim11
    - [x] tim11_div_trigger
    - [x] tima_reload
    - [x] tima_write_reloading
    - [x] tma_write_reloading
- [x] add_sp_e_timing
- [x] boot_div-dmgABCmgb
- [ ] boot_hwio-dmgABCmgb (might require stat ir blocking?)
- [x] boot_regs-dmgABC
- [x] call_cc_timing
- [x] call_cc_timing2
- [x] call_timing
- [x] call_timing2
- [x] di_timing
- [x] div_timing
- [x] ei_sequence
- [x] ei_timing
- [x] halt_ime0_ei
- [x] halt_ime0_nointr_timing
- [x] halt_ime1_timing
- [x] halt_ime1_timing2
- [x] if_ie_registers
- [x] intr_timing
- [x] jp_cc_timing
- [x] jp_timing
- [x] ld_hl_sp_e_timing
- [x] oam_dma_restart
- [x] oam_dma_start
- [x] oam_dma_timing
- [ ] pop_timing
- [ ] push_timing
- [ ] rapid_di_ei
- [x] ret_cc_timing
- [x] ret_timing
- [x] reti_intr_timing
- [x] reti_timing
- [ ] rst_timing

### TurtleTests

- [x] window_y_trigger
- [x] window_y_trigger_wx_offscreen

## References Used
- https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
- https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
- https://izik1.github.io/gbops/
- http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
- http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-The-CPU
- https://gbdev.io/pandocs/
- https://robertovaccari.com/blog/2020_09_26_gameboy/
- http://www.devrs.com/gb/files/faqs.html
- The kind people of the EmuDev discord
