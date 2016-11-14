use num::*;
use sound::*;
use std::cell::Cell;

/// It provides the timing functionality required for making sequences.
pub trait HasTimer {
    /// Sets the timing (duration) of the sequence item, and restarts the internal timer.
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()>;

    /// Provides the `TimingOption`.
    fn get_timing(&self) -> TimingOption;

    /// Restarts the internal timer.
    fn restart(&self);

    /// Applies the parent's timing to calculate it's own relative timing.
    /// It is used for sequence items.
    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()>;
}

/// Optional duration type, for timings in sequences.
#[derive(Debug, Copy, Clone)]
pub enum TimingOption {
    /// Timing is turned off (= unlimited duration)
    None,

    /// Constant amount of time independent of rhythm.
    /// For a sequence item, it is independent of the sequence's whole duration.
    TimeConst(SampleCalc),

    /// Proportion relative to a sequence's whole duration (in time).
    /// The sequence's timing must be time based!
    TimeRatio {
        /// Proportion.
        ratio: SampleCalc,
        /// The calculated duration value (from the ratio).
        duration: SampleCalc,
    },

    /// Duration relative to the beat duration.
    /// For a sequence item, it is independent of the sequence's whole duration.
    TempoConst(NoteValue),

    /// Proportion relative to a sequence's whole duration.
    /// The sequence's timing must be tempo based!
    TempoRatio {
        /// Proportion.
        ratio: NoteValue,
        /// The calculated duration in beats.
        duration: NoteValue,
    },
}

/// Timer for sequence items, based on optional duration unit types.
#[derive(Debug, Clone)]
pub struct Timer {
    sample_time: SampleCalc,
    timing: Cell<TimingOption>,
    remaining: Cell<SampleCalc>,
}

impl Timer {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc) -> SoundResult<Timer> {
        let sample_time = get_sample_time(sample_rate)?;
        Ok(Timer {
            sample_time: sample_time,
            timing: Cell::new(TimingOption::None),
            remaining: Cell::new(0.0),
        })
    }

    /// Provides the sample time value.
    pub fn get_sample_time(&self) -> SampleCalc {
        self.sample_time
    }

    /// Moves forward `sample_count` steps in time. If the elapsed time reaches the timing
    /// duration, it returns the count of samples wrapped in `Error::ItemsCompleted()`.
    pub fn jump_by_time(&self, sample_count: usize) -> SoundResult<()> {
        match self.timing.get() {
            TimingOption::None => Ok(()),
            TimingOption::TimeConst(_) |
            TimingOption::TimeRatio { .. } => {
                let time_change = (sample_count as SampleCalc) * self.sample_time;
                if self.remaining.get() >= time_change {
                    self.remaining.set(self.remaining.get() - time_change);
                    return Ok(());
                }
                let samples_left = (self.remaining.get() / self.sample_time) as usize;
                self.remaining.set(0.0);
                Err(Error::ItemsCompleted(samples_left))
            }
            TimingOption::TempoConst(_) |
            TimingOption::TempoRatio { .. } => Err(Error::TimingInvalid),
        }
    }

    /// Moves forward `tempo.len()` steps. If the duration (in beats) is reached,
    /// it returns the count of samples wrapped in `Error::ItemsCompleted()`.
    /// Tempo values are given in beats per second.
    pub fn jump_by_tempo(&self, tempo: &[SampleCalc]) -> SoundResult<()> {
        match self.timing.get() {
            TimingOption::None => Ok(()),
            TimingOption::TimeConst(_) |
            TimingOption::TimeRatio { .. } => Err(Error::TimingInvalid),
            TimingOption::TempoConst(_) |
            TimingOption::TempoRatio { .. } => {
                for (index, beats_per_second) in tempo.iter().enumerate() {
                    self.remaining
                        .set(self.remaining.get() - (*beats_per_second * self.sample_time));
                    if self.remaining.get() <= 0.0 {
                        self.remaining.set(0.0);
                        return Err(Error::ItemsCompleted(index));
                    }
                }
                Ok(())
            }
        }
    }

    /// Moves forward one sample step in time. If the elapsed time reaches the timing
    /// duration, it returns `Error::ProgressCompleted`.
    pub fn next_by_time(&self) -> SoundResult<()> {
        match self.timing.get() {
            TimingOption::None => Ok(()),
            TimingOption::TimeConst(_) |
            TimingOption::TimeRatio { .. } => {
                if self.remaining.get() >= self.sample_time {
                    self.remaining.set(self.remaining.get() - self.sample_time);
                    return Ok(());
                }
                self.remaining.set(0.0);
                Err(Error::ProgressCompleted)
            }
            TimingOption::TempoConst(_) |
            TimingOption::TempoRatio { .. } => Err(Error::TimingInvalid),
        }
    }

    /// Moves forward one step. If the duration (in beats) is reached,
    /// it returns `Error::ProgressCompleted`.
    /// Tempo value is given in beats per second.
    pub fn next_by_tempo(&self, tempo: SampleCalc) -> SoundResult<()> {
        match self.timing.get() {
            TimingOption::None => Ok(()),
            TimingOption::TimeConst(_) |
            TimingOption::TimeRatio { .. } => Err(Error::TimingInvalid),
            TimingOption::TempoConst(_) |
            TimingOption::TempoRatio { .. } => {
                self.remaining
                    .set(self.remaining.get() - (tempo * self.sample_time));
                if self.remaining.get() <= 0.0 {
                    self.remaining.set(0.0);
                    return Err(Error::ProgressCompleted);
                }
                Ok(())
            }
        }
    }
}

