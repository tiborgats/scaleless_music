use sound::*;
use rayon::prelude::*;

/// Input and output definition for the amplitude functions.
pub trait AmplitudeFunction {
    /// Provides the results of the amplitude calculations.
    fn get(&self,
           time_start: SampleCalc,
           overtone: usize,
           result: &mut [SampleCalc])
           -> SoundResult<()>;
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

impl AmplitudeFunction for AmplitudeConstOvertones {
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
}

/// Amplitude is decaying exponentially, also for overtones
/// [Exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
/// index: 0 = fundamental tone, 1.. = overtones.
#[derive(Debug, Clone)]
pub struct AmplitudeDecayExpOvertones {
    sample_rate: SampleCalc,
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
        if sample_rate <= 0.0 {
            return Err(Error::SampleRateInvalid);
        };
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
            sample_rate: sample_rate,
            amplitude: amplitude,
            rate: rate,
        })
    }
}

impl AmplitudeFunction for AmplitudeDecayExpOvertones {
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
        // sample_time: a variable for speed optimization: multiplication is faster than
        // division, so division is done out of the loop
        let sample_time: SampleCalc = 1.0 / self.sample_rate;

        // for (index, item) in result.iter_mut().enumerate() {
        result.par_iter_mut()
//            .weight(20.0)
            .enumerate()
            .for_each(|(index, item)| {
                let time: SampleCalc = (index as SampleCalc * sample_time) + time_start;
                // TODO: speed optimization, .exp() is very slow
                *item = self.amplitude[overtone] * (time * self.rate[overtone]).exp();
            });
        Ok(())
    }
}

/// [Equal-loudness contour](https://en.wikipedia.org/wiki/Equal-loudness_contour)
/// data used is described by the ISO 226:2003 standard
/// see also: https://plot.ly/~mrlyule/16/equal-loudness-contours-iso-226-2003/
pub struct AmplitudeEqualLoudness;
