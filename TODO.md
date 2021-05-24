# TODO
- Improve disassembler code. E.g. replace blank closures with func that inserts a delay instruction step?
- Implement LCD Control bit 7 behaviour
- Am I handling interupts in the correct order if more than 1 are valid to fire?
- really need start irq blocking implemented probs

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

# PPU Advanced
- https://github.com/Powerlated/TurtleTests ?
- https://github.com/mattcurrie/mealybug-tearoom-tests
- STAT IRQs and Blocking


# Sound
- https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
- https://www.reddit.com/r/EmuDev/comments/5gkwi5/gb_apu_sound_emulation/dat3zni/
- https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware
- https://gbdev.gg8.se/wiki/articles/Sound_Controller
- https://gbdev.io/pandocs/#sound-controller
- https://www.reddit.com/r/EmuDev/comments/5gkwi5/gb_apu_sound_emulation/

- PCM: Pulse code modulation
- 3 params
    - nyquest(?) freq: half the sampling rate (usually 44.1 or 48 kHz). The
        highest freq the pcm will be able to "describe"
    - sample format: (8 bit, a-law, mew-law, 16/24 signed bit, 32 bit float) the format used to
        describe the value of the amplitude of the signal 
    - number of channels: usually 2. values are interleaved between the channels in the data stream.

- https://sound.stackexchange.com/questions/34816/what-is-a-noise-sweep
- https://github.com/simias/gb-rs/blob/master/src/ui/sdl2/audio.rs
- https://www.youtube.com/watch?v=72dI7dB3ZvQ
- https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/audio-squarewave.rs
- https://emudev.de/gameboy-emulator/bleeding-ears-time-to-add-audio/

# Node FFI
- https://stackoverflow.com/questions/36604010/how-can-i-build-multiple-binaries-with-cargo
- https://www.reddit.com/r/rust/comments/jg1qm2/electron_rust_how_to_talk_to_each_other/
- napi-rs

# Misc
- DMA bus conflicts (ram trashing thing?)
- Implement oam dma as a completely separate thing that ticks over time