use sound::*;
use sound::wave::*;
use piston::input::*;

///
#[allow(dead_code)]
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

pub type GeneratorCommand = Command;

#[allow(dead_code)]
pub struct InstrumentBasic {
    sample_rate: f64,
}

#[allow(dead_code)]
impl InstrumentBasic {
    /// Custom constructor
    pub fn new(sample_rate: f64) -> InstrumentBasic {
        InstrumentBasic { sample_rate: sample_rate }
    }
}




pub struct Synthesizer {
    sample_rate: f64,
    frequency: f64,
    amplitude: SampleCalc,
    sample_index: f64,
}

impl Synthesizer {
    /// Custom constructor
    #[allow(dead_code)]
    pub fn new(sample_rate: f64, frequency: f64) -> Synthesizer {
        Synthesizer {
            sample_rate: sample_rate,
            frequency: frequency,
            amplitude: 1.0,
            sample_index: 0.0,
        }
    }

    /// Change frequency in harmony with the previous value
    /// # Todo
    /// error handling is missing
    #[allow(dead_code)]
    pub fn change_frequency(&mut self, numerator: u16, denominator: u16) {
        if denominator > 0 {
            let new_frequency = (440.0 * numerator as f64) / denominator as f64;
            // self.frequency = (self.frequency * numerator as f64) / denominator as f64;
            if (new_frequency > TONE_FREQUENCY_MIN) && (new_frequency < TONE_FREQUENCY_MAX) {
                self.frequency = new_frequency;
            }
        }
        self.amplitude = 1.0;
    }
}

impl SoundGenerator for Synthesizer {
    fn sample_next(&mut self) -> SampleOutput {
        self.sample_index += 1.0;
        //        if self.sample_index >= self.sample_rate {
        // self.sample_index -= self.sample_rate;
        // }
        let mut sample = (((self.sample_index / self.sample_rate) * self.frequency * PI2)
                              .sin() * self.amplitude) as SampleOutput;
        sample += (((self.sample_index / self.sample_rate) *
                    440.0 * PI2)
                       .sin() * 1.0) as SampleOutput;
        sample *= 0.5;
        return sample;
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::Keypress {key} => {
                match key {
                    Key::Q => self.change_frequency(1, 3),
                    Key::W => self.change_frequency(2, 3),
                    Key::E => self.change_frequency(4, 3),
                    Key::R => self.change_frequency(5, 3),
                    Key::T => self.change_frequency(1, 4),
                    Key::Y => self.change_frequency(2, 4),
                    Key::U => self.change_frequency(3, 4),
                    Key::I => self.change_frequency(5, 4),
                    Key::O => self.change_frequency(6, 4),
                    Key::P => self.change_frequency(7, 4),
                    Key::A => self.change_frequency(1, 5),
                    Key::S => self.change_frequency(2, 5),
                    Key::D => self.change_frequency(3, 5),
                    Key::F => self.change_frequency(4, 5),
                    Key::G => self.change_frequency(6, 5),
                    Key::H => self.change_frequency(7, 5),
                    Key::J => self.change_frequency(8, 5),
                    Key::K => self.change_frequency(9, 5),
                    Key::L => self.change_frequency(1, 6),
                    _ => {
                        self.amplitude = 0.0;
                    }
                };
            }
            Command::Mute => {
                self.amplitude = 0.0;
            }
            Command::FrequencyMultiple {numerator, denominator} => {
                self.change_frequency(numerator, denominator);
                self.amplitude = 1.0;
            }
        }
    }
}
