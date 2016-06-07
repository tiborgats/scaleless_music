use sound::*;
// use std::cell::Cell;
use std::fmt;

/// Harmonic musical interval, represented by a rational number.
#[derive(Debug, Copy, Clone)]
pub struct Interval {
    numerator: u16,
    denominator: u16,
    ratio: SampleCalc,
    reciprocal: SampleCalc,
}

impl Interval {
    /// custom constructor
    pub fn new(numerator: u16, denominator: u16) -> SoundResult<Interval> {
        if numerator == 0 {
            return Err(Error::NumeratorInvalid);
        };
        if denominator == 0 {
            return Err(Error::DenominatorInvalid);
        };
        let mut i = Interval {
            numerator: numerator,
            denominator: denominator,
            ratio: (numerator as SampleCalc / denominator as SampleCalc),
            reciprocal: (denominator as SampleCalc / numerator as SampleCalc),
        };
        i.simplify();
        Ok(i)
    }
    /// Simplifies the ratio with dividing by the greatest common divisor.
    fn simplify(&mut self) {
        use num::*;
        let d = self.numerator.gcd(&self.denominator);
        self.numerator /= d;
        self.denominator /= d;
    }
    /// Returns the ratio of the frequency interval.
    pub fn get(&self) -> SampleCalc {
        self.ratio
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

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.numerator, self.denominator)
    }
}
