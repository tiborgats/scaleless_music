use sound::*;
// use rayon::prelude::*;

/// Input and output definition for the amplitude functions.
pub trait AmplitudeFunction {
    /// Provides the results of the amplitude calculations.
    fn get(&self, time_start: SampleCalc, result: &mut [SampleCalc]) -> SoundResult<()>;
    /// Applies the amplitude function over an existing sample. It multiplies each sample with
    /// it's new amplitude.
    fn apply(&self, time_start: SampleCalc, samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// Provides rhythmic amlitude changes. As phase depends on the integral of tempo, only sequential
/// reading is possible (cannot be parallelized).
pub trait AmplitudeRhythm {
    /// Provides the results of the amplitude calculations. Tempo is given in beats per second.
    fn get(&mut self, tempo: &[SampleCalc], result: &mut [SampleCalc]);
    /// Applies the amplitude function over already existing samples. It multiplies each sample
    /// with it's new amplitude. Tempo is given in beats per second.
    fn apply(&mut self, tempo: &[SampleCalc], samples: &mut [SampleCalc]);
}


/// Linearly increasing amplitude.
#[derive(Debug, Copy, Clone)]
pub struct FadeInLinear {
    sample_time: SampleCalc,
    duration: SampleCalc,
    fade_rate: SampleCalc,
}

impl FadeInLinear {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc, duration: SampleCalc) -> SoundResult<FadeInLinear> {
        let sample_time = try!(get_sample_time(sample_rate));
        if duration <= 0.0 {
            return Err(Error::AmplitudeTimeInvalid);
        }
        let fade_rate = 1.0 / duration;
        Ok(FadeInLinear {
            sample_time: sample_time,
            duration: duration,
            fade_rate: fade_rate,
        })
    }
}

impl AmplitudeFunction for FadeInLinear {
    fn get(&self, time_start: SampleCalc, result: &mut [SampleCalc]) -> SoundResult<()> {
        for (index, item) in result.iter_mut().enumerate() {
            let time = (index as SampleCalc * self.sample_time) + time_start;
            *item = if time < self.duration {
                time * self.fade_rate
            } else {
                1.0
            }
        }
        Ok(())
    }
    fn apply(&self, time_start: SampleCalc, samples: &mut [SampleCalc]) -> SoundResult<()> {
        for (index, item) in samples.iter_mut().enumerate() {
            let time = (index as SampleCalc * self.sample_time) + time_start;
            *item *= if time < self.duration {
                time * self.fade_rate
            } else {
                1.0
            }
        }
        Ok(())
    }
}

/// Linearly decreasing amplitude.
#[derive(Debug, Copy, Clone)]
pub struct FadeOutLinear {
    sample_time: SampleCalc,
    duration: SampleCalc,
    fade_rate: SampleCalc,
}

impl FadeOutLinear {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc, duration: SampleCalc) -> SoundResult<FadeOutLinear> {
        let sample_time = try!(get_sample_time(sample_rate));
        if duration <= 0.0 {
            return Err(Error::AmplitudeTimeInvalid);
        }
        let fade_rate = 1.0 / duration;
        Ok(FadeOutLinear {
            sample_time: sample_time,
            duration: duration,
            fade_rate: fade_rate,
        })
    }
}

impl AmplitudeFunction for FadeOutLinear {
    fn get(&self, time_start: SampleCalc, result: &mut [SampleCalc]) -> SoundResult<()> {
        for (index, item) in result.iter_mut().enumerate() {
            let time_left = self.duration - ((index as SampleCalc * self.sample_time) + time_start);
            *item = if time_left > 0.0 {
                time_left * self.fade_rate
            } else {
                0.0
            }
        }
        Ok(())
    }
    fn apply(&self, time_start: SampleCalc, samples: &mut [SampleCalc]) -> SoundResult<()> {
        for (index, item) in samples.iter_mut().enumerate() {
            let time_left = self.duration - ((index as SampleCalc * self.sample_time) + time_start);
            *item *= if time_left > 0.0 {
                time_left * self.fade_rate
            } else {
                0.0
            }
        }
        Ok(())
    }
}

/// [tremolo](https://en.wikipedia.org/wiki/Tremolo), as sine variation of the amplitude.
#[derive(Debug, Copy, Clone)]
pub struct Tremolo {
    sample_time: SampleCalc,
    /// The speed with which the amplitude is varied.
    note_value: NoteValue,
    /// The ratio of maximum shift away from the base amplitude (must be > 0.0).
    extent_ratio: SampleCalc,
    /// The phase of the sine function.
    phase: SampleCalc,
    amplitude_normalized: SampleCalc,
}

impl Tremolo {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc,
               note_value: NoteValue,
               extent_ratio: SampleCalc)
               -> SoundResult<Tremolo> {
        let sample_time = try!(get_sample_time(sample_rate));
        if extent_ratio <= 1.0 {
            return Err(Error::AmplitudeInvalid);
        }
        let amplitude_normalized = 1.0 / extent_ratio;
        Ok(Tremolo {
            sample_time: sample_time,
            note_value: note_value,
            extent_ratio: extent_ratio,
            phase: 0.0,
            amplitude_normalized: amplitude_normalized,
        })
    }

    /// Sets a new phase value.
    pub fn set_phase(&mut self, phase: SampleCalc) -> SoundResult<()> {
        self.phase = phase % PI2;
        Ok(())
    }
}

impl AmplitudeRhythm for Tremolo {
    fn get(&mut self, tempo: &[SampleCalc], result: &mut [SampleCalc]) {
        let phase_change = self.sample_time * self.note_value.get_notes_per_beat() * PI2;
        for (item, beats_per_second) in result.iter_mut().zip(tempo.iter()) {
            self.phase += phase_change * beats_per_second;
            *item = self.amplitude_normalized * (self.extent_ratio.powf(self.phase.sin()));
        }
        self.phase %= PI2;
    }

    fn apply(&mut self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) {
        let phase_change = self.sample_time * self.note_value.get_notes_per_beat() * PI2;
        for (item, beats_per_second) in samples.iter_mut().zip(tempo.iter()) {
            self.phase += phase_change * beats_per_second;
            *item *= self.amplitude_normalized * (self.extent_ratio.powf(self.phase.sin()));
        }
        self.phase %= PI2;
    }
}

/// Input and output definition for the amplitude functions with overtones.
pub trait AmplitudeFunctionOvertones {
    /// Provides the results of the amplitude calculations for a given overtone.
    /// For the fundamental tone `overtone = 0`.
    fn get(&self,
           time_start: SampleCalc,
           overtone: usize,
           result: &mut [SampleCalc])
           -> SoundResult<()>;
    /// Applies the amplitude function over an existing sample for a given overtone.
    /// For the fundamental tone `overtone = 0`.
    fn apply(&self,
             time_start: SampleCalc,
             overtone: usize,
             samples: &mut [SampleCalc])
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

impl AmplitudeFunctionOvertones for AmplitudeConstOvertones {
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

impl AmplitudeFunctionOvertones for AmplitudeDecayExpOvertones {
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

/// Combination of several amplitude functions.
pub struct AmplitudeCombination;


/// [Equal-loudness contour](https://en.wikipedia.org/wiki/Equal-loudness_contour)
/// data used is described by the ISO 226:2003 standard
/// see also: https://plot.ly/~mrlyule/16/equal-loudness-contours-iso-226-2003/
pub struct AmplitudeEqualLoudness;
