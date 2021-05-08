# TODO
- stack push and pops are implemented as a blank step and then a write to temp step and assign step. Write (or read) step
should be split over 2 steps and the blank step should be removed. (Though this probably isn't a big deal in terms of timming accuracy)
- Review all CC Ops for correct timing
- Switch to SDL2/OpenGL? http://nercury.github.io/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-01-window.html
- make use of `unreachable!()` ?

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
https://github.com/Gekkio/mooneye-gb/tree/master/tests/acceptance
https://gekkio.fi/files/mooneye-gb/latest/tests/acceptance/
https://github.com/Powerlated/TurtleTests

- 01 PASSED!
- 02 https://www.reddit.com/r/EmuDev/comments/5qa3x1/timer_doesnt_work_properly_failed_2/ need to check all timings in the disassembler
- 03 PASSED!
- 04 PASSED!
- 05 PASSED!
- 06 PASSED!
- 07 PASSED!
- 08 PASSED!
- 09 PASSED!
- 10 PASSED!
- 11 PASSED!
