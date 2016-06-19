use sound::*;
// use rayon::prelude::*;

/// Provides time dependent amlitude changes.
pub trait AmplitudeProvider {
    /// Provides the results of the amplitude calculations.
    fn get(&self, time_start: SampleCalc, result: &mut [SampleCalc]) -> SoundResult<()>;
    /// Applies the amplitude function over an existing sample. It multiplies each sample with
    /// it's new amplitude.
    fn apply(&self, time_start: SampleCalc, samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// Provides rhythmic amlitude changes. As phase depends on the integral of tempo, only sequential
/// reading is possible (cannot be parallelized).
pub trait AmplitudeRhythmProvider {
    /// Provides the results of the amplitude calculations. Tempo is given in beats per second.
    fn get(&mut self, tempo: &[SampleCalc], result: &mut [SampleCalc]) -> SoundResult<()>;
    /// Applies the amplitude function over already existing samples. It multiplies each sample
    /// with it's new amplitude. Tempo is given in beats per second.
    fn apply(&mut self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// The `AmplitudeJoinable` trait is used to specify the ability of joining amplitude structures
/// together, forming a sequence of them.
pub trait AmplitudeJoinable {
    /// Sets the initial amplitude, and resets time.
    fn set_amplitude_start(&mut self, amplitude: SampleCalc) -> SoundResult<()>;
    // Provides the last amplitude.
    // fn get_last_amplitude() -> SampleCalc;
    /// Provides the maximal possible amplitude (for normalization).
    fn get_max(&self) -> SampleCalc;
}

/// Checks if the given value is in the valid amplitude range.
pub fn is_valid_amplitude(amplitude: SampleCalc) -> SoundResult<()> {
    if amplitude < 0.0 {
        return Err(Error::AmplitudeInvalid);
    }
    if amplitude > 1.0 {
        return Err(Error::AmplitudeInvalid);
    }
    Ok(())
}

/// Linearly changing amplitude.
#[derive(Debug, Copy, Clone)]
pub struct FadeLinear {
    /// Tempo or time based progress.
    pub progress: ProgressOption,
    amplitude_start: SampleCalc,
    amplitude_end: SampleCalc,
}

impl FadeLinear {
    /// custom constructor
    pub fn new(mut progress: ProgressOption, amplitude_end: SampleCalc) -> SoundResult<FadeLinear> {
        try!(is_valid_amplitude(amplitude_end));
        let amplitude_start = 0.0;
        progress.set_period(amplitude_end - amplitude_start);
        Ok(FadeLinear {
            progress: progress,
            amplitude_start: amplitude_start,
            amplitude_end: amplitude_end,
        })
    }

    /// Custom constructor with time based progress.
    pub fn new_with_time(sample_rate: SampleCalc,
                         duration: SampleCalc,
                         amplitude_end: SampleCalc)
                         -> SoundResult<FadeLinear> {
        let progress = try!(ProgressTime::new(sample_rate, duration));
        Self::new(ProgressOption::Time(progress), amplitude_end)
    }

    /// Constructor with tempo based progress.
    ///
    /// `note_value` = The (tempo relative) speed with which the amplitude is varied.
    pub fn new_with_tempo(sample_rate: SampleCalc,
                          note_value: NoteValue,
                          amplitude_end: SampleCalc)
                          -> SoundResult<FadeLinear> {
        let progress = try!(ProgressTempo::new(sample_rate, note_value));
        Self::new(ProgressOption::Tempo(progress), amplitude_end)
    }
}

impl AmplitudeJoinable for FadeLinear {
    fn set_amplitude_start(&mut self, amplitude: SampleCalc) -> SoundResult<()> {
        try!(is_valid_amplitude(amplitude));
        self.amplitude_start = amplitude;
        // self.time = 0.0;
        Ok(())
    }

    fn get_max(&self) -> SampleCalc {
        self.amplitude_start.max(self.amplitude_end)
    }
}

impl AmplitudeRhythmProvider for FadeLinear {
    fn get(&mut self, tempo: &[SampleCalc], result: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != result.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref mut p) => {
                for (item, beats_per_second) in result.iter_mut().zip(tempo) {
                    *item = p.next_phase(*beats_per_second);
                }
                p.simplify();
            }
            ProgressOption::Time(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }

    fn apply(&mut self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref mut p) => {
                for (item, beats_per_second) in samples.iter_mut().zip(tempo) {
                    *item *= p.next_phase(*beats_per_second);
                }
                p.simplify();
            }
            ProgressOption::Time(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }
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

impl AmplitudeProvider for FadeInLinear {
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

impl AmplitudeProvider for FadeOutLinear {
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
    /// Tempo or time based progress.
    pub progress: ProgressOption,
    /// The ratio of maximum shift away from the base amplitude (must be > 1.0).
    extent_ratio: SampleCalc,
    /// The phase of the sine function.
    amplitude_normalized: SampleCalc,
}

impl Tremolo {
    /// Custom constructor.
    ///
    /// `extent_ratio` = The ratio of maximum shift away from the base amplitude (must be > 1.0).
    pub fn new(progress: ProgressOption, extent_ratio: SampleCalc) -> SoundResult<Tremolo> {
        if extent_ratio <= 1.0 {
            return Err(Error::AmplitudeInvalid);
        }
        let amplitude_normalized = 1.0 / extent_ratio;
        //        let progress = try!(ProgressTempo::new(sample_rate, note_value));
        Ok(Tremolo {
            progress: progress,
            extent_ratio: extent_ratio,
            amplitude_normalized: amplitude_normalized,
        })
    }

    /// Custom constructor with time based progress.
    pub fn new_with_time(sample_rate: SampleCalc,
                         duration: SampleCalc,
                         extent_ratio: SampleCalc)
                         -> SoundResult<Tremolo> {
        let progress = try!(ProgressTime::new(sample_rate, duration));
        Self::new(ProgressOption::Time(progress), extent_ratio)
    }

    /// Constructor with tempo based progress.
    ///
    /// `note_value` = The (tempo relative) speed with which the amplitude is varied.
    pub fn new_with_tempo(sample_rate: SampleCalc,
                          note_value: NoteValue,
                          extent_ratio: SampleCalc)
                          -> SoundResult<Tremolo> {
        let progress = try!(ProgressTempo::new(sample_rate, note_value));
        Self::new(ProgressOption::Tempo(progress), extent_ratio)
    }
}

impl AmplitudeRhythmProvider for Tremolo {
    fn get(&mut self, tempo: &[SampleCalc], result: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != result.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref mut p) => {
                for (item, beats_per_second) in result.iter_mut().zip(tempo) {
                    *item = self.amplitude_normalized *
                            (self.extent_ratio.powf(p.next_phase(*beats_per_second).sin()));
                }
                p.simplify();
            }
            ProgressOption::Time(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }

    fn apply(&mut self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref mut p) => {
                for (item, beats_per_second) in samples.iter_mut().zip(tempo) {
                    *item *= self.amplitude_normalized *
                             (self.extent_ratio.powf(p.next_phase(*beats_per_second).sin()));
                }
                p.simplify();
            }
            ProgressOption::Time(ref _p) => return Err(Error::ProgressInvalid),
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