impl HasTimer for Timer {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        match timing {
            TimingOption::None => {
                self.remaining.set(0.0);
            }
            TimingOption::TimeConst(duration) |
            TimingOption::TimeRatio { duration, .. } => {
                if duration <= 0.0 {
                    return Err(Error::DurationInvalid);
                }
                self.remaining.set(duration);
            }
            TimingOption::TempoConst(note_value) => {
                self.remaining.set(note_value.get_duration_in_beats());
            }
            TimingOption::TempoRatio { duration, .. } => {
                self.remaining.set(duration.get_duration_in_beats());
            }
        }
        self.timing.set(timing);
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timing.get()
    }

    fn restart(&self) {
        match self.timing.get() {
            TimingOption::None => {}
            TimingOption::TimeConst(duration) |
            TimingOption::TimeRatio { duration, .. } => {
                self.remaining.set(duration);
            }
            TimingOption::TempoConst(note_value) => {
                self.remaining.set(note_value.get_duration_in_beats());
            }
            TimingOption::TempoRatio { duration, .. } => {
                self.remaining.set(duration.get_duration_in_beats());
            }
        }
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        match self.timing.get() {
            TimingOption::None |
            TimingOption::TimeConst(..) |
            TimingOption::TempoConst(..) => {}
            TimingOption::TimeRatio { ratio, duration } => {
                let parent_duration = match parent_timing {
                    TimingOption::None |
                    TimingOption::TempoConst(_) |
                    TimingOption::TempoRatio { .. } => return Err(Error::TimingInvalid),
                    TimingOption::TimeConst(duration) |
                    TimingOption::TimeRatio { duration, .. } => duration,
                };
                self.timing.set(TimingOption::TimeRatio {
                    ratio: ratio,
                    duration: duration * parent_duration,
                });
            }
            TimingOption::TempoRatio { ratio, duration } => {
                let parent_duration = match parent_timing {
                    TimingOption::None |
                    TimingOption::TimeConst(_) |
                    TimingOption::TimeRatio { .. } => return Err(Error::TimingInvalid),
                    TimingOption::TempoConst(duration) |
                    TimingOption::TempoRatio { duration, .. } => duration,
                };
                let new_duration = duration.checked_mul(&parent_duration)
                    .ok_or(Error::Overflow)?;
                self.timing.set(TimingOption::TempoRatio {
                    ratio: ratio,
                    duration: new_duration,
                });
            }
        }
        Ok(())
    }
}
