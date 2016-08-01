use sound::*;
use std::cell::Cell;

/// It provides the timing functionality required for making sequences.
pub trait SequenceItem {
    /// Sets the timing (duration) of the sequence item.
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()>;
}

/// Optional duration type, for timings in sequences.
#[derive(Debug, Copy, Clone)]
pub enum TimingOption {
    /// Timing is turned off (= unlimited duration)
    None,
    /// Constant amount of time independent of rhythm
    TimeConst(SampleCalc),
    /// Proportion relative to a sequence's whole duration (in time).
    TimeRatio {
        /// Proportion.
        ratio: SampleCalc,
        /// The calculated duration value (from the ratio).
        duration: SampleCalc,
    },
    /// Duration relative to the beat duration.
    Tempo(NoteValue),
} // TODO: TempoRatio

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
        let sample_time = try!(get_sample_time(sample_rate));
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

    /// Sets the duration, and restarts the timer.
    pub fn set(&self, timing: TimingOption) -> SoundResult<()> {
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
            TimingOption::Tempo(note_value) => {
                self.remaining.set(note_value.get_duration_in_beats());
            }
        }
        self.timing.set(timing);
        Ok(())
    }

    /// Provides the `TimingOption`.
    pub fn get_timing(&self) -> TimingOption {
        self.timing.get()
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
            TimingOption::Tempo(_) => Err(Error::TimingInvalid),
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
            TimingOption::Tempo(_) => {
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
            TimingOption::Tempo(_) => Err(Error::TimingInvalid),
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
            TimingOption::Tempo(_) => {
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

    /// Restarts the timer.
    pub fn restart(&self) {
        match self.timing.get() {
            TimingOption::None => {}
            TimingOption::TimeConst(duration) |
            TimingOption::TimeRatio { duration, .. } => {
                self.remaining.set(duration);
            }
            TimingOption::Tempo(note_value) => {
                self.remaining.set(note_value.get_duration_in_beats());
            }
        }
    }
}
