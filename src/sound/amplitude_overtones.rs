use sound::*;

/// Provides time dependent amlitude changes both for the fundamental tone and for overtones.
pub trait AmplitudeOvertonesProvider {
    /// Provides the results of the amplitude calculations for a given overtone.
    /// For the fundamental tone `overtone = 0`.
    fn get(&self,
           time_start: SampleCalc,
           overtone: usize,
           result: &mut [SampleCalc])
           -> SoundResult<()>;
    /// Applies the amplitude function over existing samples for a given overtone.
    /// For the fundamental tone `overtone = 0`. It multiplies each sample with it's new amplitude.
    fn apply(&self,
             time_start: SampleCalc,
             overtone: usize,
             samples: &mut [SampleCalc])
             -> SoundResult<()>;
}

/// The `AmplitudeOvertonesJoinable` trait is used to specify the ability of joining
/// amplitude structures (with overtones) together, forming a sequence of them.
pub trait AmplitudeOvertonesJoinable {
    /// Sets the initial amplitude, and resets time.
    fn set_amplitude_start(&mut self, amplitude: Vec<SampleCalc>) -> SoundResult<()>;
    // Provides the last amplitude.
    // fn get_last_amplitude() -> SampleCalc;
}

/// Amplitude is not changing by time, this function gives the overtone amplitudes too.
#[derive(Debug, Clone)]
pub struct AmplitudeConstOvertones {
    amplitude: Vec<SampleCalc>,
}

impl AmplitudeConstOvertones {
    /// custom constructor
    /// It normalizes the amplitudes, so the sum of them will be 1.0.
    pub fn new(mut amplitude: Vec<SampleCalc>) -> SoundResult<AmplitudeConstOvertones> {
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in &amplitude {
            if *amplitude_check < 0.0 {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum == 0.0 {
            return Err(Error::AmplitudeInvalid);
        };
        // normalization
        for item in &mut amplitude {
            *item /= amplitude_sum;
        }

        Ok(AmplitudeConstOvertones { amplitude: amplitude })
    }
}

impl AmplitudeOvertonesProvider for AmplitudeConstOvertones {
    fn get(&self,
           _time_start: SampleCalc,
           overtone: usize,
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if overtone >= self.amplitude.len() {
            for item in result.iter_mut() {
                *item = 0.0;
            }
            return Ok(());
        };
        for item in result.iter_mut() {
            *item = self.amplitude[overtone];
        }
        Ok(())
    }

    fn apply(&self,
             _time_start: SampleCalc,
             overtone: usize,
             samples: &mut [SampleCalc])
             -> SoundResult<()> {
        if overtone >= self.amplitude.len() {
            for item in samples.iter_mut() {
                *item = 0.0;
            }
            return Ok(());
        };
        for item in samples.iter_mut() {
            *item *= self.amplitude[overtone];
        }
        Ok(())
    }
}

impl AmplitudeOvertonesJoinable for AmplitudeConstOvertones {
    fn set_amplitude_start(&mut self, amplitude: Vec<SampleCalc>) -> SoundResult<()> {
        if amplitude.len() != self.amplitude.len() {
            return Err(Error::OvertoneCountInvalid);
        }
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in &amplitude {
            if (*amplitude_check < 0.0) || (*amplitude_check > 1.0) {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum == 0.0 {
            return Err(Error::AmplitudeInvalid);
        };
        for (item, amplitude) in self.amplitude.iter_mut().zip(amplitude) {
            *item = amplitude;
        }
        Ok(())
    }
}

/// Amplitude is decaying exponentially, also for overtones
/// [Exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
/// index: 0 = fundamental tone, 1.. = overtones.
#[derive(Debug, Clone)]
pub struct AmplitudeDecayExpOvertones {
    sample_time: SampleCalc,
    amplitude: Vec<SampleCalc>, // starting amplitudes
    rate: Vec<SampleCalc>, // rate must be negative!
}

impl AmplitudeDecayExpOvertones {
    /// custom constructor
    /// It normalizes the amplitudes, so the sum of the starting amplitudes will be 1.0.
    /// Rate must be negative!
    pub fn new(sample_rate: SampleCalc,
               mut amplitude: Vec<SampleCalc>,
               rate: Vec<SampleCalc>)
               -> SoundResult<AmplitudeDecayExpOvertones> {
        let sample_time = try!(get_sample_time(sample_rate));
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in &amplitude {
            if *amplitude_check < 0.0 {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum == 0.0 {
            return Err(Error::AmplitudeInvalid);
        };
        // normalization
        for item in &mut amplitude {
            *item /= amplitude_sum;
        }
        for rate_check in &rate {
            if *rate_check > 0.0 {
                return Err(Error::AmplitudeRateInvalid);
            }
        }
        Ok(AmplitudeDecayExpOvertones {
            sample_time: sample_time,
            amplitude: amplitude,
            rate: rate,
        })
    }
}

impl AmplitudeOvertonesProvider for AmplitudeDecayExpOvertones {
    fn get(&self,
           time_start: SampleCalc,
           overtone: usize,
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if (overtone >= self.amplitude.len()) || (overtone >= self.rate.len()) {
            for item in result.iter_mut() {
                *item = 0.0;
            }
            return Ok(());
        };
        let amplitude_overtone = self.amplitude[overtone];
        let position_start = time_start * self.rate[overtone];
        let position_change = self.sample_time * self.rate[overtone];
        for (index, item) in result.iter_mut().enumerate() {
            let position: SampleCalc = (index as SampleCalc * position_change) + position_start;
            // TODO: speed optimization, .exp() is very slow
            *item = amplitude_overtone * position.exp();
        }
        Ok(())
    }

    fn apply(&self,
             time_start: SampleCalc,
             overtone: usize,
             samples: &mut [SampleCalc])
             -> SoundResult<()> {
        if (overtone >= self.amplitude.len()) || (overtone >= self.rate.len()) {
            for item in samples.iter_mut() {
                *item = 0.0;
            }
            return Ok(());
        };
        let amplitude_overtone = self.amplitude[overtone];
        let position_start = time_start * self.rate[overtone];
        let position_change = self.sample_time * self.rate[overtone];
        for (index, item) in samples.iter_mut().enumerate() {
            let position: SampleCalc = (index as SampleCalc * position_change) + position_start;
            // TODO: speed optimization, .exp() is very slow
            *item *= amplitude_overtone * position.exp();
        }
        Ok(())
    }
}

impl AmplitudeOvertonesJoinable for AmplitudeDecayExpOvertones {
    fn set_amplitude_start(&mut self, amplitude: Vec<SampleCalc>) -> SoundResult<()> {
        if amplitude.len() != self.amplitude.len() {
            return Err(Error::OvertoneCountInvalid);
        }
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in &amplitude {
            if (*amplitude_check < 0.0) || (*amplitude_check > 1.0) {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum == 0.0 {
            return Err(Error::AmplitudeInvalid);
        };
        for (item, amplitude) in self.amplitude.iter_mut().zip(amplitude) {
            *item = amplitude;
        }
        Ok(())
    }
}

/// A sequence of amplitude functions with overtones.
pub struct AmplitudeOvertonesSequence;
