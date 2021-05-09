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

# Mooneye todo's
ei_timings: when ei happens, if IE & IF != 0 already then it takes 4 clock cycles before the interrupt takes place?