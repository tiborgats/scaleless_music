use sound::*;
use num::*;
use std::fmt;
use std::ops::{Div, Mul};

/// unison (1:1)
pub const INTERVAL_UNISON: Interval = Interval {
    numerator: 1,
    denominator: 1,
    ratio: 1.0,
    reciprocal: 1.0,
};

/// Harmonic musical interval (of frequencies), represented by a rational number.
#[derive(Debug, Copy, Clone)]
pub struct Interval {
    numerator: u16,
    denominator: u16,
    ratio: SampleCalc,
    reciprocal: SampleCalc,
}

impl Default for Interval {
    fn default() -> Interval {
        Interval {
            numerator: 1,
            denominator: 1,
            ratio: 1.0,
            reciprocal: 1.0,
        }
    }
}

impl Interval {
    /// custom constructor
    pub fn new(numerator: u16, denominator: u16) -> SoundResult<Interval> {
        let mut interval = Interval::default();
        try!(interval.set(numerator, denominator));
        Ok(interval)
    }

    /// Reduces to lowest terms with dividing by the greatest common divisor.
    fn reduce(&mut self) {
        let d = self.numerator.gcd(&self.denominator);
        self.numerator /= d;
        self.denominator /= d;
    }

    /// Changes the interval.
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
        self.reciprocal = denominator as SampleCalc / numerator as SampleCalc;
        self.reduce();
        Ok(())
    }

    /// Returns the ratio of the frequency interval.
    pub fn get_ratio(&self) -> SampleCalc {
        self.ratio
    }

    /// Returns the reciprocal of the frequency interval.
    pub fn get_recip(&self) -> SampleCalc {
        self.reciprocal
    }

    /// True, if the interval is `1:1`, aka. unison.
    pub fn is_unison(&self) -> bool {
        self.numerator == self.denominator
    }

    /// Gives the common name of the interval (if there is any).
    pub fn get_name(&self) -> &str {
        let ratio = if self.numerator > self.denominator {
            (self.numerator, self.denominator)
        } else {
            (self.denominator, self.numerator)
        };
        // https://en.wikipedia.org/wiki/List_of_pitch_intervals
        // https://gist.github.com/endolith/3098720
        match ratio {
            (1, 1) => "unison",
            (2, 1) => "octave",
            (3, 2) => "perfect fifth",
            (4, 3) => "perfect fourth",
            (5, 4) => "major third",
            (5, 3) => "major sixth",
            (6, 5) => "minor third",
            (7, 6) => "septimal minor third",
            (7, 5) => "lesser septimal tritone",
            (7, 4) => "augmented sixth", // "harmonic seventh", "septimal minor seventh" too
            (8, 7) => "septimal major second",
            (8, 5) => "minor sixth",
            (9, 8) => "major second", // "major tone" too
            (9, 7) => "septimal major third",
            (9, 5) => "minor seventh",

            (10, 9) => "minor tone",
            // (10, 8) => "",
            (10, 7) => "greater septimal tritone",
            // (11, 6) => "major seventh",
            (11, 8) => "lesser undecimal tritone",
            // (12, 11) => "minor second",
            (13, 8) => "acute minor sixth",
            (15, 8) => "major seventh",
            (16, 15) => "semitone", // "minor second" too
            (16, 9) => "grave minor seventh",
            // (29, 16) => "minor seventh", // "twenty-ninth harmonic"
            (31, 16) => "augmented seventh",
            (45, 32) => "augmented fourth",
            (64, 45) => "diminished fifth",
            _ => "",
        }
    }

    /// Change a frequency according to the interval.
    pub fn change_frequency(&self, frequency: SampleCalc) -> SoundResult<SampleCalc> {
        let new_frequency = frequency * self.ratio;
        if new_frequency < TONE_FREQUENCY_MIN {
            return Err(Error::FrequencyTooLow);
        };
        if new_frequency > TONE_FREQUENCY_MAX {
            return Err(Error::FrequencyTooHigh);
        };
        Ok(new_frequency)
    }

    /// Change a frequency according to the interval's reciprocal.
    pub fn reverse_frequency(&self, frequency: SampleCalc) -> SoundResult<SampleCalc> {
        let new_frequency = frequency * self.reciprocal;
        if new_frequency < TONE_FREQUENCY_MIN {
            return Err(Error::FrequencyTooLow);
        };
        if new_frequency > TONE_FREQUENCY_MAX {
            return Err(Error::FrequencyTooHigh);
        };
        Ok(new_frequency)
    }

    /// Change a frequency according to the interval.
    pub fn transpose(&self,
                     base_frequency: &[SampleCalc],
                     result: &mut [SampleCalc])
                     -> SoundResult<()> {
        if base_frequency.len() != result.len() {
            return Err(Error::BufferSize);
        }
        for (new_frequency, frequency) in result.iter_mut().zip(base_frequency) {
            *new_frequency = *frequency * self.ratio;
            if *new_frequency < TONE_FREQUENCY_MIN {
                return Err(Error::FrequencyTooLow);
            };
            if *new_frequency > TONE_FREQUENCY_MAX {
                return Err(Error::FrequencyTooHigh);
            };
        }
        Ok(())
    }
}

impl Mul for Interval {
    type Output = Interval;

    fn mul(self, rhs: Interval) -> Interval {
        let mut interval = Interval::default();
        interval.numerator = self.numerator * rhs.numerator;
        interval.denominator = self.denominator * rhs.denominator;
        interval.reduce();
        interval.ratio = interval.numerator as SampleCalc / interval.denominator as SampleCalc;
        interval.reciprocal = interval.denominator as SampleCalc / interval.numerator as SampleCalc;
        interval
    }
}

impl Div for Interval {
    type Output = Interval;

    fn div(self, rhs: Interval) -> Interval {
        let mut interval = Interval::default();
        interval.numerator = self.numerator * rhs.denominator;
        interval.denominator = self.denominator * rhs.numerator;
        interval.reduce();
        interval.ratio = interval.numerator as SampleCalc / interval.denominator as SampleCalc;
        interval.reciprocal = interval.denominator as SampleCalc / interval.numerator as SampleCalc;
        interval
    }
}

impl From<Interval> for SampleCalc {
    fn from(interval: Interval) -> Self {
        interval.ratio
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.numerator, self.denominator)
    }
}
