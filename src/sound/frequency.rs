use sound::*;
use std::cell::Cell;
// use std::fmt;
// use rayon::prelude::*;

/// Input and output definition for the frequency functions.
pub trait FrequencyFunction {
    /// Provides the results of the frequency calculations.
    fn get(&self,
           time_start: SampleCalc,
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

/// Changing frequency linearly. Linearity means constant multiplication over time slices.
#[allow(dead_code)]
pub struct FrequencyChangeLinear {
    sample_rate: SampleCalc,
    frequency_begin: SampleCalc,
    frequency_end: SampleCalc,
    timeframe: SampleCalc,
    time: SampleCalc,
}

/// Input and output definition for the frequency functions.
pub trait FrequencyModulator {
    /// Provides the results of the modulation of an array of frequencies.
    fn get(&self,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc],
           result: &mut [SampleCalc])
           -> SoundResult<()>;
    /// Applies the modulation on an already existing array of frequencies.
    fn apply(&self, time_start: SampleCalc, samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// Vibrato: sinusoidal modulation of the base frequency.
#[allow(dead_code)]
pub struct Vibrato {
    sample_time: SampleCalc,
    /// The speed with which the pitch is varied.
    rate: SampleCalc,
    /// The ratio of maximum shift away from the base frequency (must be > 0.0).
    extent_ratio: SampleCalc,
}
impl Vibrato {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc,
               rate: SampleCalc,
               extent_ratio: SampleCalc)
               -> SoundResult<Vibrato> {
        let sample_time = try!(get_sample_time(sample_rate));
        if rate <= 0.0 {
            return Err(Error::PeriodInvalid);
        }
        if extent_ratio <= 0.0 {
            return Err(Error::FrequencyTooLow);
        }
        Ok(Vibrato {
            sample_time: sample_time,
            rate: rate,
            extent_ratio: extent_ratio,
        })
    }
}

impl FrequencyModulator for Vibrato {
    fn get(&self,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc],
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if base_frequency.len() != result.len() {
            return Err(Error::BufferSize);
        }
        let rate_in_rad = self.rate * PI2;
        for ((index, item), frequency) in result.iter_mut().enumerate().zip(base_frequency) {
            let time = (index as SampleCalc * self.sample_time) + time_start;
            *item = *frequency * (self.extent_ratio.powf((time * rate_in_rad).sin()));
        }
        Ok(())
    }

    fn apply(&self, time_start: SampleCalc, samples: &mut [SampleCalc]) -> SoundResult<()> {
        let rate_in_rad = self.rate * PI2;
        for (index, item) in samples.iter_mut().enumerate() {
            let time = (index as SampleCalc * self.sample_time) + time_start;
            *item *= self.extent_ratio.powf((time * rate_in_rad).sin());
        }
        Ok(())
    }
}
