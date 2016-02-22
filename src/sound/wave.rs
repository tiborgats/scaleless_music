use sound::*;
use sound::frequency::*;
use sound::amplitude::*;
use std::rc::Rc;
use std::cell::RefCell;

pub const PI2: SampleCalc = ::std::f64::consts::PI * 2.0;
// pub const PI2: SampleCalc = ::std::f32::consts::PI * 2.0;

// #![allow(dead_code)]

/// A tone with optional overtones and amplitude modulation.
#[allow(dead_code)]
pub struct Note {
    sample_rate: SampleCalc,
    // frequency_function: Rc<FrequencyFunction>,
    frequency_buffer: RefCell<Vec<SampleCalc>>,
    amplitude_function: Rc<AmplitudeFunction>,
    amplitude_buffer: RefCell<Vec<SampleCalc>>,
    // amplitude_buffer: Vec<SampleCalc>,
    overtone_max: usize,
}

impl Note {
    /// Custom constructor
    #[allow(dead_code)]
    pub fn new(sample_rate: SampleCalc,
               amplitude_function: Rc<AmplitudeFunction>,
               overtone_max: usize)
               -> SoundResult<Note> {
        Ok(Note {
            sample_rate: sample_rate,
            frequency_buffer: RefCell::new(vec![0.0; BUFFER_SIZE]),
            amplitude_function: amplitude_function,
            amplitude_buffer: RefCell::new(vec![0.0; BUFFER_SIZE]),
            // amplitude_buffer: vec![0.0; BUFFER_SIZE],
            overtone_max: overtone_max,
        })
    }

    #[allow(dead_code)]
    pub fn get(&self,
               sample_count: usize,
               time_start: SampleCalc,
               base_frequency: &Vec<SampleCalc>,
               result: &mut Vec<SampleCalc>)
               -> SoundResult<()> {
        if sample_count > BUFFER_SIZE {
            return Err(Error::BufferSize);
        }
        if base_frequency.len() < sample_count {
            return Err(Error::BufferSize);
        }
        if result.len() < sample_count {
            return Err(Error::BufferSize);
        }
        for sample_idx in 0..sample_count {
            *result.get_mut(sample_idx).unwrap() = 0.0;
        }
        let time_sample: SampleCalc = 1.0 / self.sample_rate;
        let mut sample: SampleCalc;
        // let frequency_b = self.frequency_buffer.borrow_mut();
        for overtone in 0..self.overtone_max {
            let freq_multiplier = (overtone as SampleCalc + 1.0) * PI2;
            for sample_idx in 0..sample_count {
                *self.frequency_buffer.borrow_mut().get_mut(sample_idx).unwrap() =
                    *base_frequency.get(sample_idx).unwrap();
            }
            try!(self.amplitude_function.get(sample_count,
                                             time_start,
                                             base_frequency,
                                             overtone,
                                             &mut self.amplitude_buffer.borrow_mut()));
            for sample_idx in 0..sample_count {
                let time: SampleCalc = (sample_idx as SampleCalc * time_sample) + time_start;
                let frequency: SampleCalc = *base_frequency.get(sample_idx).unwrap();
                sample = (time * frequency * freq_multiplier).sin() *
                         self.amplitude_buffer.borrow().get(sample_idx).unwrap();
                *result.get_mut(sample_idx).unwrap() += sample;  // vectors must match sample_count in size
            }
        }
        // sample
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_amplitude(&mut self, amplitude_function: Rc<AmplitudeFunction>) -> &mut Note {
        self.amplitude_function = amplitude_function;
        self
    }
}
