use sound::*;
use std::cell::Cell;
use std::rc::Rc;

/// Provides time dependent amlitude changes.
pub trait AmplitudeProvider {
    /// Applies the amplitude function over already existing samples. It multiplies each sample
    /// with it's new amplitude.
    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()>;

    /// Applies the amplitude function over already existing samples. It multiplies each sample
    /// with it's new amplitude. Tempo is given in beats per second.
    /// Note: as phase depends on the integral of tempo, only sequential reading is possible
    /// (cannot be parallelized).
    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// The `AmplitudeJoinable` trait is used to specify the ability of joining amplitude structures
/// together, forming a sequence of them.
pub trait AmplitudeJoinable: AmplitudeProvider + HasTimer {
    /// Sets the initial amplitude, and resets time.
    fn set_amplitude_start(&self, amplitude: SampleCalc) -> SoundResult<()>;
    /// Provides the actual amplitude value.
    fn get_amplitude(&self) -> SampleCalc;
    /// Provides the maximal possible future amplitude (for normalization).
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

/// Constant amplitude.
#[derive(Debug, Clone)]
pub struct AmplitudeConst {
    timer: Timer,
    amplitude: Cell<SampleCalc>,
}

impl AmplitudeConst {
    /// Custom constructor.
    pub fn new(sample_rate: SampleCalc) -> SoundResult<AmplitudeConst> {
        Ok(AmplitudeConst {
            timer: Timer::new(sample_rate)?,
            amplitude: Cell::new(1.0),
        })
    }
}

impl AmplitudeProvider for AmplitudeConst {
    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()> {
        let timer_result = self.timer.jump_by_time(samples.len());
        match timer_result {
            Ok(()) => {
                for item in samples.iter_mut() {
                    *item *= self.amplitude.get();
                }
            }
            Err(Error::ItemsCompleted(completed)) => {
                for item in samples.iter_mut().take(completed) {
                    *item *= self.amplitude.get();
                }
            }
            Err(ref _e) => {}
        }
        timer_result
    }

    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        let timer_result = self.timer.jump_by_tempo(tempo);
        match timer_result {
            Ok(()) => {
                for item in samples.iter_mut() {
                    *item *= self.amplitude.get();
                }
            }
            Err(Error::ItemsCompleted(completed)) => {
                for item in samples.iter_mut().take(completed) {
                    *item *= self.amplitude.get();
                }
            }
            Err(ref _e) => {}
        }
        timer_result
    }
}

impl HasTimer for AmplitudeConst {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        self.timer.set_timing(timing)?;
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timer.get_timing()
    }

    fn restart(&self) {
        self.timer.restart();
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        self.timer.apply_parent_timing(parent_timing)
    }
}

impl AmplitudeJoinable for AmplitudeConst {
    fn set_amplitude_start(&self, amplitude: SampleCalc) -> SoundResult<()> {
        is_valid_amplitude(amplitude)?;
        self.amplitude.set(amplitude);
        self.timer.restart();
        Ok(())
    }

    fn get_amplitude(&self) -> SampleCalc {
        self.amplitude.get()
    }

    fn get_max(&self) -> SampleCalc {
        self.amplitude.get()
    }
}

/// Linearly changing amplitude.
#[derive(Debug, Clone)]
pub struct FadeLinear {
    /// Tempo or time based progress.
    progress: ProgressOption,
    amplitude_start: Cell<SampleCalc>,
    amplitude_end: SampleCalc,
}

impl FadeLinear {
    /// Custom constructor.
    pub fn new(progress: ProgressOption, amplitude_end: SampleCalc) -> SoundResult<FadeLinear> {
        is_valid_amplitude(amplitude_end)?;
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
        let progress = ProgressTime::new(sample_rate, duration)?;
        Self::new(ProgressOption::Time(progress), amplitude_end)
    }

    /// Constructor with tempo based progress.
    /// `note_value` is the tempo relative fade duration.
    pub fn new_with_tempo(sample_rate: SampleCalc,
                          note_value: NoteValue,
                          amplitude_end: SampleCalc)
                          -> SoundResult<FadeLinear> {
        let progress = ProgressTempo::new(sample_rate, note_value)?;
        Self::new(ProgressOption::Tempo(progress), amplitude_end)
    }
}

