use sound::*;
use sound::frequency::*;
use sound::amplitude::*;
use std::rc::Rc;

pub const PI2: SampleCalc = ::std::f64::consts::PI * 2.0;

// #![allow(dead_code)]

/// A tone with optional overtones and amplitude modulation.
#[allow(dead_code)]
pub struct Note {
    sample_rate: SampleCalc,
    // frequency_function: Rc<FrequencyFunction>,
    amplitude_function: Rc<AmplitudeFunction>,
    amplitude_buffer: Vec<SampleCalc>,
    overtone_max: usize,
}

impl Note {
    /// Custom constructor
    #[allow(dead_code)]
    pub fn new(sample_rate: SampleCalc,
               amplitude_function: Rc<AmplitudeFunction>,
               overtone_max: usize)
               -> Note {
        Note {
            sample_rate: sample_rate,
            amplitude_function: amplitude_function,
            amplitude_buffer: vec![0.0; BUFFER_SIZE], // Vec::with_capacity(BUFFER_SIZE),
            overtone_max: overtone_max,
        }
    }

    #[allow(dead_code)]
    pub fn get(&self,
               sample_count: usize,
               time_start: SampleCalc,
               base_frequency: &Vec<SampleCalc>,
               result: &mut Vec<SampleCalc>)
               -> SoundResult<()> {
        if base_frequency.len() < sample_count {
            return Err(Error::BufferSize);
        }
        if result.len() < sample_count {
            return Err(Error::BufferSize);
        }
        for sample_idx in 0..sample_count {
            *result.get_mut(sample_idx).unwrap() = 0.0;
        }
        let mut sample: SampleCalc;
        for overtone in 0..self.overtone_max {
            let freq_multiplier = (overtone as SampleCalc + 1.0) * PI2;
            for sample_idx in 0..sample_count {
                let time: SampleCalc = (sample_idx as SampleCalc / self.sample_rate) + time_start;
                let frequency: SampleCalc = *base_frequency.get(sample_idx).unwrap();
                sample = (time * frequency * freq_multiplier).sin() *
                         self.amplitude_function.get(time, frequency, overtone);
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
