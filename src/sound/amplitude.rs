use sound::*;
// use std::rc::Rc;
use std::cell::RefCell;
// use std::cell::{RefCell, RefMut};

pub trait AmplitudeFunction {
    fn get(&self,
           sample_count: usize,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc], // &Vec<SampleCalc>,
           overtone: usize,
           result: &mut Vec<SampleCalc>)
           -> SoundResult<()>;
}


/// Amplitude is not changing by time
#[allow(dead_code)]
pub struct AmplitudeConstOvertones {
    amplitude: RefCell<Vec<SampleCalc>>,
}

#[allow(dead_code)]
impl AmplitudeConstOvertones {
    pub fn new(mut amplitude: Vec<SampleCalc>) -> SoundResult<AmplitudeConstOvertones> {
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in &amplitude {
            if *amplitude_check < 0.0 {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        // normalization
        for amp in &mut amplitude {
            *amp /= amplitude_sum;
        }
        Ok(AmplitudeConstOvertones { amplitude: RefCell::new(amplitude) })
    }
}

impl AmplitudeFunction for AmplitudeConstOvertones {
    fn get(&self,
           sample_count: usize,
           _time_start: SampleCalc,
           base_frequency: &[SampleCalc], // &Vec<SampleCalc>,
           overtone: usize,
           result: &mut Vec<SampleCalc>)
           -> SoundResult<()> {
        // self.amplitude
        if base_frequency.len() < sample_count {
            return Err(Error::BufferSize);
        }
        if result.len() < sample_count {
            return Err(Error::BufferSize);
        }
        let amplitude_b = self.amplitude.borrow();
        if overtone >= amplitude_b.len() {
            for sample_idx in 0..sample_count {
                *result.get_mut(sample_idx).unwrap() = 0.0;
            }
            return Ok(());
        };
        for sample_idx in 0..sample_count {
            *result.get_mut(sample_idx).unwrap() = *amplitude_b.get(overtone).unwrap();
        }
        Ok(())
    }
}
// Amplitude is decaying exponentially
// See also: [Exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
// #[allow(dead_code)]
// pub struct AmplitudeDecayExp {
// amplitude_function: Rc<AmplitudeFunction>,
// rate: SampleCalc,
// }
//
// #[allow(dead_code)]
// impl AmplitudeDecayExp {
// rate must be negative!
// pub fn new(amplitude_function: Rc<AmplitudeFunction>, rate: SampleCalc) -> AmplitudeDecayExp {
// let mut rate_neg = rate;
// if rate_neg > 0.0 {
// rate_neg = -rate;
// }
// AmplitudeDecayExp {
// amplitude_function: amplitude_function,
// rate: rate_neg,
// }
// }
// }
//
// impl AmplitudeFunction for AmplitudeDecayExp {
// fn get(&self,
// _sample_count: usize,
// _time_start: SampleCalc,
// _base_frequency: &Vec<SampleCalc>,
// _overtone: usize,
// _result: &mut Vec<SampleCalc>)
// -> SoundResult<()> {
// self.amplitude_function.get(time, frequency, overtone) * (time * self.rate).exp()
// Ok(())
// }
// }

/// Amplitude is decaying exponentially, also for overtones
/// [Exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
/// index: 0 = fundamental tone, 1.. = overtones
#[allow(dead_code)]
pub struct AmplitudeDecayExpOvertones {
    sample_rate: SampleCalc,
    amplitude: Vec<SampleCalc>, // starting amplitudes
    rate: Vec<SampleCalc>, // rate must be negative!
}

#[allow(dead_code)]
impl AmplitudeDecayExpOvertones {
    /// rate must be negative!
    pub fn new(sample_rate: SampleCalc,
               mut amplitude: Vec<SampleCalc>,
               rate: Vec<SampleCalc>)
               -> SoundResult<AmplitudeDecayExpOvertones> {
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in &amplitude {
            if *amplitude_check < 0.0 {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        // normalization
        for amp in &mut amplitude {
            *amp /= amplitude_sum;
        }
        for rate_check in &rate {
            if *rate_check > 0.0 {
                return Err(Error::AmplitudeRateInvalid);
            }
        }
        Ok(AmplitudeDecayExpOvertones {
            sample_rate: sample_rate,
            amplitude: amplitude,
            rate: rate,
        })
    }
}

impl AmplitudeFunction for AmplitudeDecayExpOvertones {
    fn get(&self,
           sample_count: usize,
           time_start: SampleCalc,
           _base_frequency: &[SampleCalc], // &Vec<SampleCalc>,
           overtone: usize,
           // result: RefMut<Vec<SampleCalc>>)
           result: &mut Vec<SampleCalc>)
           -> SoundResult<()> {
        //        if base_frequency.len() < sample_count {
        // return Err(Error::BufferSize);
        // }
        if result.len() < sample_count {
            return Err(Error::BufferSize);
        }
        if (overtone >= self.amplitude.len()) || (overtone >= self.rate.len()) {
            for sample_idx in 0..sample_count {
                *result.get_mut(sample_idx).unwrap() = 0.0;
            }
            return Ok(());
        };
        for sample_idx in 0..sample_count {
            let time: SampleCalc = (sample_idx as SampleCalc / self.sample_rate) + time_start;
            *result.get_mut(sample_idx).unwrap() = self.amplitude.get(overtone).unwrap() *
                                                   (time * self.rate.get(overtone).unwrap()).exp();
        }
        Ok(())
    }
}

/// [Equal-loudness contour](https://en.wikipedia.org/wiki/Equal-loudness_contour)
/// data used is described by the ISO 226:2003 standard
/// see also: https://plot.ly/~mrlyule/16/equal-loudness-contours-iso-226-2003/
#[allow(dead_code)]
pub struct AmplitudeEqualLoudness;
