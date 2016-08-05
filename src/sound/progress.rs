use sound::*;
use std::cell::Cell;

/// Common methods of the Progress types.
pub trait Progress: HasTimer {
    /// Sets a new period unit, and restarts the progress. Period unit is the amount of phase
    /// change during one period.
    fn set_period_unit(&self, period_unit: SampleCalc);

    /// Sets a new initial phase value, and restarts the progress.
    fn set_phase_init(&self, phase: SampleCalc);

    /// Simplifies the phase to achieve higher accuracy. It is only used for periodic functions.
    fn simplify(&self);

    /// Provides the next phase value, or `Error::ProgressCompleted` if the progress is finished.
    fn next_by_time(&self) -> SoundResult<SampleCalc>;

    /// Provides the next phase value depending on the actual tempo, or
    /// `Error::ProgressCompleted` if progress is finished.
    /// Tempo value is given in beats per second.
    fn next_by_tempo(&self, tempo: SampleCalc) -> SoundResult<SampleCalc>;

    // Returns the final phase value. This phase value will be the last one when the progress
    // reaches it's duration.
    // fn get_phase_final(&self) -> SoundResult<SampleCalc>;

    /// Returns the actual phase value.
    fn get_phase(&self) -> SampleCalc;
}


/// Time based progress measurement. It provides the sequence of phases (for sound functions) by
/// calling `next_phase()`. The whole duration can be divided to periods, for periodic functions.
#[derive(Debug, Clone)]
pub struct ProgressTime {
    timer: Timer,
    /// For periodic functions: the duration of one period.
    period: Cell<SampleCalc>,
    /// The amount of phase change during one period.
    period_unit: Cell<SampleCalc>,
    /// Initial value of `phase`.
    phase_init: Cell<SampleCalc>,
    phase_change: Cell<SampleCalc>,
    /// The phase of the progress.
    phase: Cell<SampleCalc>,
}

impl ProgressTime {
    /// Custom constructor with duration. The default duration is the period.
    /// The default period unit is π x 2.
    pub fn new(sample_rate: SampleCalc, period: SampleCalc) -> SoundResult<ProgressTime> {
        let timer = try!(Timer::new(sample_rate));
        if period <= 0.0 {
            return Err(Error::PeriodInvalid);
        }
        try!(timer.set_timing(TimingOption::TimeConst(period)));
        let period_unit = PI2;
        let phase_change = (timer.get_sample_time() / period) * period_unit;
        Ok(ProgressTime {
            timer: timer,
            period: Cell::new(period),
            period_unit: Cell::new(period_unit),
            phase_init: Cell::new(0.0),
            phase_change: Cell::new(phase_change),
            phase: Cell::new(0.0),
        })
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

impl HasTimer for ProgressTime {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        try!(self.timer.set_timing(timing));
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timer.get_timing()
    }

    fn restart(&self) {
        self.timer.restart();
        self.phase.set(self.phase_init.get());
        self.phase_change
            .set((self.timer.get_sample_time() / self.period.get()) * self.period_unit.get());
    }
}

impl Progress for ProgressTime {
    fn set_period_unit(&self, period_unit: SampleCalc) {
        self.period_unit.set(period_unit);
        self.restart();
    }

    fn set_phase_init(&self, phase: SampleCalc) {
        self.phase_init.set(phase);
        self.restart();
    }

    fn simplify(&self) {
        self.phase.set(self.phase.get() % self.period_unit.get());
    }

    fn next_by_time(&self) -> SoundResult<SampleCalc> {
        try!(self.timer.next_by_time());
        self.phase.set(self.phase.get() + self.phase_change.get());
        Ok(self.phase.get())
    }

    /// Note: it means: the duration is tempo dependent, but the phase change is time dependent.
    fn next_by_tempo(&self, tempo: SampleCalc) -> SoundResult<SampleCalc> {
        try!(self.timer.next_by_tempo(tempo));
        self.phase.set(self.phase.get() + self.phase_change.get());
        Ok(self.phase.get())
    }

    // fn get_phase_final(&self) -> SoundResult<SampleCalc> {
    // match self.duration.get() {
    // Some(d) => {
    // Ok(self.phase_init.get() + ((d / self.period.get()) * self.period_unit.get()))
    // }
    // None => Err(Error::DurationInvalid),
    // }
    // }

