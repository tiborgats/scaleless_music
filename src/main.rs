//!
//! Create scaleless music.
//!
#[cfg(target_os = "android")]
#[macro_use]
extern crate android_glue;

extern crate portaudio;
#[macro_use] extern crate lazy_static;
extern crate rayon;

#[macro_use] extern crate conrod;
extern crate piston_window;
extern crate piston;
//extern crate graphics;
//extern crate glium_graphics;
//extern crate glium;

mod sound;

use piston_window::*;

#[cfg(target_os = "android")]
android_start!(main);

fn main() {
    use sound::interface::*;
    use sound::instrument::*;
    let sound_generator = Box::new(InstrumentBasic::new(48000.0, 440.0).unwrap());
    let mut sound = SoundInterface::new(48000, 2, sound_generator).unwrap();

    let window: PistonWindow = WindowSettings::new("Music", [320, 200])
            .exit_on_esc(true)
            .vsync(true)
            .build()
            .unwrap();

    sound.start().unwrap();

    for event in window {
        if let Some(button) = event.press_args() {
            if let Button::Keyboard(key) = button {
                sound.send_command(Command::Keypress{ key: key });
            } else {
                println!("Pressed {:?}", button);
            }
        }
        event.draw_2d(|_c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
        });
    }
}
