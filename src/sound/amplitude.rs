use sound::*;
use std::rc::Rc;
use std::cell::RefCell;

pub trait AmplitudeFunction {
    fn get(&self, time: SampleCalc, frequency: SampleCalc, overtone: usize) -> SampleCalc;
}


/// Amplitude is not changing by time
#[allow(dead_code)]
pub struct AmplitudeConst {
    amplitude: SampleCalc,
}

#[allow(dead_code)]
impl AmplitudeConst {
    pub fn new(amplitude: SampleCalc) -> AmplitudeConst {
        AmplitudeConst { amplitude: amplitude }
    }
}

impl AmplitudeFunction for AmplitudeConst {
    fn get(&self, _time: SampleCalc, _frequency: SampleCalc, _overtone: usize) -> SampleCalc {
        self.amplitude
    }
}

/// Amplitude is decaying exponentially
/// See also: [Exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
#[allow(dead_code)]
pub struct AmplitudeDecayExp {
    amplitude_function: Rc<AmplitudeFunction>,
    rate: SampleCalc,
}

#[allow(dead_code)]
impl AmplitudeDecayExp {
    /// rate must be negative!
    pub fn new(amplitude_function: Rc<AmplitudeFunction>, rate: SampleCalc) -> AmplitudeDecayExp {
        let mut rate_neg = rate;
        if rate_neg > 0.0 {
            rate_neg = -rate;
        }
        AmplitudeDecayExp {
            amplitude_function: amplitude_function,
            rate: rate_neg,
        }
    }
}

impl AmplitudeFunction for AmplitudeDecayExp {
    fn get(&self, time: SampleCalc, frequency: SampleCalc, overtone: usize) -> SampleCalc {
        self.amplitude_function.get(time, frequency, overtone) * (time * self.rate).exp()
    }
}

/// Amplitude is decaying exponentially, also for overtones
/// https://en.wikipedia.org/wiki/Exponential_decay
#[allow(dead_code)]
pub struct AmplitudeDecayExpOvertones {
    amplitude: RefCell<Vec<SampleCalc>>, // starting amplitudes
    rate: RefCell<Vec<SampleCalc>>, // rate must be negative!
}

#[allow(dead_code)]
impl AmplitudeDecayExpOvertones {
    /// rate must be negative!
    pub fn new(amplitude: Vec<SampleCalc>,
               rate: Vec<SampleCalc>)
               -> SoundResult<AmplitudeDecayExpOvertones> {
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in &amplitude {
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum > 1.0 {
            return Err(Error::AmplitudeInvalid);
        }
        for rate_check in &rate {
            if *rate_check > 0.0 {
                return Err(Error::AmplitudeRateInvalid);
            }
        }
        Ok(AmplitudeDecayExpOvertones {
            amplitude: RefCell::new(amplitude),
            rate: RefCell::new(rate),
        })
    }
}

impl AmplitudeFunction for AmplitudeDecayExpOvertones {
    fn get(&self, time: SampleCalc, _frequency: SampleCalc, overtone: usize) -> SampleCalc {
        let amplitude_b = self.amplitude.borrow();
        let rate_b = self.rate.borrow();
        if overtone >= amplitude_b.len() {return 0.0;};
        if overtone >= rate_b.len() {return 0.0;};
        amplitude_b.get(overtone).unwrap() * (time * rate_b.get(overtone).unwrap()).exp()
    }
}
