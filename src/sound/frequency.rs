use sound::*;
use std::cell::Cell;
use std::fmt;
// use rayon::prelude::*;

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
        Ok(Interval {
            numerator: numerator,
            denominator: denominator,
            ratio: (numerator as SampleCalc / denominator as SampleCalc),
            reciprocal: (denominator as SampleCalc / numerator as SampleCalc),
        })
    }
    /// Returns the ratio of the frequency interval.
    pub fn get(&self) -> SampleCalc {
        self.ratio
    }
    /// Gives the common name of the interval (if there is any).
    pub fn get_name(&self) -> &str {
        let ratio: (u16, u16);
        if self.numerator > self.denominator {
            ratio = (self.numerator, self.denominator);
        } else {
            ratio = (self.denominator, self.numerator);
        }
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
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.numerator, self.denominator)
    }
}

/// Input and output definition for the frequency functions.
pub trait FrequencyFunction {
    /// Provides the results of the frequency calculations.
    fn get(&self, time_begin: SampleCalc, result: &mut [SampleCalc]) -> SoundResult<()>;
    //    fn set_time(&self, time: SampleCalc) -> SoundResult<()>;
}

/// Frequency is not changing by time.
#[derive(Debug, Clone)]
pub struct FrequencyConst {
    frequency: Cell<SampleCalc>,
}

impl FrequencyConst {
    /// custom constructor
    pub fn new(frequency: SampleCalc) -> SoundResult<FrequencyConst> {
        Ok(FrequencyConst { frequency: Cell::new(frequency) })
    }

    /// Change frequency in harmony with it's previous value.
    pub fn change(&self, interval: Interval) -> SoundResult<&FrequencyConst> {
        self.frequency.set(try!(interval.change_frequency(self.frequency.get())));
        Ok(self)
    }
}

impl FrequencyFunction for FrequencyConst {
    fn get(&self, _time_begin: SampleCalc, result: &mut [SampleCalc]) -> SoundResult<()> {
        for item in result.iter_mut() {
            *item = self.frequency.get();
        }
        Ok(())
    }

    //    fn set_time(&self, _time: SampleCalc) -> SoundResult<()> {
    // Ok(())
    // }
}

/// Vibrato around the base frequency (= frequency modulation).
#[allow(dead_code)]
pub struct FrequencyVibrato {
    sample_rate: SampleCalc,
    frequency_base: SampleCalc,
    /// The ratio of maximum shift away from the base frequency (0.0 - 1.0).
    frequency_deviation: SampleCalc,
    rhythm: SampleCalc,
    time: SampleCalc,
}

/// Changing frequency linearly. Linearity means constant multiplication over time slices.
#[allow(dead_code)]
pub struct FrequencyChangeLinear {
    sample_rate: SampleCalc,
    frequency_begin: SampleCalc,
    frequency_end: SampleCalc,
    timeframe: SampleCalc,
    time: SampleCalc,
}
