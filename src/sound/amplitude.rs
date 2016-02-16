use sound::*;
use std::rc::Rc;

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
/// https://en.wikipedia.org/wiki/Exponential_decay
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
