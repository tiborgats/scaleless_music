//! This example is a basic instrument playing only one tone. Keyboard is the user input.
//! The tone is a very simple function (nothing like a real instrument). It's purpose is
//! only the testing of some intervals.
//!
//! The keys from 'Q' to 'O' changes the frequency to be higher,
//! the keys from 'A' to 'L' changes the frequency to be lower. Other keys play the previous
/// frequency. To quit press 'Esc'.

// #![feature(question_mark)]

extern crate music;
extern crate rayon;

#[macro_use]
extern crate conrod;
extern crate piston_window;
extern crate piston;

use music::sound::*;
use music::sound::wave::*;
use music::sound::frequency::*;
use music::sound::amplitude::*;
use std::rc::Rc;
use rayon::prelude::*;
use piston_window::*;

/// Commands of the messages from the UI thread to the playback thread.
pub enum Command {
    /// Mute
    Mute,
    /// Keyboard event
    Keypress {
        key: keyboard::Key,
    },
    /// Multiply frequency by value1
    FrequencyMultiple {
        numerator: u16,
        denominator: u16,
    },
}

pub struct InstrumentBasic {
    sample_rate: SampleCalc,
    note1: Note,
    frequency1: Rc<FrequencyConst>,
    frequency1_buffer: Vec<SampleCalc>,
    time: SampleCalc,
}

impl InstrumentBasic {
    /// Custom constructor
    pub fn new(sample_rate: SampleCalc,
               frequency_start: SampleCalc)
               -> SoundResult<InstrumentBasic> {
        let frequency1 = Rc::new(try!(FrequencyConst::new(440.0)));
        let amplitude = {
            let overtones_amplitude: Vec<SampleCalc> = vec![3.0, 4.5, 1.0, 0.9, 0.7, 0.5, 0.4, 3.5];
            let overtones_dec_rate: Vec<SampleCalc> = vec![-1.0, -1.4, -1.9, -2.1, -2.4, -3.0,
                                                           -3.5, -3.7, -3.8, -4.0, -4.2, -4.4,
                                                           -4.8, -5.3, -6.1, -7.0];
            try!(AmplitudeDecayExpOvertones::new(sample_rate,
                                                 overtones_amplitude,
                                                 overtones_dec_rate))
        };
        let note1 = try!(Note::new(sample_rate, frequency1.clone(), Rc::new(amplitude), 8));
        Ok(InstrumentBasic {
            sample_rate: sample_rate,
            note1: note1,
            frequency1: frequency1,
            frequency1_buffer: vec![frequency_start; BUFFER_SIZE],
            time: 0.0,
        })
    }

    /// Change frequency in harmony with the previous value
    #[allow(dead_code)]
    pub fn change_frequency(&mut self, numerator: u16, denominator: u16) -> SoundResult<()> {
        try!(self.frequency1.change_harmonically(numerator, denominator));
        self.time = 0.0;
        Ok(())
    }
}

impl SoundGenerator<Command> for InstrumentBasic {
    fn get_samples(&mut self, sample_count: usize, result: &mut Vec<SampleCalc>) {
        result.par_iter_mut()
            .enumerate()
            .filter(|&(index, _)| index < sample_count)
            .for_each(|(_index, value)| {
                *value = 0.0;
            });
        self.frequency1.get(sample_count, self.time, &mut self.frequency1_buffer).unwrap();
        self.note1.get(sample_count, self.time, &self.frequency1_buffer, result).unwrap();
        self.time += sample_count as SampleCalc / self.sample_rate;
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::Keypress { key } => {
                let _ = match key {
                    Key::Q => self.change_frequency(7, 6),
                    Key::W => self.change_frequency(6, 5),
                    Key::E => self.change_frequency(5, 4),
                    Key::R => self.change_frequency(4, 3),
                    Key::T => self.change_frequency(7, 5),
                    Key::Y => self.change_frequency(3, 2),
                    Key::U => self.change_frequency(5, 3),
                    Key::I => self.change_frequency(7, 4),
                    Key::O => self.change_frequency(2, 1),
                    // Key::P => self.change_frequency(1, 1),
                    Key::A => self.change_frequency(6, 7),
                    Key::S => self.change_frequency(5, 6),
                    Key::D => self.change_frequency(4, 5),
                    Key::F => self.change_frequency(3, 4),
                    Key::G => self.change_frequency(5, 7),
                    Key::H => self.change_frequency(2, 3),
                    Key::J => self.change_frequency(3, 5),
                    Key::K => self.change_frequency(4, 7),
                    Key::L => self.change_frequency(1, 2),
                    _ => self.change_frequency(1, 1),
                };
            }
            Command::Mute => {
                let _ = self.change_frequency(1, 1);
            }
            Command::FrequencyMultiple { numerator, denominator } => {
                let _ = self.change_frequency(numerator, denominator);
            }
        }
    }
}

fn main() {
    use music::sound::backend_portaudio::*;
    let sound_generator = Box::new(InstrumentBasic::new(48000.0, 440.0).unwrap());
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
                sound.send_command(Command::Keypress { key: key });
            } else {
                println!("Pressed {:?}", button);
            }
        }
        window.draw_2d(&event, |_c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
        });
    }
}
