//!
//! Create scaleless music.
//!

//#![feature(question_mark)]

extern crate portaudio;
#[macro_use] extern crate lazy_static;
extern crate rayon;

#[macro_use] extern crate conrod;
extern crate piston_window;
extern crate piston;

mod sound;

use piston_window::*;

fn main() {
    use sound::backend_portaudio::*;
    use sound::instrument::*;
    let sound_generator = Box::new(InstrumentBasic2::new(48000.0, 440.0).unwrap());
    let mut sound = SoundInterface::new(48000, 2, sound_generator).unwrap();

    let mut window: PistonWindow = WindowSettings::new("Music", [320, 200])
            .exit_on_esc(true)
            .vsync(true)
            .build()
            .unwrap();

    sound.start().unwrap();

    while let Some(event) = window.next() {
        if let Some(button) = event.press_args() {
            if let Button::Keyboard(key) = button {
                sound.send_command(Command::Keypress{ key: key });
            } else {
                println!("Pressed {:?}", button);
            }
        }
        window.draw_2d(&event, |_c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
        });
    }
}
