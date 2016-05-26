use sound::*;
use sound::wave::*;
use sound::frequency::*;
use sound::amplitude::*;
use std::rc::Rc;
use rayon::prelude::*;
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
    note2: Note,
    frequency1: Rc<FrequencyConst>,
    frequency1_buffer: Vec<SampleCalc>,
    frequency2: Rc<FrequencyConst>,
    frequency2_buffer: Vec<SampleCalc>,
    time: SampleCalc,
}

#[allow(dead_code)]
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
        let frequency2 = Rc::new(try!(FrequencyConst::new(440.0)));
        let amplitude = {
            let overtones_amplitude: Vec<SampleCalc> = vec![0.0, 3.0, 5.0, 2.0, 1.0, 0.5, 0.1,
                                                            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
                                                            0.1, 0.1, 0.1, 0.1, 0.1, 0.1];
            try!(AmplitudeConstOvertones::new(overtones_amplitude))
        };
        let note2 = try!(Note::new(sample_rate, frequency2.clone(), Rc::new(amplitude), 6));
        Ok(InstrumentBasic {
            sample_rate: sample_rate,
            note1: note1,
            note2: note2,
            frequency1: frequency1,
            frequency1_buffer: vec![frequency_start; BUFFER_SIZE],
            frequency2: frequency2,
            frequency2_buffer: vec![frequency_start; BUFFER_SIZE],
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

impl SoundGenerator for InstrumentBasic {
    fn get_samples(&mut self, sample_count: usize, result: &mut Vec<SampleCalc>) {
        result.par_iter_mut()
              .enumerate()
              .filter(|&(index, _)| index < sample_count)
              .for_each(|(_index, value)| {
                  *value = 0.0;
              });
        self.frequency1.get(sample_count, self.time, &mut self.frequency1_buffer).unwrap();
        // self.frequency2.get(sample_count, self.time, &mut self.frequency2_buffer).unwrap();
        self.note1.get(sample_count, self.time, &self.frequency1_buffer, result).unwrap();
        // self.note2.get(sample_count, self.time, &self.frequency2_buffer, result).unwrap();
        self.time += sample_count as SampleCalc / self.sample_rate;
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


#[allow(dead_code)]
pub struct InstrumentBasic2 {
    sample_rate: SampleCalc,
    note1: Note,
    frequency1: Rc<FrequencyConst>,
    frequency1_buffer: Vec<SampleCalc>,
    time: SampleCalc,
}

#[allow(dead_code)]
impl InstrumentBasic2 {
    /// Custom constructor
    pub fn new(sample_rate: SampleCalc,
               frequency_start: SampleCalc)
               -> SoundResult<InstrumentBasic2> {
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
        Ok(InstrumentBasic2 {
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

impl SoundGenerator for InstrumentBasic2 {
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
            Command::Keypress {key} => {
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
                    Key::P => self.change_frequency(1, 1),
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
            Command::FrequencyMultiple {numerator, denominator} => {
                let _ = self.change_frequency(numerator, denominator);
            }
        }
    }
}
