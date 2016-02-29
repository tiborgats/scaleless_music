use sound::*;
use rayon::prelude::*;
use std::cell::Cell;

pub trait FrequencyFunction {
    fn get(&self,
           sample_count: usize,
           time_begin: SampleCalc,
           frequency_buffer: &mut Vec<SampleCalc>)
           -> SoundResult<()>;

    fn set_time(&self, time: SampleCalc) -> SoundResult<()>;
}

/// Frequency is not changing by time
#[allow(dead_code)]
pub struct FrequencyConst {
    frequency: Cell<SampleCalc>,
}

#[allow(dead_code)]
impl FrequencyConst {
    pub fn new(frequency: SampleCalc) -> SoundResult<FrequencyConst> {
        Ok(FrequencyConst { frequency: Cell::new(frequency) })
    }

    /// Change frequency in harmony with it's previous value
    #[allow(dead_code)]
    pub fn change_harmonically(&self,
                               numerator: u16,
                               denominator: u16)
                               -> SoundResult<(&FrequencyConst)> {
        if denominator <= 0 {
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
    fn get(&self,
           sample_count: usize,
           _time_begin: SampleCalc,
           frequency_buffer: &mut Vec<SampleCalc>)
           -> SoundResult<()> {
        if frequency_buffer.len() < sample_count {
            return Err(Error::BufferSize);
        }
        let frequency = self.frequency.get();
        frequency_buffer.par_iter_mut()
                        .enumerate()
                        .filter(|&(index, _)| index < sample_count)
                        .for_each(|(_index, f)| {
                            *f = frequency;
                        });
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
