use sound::*;
use std::cell::Cell;

/// It provides the timing functionality required for making sequences.
pub trait SequenceItem {
    /// It returns a reference to the `Timer`.
    fn get_timer(&self) -> &Timer;
}

/// Optional duration type, for timings in sequences.
#[derive(Debug, Copy, Clone)]
pub enum TimingOption {
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
}

/// Timer for sequence items, based on optional duration unit types.
#[derive(Debug, Clone)]
pub struct Timer {
    sample_time: SampleCalc,
    duration: Cell<TimingOption>,
    remaining: Cell<SampleCalc>,
}

impl Timer {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc) -> SoundResult<Timer> {
        let sample_time = try!(get_sample_time(sample_rate));
        Ok(Timer {
            sample_time: sample_time,
            duration: Cell::new(TimingOption::TimeConst(60.0)),
            remaining: Cell::new(60.0),
        })
    }

    /// Sets the duration, and restarts the timer.
    pub fn set_duration(&self, duration: TimingOption) -> SoundResult<()> {
        match duration {
            TimingOption::TimeConst(d) => {
                if d <= 0.0 {
                    return Err(Error::DurationInvalid);
                }
                self.remaining.set(d);
            }
            TimingOption::TimeRatio { ratio, duration } => {
                let _ = ratio;
                if duration <= 0.0 {
                    return Err(Error::DurationInvalid);
                }
                self.remaining.set(duration);
            }
            TimingOption::Tempo(note_value) => {
                self.remaining.set(note_value.get_duration_in_beats());
            }
        }
        self.duration.set(duration);
        Ok(())
    }

    /// Moves forward `sample_count` steps in time. If the elapsed time reaches the timing
    /// duration, it returns the count of samples wrapped in `Error::ItemsCompleted()`.
    pub fn step_time(&self, sample_count: usize) -> SoundResult<()> {
        let time_change = (sample_count as SampleCalc) * self.sample_time;
        if self.remaining.get() >= time_change {
            self.remaining.set(self.remaining.get() - time_change);
            return Ok(());
        }
        let samples_left = (self.remaining.get() / self.sample_time) as usize;
        self.remaining.set(0.0);
        Err(Error::ItemsCompleted(samples_left))
    }

    /// Moves forward `tempo.len()` steps. If the duration (in beats) is reached,
    /// it returns the count of samples wrapped in `Error::ItemsCompleted()`.
    /// Tempo values are given in beats per second.
    pub fn step_tempo(&self, tempo: &[SampleCalc]) -> SoundResult<()> {
        match self.duration.get() {
            TimingOption::Tempo(_) => {
                for (index, beats_per_second) in tempo.iter().enumerate() {
                    self.remaining
                        .set(self.remaining.get() - (*beats_per_second * self.sample_time));
                    if self.remaining.get() <= 0.0 {
                        self.remaining.set(0.0);
                        return Err(Error::ItemsCompleted(index));
                    }
                }
            }
            _ => {
                return Err(Error::ProgressInvalid);
            }
        }
        Ok(())
    }

    /// Restarts the timer.
    pub fn restart(&self) {
        match self.duration.get() {
            TimingOption::TimeConst(d) => {
                self.remaining.set(d);
            }
            TimingOption::TimeRatio { ratio, duration } => {
                let _ = ratio;
                self.remaining.set(duration);
            }
            TimingOption::Tempo(note_value) => {
                self.remaining.set(note_value.get_duration_in_beats());
            }
        }
    }
}
