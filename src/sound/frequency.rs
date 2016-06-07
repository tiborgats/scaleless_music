use sound::*;
use std::cell::Cell;
// use std::fmt;
// use rayon::prelude::*;

/// Input and output definition for the frequency functions.
pub trait FrequencyFunction {
    /// Provides the results of the frequency calculations.
    fn get(&self,
           time_begin: SampleCalc,
           base_frequency: Option<&[SampleCalc]>,
           result: &mut [SampleCalc])
           -> SoundResult<()>;
}

/// Frequency is not changing by time.
#[derive(Debug, Clone)]
pub struct FrequencyConst {
    frequency: Cell<SampleCalc>,
}

impl FrequencyConst {
    /// custom constructor
    pub fn new(frequency: SampleCalc) -> SoundResult<FrequencyConst> {
        Ok(FrequencyConst { frequency: Cell::new(frequency) })
    }

    /// Change frequency in harmony with it's previous value.
    pub fn change(&self, interval: Interval) -> SoundResult<&FrequencyConst> {
        self.frequency.set(try!(interval.change_frequency(self.frequency.get())));
        Ok(self)
    }
}

impl FrequencyFunction for FrequencyConst {
    fn get(&self,
           _time_begin: SampleCalc,
           base_frequency: Option<&[SampleCalc]>,
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if base_frequency.is_some() {
            return Err(Error::FrequencySource);
        }
        for item in result.iter_mut() {
            *item = self.frequency.get();
        }
        Ok(())
    }
}

/// Vibrato around the base frequency (= frequency modulation).
#[allow(dead_code)]
pub struct FrequencyVibrato {
    sample_rate: SampleCalc,
    frequency_base: SampleCalc,
    /// The ratio of maximum shift away from the base frequency (0.0 - 1.0).
    frequency_deviation: SampleCalc,
    rhythm: SampleCalc,
    time: SampleCalc,
}

/// Changing frequency linearly. Linearity means constant multiplication over time slices.
#[allow(dead_code)]
pub struct FrequencyChangeLinear {
    sample_rate: SampleCalc,
    frequency_begin: SampleCalc,
    frequency_end: SampleCalc,
    timeframe: SampleCalc,
    time: SampleCalc,
}
