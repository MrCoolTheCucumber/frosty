# TODO
- stack push and pops are implemented as a blank step and then a write to temp step and assign step. Write (or read) step
should be split over 2 steps and the blank step should be removed. (Though this probably isn't a big deal in terms of timming accuracy)
- Review all CC Ops for correct timing
- Switch to SDL2/OpenGL? http://nercury.github.io/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-01-window.html

- Is something like this possible in rust? 
``` C++
union Register {
        struct {
            u8 lo;
            u8 hi;
        };
        u16 value;
    };
Register AF;
```
- 01 PASSED!
- 03 hangs?
- 04 ADC, SBC FAIL
- 05 PASSED!
- 06 PASSED!

07 MAY BE THE THE REASON FOR WEIRD RESTARTS?
- 07 need halt impl for this to work

https://github.com/retrio/gb-test-roms/blob/master/cpu_instrs/source/08-misc%20instrs.s
- 08 just restarts endlessly? one of them must be borked completely NEED TO FIX THIS

- 09
- 10 PASSED!
- 11 RES  x,(HL) where x = 0, 1, 3 failed ??????????? 
