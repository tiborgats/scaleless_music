use sound::*;
use std::cell::Cell;

/// Common methods of the Progress types.
pub trait Progress {
    /// Simplifies the phase to achieve higher accuracy. It is only used for periodic functions.
    fn simplify(&self);
    /// Sets a new period unit, and restarts the progress. Period unit is the amount of phase
    /// change during one period.
    fn set_period_unit(&self, period_unit: SampleCalc);
    /// Sets a new initial phase value, and restarts the progress.
    fn set_phase_init(&self, phase: SampleCalc);
    /// Restarts the progress.
    fn restart(&self);
}

/// Time based progress measurement. It provides the sequence of phases (for sound functions) by
/// calling `next_phase()`. The whole duration can be divided to periods, for periodic functions.
#[derive(Debug, Clone)]
pub struct ProgressTime {
    sample_time: SampleCalc,
    duration: Cell<SampleCalc>,
    /// Only for periodic functions: the duration of one period.
    period: Cell<SampleCalc>,
    /// The amount of phase change during one period.
    period_unit: Cell<SampleCalc>,
    /// Initial value of `phase`.
    phase_init: Cell<SampleCalc>,
    phase_change: Cell<SampleCalc>,
    /// The remaining time.
    remaining_time: Cell<SampleCalc>,
    /// The phase of the progress.
    phase: Cell<SampleCalc>,
}

impl ProgressTime {
    /// Custom constructor with duration. The default period is the whole duration.
    /// The default period unit is π x 2.
    pub fn new(sample_rate: SampleCalc, duration: SampleCalc) -> SoundResult<ProgressTime> {
        let sample_time = try!(get_sample_time(sample_rate));
        if duration <= 0.0 {
            return Err(Error::PeriodInvalid);
        }
        let period = duration;
        let period_unit = PI2;
        let phase_change = (sample_time / period) * period_unit;
        Ok(ProgressTime {
            sample_time: sample_time,
            duration: Cell::new(duration),
            period: Cell::new(period),
            period_unit: Cell::new(period_unit),
            phase_init: Cell::new(0.0),
            phase_change: Cell::new(phase_change),
            remaining_time: Cell::new(duration),
            phase: Cell::new(0.0),
        })
    }

    /// Provides the next phase value, or `Error::ProgressCompleted` if the progress is finished.
    pub fn next_phase(&self) -> SoundResult<SampleCalc> {
        self.remaining_time.set(self.remaining_time.get() - self.sample_time);
        if self.remaining_time.get() <= 0.0 {
            return Err(Error::ProgressCompleted);
        }
        self.phase.set(self.phase.get() + self.phase_change.get());
        Ok(self.phase.get())
    }

    /// Sets the duration, and restarts the progress.
    pub fn set_duration(&self, duration: SampleCalc) -> SoundResult<()> {
        if duration <= 0.0 {
            return Err(Error::DurationInvalid);
        }
        self.duration.set(duration);
        self.remaining_time.set(self.duration.get());
        self.restart();
        Ok(())
    }

    /// Sets a new period, and restarts the progress.
    pub fn set_period(&self, period: SampleCalc) -> SoundResult<()> {
        if period <= 0.0 {
            return Err(Error::PeriodInvalid);
        }
        self.period.set(period);
        self.restart();
        Ok(())
    }

    /// Sets a new period calculated from the given frequency, and restarts the progress.
    pub fn set_frequency(&self, frequency: SampleCalc) -> SoundResult<()> {
        if frequency <= 0.0 {
            return Err(Error::FrequencyInvalid);
        }
        self.set_period(1.0 / frequency)
    }
}

impl Progress for ProgressTime {
    fn simplify(&self) {
        self.phase.set(self.phase.get() % self.period_unit.get());
    }

    fn set_period_unit(&self, period_unit: SampleCalc) {
        self.period_unit.set(period_unit);
        self.restart();
    }

    fn set_phase_init(&self, phase: SampleCalc) {
        self.phase_init.set(phase);
        self.restart();
    }

    fn restart(&self) {
        self.phase.set(self.phase_init.get());
        self.remaining_time.set(self.duration.get());
        self.phase_change.set((self.sample_time / self.period.get()) * self.period_unit.get());
    }
}

