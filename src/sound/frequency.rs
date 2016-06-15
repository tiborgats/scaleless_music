use sound::*;
use std::cell::Cell;
// use std::fmt;
// use rayon::prelude::*;

/// Input and output definition for the frequency functions.
pub trait FrequencyFunction {
    /// Provides the results of the frequency calculations.
    fn get(&self,
           time_start: SampleCalc,
           base_frequency: Option<&[SampleCalc]>,
           result: &mut [SampleCalc])
           -> SoundResult<()>;
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
    fn get(&self,
           _time_begin: SampleCalc,
           base_frequency: Option<&[SampleCalc]>,
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if base_frequency.is_some() {
            return Err(Error::FrequencySource);
        }
        for item in result.iter_mut() {
            *item = self.frequency.get();
        }
        Ok(())
    }
}

/// Changing frequency linearly. Linearity means constant multiplication over time slices.
#[allow(dead_code)]
pub struct FrequencyChangeLinear {
    sample_time: SampleCalc,
    frequency_begin: SampleCalc,
    frequency_end: SampleCalc,
    timeframe: SampleCalc,
    time: SampleCalc,
}

/// Provides rhythmic frequency changes. As phase depends on the integral of tempo, only
/// sequential reading is possible (cannot be parallelized).
pub trait FrequencyModulator {
    /// Provides the results of the modulation of an array of frequencies.
    /// Tempo is given in beats per second.
    fn get(&mut self,
           tempo: &[SampleCalc],
           base_frequency: &[SampleCalc],
           result: &mut [SampleCalc])
           -> SoundResult<()>;
    /// Applies the modulation on an already existing array of frequencies. It multiplies
    /// each sample with it's new amplitude. Tempo is given in beats per second.
    fn apply(&mut self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// Vibrato: sinusoidal modulation of the base frequency.
#[derive(Debug, Copy, Clone)]
pub struct Vibrato {
    sample_time: SampleCalc,
    /// The (tempo relative) speed with which the amplitude is varied.
    note_value: NoteValue,
    /// The ratio of maximum shift away from the base frequency (must be > 0.0).
    extent_ratio: SampleCalc,
    /// The phase of the sine function.
    phase: SampleCalc,
}

impl Vibrato {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc,
               note_value: NoteValue,
               extent_ratio: SampleCalc)
               -> SoundResult<Vibrato> {
        let sample_time = try!(get_sample_time(sample_rate));
        if extent_ratio <= 0.0 {
            return Err(Error::FrequencyTooLow);
        }
        Ok(Vibrato {
            sample_time: sample_time,
            note_value: note_value,
            extent_ratio: extent_ratio,
            phase: 0.0,
        })
    }

    /// Sets a new phase value.
    pub fn set_phase(&mut self, phase: SampleCalc) -> SoundResult<()> {
        self.phase = phase % PI2;
        Ok(())
    }
}

impl FrequencyModulator for Vibrato {
    fn get(&mut self,
           tempo: &[SampleCalc],
           base_frequency: &[SampleCalc],
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if base_frequency.len() != result.len() {
            return Err(Error::BufferSize);
        }

        let phase_change = self.sample_time * self.note_value.get_notes_per_beat() * PI2;
        for ((item, frequency), beats_per_second) in result.iter_mut()
            .zip(base_frequency)
            .zip(tempo) {
            self.phase += phase_change * beats_per_second;
            *item = *frequency * (self.extent_ratio.powf(self.phase.sin()));
        }
        self.phase %= PI2;
        Ok(())
    }

    fn apply(&mut self, tempo: &[SampleCalc], samples: &mut [SampleCalc]) -> SoundResult<()> {
        let phase_change = self.sample_time * self.note_value.get_notes_per_beat() * PI2;
        for (item, beats_per_second) in samples.iter_mut()
            .zip(tempo) {
            self.phase += phase_change * beats_per_second;
            *item *= self.extent_ratio.powf(self.phase.sin());
        }
        self.phase %= PI2;
        Ok(())
    }
}
