//! This example is a basic instrument playing only one tone. Keyboard is the user input.
//! The tone is a very simple function (nothing like a real instrument). It's purpose is
//! only the testing of some intervals.
//!
//! The keys from <kbd>Q</kbd> to <kbd>O</kbd> changes the frequency to be higher,
//! the keys from <kbd>A</kbd> to <kbd>L</kbd> changes the frequency to be lower.
/// Other keys play the previous frequency. To quit press <kbd>Esc</kbd>.

extern crate scaleless_music;

#[macro_use]
extern crate conrod;
extern crate piston_window;
extern crate piston;

use piston_window::*;
use scaleless_music::sound::*;
use std::rc::Rc;

/// Commands of the messages from the UI thread to the playback thread.
pub enum GeneratorCommand {
    /// Mute
    Mute,
    /// Keyboard event
    Keypress { key: keyboard::Key },
    /// Multiply frequency by a rational number
    FrequencyMultiple { numerator: u16, denominator: u16 },
}

pub struct InstrumentBasic {
    sample_rate: SampleCalc,
    timbre1: Timbre,
    frequency1: Rc<FrequencyConst>,
    frequency1_buffer: Vec<SampleCalc>,
    time: SampleCalc,
}

impl InstrumentBasic {
    /// Custom constructor
    pub fn new(sample_rate: SampleCalc) -> SoundResult<InstrumentBasic> {
        let frequency1 = Rc::new(FrequencyConst::new(220.0)?);
        let amplitude = {
            let overtones_amplitude: Vec<SampleCalc> = vec![10.0, 1.0, 1.0, 0.95, 0.9, 0.9, 0.86,
                                                            0.83, 0.80, 0.78, 0.76, 0.74, 0.73,
                                                            0.72, 0.71, 0.70];
            let overtones_half_life: Vec<SampleCalc> = vec![1.0, 0.2, 0.1, 0.06, 0.04, 0.03, 0.02,
                                                            0.015, 0.01, 0.008, 0.007, 0.006,
                                                            0.005, 0.004, 0.002, 0.001];
            AmplitudeDecayExpOvertones::new(sample_rate,
                                            4,
                                            &overtones_amplitude,
                                            &overtones_half_life)?
        };
        let timbre1 = Timbre::new(sample_rate, BUFFER_SIZE_DEFAULT, Rc::new(amplitude), 4)?;
        Ok(InstrumentBasic {
            sample_rate: sample_rate,
            timbre1: timbre1,
            frequency1: frequency1,
            frequency1_buffer: vec![1.0; BUFFER_SIZE_DEFAULT],
            time: 0.0,
        })
    }

    /// Change frequency in harmony with the previous value
    #[allow(dead_code)]
    pub fn change_frequency(&mut self, numerator: u16, denominator: u16) -> SoundResult<()> {
        let interval = Interval::new(numerator, denominator)?;
        self.frequency1.change(interval)?;
        self.time = 0.0;
        self.timbre1.restart();
        println!("{}  {}", interval, interval.get_name());
        Ok(())
    }
}
// TODO: -unwrap()
impl SoundGenerator for InstrumentBasic {
    type Command = GeneratorCommand;

    fn get_samples(&mut self, sample_count: usize, result: &mut Vec<SampleCalc>) {
        self.frequency1.get(self.time, None, &mut self.frequency1_buffer).unwrap();
        self.timbre1.get(&self.frequency1_buffer, result).unwrap();
        self.time += sample_count as SampleCalc / self.sample_rate;
    }

    fn process_command(&mut self, command: GeneratorCommand) {
        match command {
            GeneratorCommand::Keypress { key } => {
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
            GeneratorCommand::Mute => {
                let _ = self.change_frequency(1, 1);
            }
            GeneratorCommand::FrequencyMultiple { numerator, denominator } => {
                let _ = self.change_frequency(numerator, denominator);
            }
        }
    }
}

fn main() {
    use scaleless_music::sound::backend_portaudio::*;
    println!("scaleless_music v{} example", env!("CARGO_PKG_VERSION"));
    let sound_generator = Box::new(InstrumentBasic::new(48000.0)
        .expect("InstrumentBasic construction shouldn't fail."));
    let mut sound = SoundInterface::new(48000, BUFFER_SIZE_DEFAULT, 2, sound_generator)
        .expect("SoundInterface construction shouldn't fail.");

    let mut window: PistonWindow = WindowSettings::new("Music", [320, 200])
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .expect("PistonWindow construction shouldn't fail.");

    sound.start().expect("sound.start() shouldn't fail.");
    println!("\n\nThe keys from [Q] to [O] changes the frequency to be higher,");
    println!("the keys from [A] to [L] changes the frequency to be lower.");
    println!("Other keys play the previous frequency. To quit press [Esc].");
    while let Some(event) = window.next() {
        if let Some(button) = event.press_args() {
            if let Button::Keyboard(key) = button {
                sound.send_command(GeneratorCommand::Keypress { key: key })
                    .expect("send_command failed.");
            } else {
                println!("Pressed {:?}", button);
            }
        }
        window.draw_2d(&event, |_c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
        });
    }
}
