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
