// #![windows_subsystem = "windows"]

extern crate sdl2;
extern crate imgui;
extern crate imgui_sdl2;
extern crate gl;
extern crate imgui_opengl_renderer;

use std::{cell::RefCell, collections::VecDeque, ffi::c_void, process, rc::Rc};

use gameboy_rs::{gameboy::{GameBoy, spu::SAMPLES_PER_BUFFER}};
use gl::types::GLuint;
use imgui::{MenuItem, im_str};
use nfd2::Response;
use sdl2::{audio::AudioSpecDesired, pixels::PixelFormatEnum, surface::Surface, video::Window};

const SCALE: u32 = 2;
const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const MENU_BAR_HEIGHT: u32 = 19;
const CYCLES_PER_SCREEN_DRAW: u64 = 70_224;

fn main() {
    let mut gb: Option<GameBoy> = None;

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let audio_subsystem = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(48000 as i32),
        channels: Some(1),
        samples: Some(SAMPLES_PER_BUFFER as u16)
    };

    let audio_device = match audio_subsystem.open_queue::<f32, _>(None, &desired_spec) {
        Ok(d) => Rc::new(RefCell::new(d)),
        Err(e) => panic!("Unable to initialize audio queue: {:?}", e)
    };

    {
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 0);
    }

    let mut window = video.window("Frosty", WIDTH * SCALE, (HEIGHT * SCALE) + MENU_BAR_HEIGHT)
        .position_centered()
        .opengl()
        .allow_highdpi()
        .build()
        .unwrap();

    set_window_icon(&mut window);

    let _gl_context = window.gl_create_context().expect("Couldn't create GL context");
    gl::load_with(|s| video.gl_get_proc_address(s) as _);

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video.gl_get_proc_address(s) as _);

    let mut event_pump = sdl.event_pump().unwrap();

    // https://stackoverflow.com/questions/30488155/opengl-fastest-way-to-draw-2d-image

    let mut fb_id: GLuint = 0;
    let mut tex_id: GLuint = 0;
    init_gl_state(&mut tex_id, &mut fb_id);

    let mut paused = true;

    let timer = sdl.timer().unwrap();
    let mut turbo = false;

    let mut elapsed_ns: u64 = 0;
    const FRAMERATE_UPDATE_NS: u64 = 1000000;
    let mut comutative_speed: VecDeque<f64> = VecDeque::new();

    'running: loop {
        use sdl2::event::Event;

        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) { continue; }

            match event {
                sdl2::event::Event::KeyDown {
                    timestamp: _, 
                    window_id: _, 
                    keycode, 
                    scancode: _, 
                    keymod: _, 
                    repeat 
                } => {
                    if !repeat && keycode.is_some() {
                        let keycode = keycode.unwrap();
                        match keycode {
                            sdl2::keyboard::Keycode::Tab => {
                                turbo = true;
                                (*audio_device).borrow().pause();
                                comutative_speed.clear();
                            },
                            _ => {
                                if gb.is_some() && !paused {
                                    gb.as_mut().unwrap().key_down(keycode)
                                }                                
                            }
                        }
                    }
                },

                sdl2::event::Event::KeyUp {
                    timestamp: _, 
                    window_id: _, 
                    keycode, 
                    scancode: _, 
                    keymod: _, 
                    repeat 
                } => {
                    if !repeat && keycode.is_some() {
                        let keycode = keycode.unwrap();
                        match keycode {
                            sdl2::keyboard::Keycode::Tab => {
                                turbo = false;
                                let ad = (*audio_device).borrow();
                                ad.clear();
                                ad.queue(&[0.0; SAMPLES_PER_BUFFER]);
                                ad.resume();
                                comutative_speed.clear();
                            },
                            _ => {
                                if gb.is_some() && !paused {
                                    gb.as_mut().unwrap().key_up(keycode)
                                }                                
                            }
                        }
                    }
                },

                Event::Quit {..} => {
                    break 'running
                },
                _ => {}
            }
        }

        let start = timer.performance_counter();

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        unsafe {
            gl::ClearColor(0.4549, 0.92549, 0.968627, 0.7);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        if gb.is_some() && !paused {
            for _ in 0..CYCLES_PER_SCREEN_DRAW {
                gb.as_mut().unwrap().tick();
            }

            render_gb(gb.as_ref().unwrap(), fb_id, tex_id);
        }

        else if gb.is_some() && paused{
            render_paused_frame(fb_id, tex_id);
        }

        if gb.is_some() && !turbo {
            while (*audio_device).borrow().size() > SAMPLES_PER_BUFFER as u32 * 4 { }
        }

        let ui = imgui.frame();
        match ui.begin_main_menu_bar() {
            Some(mmb_token) => {
                match ui.begin_menu(im_str!("File"), true) {
                    Some(mm_token) => {
                        if MenuItem::new(im_str!("Load ROM")).build(&ui) {
                            (*audio_device).borrow().pause();

                            match nfd2::open_file_dialog(Some("gb"), None).expect("Hmm?") {
                                Response::Okay(file_path) => {
                                    let _gb = GameBoy::new(
                                        file_path.to_str().unwrap(), 
                                        Some(audio_device.clone())
                                    );
                                    gb = Some(_gb);

                                    let ad = (*audio_device).borrow();
                                    ad.queue(&[0.0; SAMPLES_PER_BUFFER]);
                                    ad.resume();
                                    paused = false;
                                },
                                
                                Response::OkayMultiple(files) => println!("Files {:?}", files),
                                Response::Cancel => println!("User canceled"),
                            }
                        }

                        let pause_resume_str = if paused { im_str!("Resume") } else { im_str!("Pause") };

                        if MenuItem::new(pause_resume_str).build(&ui) {
                            if !(paused && gb.is_none()) {
                                paused = !paused;

                                if paused {
                                    (*audio_device).borrow().pause();
                                } else {
                                    (*audio_device).borrow().resume();
                                }
                            }
                        }

                        if MenuItem::new(im_str!("Exit")).build(&ui) {
                            process::exit(0);
                        }

                        mm_token.end(&ui);
                    }
                    None => {}
                }

                mmb_token.end(&ui);
            }
            None => {}
        }

        let end = timer.performance_counter();

        elapsed_ns += end - start;

        if elapsed_ns > FRAMERATE_UPDATE_NS {
            elapsed_ns -= FRAMERATE_UPDATE_NS;
            let elapsed = (end - start) as f64 / timer.performance_frequency() as f64;
            let fps = 1.0f64 / elapsed;
            let percent = (fps / 59.7) * 100.0;

            comutative_speed.push_back(percent);
            if comutative_speed.len() > 100 {
                comutative_speed.pop_front();
            }

            let average_percent = comutative_speed.iter().sum::<f64>() / comutative_speed.len() as f64;

            window.set_title(format!("Frosty  [{:.0}%]", average_percent).as_str()).unwrap()
        }

        imgui_sdl2.prepare_render(&ui, &window);
        renderer.render(ui);

        window.gl_swap_window();
    }
}

