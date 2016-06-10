//! This example is an overtone instrument. Keyboard is the user input.
//! The tone is a very simple function (nothing like a real instrument). It's purpose is
//! only testing.
//! See also: [Overtone flute](https://en.wikipedia.org/wiki/Overtone_flute)
extern crate music;
extern crate rayon;

#[macro_use]
extern crate conrod;
extern crate piston_window;
extern crate piston;

use music::sound::*;
use std::rc::Rc;
use piston_window::*;

/// Commands of the messages from the UI thread to the playback thread.
pub enum Command {
    /// Mute
    Mute,
    /// Keyboard event
    Keypress {
        key: keyboard::Key,
    },
    /// Multiply frequency by a rational number
    FrequencyMultiple {
        numerator: u16,
        denominator: u16,
    },
}

pub struct InstrumentBasic {
    sample_rate: SampleCalc,
    frequency1: Rc<FrequencyConst>,
    frequency1_buffer: Vec<SampleCalc>,
    mixer: Rc<Mixer>,
    time: SampleCalc,
}

impl InstrumentBasic {
    /// Custom constructor
    pub fn new(sample_rate: SampleCalc) -> SoundResult<InstrumentBasic> {
        let frequency1 = Rc::new(try!(FrequencyConst::new(110.0)));
        let amplitude = {
            let overtones_amplitude: Vec<SampleCalc> = vec![10.0, 1.0, 1.0, 0.95, 0.9, 0.9, 0.86,
                                                            0.83, 0.80, 0.78, 0.76, 0.74, 0.73,
                                                            0.72, 0.71, 0.70];
            let overtones_dec_rate: Vec<SampleCalc> = vec![-0.5, -1.4, -1.9, -2.1, -2.4, -3.0,
                                                           -3.5, -3.7, -3.8, -4.0, -4.2, -4.4,
                                                           -4.8, -5.3, -6.1, -7.0];
            try!(AmplitudeDecayExpOvertones::new(sample_rate,
                                                 overtones_amplitude,
                                                 overtones_dec_rate))
        };
        let note1 =
            Rc::new(try!(Note::new(sample_rate, BUFFER_SIZE_DEFAULT, Rc::new(amplitude), 4)));
        let amplitude = {
            let overtones_amplitude: Vec<SampleCalc> = vec![1.0, 0.1, 0.1, 0.1, 0.2, 0.5, 0.1,
                                                            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
                                                            0.1, 0.1, 0.1, 0.1, 0.1, 0.1];
            try!(AmplitudeConstOvertones::new(overtones_amplitude))
        };
        let note2 =
            Rc::new(try!(Note::new(sample_rate, BUFFER_SIZE_DEFAULT, Rc::new(amplitude), 4)));
        let mixer = Rc::new(try!(Mixer::new(BUFFER_SIZE_DEFAULT)));
        try!(mixer.add(try!(Interval::new(1, 1)), note1, 4.0));
        try!(mixer.add(try!(Interval::new(1, 1)), note2, 1.0));

        Ok(InstrumentBasic {
            sample_rate: sample_rate,
            frequency1: frequency1,
            frequency1_buffer: vec![1.0; BUFFER_SIZE_DEFAULT],
            mixer: mixer,
            time: 0.0,
        })
    }

    /// Change frequency in harmony with the previous value
    #[allow(dead_code)]
    pub fn change_frequency(&mut self, numerator: u16, denominator: u16) -> SoundResult<()> {
        let interval = try!(Interval::new(numerator, denominator));
        try!(self.mixer.set_interval(0, interval));
        self.time = 0.0;
        println!("{}", interval);
        Ok(())
    }
}
// TODO: -unwrap()
impl SoundGenerator<Command> for InstrumentBasic {
    fn get_samples(&mut self, sample_count: usize, result: &mut Vec<SampleCalc>) {
        self.frequency1.get(self.time, None, &mut self.frequency1_buffer).unwrap();
        self.mixer.get(self.time, &self.frequency1_buffer, result).unwrap();
        self.time += sample_count as SampleCalc / self.sample_rate;
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::Keypress { key } => {
                let _ = match key {
                    Key::Q => self.change_frequency(1, 2),
                    Key::W => self.change_frequency(3, 2),
                    Key::E => self.change_frequency(5, 2),
                    Key::R => self.change_frequency(7, 2),
                    Key::T => self.change_frequency(9, 2),
                    Key::Y => self.change_frequency(11, 2),
                    Key::U => self.change_frequency(13, 2),
                    Key::I => self.change_frequency(15, 2),
                    Key::O => self.change_frequency(17, 2),
                    Key::P => self.change_frequency(19, 2),
                    Key::A => self.change_frequency(1, 1),
                    Key::S => self.change_frequency(2, 1),
                    Key::D => self.change_frequency(3, 1),
                    Key::F => self.change_frequency(4, 1),
                    Key::G => self.change_frequency(5, 1),
                    Key::H => self.change_frequency(6, 1),
                    Key::J => self.change_frequency(7, 1),
                    Key::K => self.change_frequency(8, 1),
                    Key::L => self.change_frequency(9, 1),
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
    println!("scaleless_music v{} example: overtone instrument\n",
             env!("CARGO_PKG_VERSION"));
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
    println!("\n\nThe keys from [Q] to [P] produces half wave resonances,");
    println!("the keys from [A] to [L] makes full wave resonances.");
    println!("To quit press [Esc].");
    while let Some(event) = window.next() {
        if let Some(button) = event.press_args() {
            if let Button::Keyboard(key) = button {
                sound.send_command(Command::Keypress { key: key })
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
