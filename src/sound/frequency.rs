use sound::*;

pub trait FrequencyFunction {
    fn get(&self, time: SampleCalc) -> SampleCalc;
}

/// Frequency is not changing by time
#[allow(dead_code)]
pub struct FrequencyConst {
    frequency: SampleCalc,
}

#[allow(dead_code)]
impl FrequencyConst {
    pub fn new(frequency: SampleCalc) -> FrequencyConst {
        FrequencyConst { frequency: frequency }
    }
}

impl FrequencyFunction for FrequencyConst {
    fn get(&self, _time: SampleCalc) -> SampleCalc {
        self.frequency
    }
}
