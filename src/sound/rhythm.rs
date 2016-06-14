use sound::*;
use num::num_integer::*;
use std::ops::Add;
use std::fmt;

/// The `Rhythm` trait is used to specify the functionality of rhythmic time syncronization.
pub trait Rhythm {
    /// Sets the actual tempo base.
    fn set_tempo(&self, tempo: Tempo) -> SoundResult<()>;
    /// Sets the ratio of the timing compared to the tempo beat duration.
    fn set_note_value(&self, note_value: NoteValue);
}

/// The `TempoProvider` trait is used to provide tempo.
pub trait TempoProvider {
    /// Returns the beat duration for each sample in the `result` buffer.
    fn get_beat_duration(&self, time_start: SampleCalc, result: &mut [SampleCalc]);
}

/// Constant speed of the music. See also: [Tempo](https://en.wikipedia.org/wiki/Tempo)
#[derive(Debug, Copy, Clone)]
pub struct Tempo {
    beats_per_minute: SampleCalc,
    beat_duration: SampleCalc,
}

impl Default for Tempo {
    /// The default value is 120bpm (= allegretto)
    fn default() -> Tempo {
        Tempo {
            beats_per_minute: 120.0,
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
            beats_per_minute: beats_per_minute,
            beat_duration: beat_duration,
        })
    }

    /// Sets the tempo from the beats per minute.
    pub fn set_bpm(&mut self, beats_per_minute: SampleCalc) -> SoundResult<()> {
        if beats_per_minute <= 0.0 {
            return Err(Error::TempoInvalid);
        };
        self.beats_per_minute = beats_per_minute;
        self.beat_duration = 60.0 / beats_per_minute;
        Ok(())
    }

    /// Sets the tempo from beat duration.
    pub fn set_beat_duration(&mut self, beat_duration: SampleCalc) -> SoundResult<()> {
        if beat_duration <= 0.0 {
            return Err(Error::TempoInvalid);
        };
        self.beat_duration = beat_duration;
        self.beats_per_minute = 60.0 / beat_duration;
        Ok(())
    }

    /// Returns the duration of one beat.
    pub fn get_duration(&self) -> SampleCalc {
        self.beat_duration
    }

    /// Returns the number of beats per minute.
    pub fn get_bpm(&self) -> SampleCalc {
        self.beats_per_minute
    }
}

impl TempoProvider for Tempo {
    fn get_beat_duration(&self, _time_start: SampleCalc, result: &mut [SampleCalc]) {
        for item in result {
            *item = self.beat_duration;
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
    duration_change_rate: SampleCalc,
}

impl TempoChangeLinear {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc,
               tempo_start: Tempo,
               tempo_end: Tempo,
               duration: SampleCalc)
               -> SoundResult<TempoChangeLinear> {
        let sample_time = try!(get_sample_time(sample_rate));
        let duration_change_rate = (tempo_end.beat_duration - tempo_start.beat_duration) / duration;
        Ok(TempoChangeLinear {
            sample_time: sample_time,
            tempo_start: tempo_start,
            tempo_end: tempo_end,
            duration: duration,
            duration_change_rate: duration_change_rate,
        })
    }
    /// Sets duration calculated from the given note value.
    pub fn set_note_value(&mut self, note_value: NoteValue) {
        let beat_mean = (self.tempo_start.beat_duration + self.tempo_end.beat_duration) * 0.5;
        self.duration = note_value.get_duration_in_beats() * beat_mean;
    }
}

impl TempoProvider for TempoChangeLinear {
    fn get_beat_duration(&self, time_start: SampleCalc, result: &mut [SampleCalc]) {
        for (index, item) in result.iter_mut().enumerate() {
            let time = (index as SampleCalc * self.sample_time) + time_start;
            *item = if time < self.duration {
                self.tempo_start.beat_duration + (time * self.duration_change_rate)
            } else if time < 0.0 {
                self.tempo_start.beat_duration
            } else {
                self.tempo_end.beat_duration
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
