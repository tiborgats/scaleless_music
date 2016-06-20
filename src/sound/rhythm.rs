use sound::*;
use num::num_integer::*;
use std::ops::Add;
use std::fmt;

/// The `TempoProvider` trait is used to provide tempo.
pub trait TempoProvider {
    /// Returns the beat duration for each sample in the `result` buffer.
    fn get_beat_duration(&self, time_start: SampleCalc, result: &mut [SampleCalc]);
    ///
    fn get_beats_per_second(&self, time_start: SampleCalc, result: &mut [SampleCalc]);
}

/// Constant speed of the music. See also: [Tempo](https://en.wikipedia.org/wiki/Tempo)
#[derive(Debug, Copy, Clone)]
pub struct Tempo {
    /// beat frequency
    beats_per_second: SampleCalc,
    beat_duration: SampleCalc,
}

impl Default for Tempo {
    /// The default value is 120 beats per minute (= allegretto)
    fn default() -> Tempo {
        Tempo {
            beats_per_second: 2.0,
            beat_duration: 0.5,
        }
    }
}

impl Tempo {
    /// custom constructor
    pub fn new(beats_per_minute: SampleCalc) -> SoundResult<Tempo> {
        if beats_per_minute <= 0.0 {
            return Err(Error::TempoInvalid);
        };
        let beat_duration = 60.0 / beats_per_minute;
        Ok(Tempo {
            beats_per_second: beats_per_minute / 60.0,
            beat_duration: beat_duration,
        })
    }

    /// Sets the tempo from the beats per minute.
    pub fn set_bpm(&mut self, beats_per_minute: SampleCalc) -> SoundResult<()> {
        if beats_per_minute <= 0.0 {
            return Err(Error::TempoInvalid);
        };
        self.beats_per_second = beats_per_minute / 60.0;
        self.beat_duration = 60.0 / beats_per_minute;
        Ok(())
    }

    /// Sets the tempo from beat duration.
    pub fn set_beat_duration(&mut self, beat_duration: SampleCalc) -> SoundResult<()> {
        if beat_duration <= 0.0 {
            return Err(Error::TempoInvalid);
        };
        self.beat_duration = beat_duration;
        self.beats_per_second = 1.0 / beat_duration;
        Ok(())
    }

    /// Returns the duration of one beat.
    pub fn get_duration(&self) -> SampleCalc {
        self.beat_duration
    }

    /// Returns the number of beats per minute.
    pub fn get_bpm(&self) -> SampleCalc {
        self.beats_per_second * 60.0
    }
}

impl TempoProvider for Tempo {
    fn get_beat_duration(&self, _time_start: SampleCalc, result: &mut [SampleCalc]) {
        for item in result {
            *item = self.beat_duration;
        }
    }

    fn get_beats_per_second(&self, _time_start: SampleCalc, result: &mut [SampleCalc]) {
        for item in result {
            *item = self.beats_per_second;
        }
    }
}

/// Linearly changing speed of the music.
#[derive(Debug, Copy, Clone)]
pub struct TempoChangeLinear {
    sample_time: SampleCalc,
    tempo_start: Tempo,
    tempo_end: Tempo,
    duration: SampleCalc,
    /// positive for slowing down tempo, negative for speeding up
    beat_duration_change_rate: SampleCalc,
    /// negative for slowing down tempo, positive for speeding up
    bps_change_rate: SampleCalc,
}
// TODO: build pattern for the possibility to use different input variable combinations
impl TempoChangeLinear {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc,
               tempo_start: Tempo,
               tempo_end: Tempo,
               duration: SampleCalc)
               -> SoundResult<TempoChangeLinear> {
        let sample_time = try!(get_sample_time(sample_rate));
        let beat_duration_change_rate = (tempo_end.beat_duration - tempo_start.beat_duration) /
                                        duration;
        let bps_change_rate = -1.0 / beat_duration_change_rate;
        Ok(TempoChangeLinear {
            sample_time: sample_time,
            tempo_start: tempo_start,
            tempo_end: tempo_end,
            duration: duration,
            beat_duration_change_rate: beat_duration_change_rate,
            bps_change_rate: bps_change_rate,
        })
    }
    /// Sets duration calculated from the given note value.
    pub fn set_note_value(&mut self, note_value: NoteValue) {
        let beat_mean = (self.tempo_start.beat_duration + self.tempo_end.beat_duration) * 0.5;
        self.duration = note_value.get_duration_in_beats() * beat_mean;
        self.beat_duration_change_rate =
            (self.tempo_end.beat_duration - self.tempo_start.beat_duration) / self.duration;
        self.bps_change_rate = -1.0 / self.beat_duration_change_rate;
    }
}

impl TempoProvider for TempoChangeLinear {
    fn get_beat_duration(&self, time_start: SampleCalc, result: &mut [SampleCalc]) {
        for (index, item) in result.iter_mut().enumerate() {
            let time = (index as SampleCalc * self.sample_time) + time_start;
            *item = if time < self.duration {
                self.tempo_start.beat_duration + (time * self.beat_duration_change_rate)
            } else if time < 0.0 {
                self.tempo_start.beat_duration
            } else {
                self.tempo_end.beat_duration
            }
        }
    }