/// Tempo based progress measurement. It provides the sequence of phases (for sound functions) by
/// calling `next_phase()`. The whole duration can be divided to periods, for periodic functions.
#[derive(Debug, Clone)]
pub struct ProgressTempo {
    sample_time: SampleCalc,
    /// The tempo relative duration, measured in beats.
    duration: Cell<NoteValue>,
    /// Only for periodic functions: the duration of one period in beats.
    period: Cell<NoteValue>,
    /// The amount of phase change during one period.
    period_unit: Cell<SampleCalc>,
    /// Initial value of `phase`.
    phase_init: Cell<SampleCalc>,
    phase_change: Cell<SampleCalc>,
    /// The remaining beats.
    remaining_beats: Cell<SampleCalc>,
    /// The phase of the progress.
    phase: Cell<SampleCalc>,
}

impl ProgressTempo {
    /// Custom constructor. The default period is the whole duration.
    /// The default period unit is π x 2.
    pub fn new(sample_rate: SampleCalc, duration: NoteValue) -> SoundResult<ProgressTempo> {
        let sample_time = try!(get_sample_time(sample_rate));
        let period = duration;
        let period_unit = PI2;
        let phase_change = sample_time * period.get_notes_per_beat() * period_unit;
        Ok(ProgressTempo {
            sample_time: sample_time,
            duration: Cell::new(duration),
            period: Cell::new(period),
            period_unit: Cell::new(period_unit),
            phase_init: Cell::new(0.0),
            phase_change: Cell::new(phase_change),
            remaining_beats: Cell::new(duration.get_duration_in_beats()),
            phase: Cell::new(0.0),
        })
    }

    /// Provides the next phase value depending on the actual tempo, or
    /// `Error::ProgressCompleted` if progress is finished.
    pub fn next_phase(&self, beats_per_second: SampleCalc) -> SoundResult<SampleCalc> {
        self.remaining_beats.set(self.remaining_beats.get() - self.sample_time * beats_per_second);
        if self.remaining_beats.get() <= 0.0 {
            return Err(Error::ProgressCompleted);
        }
        self.phase.set(self.phase.get() + self.phase_change.get() * beats_per_second);
        Ok(self.phase.get())
    }

    /// Sets the duration in tempo beats, and restarts the progress.
    pub fn set_duration(&self, duration: NoteValue) {
        self.duration.set(duration);
        self.restart();
    }

    /// Sets a new period in tempo beats, and restarts the progress.
    pub fn set_period(&self, period: NoteValue) {
        self.period.set(period);
        self.restart();
    }
}

impl Progress for ProgressTempo {
    fn simplify(&self) {
        self.phase.set(self.phase.get() % self.period_unit.get());
    }

    fn set_period_unit(&self, period_unit: SampleCalc) {
        self.period_unit.set(period_unit);
        self.restart();
    }

    fn set_phase_init(&self, phase: SampleCalc) {
        self.phase_init.set(phase);
        self.restart();
    }

    fn restart(&self) {
        self.phase.set(self.phase_init.get());
        self.remaining_beats.set(self.duration.get().get_duration_in_beats());
        self.phase_change.set(self.sample_time * self.period.get().get_notes_per_beat() *
                              self.period_unit.get());
    }
}

/// Time or tempo based progress.
#[derive(Debug, Clone)]
pub enum ProgressOption {
    /// Time based progress.
    Time(ProgressTime),
    /// Rhythmic, tempo synchronized progress.
    Tempo(ProgressTempo),
}

impl Progress for ProgressOption {
    fn simplify(&self) {
        match *self {
            ProgressOption::Time(ref p) => p.simplify(),
            ProgressOption::Tempo(ref p) => p.simplify(),
        }
    }

    fn set_period_unit(&self, period_unit: SampleCalc) {
        match *self {
            ProgressOption::Time(ref p) => p.set_period_unit(period_unit),
            ProgressOption::Tempo(ref p) => p.set_period_unit(period_unit),
        }
    }

    fn set_phase_init(&self, phase: SampleCalc) {
        match *self {
            ProgressOption::Time(ref p) => p.set_phase_init(phase),
            ProgressOption::Tempo(ref p) => p.set_phase_init(phase),
        }
    }

    fn restart(&self) {
        match *self {
            ProgressOption::Time(ref p) => p.restart(),
            ProgressOption::Tempo(ref p) => p.restart(),
        }
    }
}

impl From<ProgressTime> for ProgressOption {
    fn from(progress: ProgressTime) -> Self {
        ProgressOption::Time(progress)
    }
}

impl From<ProgressTempo> for ProgressOption {
    fn from(progress: ProgressTempo) -> Self {
        ProgressOption::Tempo(progress)
    }
}
