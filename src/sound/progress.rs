use sound::*;

/// Common methods of the Progress types.
pub trait Progress {
    /// Simplifies the phase to achieve higher accuracy. It is only used for periodic functions.
    fn simplify(&mut self);
    /// Sets a new period unit, and restarts the progress. Period unit is the amount of phase
    /// change during one period.
    fn set_period_unit(&mut self, period_unit: SampleCalc);
    /// Sets a new initial phase value, and restarts the progress.
    fn set_phase_init(&mut self, phase: SampleCalc);
    /// Restarts the progress.
    fn restart(&mut self);
}

/// Time based progress measurement. It provides the sequence of phases (for sound functions) by
/// calling `next_phase()`. The whole duration can be divided to periods, for periodic functions.
#[derive(Debug, Copy, Clone)]
pub struct ProgressTime {
    sample_time: SampleCalc,
    duration: SampleCalc,
    /// Only for periodic functions: the duration of one period.
    period: SampleCalc,
    /// The amount of phase change during one period.
    period_unit: SampleCalc,
    /// Initial value of `phase`.
    phase_init: SampleCalc,
    phase_change: SampleCalc,
    /// The remaining time.
    remaining_time: SampleCalc,
    /// The phase of the progress.
    phase: SampleCalc,
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
            duration: duration,
            period: period,
            period_unit: period_unit,
            phase_init: 0.0,
            phase_change: phase_change,
            remaining_time: duration,
            phase: 0.0,
        })
    }

    /// Provides the next phase value, or `Error::ProgressCompleted` if the progress is finished.
    pub fn next_phase(&mut self) -> SoundResult<SampleCalc> {
        self.remaining_time -= self.sample_time;
        if self.remaining_time <= 0.0 {
            return Err(Error::ProgressCompleted);
        }
        self.phase += self.phase_change;
        Ok(self.phase)
    }

    /// Sets the duration, and restarts the progress.
    pub fn set_duration(&mut self, duration: SampleCalc) -> SoundResult<()> {
        if duration <= 0.0 {
            return Err(Error::DurationInvalid);
        }
        self.duration = duration;
        self.remaining_time = self.duration;
        self.restart();
        Ok(())
    }

    /// Sets a new period, and restarts the progress.
    pub fn set_period(&mut self, period: SampleCalc) -> SoundResult<()> {
        if period <= 0.0 {
            return Err(Error::PeriodInvalid);
        }
        self.period = period;
        self.restart();
        Ok(())
    }

    /// Sets a new period calculated from the given frequency, and restarts the progress.
    pub fn set_frequency(&mut self, frequency: SampleCalc) -> SoundResult<()> {
        if frequency <= 0.0 {
            return Err(Error::FrequencyInvalid);
        }
        self.set_period(1.0 / frequency)
    }
}

impl Progress for ProgressTime {
    fn simplify(&mut self) {
        self.phase %= self.period_unit;
    }

    fn set_period_unit(&mut self, period_unit: SampleCalc) {
        self.period_unit = period_unit;
        self.restart();
    }

    fn set_phase_init(&mut self, phase: SampleCalc) {
        self.phase_init = phase;
        self.restart();
    }

    fn restart(&mut self) {
        self.phase = self.phase_init;
        self.remaining_time = self.duration;
        self.phase_change = (self.sample_time / self.period) * self.period_unit;
    }
}

/// Tempo based progress measurement. It provides the sequence of phases (for sound functions) by
/// calling `next_phase()`. The whole duration can be divided to periods, for periodic functions.
#[derive(Debug, Copy, Clone)]
pub struct ProgressTempo {
    sample_time: SampleCalc,
    /// The tempo relative duration, measured in beats.
    duration: NoteValue,
    /// The remaining beats.
    remaining_beats: SampleCalc,
    /// Only for periodic functions: the duration of one period in beats.
    period: NoteValue,
    /// The amount of phase change during one period.
    period_unit: SampleCalc,
    /// The phase of the progress.
    phase: SampleCalc,
    /// Initial value of `phase`.
    phase_init: SampleCalc,
    phase_change: SampleCalc,
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
            duration: duration,
            remaining_beats: duration.get_duration_in_beats(),
            period: period,
            period_unit: period_unit,
            phase: 0.0,
            phase_init: 0.0,
            phase_change: phase_change,
        })
    }

    /// Provides the next phase value depending on the actual tempo, or
    /// `Error::ProgressCompleted` if progress is finished.
    pub fn next_phase(&mut self, beats_per_second: SampleCalc) -> SoundResult<SampleCalc> {
        self.remaining_beats -= self.sample_time * beats_per_second;
        if self.remaining_beats <= 0.0 {
            return Err(Error::ProgressCompleted);
        }
        self.phase += self.phase_change * beats_per_second;
        Ok(self.phase)
    }

    /// Sets the duration in tempo beats, and restarts the progress.
    pub fn set_duration(&mut self, duration: NoteValue) {
        self.duration = duration;
        self.restart();
    }

    /// Sets a new period in tempo beats, and restarts the progress.
    pub fn set_period(&mut self, period: NoteValue) {
        self.period = period;
        self.restart();
    }
}

impl Progress for ProgressTempo {
    fn simplify(&mut self) {
        self.phase %= self.period_unit;
    }

    fn set_period_unit(&mut self, period_unit: SampleCalc) {
        self.period_unit = period_unit;
        self.restart();
    }

    fn set_phase_init(&mut self, phase: SampleCalc) {
        self.phase_init = phase;
        self.restart();
    }

    fn restart(&mut self) {
        self.phase = self.phase_init;
        self.remaining_beats = self.duration.get_duration_in_beats();
        self.phase_change = self.sample_time * self.period.get_notes_per_beat() * self.period_unit;
    }
}

/// Time or tempo based progress.
#[derive(Debug, Copy, Clone)]
pub enum ProgressOption {
    /// Time based progress.
    Time(ProgressTime),
    /// Rhythmic, tempo synchronized progress.
    Tempo(ProgressTempo),
}

impl Progress for ProgressOption {
    fn simplify(&mut self) {
        match *self {
            ProgressOption::Time(ref mut p) => p.simplify(),
            ProgressOption::Tempo(ref mut p) => p.simplify(),
        }
    }

    fn set_period_unit(&mut self, period_unit: SampleCalc) {
        match *self {
            ProgressOption::Time(ref mut p) => p.set_period_unit(period_unit),
            ProgressOption::Tempo(ref mut p) => p.set_period_unit(period_unit),
        }
    }

    fn set_phase_init(&mut self, phase: SampleCalc) {
        match *self {
            ProgressOption::Time(ref mut p) => p.set_phase_init(phase),
            ProgressOption::Tempo(ref mut p) => p.set_phase_init(phase),
        }
    }

    fn restart(&mut self) {
        match *self {
            ProgressOption::Time(ref mut p) => p.restart(),
            ProgressOption::Tempo(ref mut p) => p.restart(),
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
