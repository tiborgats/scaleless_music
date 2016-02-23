//!
//! Create scaleless music.
//!
#[cfg(target_os = "android")]
#[macro_use]
extern crate android_glue;

extern crate portaudio;
#[macro_use] extern crate lazy_static;

#[macro_use] extern crate conrod;
extern crate piston_window;
extern crate piston;
//extern crate graphics;
//extern crate glium_graphics;
//extern crate glium;

mod sound;

use piston_window::*;
//use std::thread;
//use std::time::Duration;

#[cfg(target_os = "android")]
android_start!(main);

fn main() {
    use sound::interface::*;
    use sound::instrument::*;

//    let opengl = OpenGL::V3_2;
    let window: PistonWindow = WindowSettings::new("Music", [320, 200])
            .exit_on_esc(true)
            .vsync(true)
//            .opengl(opengl)
            .build()
            .unwrap();

    let sound_generator = Box::new(InstrumentBasic::new(48000.0, 440.0).unwrap());
    let mut sound = SoundInterface::new(48000, 2, sound_generator).unwrap();

//    while let Some(event) = { let mut b = glutin_window.borrow_mut(); events.next(&mut *b) } {
    for event in window.ups(120) {
        event.draw_2d(|_c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
        });
        if let Some(Button::Keyboard(key)) = event.press_args() {
            sound.send_command(Command::Keypress{ key: key });
        }

//        thread::sleep(Duration::from_millis(1));
//        thread::sleep_ms(1);
    }

//    println!("Ready.")
}
