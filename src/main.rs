//!
//! Create scaleless music.
//!
#[cfg(target_os = "android")]
#[macro_use]
extern crate android_glue;

extern crate portaudio;
#[macro_use] extern crate lazy_static;

extern crate piston;
extern crate glutin_window;
extern crate graphics;
extern crate glium_graphics;
extern crate glium;
#[macro_use] extern crate conrod;

mod sound;

//use std::thread;
//use std::time::Duration;
use std::rc::Rc;
use std::cell::RefCell;
use glium::Surface;
use glium_graphics::*;
use piston::event_loop::*;
//use piston::input::{RenderEvent, PressEvent, ReleaseEvent, TextEvent, UpdateEvent, Key};
use piston::input::*;
use piston::window::{ AdvancedWindow, WindowSettings, Size };
use glutin_window::*;//{ GlutinWindow, OpenGL };
//use glium::{DisplayBuild, Surface};

#[cfg(target_os = "android")]
android_start!(main);

fn main() {
    use sound::interface::*;
    use sound::instrument::*;

    let sound_generator = Box::new(InstrumentBasic::new(192000.0, 440.0).unwrap());
    let mut sound = SoundInterface::new(192000, 2, sound_generator).unwrap();

    let opengl = OpenGL::V3_2;
    let size = Size { width: 1024, height: 768 };
    let ref glutin_window: Rc<RefCell<GlutinWindow>> = Rc::new(RefCell::new(WindowSettings::new("Music", size)
            .exit_on_esc(true)
            .vsync(true)
            .opengl(opengl)
            .build()
            .unwrap()
        ));
    let ref mut glium_window = GliumWindow::new(glutin_window).unwrap();

//    let mut clip_inside = true;
//    let mut g2d = Glium2d::new(opengl, glium_window);

//    for event in glutin_window.events().ups(120).swap_buffers(false) {
    let mut events = glutin_window.borrow().events().swap_buffers(false);
    // Temporary fix for https://github.com/rust-lang/rust/issues/30832.
    while let Some(event) = { let mut b = glutin_window.borrow_mut(); events.next(&mut *b) } {
        if let Some(_args) = event.render_args() {
//            use graphics::*;

            let mut target = glium_window.draw();
            {
                target.clear_all((0.8, 0.8, 0.8, 1.0),1.0,0);
//                let ref mut g = GliumGraphics::new(&mut g2d, &mut target);
//                let c = Context::new_viewport(args.viewport());

//                        clear([0.8, 0.8, 0.8, 1.0], g);
//                        g.clear_stencil(0);
//                Rectangle::new([1.0, 0.0, 0.0, 1.0]);
//                            .draw([0.0, 0.0, 100.0, 100.0], &c.draw_state, c.transform, g);
            }

            target.finish().unwrap();
        }
        if let Some(Button::Keyboard(key)) = event.press_args() {
            sound.send_command(Command::Keypress{ key: key });
        }


//        thread::sleep(Duration::from_millis(1));
//        thread::sleep_ms(1);
    }

//    println!("Ready.")
}
