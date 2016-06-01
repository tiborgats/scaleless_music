use sound::*;
// use rayon::prelude::*;
use std::cell::Cell;

/// Input and output definition for the frequency functions.
pub trait FrequencyFunction {
    /// Provides the results of the frequency calculations.
    fn get(&self, time_begin: SampleCalc, result: &mut [SampleCalc]) -> SoundResult<()>;
    ///
    fn set_time(&self, time: SampleCalc) -> SoundResult<()>;
}

/// Frequency is not changing by time.
pub struct FrequencyConst {
    frequency: Cell<SampleCalc>,
}

impl FrequencyConst {
    /// custom constructor
    pub fn new(frequency: SampleCalc) -> SoundResult<FrequencyConst> {
        Ok(FrequencyConst { frequency: Cell::new(frequency) })
    }

    /// Change frequency in harmony with it's previous value
    pub fn change_harmonically(&self,
                               numerator: u16,
                               denominator: u16)
                               -> SoundResult<(&FrequencyConst)> {
        if denominator == 0 {
            return Err(Error::DenominatorInvalid);
        };
        let new_frequency = (self.frequency.get() * numerator as SampleCalc) /
                            denominator as SampleCalc;
        if new_frequency < TONE_FREQUENCY_MIN {
            return Err(Error::DenominatorInvalid);
        };
        if new_frequency > TONE_FREQUENCY_MAX {
            return Err(Error::DenominatorInvalid);
        };
        self.frequency.set(new_frequency);
        Ok(self)
    }
}

impl FrequencyFunction for FrequencyConst {
    fn get(&self, _time_begin: SampleCalc, result: &mut [SampleCalc]) -> SoundResult<()> {
        let frequency = self.frequency.get();
        for item in result.iter_mut() {
            *item = frequency;
        }
        Ok(())
    }

    fn set_time(&self, _time: SampleCalc) -> SoundResult<()> {
        Ok(())
    }
}

/// Vibrato around the base frequency (= frequency modulation)
#[allow(dead_code)]
pub struct FrequencyVibrato {
    sample_rate: SampleCalc,
    frequency_base: SampleCalc,
    frequency_deviation: SampleCalc, // = maximum shift away from frequency_base
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
