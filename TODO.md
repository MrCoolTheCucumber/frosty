# TODO
- Review all CC Ops for correct timing, write a rust test that goes through the opcode timing table in discord?!
- Improve disassembler code. E.g. replace blank closures with func that inserts a delay instruction step?

- Switch to SDL2/OpenGL? http://nercury.github.io/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-01-window.html

https://github.com/Gekkio/mooneye-gb/tree/master/tests/acceptance
https://gekkio.fi/files/mooneye-gb/latest/tests/acceptance/
https://github.com/Powerlated/TurtleTests

# Mooneye todo's
ei_timings: when ei happens, if IE & IF != 0 already then it takes 4 clock cycles before the interrupt takes place?

# PPU
- https://github.com/Hacktix/scribbltests

- https://gbdev.io/pandocs/#pixel-fifo
- https://hacktixme.ga/GBEDG/ppu/#:~:text=The%20PPU%20(which%20stands%20for,beats%20the%20CPU%20by%20far
- https://github.com/Gekkio/mooneye-gb/tree/2d52008228557f9e713545e702d5b7aa233d09bb/tests/acceptance/ppu
- https://gbdev.io/pandocs/#int-48-stat-interrupt RISING EDGE, need to store prev val?
- https://www.reddit.com/r/EmuDev/comments/aihkvs/gb_some_interupt_questions/
- LY = LYC is triggered on every LYC write too! Need to handle this in the mmu struct?
- STAT IRQs and Blocking
- https://github.com/mattcurrie/mealybug-tearoom-tests

# Sound
- https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
- https://www.reddit.com/r/EmuDev/comments/5gkwi5/gb_apu_sound_emulation/dat3zni/
- https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware
- https://gbdev.gg8.se/wiki/articles/Sound_Controller
- https://gbdev.io/pandocs/#sound-controller

# Misc
- DMA bus conflicts (ram trashing thing?)