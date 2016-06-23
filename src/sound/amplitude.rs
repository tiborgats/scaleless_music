use sound::*;
use std::cell::Cell;
// use rayon::prelude::*;

/// Provides time dependent amlitude changes.
pub trait AmplitudeProvider {
    /// Provides the results of the amplitude calculations.
    fn get(&self, result: &mut [SampleCalc]) -> SoundResult<()>;
    /// Applies the amplitude function over already existing samples. It multiplies each sample
    /// with it's new amplitude.
    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// Provides rhythmic amlitude changes. As phase depends on the integral of tempo, only sequential
/// reading is possible (cannot be parallelized).
pub trait AmplitudeRhythmProvider {
    /// Provides the results of the amplitude calculations. Tempo is given in beats per second.
    fn get_rhythmic(&self, tempo: &[SampleCalc], result: &mut [SampleCalc]) -> SoundResult<()>;
    /// Applies the amplitude function over already existing samples. It multiplies each sample
    /// with it's new amplitude. Tempo is given in beats per second.
    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// The `AmplitudeJoinable` trait is used to specify the ability of joining amplitude structures
/// together, forming a sequence of them.
pub trait AmplitudeJoinable {
    /// Sets the initial amplitude, and resets time.
    fn set_amplitude_start(&self, amplitude: SampleCalc) -> SoundResult<()>;
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
#[derive(Debug, Clone)]
pub struct FadeLinear {
    /// Tempo or time based progress.
    pub progress: ProgressOption,
    amplitude_start: Cell<SampleCalc>,
    amplitude_end: SampleCalc,
}

impl FadeLinear {
    /// Custom constructor.
    pub fn new(progress: ProgressOption, amplitude_end: SampleCalc) -> SoundResult<FadeLinear> {
        try!(is_valid_amplitude(amplitude_end));
        let amplitude_start = 0.0;
        progress.set_period_unit(amplitude_end - amplitude_start);
        Ok(FadeLinear {
            progress: progress,
            amplitude_start: Cell::new(amplitude_start),
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
    /// `note_value` is the tempo relative fade duration.
    pub fn new_with_tempo(sample_rate: SampleCalc,
                          note_value: NoteValue,
                          amplitude_end: SampleCalc)
                          -> SoundResult<FadeLinear> {
        let progress = try!(ProgressTempo::new(sample_rate, note_value));
        Self::new(ProgressOption::Tempo(progress), amplitude_end)
    }
}

impl AmplitudeJoinable for FadeLinear {
    fn set_amplitude_start(&self, amplitude: SampleCalc) -> SoundResult<()> {
        try!(is_valid_amplitude(amplitude));
        self.amplitude_start.set(amplitude);
        self.progress.set_phase_init(self.amplitude_start.get());
        self.progress.set_period_unit(self.amplitude_end - self.amplitude_start.get());
        Ok(())
    }

    fn get_max(&self) -> SampleCalc {
        self.amplitude_start.get().max(self.amplitude_end)
    }
}

impl AmplitudeProvider for FadeLinear {
    fn get(&self, result: &mut [SampleCalc]) -> SoundResult<()> {
        match self.progress {
            ProgressOption::Time(ref p) => {
                for (index, item) in result.iter_mut().enumerate() {
                    match p.next_phase() {
                        Ok(phase) => *item = phase,
                        Err(Error::ProgressCompleted) => return Err(Error::ItemsCompleted(index)),
                        Err(e) => return Err(e),
                    }
                }
            }
            ProgressOption::Tempo(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }

    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()> {
        match self.progress {
            ProgressOption::Time(ref p) => {
                for (index, item) in samples.iter_mut().enumerate() {
                    match p.next_phase() {
                        Ok(phase) => *item *= phase,
                        Err(Error::ProgressCompleted) => return Err(Error::ItemsCompleted(index)),
                        Err(e) => return Err(e),
                    }
                }
            }
            ProgressOption::Tempo(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }
}

impl AmplitudeRhythmProvider for FadeLinear {
    fn get_rhythmic(&self, tempo: &[SampleCalc], result: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != result.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref p) => {
                for ((index, item), beats_per_second) in result.iter_mut().enumerate().zip(tempo) {
                    match p.next_phase(*beats_per_second) {
                        Ok(phase) => *item = phase,
                        Err(Error::ProgressCompleted) => return Err(Error::ItemsCompleted(index)),
                        Err(e) => return Err(e),
                    }
                }
            }
            ProgressOption::Time(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }

    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref p) => {
                for ((index, item), beats_per_second) in samples.iter_mut()
                    .enumerate()
                    .zip(tempo) {
                    match p.next_phase(*beats_per_second) {
                        Ok(phase) => *item *= phase,
                        Err(Error::ProgressCompleted) => return Err(Error::ItemsCompleted(index)),
                        Err(e) => return Err(e),
                    }
                }
            }
            ProgressOption::Time(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }
}

/// [Tremolo](https://en.wikipedia.org/wiki/Tremolo), as sine variation of the amplitude.
#[derive(Debug, Clone)]
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
    /// `extent_ratio` is the ratio of maximum shift away from the base amplitude (must be > 1.0).
    pub fn new(progress: ProgressOption, extent_ratio: SampleCalc) -> SoundResult<Tremolo> {
        if extent_ratio <= 1.0 {
            return Err(Error::AmplitudeInvalid);
        }
        let amplitude_normalized = 1.0 / extent_ratio;
        Ok(Tremolo {
            progress: progress,
            extent_ratio: extent_ratio,
            amplitude_normalized: amplitude_normalized,
        })
    }

    /// Custom constructor with time based progress.
    pub fn new_with_time(sample_rate: SampleCalc,
                         duration: SampleCalc,
                         period: SampleCalc,
                         extent_ratio: SampleCalc)
                         -> SoundResult<Tremolo> {
        let progress = try!(ProgressTime::new(sample_rate, duration));
        try!(progress.set_period(period));
        Self::new(ProgressOption::Time(progress), extent_ratio)
    }

    /// Constructor with tempo based progress.
    pub fn new_with_tempo(sample_rate: SampleCalc,
                          note_value: NoteValue,
                          period: NoteValue,
                          extent_ratio: SampleCalc)
                          -> SoundResult<Tremolo> {
        let progress = try!(ProgressTempo::new(sample_rate, note_value));
        progress.set_period(period);
        Self::new(ProgressOption::Tempo(progress), extent_ratio)
    }
}

impl AmplitudeProvider for Tremolo {
    fn get(&self, result: &mut [SampleCalc]) -> SoundResult<()> {
        match self.progress {
            ProgressOption::Time(ref p) => {
                for (index, item) in result.iter_mut().enumerate() {
                    match p.next_phase() {
                        Ok(phase) => {
                            *item = self.amplitude_normalized *
                                    (self.extent_ratio.powf(phase.sin()))
                        }
                        Err(Error::ProgressCompleted) => return Err(Error::ItemsCompleted(index)),
                        Err(e) => return Err(e),
                    }
                }
                p.simplify();
            }
            ProgressOption::Tempo(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }

    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()> {
        match self.progress {
            ProgressOption::Time(ref p) => {
                for (index, item) in samples.iter_mut().enumerate() {
                    match p.next_phase() {
                        Ok(phase) => {
                            *item *= self.amplitude_normalized *
                                     (self.extent_ratio.powf(phase.sin()))
                        }
                        Err(Error::ProgressCompleted) => return Err(Error::ItemsCompleted(index)),
                        Err(e) => return Err(e),
                    }
                }
                p.simplify();
            }
            ProgressOption::Tempo(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }
}

impl AmplitudeRhythmProvider for Tremolo {
    fn get_rhythmic(&self, tempo: &[SampleCalc], result: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != result.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref p) => {
                for ((index, item), beats_per_second) in result.iter_mut().enumerate().zip(tempo) {
                    match p.next_phase(*beats_per_second) {
                        Ok(phase) => {
                            *item = self.amplitude_normalized *
                                    (self.extent_ratio.powf(phase.sin()))
                        }
                        Err(Error::ProgressCompleted) => return Err(Error::ItemsCompleted(index)),
                        Err(e) => return Err(e),
                    }
                }
                p.simplify();
            }
            ProgressOption::Time(ref _p) => return Err(Error::ProgressInvalid),
        }
        Ok(())
    }

    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref p) => {
                for ((index, item), beats_per_second) in samples.iter_mut().enumerate().zip(tempo) {
                    match p.next_phase(*beats_per_second) {
                        Ok(phase) => {
                            *item *= self.amplitude_normalized *
                                     (self.extent_ratio.powf(phase.sin()))
                        }
                        Err(Error::ProgressCompleted) => return Err(Error::ItemsCompleted(index)),
                        Err(e) => return Err(e),
                    }
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
