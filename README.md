# GameBoy Emulator written in Rust

Still a work in progress but 32KB roms should mostly work (e.g. Tetris) and MBC1 cartridges that dont have ram dimms in it

![image](https://user-images.githubusercontent.com/16002713/117520192-ebc23900-af9e-11eb-94b0-c4e67b1e6ac6.png)
![image](https://user-images.githubusercontent.com/16002713/118184155-e8153300-b432-11eb-8449-ef6f9a58b9cc.png)


## Tests
All Blargg cpu_instrs and instr_timing tests passing, as well as the dmg-acid2 ppu test!

![image](https://user-images.githubusercontent.com/16002713/117557463-0235c680-b06b-11eb-969c-40ea69976beb.png)
![image](https://user-images.githubusercontent.com/16002713/117734032-83679780-b1ea-11eb-868f-7b937e2e6cd8.png)
![image](https://user-images.githubusercontent.com/16002713/117857229-74cdbe80-b284-11eb-833e-98285873fbfe.png)


## TODO:
- Imlpement the HALT bug for true accuracy?
- Implement Sound? Not sure how hard it is
- Implement all cartridge types
- [Bonus] Pass all mooneye tests? A lot already pass

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
