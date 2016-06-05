use sound::*;
use sound::amplitude::*;
use std::rc::Rc;
use std::cell::RefCell;
// use rayon::prelude::*;
// use std::fmt::Debug;

/// A tone with optional overtones and amplitude modulation.
#[derive(Clone)]
pub struct Note {
    sample_rate: SampleCalc,
    //    frequency_function: Rc<FrequencyFunction>,
    //    frequency_buffer: RefCell<Vec<SampleCalc>>,
    amplitude_function: Rc<AmplitudeFunction>,
    amplitude_buffer: RefCell<Vec<SampleCalc>>,
    wave_buffer: RefCell<Vec<SampleCalc>>,
    overtone_max: usize,
}

impl Note {
    /// Custom constructor
    pub fn new(sample_rate: SampleCalc,
               buffer_size: usize,
               amplitude_function: Rc<AmplitudeFunction>,
               overtone_max: usize)
               -> SoundResult<Note> {
        Ok(Note {
            sample_rate: sample_rate,
            //            frequency_function: frequency_function,
            //            frequency_buffer: RefCell::new(vec![0.0; buffer_size]),
            amplitude_function: amplitude_function,
            amplitude_buffer: RefCell::new(vec![0.0; buffer_size]),
            wave_buffer: RefCell::new(vec![0.0; buffer_size]),
            overtone_max: overtone_max,
        })
    }
    /// Set a new amplitude function
    pub fn set_amplitude(&mut self, amplitude_function: Rc<AmplitudeFunction>) -> &mut Note {
        self.amplitude_function = amplitude_function;
        self
    }
}

impl SoundStructure for Note {
    fn get(&self,
           //           sample_count: usize,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc], // &Vec<SampleCalc>,
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        let buffer_size = self.wave_buffer.borrow().len();
        // if self.amplitude_buffer.borrow().len() != buffer_size {
        // return Err(Error::BufferSize);
        // }
        if base_frequency.len() != buffer_size {
            return Err(Error::BufferSize);
        }
        if result.len() != buffer_size {
            return Err(Error::BufferSize);
        }
        let time_sample: SampleCalc = 1.0 / self.sample_rate;
        // let frequency_b = self.frequency_buffer.borrow_mut();
        for item in result.iter_mut() {
            *item = 0.0;
        }
        for overtone in 0..self.overtone_max {
            let freq_multiplier = (overtone as SampleCalc + 1.0) * PI2;
            //            for sample_idx in 0..buffer_size {
            // self.frequency_buffer.borrow_mut().get_mut(sample_idx).unwrap() =
            // base_frequency.get(sample_idx).unwrap();
            // }
            try!(self.amplitude_function.get(time_start,
                                             overtone,
                                             &mut self.amplitude_buffer.borrow_mut()));

            for (((index, item), frequency), amplitude) in result.iter_mut()
                .enumerate()
                .zip(base_frequency)
                .zip(self.amplitude_buffer.borrow().iter()) {
                    let time = (index as SampleCalc * time_sample) + time_start;
                    *item += (time * frequency * freq_multiplier).sin() * *amplitude;
                }
/*            self.wave_buffer
                .borrow_mut()
                .par_iter_mut()
                .zip(base_frequency.par_iter())
                .enumerate()
                .for_each(|(i, (w, f))| {
                    let time = (i as SampleCalc * time_sample) + time_start;
                    *w = (time * f * freq_multiplier).sin();
                });*/
/*            for ((item, wave), amplitude) in result.iter_mut()
                .zip(self.wave_buffer.borrow().iter())
                .zip(self.amplitude_buffer.borrow().iter()) {
                *item += *wave * *amplitude;
            }*/
        }
        Ok(())
    }
}

#[allow(dead_code)]
struct MixerChannel {
    sound: Rc<SoundStructure>,
    amplitude: Rc<AmplitudeFunction>,
}

/// Mixing sound channels (structures).
#[allow(dead_code)]
pub struct Mixer {
    sample_rate: SampleCalc,
    channels: Vec<MixerChannel>,
}

impl Mixer {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc) -> SoundResult<Mixer> {
        Ok(Mixer {
            sample_rate: sample_rate,
            channels: Vec::new(),
        })
    }
    /// Add a new channel to the mixer.
    pub fn add(&mut self,
               sound: Rc<SoundStructure>,
               amplitude: Rc<AmplitudeFunction>)
               -> SoundResult<&mut Mixer> {
        let channel = MixerChannel {
            sound: sound.clone(),
            amplitude: amplitude.clone(),
        };
        self.channels.push(channel);
        Ok(self)
    }
}

impl SoundStructure for Mixer {
    fn get(&self,
           _time_start: SampleCalc,
           base_frequency: &[SampleCalc], // &Vec<SampleCalc>,
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if base_frequency.len() != result.len() {
            return Err(Error::BufferSize);
        }




        Ok(())
    }
}
