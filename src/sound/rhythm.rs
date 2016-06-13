use sound::*;
use num::num_integer::*;
use std::ops::Add;
use std::fmt;

/// The speed of the music. See also: [Tempo](https://en.wikipedia.org/wiki/Tempo)
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


/// The duration of a note relative to the duration of a beat.
/// See also: [Note value](https://en.wikipedia.org/wiki/Note_value)
#[derive(Debug, Copy, Clone)]
pub struct NoteValue {
    numerator: u16,
    denominator: u16,
    ratio: SampleCalc,
}

impl Default for NoteValue {
    fn default() -> NoteValue {
        NoteValue {
            numerator: 1,
            denominator: 1,
            ratio: 1.0,
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
        self.ratio = numerator as SampleCalc / denominator as SampleCalc;
        // self.reciprocal = denominator as SampleCalc / numerator as SampleCalc;
        self.reduce();
        Ok(())
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
            ratio: n as SampleCalc / d as SampleCalc,
        }
    }
}

impl From<NoteValue> for SampleCalc {
    fn from(note_value: NoteValue) -> Self {
        note_value.ratio
    }
}

impl fmt::Display for NoteValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}