    fn get_beats_per_second(&self, time_start: SampleCalc, result: &mut [SampleCalc]) {
        for (index, item) in result.iter_mut().enumerate() {
            let time = (index as SampleCalc * self.sample_time) + time_start;
            *item = if time < self.duration {
                self.tempo_start.beats_per_second + (time * self.bps_change_rate)
            } else if time < 0.0 {
                self.tempo_start.beats_per_second
            } else {
                self.tempo_end.beats_per_second
            }
        }
    }
}

/// The duration of a note relative to the duration of a beat.
/// See also: [Note value](https://en.wikipedia.org/wiki/Note_value)
#[derive(Debug, Copy, Clone)]
pub struct NoteValue {
    numerator: u16,
    denominator: u16,
    duration_in_beats: SampleCalc,
    notes_per_beat: SampleCalc,
}

impl Default for NoteValue {
    fn default() -> NoteValue {
        NoteValue {
            numerator: 1,
            denominator: 1,
            duration_in_beats: 1.0,
            notes_per_beat: 1.0,
        }
    }
}

impl NoteValue {
    /// custom constructor
    pub fn new(numerator: u16, denominator: u16) -> SoundResult<NoteValue> {
        let mut note_value = NoteValue::default();
        try!(note_value.set(numerator, denominator));
        Ok(note_value)
    }

    /// Reduces to lowest terms with dividing by the greatest common divisor.
    fn reduce(&mut self) {
        let d = self.numerator.gcd(&self.denominator);
        self.numerator /= d;
        self.denominator /= d;
    }

    /// Changes the note value.
    pub fn set(&mut self, numerator: u16, denominator: u16) -> SoundResult<()> {
        if numerator == 0 {
            return Err(Error::NumeratorInvalid);
        };
        if denominator == 0 {
            return Err(Error::DenominatorInvalid);
        };
        self.numerator = numerator;
        self.denominator = denominator;
        self.duration_in_beats = numerator as SampleCalc / denominator as SampleCalc;
        self.notes_per_beat = denominator as SampleCalc / numerator as SampleCalc;
        self.reduce();
        Ok(())
    }

    /// Provides the number of notes per beat.
    pub fn get_notes_per_beat(&self) -> SampleCalc {
        self.notes_per_beat
    }

    /// Provides the duration measured in beats.
    pub fn get_duration_in_beats(&self) -> SampleCalc {
        self.duration_in_beats
    }
}

impl Add for NoteValue {
    type Output = NoteValue;

    fn add(self, rhs: NoteValue) -> NoteValue {
        let d = self.denominator.lcm(&rhs.denominator);
        let mut n = self.numerator * (d / self.denominator);
        n += rhs.numerator * (d / rhs.denominator);
        NoteValue {
            numerator: n,
            denominator: d,
            duration_in_beats: n as SampleCalc / d as SampleCalc,
            notes_per_beat: d as SampleCalc / n as SampleCalc,
        }
    }
}

impl From<NoteValue> for SampleCalc {
    fn from(note_value: NoteValue) -> Self {
        note_value.duration_in_beats
    }
}

impl fmt::Display for NoteValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

/// Common methods of the Progress types.
pub trait Progress {
    /// Simplifies the phase to achieve higher accuracy. It is only used for periodic functions.
    fn simplify(&mut self);
    /// Sets a new period unit.
    fn set_period_unit(&mut self, period_unit: SampleCalc);
    /// Sets a new phase value.
    fn set_phase(&mut self, phase: SampleCalc);
}

/// Time based progress calculations.
#[derive(Debug, Copy, Clone)]
pub struct ProgressTime {
    sample_time: SampleCalc,
    duration: SampleCalc,
    /// The remaining beats.
    remaining: SampleCalc, // elapsed?
    /// Only for periodic functions: the duration of one period in beats.
    period: SampleCalc,
    /// The amount of phase change during one period.
    period_unit: SampleCalc,
    /// The phase of the progress.
    phase: SampleCalc,
    phase_change: SampleCalc,
}

impl ProgressTime {
    /// Custom constructor with duration.
    pub fn new(sample_rate: SampleCalc, duration: SampleCalc) -> SoundResult<ProgressTime> {
        let sample_time = try!(get_sample_time(sample_rate));
        if duration <= 0.0 {
            return Err(Error::PeriodInvalid);
        }
        let period = duration;
        let period_unit = PI2;
        let phase_change = (sample_time / duration) * period_unit;
        Ok(ProgressTime {
            sample_time: sample_time,
            duration: duration,
            remaining: duration,
            period: period,
            period_unit: period_unit,
            phase: 0.0,
            phase_change: phase_change,
        })
    }