    fn get_phase(&self) -> SampleCalc {
        self.phase.get()
    }
}

/// Tempo based progress measurement. It provides the sequence of phases (for sound functions) by
/// calling `next_phase()`. The whole duration can be divided to periods, for periodic functions.
#[derive(Debug, Clone)]
pub struct ProgressTempo {
    timer: Timer,
    /// For periodic functions: the duration of one period in beats.
    period: Cell<NoteValue>,
    /// The amount of phase change during one period.
    period_unit: Cell<SampleCalc>,
    /// Initial value of `phase`.
    phase_init: Cell<SampleCalc>,
    phase_change: Cell<SampleCalc>,
    /// The phase of the progress.
    phase: Cell<SampleCalc>,
}

impl ProgressTempo {
    /// Custom constructor. The default period is the whole duration.
    /// The default period unit is π x 2.
    pub fn new(sample_rate: SampleCalc, period: NoteValue) -> SoundResult<ProgressTempo> {
        let timer = try!(Timer::new(sample_rate));
        try!(timer.set_timing(TimingOption::Tempo(period)));
        let period_unit = PI2;
        let phase_change = timer.get_sample_time() * period.get_notes_per_beat() * period_unit;
        Ok(ProgressTempo {
            timer: timer,
            period: Cell::new(period),
            period_unit: Cell::new(period_unit),
            phase_init: Cell::new(0.0),
            phase_change: Cell::new(phase_change),
            phase: Cell::new(0.0),
        })
    }

    /// Sets a new period in tempo beats, and restarts the progress.
    pub fn set_period(&self, period: NoteValue) {
        self.period.set(period);
        self.restart();
    }
}

impl HasTimer for ProgressTempo {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        try!(self.timer.set_timing(timing));
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timer.get_timing()
    }

    fn restart(&self) {
        self.timer.restart();
        self.phase.set(self.phase_init.get());
        self.phase_change
            .set(self.timer.get_sample_time() * self.period.get().get_notes_per_beat() *
                 self.period_unit.get());
    }
}

impl Progress for ProgressTempo {
    fn set_period_unit(&self, period_unit: SampleCalc) {
        self.period_unit.set(period_unit);
        self.restart();
    }

    fn set_phase_init(&self, phase: SampleCalc) {
        self.phase_init.set(phase);
        self.restart();
    }

    fn simplify(&self) {
        self.phase.set(self.phase.get() % self.period_unit.get());
    }

    fn next_by_time(&self) -> SoundResult<SampleCalc> {
        // TODO: investigate if we need this method implementation at all
        // try!(self.timer.next_by_time());
        // self.phase.set(self.phase.get() + self.phase_change.get() * tempo);
        // Ok(self.phase.get())
        Err(Error::ProgressInvalid)
    }

    fn next_by_tempo(&self, tempo: SampleCalc) -> SoundResult<SampleCalc> {
        try!(self.timer.next_by_tempo(tempo));
        self.phase.set(self.phase.get() + self.phase_change.get() * tempo);
        Ok(self.phase.get())
    }

    // fn get_phase_final(&self) -> SoundResult<SampleCalc> {
    // Ok(self.phase_init.get() +
    // (self.duration.get().get_duration_in_beats() * self.period.get().get_notes_per_beat() *
    // self.period_unit.get()))
    // }

    fn get_phase(&self) -> SampleCalc {
        self.phase.get()
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

impl HasTimer for ProgressOption {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        match *self {
            ProgressOption::Time(ref p) => p.set_timing(timing),
            ProgressOption::Tempo(ref p) => p.set_timing(timing),
        }
    }

    fn get_timing(&self) -> TimingOption {
        match *self {
            ProgressOption::Time(ref p) => p.get_timing(),
            ProgressOption::Tempo(ref p) => p.get_timing(),
        }
    }

    fn restart(&self) {
        match *self {
            ProgressOption::Time(ref p) => p.restart(),
            ProgressOption::Tempo(ref p) => p.restart(),
        }
    }
}

impl Progress for ProgressOption {
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

    fn simplify(&self) {
        match *self {
            ProgressOption::Time(ref p) => p.simplify(),
            ProgressOption::Tempo(ref p) => p.simplify(),
        }
    }

    fn next_by_time(&self) -> SoundResult<SampleCalc> {
        match *self {
            ProgressOption::Time(ref p) => p.next_by_time(),
            ProgressOption::Tempo(ref p) => p.next_by_time(),
        }
    }

    fn next_by_tempo(&self, tempo: SampleCalc) -> SoundResult<SampleCalc> {
        match *self {
            ProgressOption::Time(ref p) => p.next_by_tempo(tempo),
            ProgressOption::Tempo(ref p) => p.next_by_tempo(tempo),
        }
    }

    // fn get_phase_final(&self) -> SoundResult<SampleCalc> {
    // match *self {
    // ProgressOption::Time(ref p) => p.get_phase_final(),
    // ProgressOption::Tempo(ref p) => p.get_phase_final(),
    // }
    // }

    fn get_phase(&self) -> SampleCalc {
        match *self {
            ProgressOption::Time(ref p) => p.get_phase(),
            ProgressOption::Tempo(ref p) => p.get_phase(),
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