impl AmplitudeProvider for FadeLinear {
    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()> {
        match self.progress {
            ProgressOption::Time(ref p) => {
                for (index, item) in samples.iter_mut().enumerate() {
                    match p.next_by_time() {
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

    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref p) => {
                for ((index, item), beats_per_second) in
                    samples.iter_mut()
                        .enumerate()
                        .zip(tempo) {
                    match p.next_by_tempo(*beats_per_second) {
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

impl HasTimer for FadeLinear {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        self.progress.set_timing(timing)?;
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.progress.get_timing()
    }

    fn restart(&self) {
        self.progress.restart();
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        self.progress.apply_parent_timing(parent_timing)
    }
}

impl AmplitudeJoinable for FadeLinear {
    fn set_amplitude_start(&self, amplitude: SampleCalc) -> SoundResult<()> {
        is_valid_amplitude(amplitude)?;
        self.amplitude_start.set(amplitude);
        self.progress.set_phase_init(self.amplitude_start.get());
        self.progress.set_period_unit(self.amplitude_end - self.amplitude_start.get());
        Ok(())
    }

    fn get_amplitude(&self) -> SampleCalc {
        self.progress.get_phase()
    }

    fn get_max(&self) -> SampleCalc {
        self.amplitude_start.get().max(self.amplitude_end)
    }
}

/// Amplitude is decaying exponentially. The decay rate only depends on time, even when the
/// duration is tempo dependent.
/// [Exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
#[derive(Debug, Clone)]
pub struct AmplitudeDecayExp {
    timer: Timer,
    sample_time: SampleCalc,
    multiplier: SampleCalc,
    amplitude: Cell<SampleCalc>,
}

impl AmplitudeDecayExp {
    /// custom constructor
    /// `half_life` is the time required to reduce the amplitude to it's half.
    pub fn new(sample_rate: SampleCalc, half_life: SampleCalc) -> SoundResult<AmplitudeDecayExp> {
        let sample_time = get_sample_time(sample_rate)?;
        if half_life <= 0.0 {
            return Err(Error::AmplitudeRateInvalid);
        }
        let half: SampleCalc = 0.5;
        let multiplier = half.powf(sample_time / half_life);
        Ok(AmplitudeDecayExp {
            timer: Timer::new(sample_rate)?,
            sample_time: sample_time,
            multiplier: multiplier,
            amplitude: Cell::new(1.0),
        })
    }
}

impl AmplitudeProvider for AmplitudeDecayExp {
    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()> {
        let timer_result = self.timer.jump_by_time(samples.len());
        match timer_result {
            Ok(()) => {
                for item in samples.iter_mut() {
                    self.amplitude.set(self.amplitude.get() * self.multiplier);
                    *item *= self.amplitude.get();
                }
                for item in samples.iter_mut() {
                    self.amplitude.set(self.amplitude.get() * self.multiplier);
                    *item *= self.amplitude.get();
                }
            }
            Err(Error::ItemsCompleted(completed)) => {
                for item in samples.iter_mut().take(completed) {
                    *item = self.amplitude.get();
                }
            }
            Err(ref _e) => {}
        }
        timer_result
    }

    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        let timer_result = self.timer.jump_by_tempo(tempo);
        match timer_result {
            Ok(()) => {
                for item in samples.iter_mut() {
                    self.amplitude.set(self.amplitude.get() * self.multiplier);
                    *item *= self.amplitude.get();
                }
            }
            Err(Error::ItemsCompleted(completed)) => {
                for item in samples.iter_mut().take(completed) {
                    self.amplitude.set(self.amplitude.get() * self.multiplier);
                    *item *= self.amplitude.get();
                }
            }
            Err(ref _e) => {}
        }
        timer_result
    }
}

impl HasTimer for AmplitudeDecayExp {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        self.timer.set_timing(timing)?;
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timer.get_timing()
    }

    fn restart(&self) {
        self.timer.restart();
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        self.timer.apply_parent_timing(parent_timing)
    }
}

impl AmplitudeJoinable for AmplitudeDecayExp {
    fn set_amplitude_start(&self, amplitude: SampleCalc) -> SoundResult<()> {
        is_valid_amplitude(amplitude)?;
        self.amplitude.set(amplitude);
        self.timer.restart();
        Ok(())
    }

    fn get_amplitude(&self) -> SampleCalc {
        self.amplitude.get()
    }

    fn get_max(&self) -> SampleCalc {
        self.amplitude.get()
    }
}

/// [Tremolo](https://en.wikipedia.org/wiki/Tremolo), as sine variation of the amplitude.
#[derive(Debug, Clone)]
pub struct Tremolo {
    /// Tempo or time based progress.
    progress: ProgressOption,
    /// The ratio of maximum shift away from the base amplitude (must be > 1.0).
    extent_ratio: SampleCalc,
    /// The average amplitude. It is calculated in a way that the peak amplitude will be 1.0.
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
                         timing: TimingOption,
                         period: SampleCalc,
                         extent_ratio: SampleCalc)
                         -> SoundResult<Tremolo> {
        let progress = ProgressTime::new(sample_rate, period)?;
        progress.set_timing(timing)?;
        Self::new(ProgressOption::Time(progress), extent_ratio)
    }

    /// Constructor with tempo based progress.
    pub fn new_with_tempo(sample_rate: SampleCalc,
                          timing: TimingOption,
                          period: NoteValue,
                          extent_ratio: SampleCalc)
                          -> SoundResult<Tremolo> {
        let progress = ProgressTempo::new(sample_rate, period)?;
        progress.set_timing(timing)?;
        Self::new(ProgressOption::Tempo(progress), extent_ratio)
    }
}

impl AmplitudeProvider for Tremolo {
    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()> {
        match self.progress {
            ProgressOption::Time(ref p) => {
                for (index, item) in samples.iter_mut().enumerate() {
                    match p.next_by_time() {
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

    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        match self.progress {
            ProgressOption::Tempo(ref p) => {
                for ((index, item), beats_per_second) in samples.iter_mut().enumerate().zip(tempo) {
                    match p.next_by_tempo(*beats_per_second) {
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

impl HasTimer for Tremolo {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        self.progress.set_timing(timing)?;
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.progress.get_timing()
    }

    fn restart(&self) {
        self.progress.restart();
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        self.progress.apply_parent_timing(parent_timing)
    }
}

/// Sequence of several amplitude functions.
#[derive(Clone)]
pub struct AmplitudeSequence {
    timer: Timer,
    amp_funct_array: Vec<Rc<AmplitudeJoinable>>,
    array_index: Cell<usize>,
    amplitude: Cell<SampleCalc>, // the last amplitude value
}

impl AmplitudeSequence {
    /// Custom constructor.
    pub fn new(sample_rate: SampleCalc) -> SoundResult<AmplitudeSequence> {
        let v = Vec::new();
        Ok(AmplitudeSequence {
            timer: Timer::new(sample_rate)?,
            amp_funct_array: v,
            array_index: Cell::new(0),
            amplitude: Cell::new(1.0),
        })
    }

    /// Adds a new amplitude function to the sequence.
    pub fn add(&mut self, amplitude: Rc<AmplitudeJoinable>) {
        self.amp_funct_array.push(amplitude);
    }
}

impl AmplitudeProvider for AmplitudeSequence {
    fn apply(&self, samples: &mut [SampleCalc]) -> SoundResult<()> {
        if self.amp_funct_array.is_empty() {
            return Err(Error::SequenceEmpty);
        }
        let buffer: &mut [SampleCalc];
        let timer_result = self.timer.jump_by_time(samples.len());
        match timer_result {
            Ok(()) => buffer = samples,
            Err(Error::ItemsCompleted(completed)) => buffer = &mut samples[0..completed],
            Err(_) => return timer_result,
        }
        let mut index_from: usize = 0;
        loop {
            let amp_funct_act =
                self.amp_funct_array.get(self.array_index.get()).ok_or(Error::ItemInvalid)?;
            let child_result = amp_funct_act.apply(&mut buffer[index_from..]);
            match child_result {
                Ok(()) => {
                    self.amplitude.set(amp_funct_act.get_amplitude());
                    return timer_result;
                }
                Err(Error::ItemsCompleted(completed)) => {
                    index_from = completed;
                    let array_index = self.array_index.get() + 1;
                    if array_index >= self.amp_funct_array.len() {
                        return Err(Error::ItemInvalid);
                    } else {
                        self.array_index.set(array_index);
                        self.amplitude.set(amp_funct_act.get_amplitude());
                        let amp_funct_next = self.amp_funct_array
                            .get(array_index)
                            .ok_or(Error::ItemInvalid)?;
                        amp_funct_next.set_amplitude_start(self.amplitude.get())?;
                        amp_funct_next.apply_parent_timing(self.timer.get_timing())?;
                    }
                }
                Err(_) => return child_result,
            }
        }
    }

    fn apply_rhythmic(&self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        if tempo.len() != samples.len() {
            return Err(Error::BufferSize);
        }
        if self.amp_funct_array.is_empty() {
            return Err(Error::SequenceEmpty);
        }
        let buffer: &mut [SampleCalc];
        let timer_result = self.timer.jump_by_time(samples.len());
        match timer_result {
            Ok(()) => buffer = samples,
            Err(Error::ItemsCompleted(completed)) => buffer = &mut samples[0..completed],
            Err(_) => return timer_result,
        }
        let mut index_from: usize = 0;
        loop {
            let amp_funct_act =
                self.amp_funct_array.get(self.array_index.get()).ok_or(Error::ItemInvalid)?;
            let child_result = amp_funct_act.apply_rhythmic(tempo, &mut buffer[index_from..]);
            match child_result {
                Ok(()) => {
                    self.amplitude.set(amp_funct_act.get_amplitude());
                    return timer_result;
                }
                Err(Error::ItemsCompleted(completed)) => {
                    index_from = completed;
                    let array_index = self.array_index.get() + 1;
                    if array_index >= self.amp_funct_array.len() {
                        return Err(Error::ItemInvalid);
                    } else {
                        self.array_index.set(array_index);
                        self.amplitude.set(amp_funct_act.get_amplitude());
                        let amp_funct_next = self.amp_funct_array
                            .get(array_index)
                            .ok_or(Error::ItemInvalid)?;
                        amp_funct_next.set_amplitude_start(self.amplitude.get())?;
                        amp_funct_next.apply_parent_timing(self.timer.get_timing())?;
                    }
                }
                Err(_) => return child_result,
            }
        }
    }
}

impl HasTimer for AmplitudeSequence {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        self.timer.set_timing(timing)?;
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timer.get_timing()
    }

    fn restart(&self) {
        self.timer.restart();
        self.array_index.set(0);
        if let Some(amplitude) = self.amp_funct_array.first() {
            amplitude.restart();
        }
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        self.timer.apply_parent_timing(parent_timing)?;
        self.restart();
        Ok(())
    }
}

impl AmplitudeJoinable for AmplitudeSequence {
    fn set_amplitude_start(&self, amplitude: SampleCalc) -> SoundResult<()> {
        is_valid_amplitude(amplitude)?;
        self.amplitude.set(amplitude);
        self.timer.restart();
        Ok(())
    }

    fn get_amplitude(&self) -> SampleCalc {
        self.amplitude.get()
    }

    fn get_max(&self) -> SampleCalc {
        let mut amplitude_max: SampleCalc = 0.0;
        for item in &self.amp_funct_array {
            amplitude_max = amplitude_max.max(item.get_max());
        }
        amplitude_max
    }
}

/// Combination of several amplitude functions.
pub struct AmplitudeCombination;


/// [Equal-loudness contour](https://en.wikipedia.org/wiki/Equal-loudness_contour)
/// data used is described by the ISO 226:2003 standard
/// see also: https://plot.ly/~mrlyule/16/equal-loudness-contours-iso-226-2003/
pub struct AmplitudeEqualLoudness;
