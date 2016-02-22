use sound::*;
use sound::wave::*;
// use sound::frequency::*;
use sound::amplitude::*;
use std::rc::Rc;
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
    sample_rate: SampleCalc,
    note1: Note,
    frequency: SampleCalc,
    frequency_buffer: Vec<SampleCalc>,
    time: SampleCalc,
}

#[allow(dead_code)]
impl InstrumentBasic {
    /// Custom constructor
    pub fn new(sample_rate: SampleCalc,
               frequency_start: SampleCalc)
               -> SoundResult<InstrumentBasic> {
        let overtones_amplitude: Vec<SampleCalc> = vec![0.5, 0.2, 0.1, 0.05, 0.03];
        let overtones_dec_rate: Vec<SampleCalc> = vec![-0.5, -2.0, -4.0, -8.0, -16.0];
        let amplitude1 = try!(AmplitudeDecayExpOvertones::new(sample_rate,
                                                              overtones_amplitude,
                                                              overtones_dec_rate));
        let note1 = try!(Note::new(sample_rate, Rc::new(amplitude1), 4));
        Ok(InstrumentBasic {
            sample_rate: sample_rate,
            note1: note1,
            frequency: frequency_start,
            frequency_buffer: vec![frequency_start; BUFFER_SIZE],
            time: 0.0,
        })
    }

    /// Change frequency in harmony with the previous value
    #[allow(dead_code)]
    pub fn change_frequency(&mut self, numerator: u16, denominator: u16) -> SoundResult<()> {
        if denominator <= 0 {
            return Err(Error::DenominatorInvalid);
        };
        let new_frequency = (self.frequency * numerator as SampleCalc) / denominator as SampleCalc;
        if new_frequency < TONE_FREQUENCY_MIN {
            return Err(Error::DenominatorInvalid);
        };
        if new_frequency > TONE_FREQUENCY_MAX {
            return Err(Error::DenominatorInvalid);
        };
        // self.frequency = new_frequency;
        for i in self.frequency_buffer.iter_mut() {
            *i = new_frequency;
        }
        self.time = 0.0;
        Ok(())
    }
}

impl SoundGenerator for InstrumentBasic {
    fn get_samples(&mut self, count: usize, result: &mut Vec<SampleCalc>) {
        self.note1.get(count, self.time, &self.frequency_buffer, result).unwrap();
        self.time += count as SampleCalc / self.sample_rate;
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::Keypress {key} => {
                let _ = match key {
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
                    _ => self.change_frequency(1, 1),
                };
            }
            Command::Mute => {
                let _ = self.change_frequency(1, 1);
            }
            Command::FrequencyMultiple {numerator, denominator} => {
                let _ = self.change_frequency(numerator, denominator);
            }
        }
    }
}