    /// Custom constructor with frequency.
    pub fn new_f(sample_rate: SampleCalc, frequency: SampleCalc) -> SoundResult<ProgressTime> {
        let sample_time = try!(get_sample_time(sample_rate));
        if frequency <= 0.0 {
            return Err(Error::FrequencyInvalid);
        }
        let duration = 1.0 / frequency;
        let period = duration;
        let period_unit = PI2;
        let phase_change = sample_time * frequency * period_unit;
        Ok(ProgressTime {
            sample_time: sample_time,
            duration: duration,
            remaining: duration,
            period: period,
            period_unit: period_unit,
            phase: 0.0,
            phase_change: phase_change,
        })
    }

    /// Provides the next phase value depending on the actual tempo.
    pub fn next_phase(&mut self) -> SampleCalc {
        self.phase += self.phase_change;
        self.phase
    }

    /// Sets the ratio of the timing compared to the tempo beat duration.
    pub fn set_duration(&mut self, duration: SampleCalc) -> SoundResult<()> {
        if duration <= 0.0 {
            return Err(Error::DurationInvalid);
        }
        self.duration = duration;
        self.phase_change = (self.sample_time / self.duration) * self.period_unit;
        Ok(())
    }

    /// Sets the ratio of the timing compared to the tempo beat frequency.
    pub fn set_frequency(&mut self, frequency: SampleCalc) -> SoundResult<()> {
        if frequency <= 0.0 {
            return Err(Error::FrequencyInvalid);
        }
        self.duration = 1.0 / frequency;
        self.phase_change = (self.sample_time / self.duration) * self.period_unit;
        Ok(())
    }

    /// Sets a new period.
    pub fn set_period(&mut self, period: SampleCalc) {
        self.period = period;
        self.phase_change = (self.sample_time / self.duration) * self.period_unit;
    }
}

impl Progress for ProgressTime {
    fn simplify(&mut self) {
        self.phase %= self.period_unit;
    }

    fn set_period_unit(&mut self, period_unit: SampleCalc) {
        self.period_unit = period_unit;
        self.phase_change = (self.sample_time / self.duration) * self.period_unit;
    }

    fn set_phase(&mut self, phase: SampleCalc) {
        self.phase = phase;
    }
}


/// Tempo based progress calculations.
#[derive(Debug, Copy, Clone)]
pub struct ProgressTempo {
    sample_time: SampleCalc,
    /// The tempo relative duration, measured in beats.
    duration: NoteValue,
    /// The remaining beats.
    remaining: SampleCalc, // elapsed?
    /// Only for periodic functions: the duration of one period in beats.
    period: NoteValue,
    /// The amount of phase change during one period.
    period_unit: SampleCalc,
    /// The phase of the progress.
    phase: SampleCalc,
    phase_change: SampleCalc,
}

impl ProgressTempo {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc, duration: NoteValue) -> SoundResult<ProgressTempo> {
        let sample_time = try!(get_sample_time(sample_rate));
        let period = duration;
        let period_unit = PI2;
        let phase_change = sample_time * period.get_notes_per_beat() * period_unit;
        Ok(ProgressTempo {
            sample_time: sample_time,
            duration: duration,
            remaining: duration.get_duration_in_beats(),
            period: period,
            period_unit: period_unit,
            phase: 0.0,
            phase_change: phase_change,
        })
    }

    /// Provides the next phase value depending on the actual tempo, or `ProgressCompleted` if
    /// progress is finished.
    pub fn next_phase(&mut self, beats_per_second: SampleCalc) -> SoundResult<SampleCalc> {
        self.remaining -= self.sample_time * beats_per_second;
        if self.remaining <= 0.0 {
            return Err(Error::ProgressCompleted);
        }
        self.phase += self.phase_change * beats_per_second;
        Ok(self.phase)
    }

    /// Sets the ratio of the timing compared to the tempo beat duration.
    pub fn set_duration(&mut self, duration: NoteValue) {
        self.duration = duration;
        // TODO: remaining
    }

    /// Sets a new period.
    pub fn set_period(&mut self, period: NoteValue) {
        self.period = period;
        self.phase_change = self.sample_time * self.period.get_notes_per_beat() * self.period_unit;
    }
}

impl Progress for ProgressTempo {
    fn simplify(&mut self) {
        self.phase %= self.period_unit;
    }

    fn set_period_unit(&mut self, period_unit: SampleCalc) {
        self.period_unit = period_unit;
        self.phase_change = self.sample_time * self.period.get_notes_per_beat() * period_unit;
    }

    fn set_phase(&mut self, phase: SampleCalc) {
        self.phase = phase;
    }
}

/// Time or tempo based progress.
#[derive(Debug, Copy, Clone)]
pub enum ProgressOption {
    /// Time based progress.
    Time(ProgressTime),
    /// Rhythmic, tempo syncronized progress.
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

    fn set_phase(&mut self, phase: SampleCalc) {
        match *self {
            ProgressOption::Time(ref mut p) => p.set_phase(phase),
            ProgressOption::Tempo(ref mut p) => p.set_phase(phase),
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