fn init_gl_state(tex_id: &mut u32, fb_id: &mut u32) {
    unsafe {
        gl::GenTextures(1, tex_id);
        gl::BindTexture(gl::TEXTURE_2D, *tex_id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        let mut data: [u8; (WIDTH * HEIGHT * 3) as usize] = [0; (WIDTH * HEIGHT * 3) as usize];
        let mut i = 0usize;
        while i < (WIDTH * HEIGHT * 3) as usize {
            data[i] = 55;
            data[i + 1] = 55;
            data[i + 2] = 55;

            i += 3;
        }

        gl::TexImage2D(
            gl::TEXTURE_2D, 
            0, 
            gl::RGB as i32, 
            WIDTH as i32, 
            HEIGHT as i32, 
            0, 
            gl::RGB, 
            gl::UNSIGNED_BYTE, 
            data.as_ptr() as *const c_void
        );

        gl::BindTexture(gl::TEXTURE_2D, 0);

        // https://stackoverflow.com/questions/31482816/opengl-is-there-an-easier-way-to-fill-window-with-a-texture-instead-using-vbo

        gl::GenFramebuffers(1, fb_id);

        gl::ClearColor(0.4549, 0.92549, 0.968627, 0.7);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

fn render_gb(gb: &GameBoy, fb_id: GLuint, tex_id: GLuint) {
    let frame_buffer = gb.get_frame_buffer();
    let mut tex_data = [0u8; (WIDTH * HEIGHT * 3) as usize];
    let mut i: usize = 0;

    while i < (WIDTH * HEIGHT * 3) as usize {
        let index = i / 3;
        let color = frame_buffer[index];

        tex_data[i] = color;
        tex_data[i + 1] = color;
        tex_data[i + 2] = color;

        i += 3;
    }

    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex_id);
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            WIDTH as i32,
            HEIGHT as i32,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            tex_data.as_ptr() as *const c_void
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);

        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fb_id);
        gl::FramebufferTexture2D(
            gl::READ_FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            tex_id, 
            0
        );

        gl::BlitFramebuffer(
            0, 
            0, 
            WIDTH as i32, 
            HEIGHT as i32, 
            0, 
            ((HEIGHT * SCALE)) as i32, 
            (WIDTH * SCALE) as i32, 
            0, 
            gl::COLOR_BUFFER_BIT, 
            gl::NEAREST
        );
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
    }
}

fn render_paused_frame(fb_id: GLuint, tex_id: GLuint) {
    unsafe {
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fb_id);
        gl::FramebufferTexture2D(
            gl::READ_FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            tex_id, 
            0
        );

        gl::BlitFramebuffer(
            0, 
            0, 
            WIDTH as i32, 
            HEIGHT as i32, 
            0, 
            ((HEIGHT * SCALE)) as i32, 
            (WIDTH * SCALE) as i32, 
            0, 
            gl::COLOR_BUFFER_BIT, 
            gl::NEAREST
        );
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
    }
}

fn set_window_icon(window: &mut Window) {
    let icon_data: [u8; 16*16] = [
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
    ];

    let mut real_icon_data = [0u8; 16 * 16 * 3];
    let mut i = 0usize;
    while i < 16 * 16 * 3 {
        let index = i / 3;
        let color = icon_data[index];

        if color != 0 {
            real_icon_data[i] = 0;
            real_icon_data[i + 1] = 162;
            real_icon_data[i + 2] = 232;
        } else {
            real_icon_data[i] = !color;
            real_icon_data[i + 1] = !color;
            real_icon_data[i + 2] = !color;
        }

        i += 3;
    }

    let icon = Surface::from_data(
        &mut real_icon_data, 
        16, 
        16, 
        16 * 3, 
        PixelFormatEnum::RGB24
    ).unwrap();

    window.set_icon(icon);
}