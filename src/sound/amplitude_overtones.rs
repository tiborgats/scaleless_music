use sound::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;

/// Provides time dependent amlitude changes both for the fundamental tone and for overtones.
pub trait AmplitudeOvertonesProvider: HasTimer {
    /// It is only for measuring time lapse. Does nothing else.
    fn next_chunk(&self, samples: usize) -> SoundResult<()>;

    // Provides the results of the amplitude calculations for a given overtone.
    // For the fundamental tone `overtone = 0`.
    // fn get(&self, overtone: usize, result: &mut [SampleCalc]) -> SoundResult<()>;

    /// Applies the amplitude function over existing samples for a given overtone.
    /// For the fundamental tone `overtone = 0`. It multiplies each sample with it's new amplitude.
    fn apply(&self, overtone: usize, samples: &mut [SampleCalc]) -> SoundResult<()>;
}

/// The `AmplitudeOvertonesJoinable` trait is used to specify the ability of joining
/// amplitude structures (with overtones) together, forming a sequence of them.
pub trait AmplitudeOvertonesJoinable: AmplitudeOvertonesProvider {
    /// Sets the initial amplitude, and resets time.
    fn set_amplitude_start(&self, amplitude: &[SampleCalc]) -> SoundResult<()>;
    /// Provides the actual amplitude values.
    fn get_amplitudes(&self, result: &mut [SampleCalc]) -> SoundResult<()>;
    // fn get_duration(&self) -> Option<SampleCalc>;

    // fn set_duration(&self, duration: SampleCalc);
}


/// Amplitude is not changing by time, this function gives the overtone amplitudes too.
#[derive(Debug, Clone)]
pub struct AmplitudeConstOvertones {
    timer: Timer,
    amplitude: RefCell<Vec<SampleCalc>>,
}

impl AmplitudeConstOvertones {
    /// custom constructor
    /// It normalizes the amplitudes, so the sum of them will be 1.0.
    /// `overtone_count` is independent of the size of `amplitude`.
    pub fn new(sample_rate: SampleCalc,
               overtone_count: usize,
               amplitude: &[SampleCalc])
               -> SoundResult<AmplitudeConstOvertones> {
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in amplitude.iter().take(overtone_count + 1) {
            if *amplitude_check < 0.0 {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum == 0.0 {
            return Err(Error::AmplitudeInvalid);
        };
        // fundamental tone is included in size
        let mut amplitude_new = vec![0.0; overtone_count + 1];
        // normalization
        for (item, amplitude_old) in amplitude_new.iter_mut().zip(amplitude) {
            *item = amplitude_old / amplitude_sum;
        }
        Ok(AmplitudeConstOvertones {
            timer: try!(Timer::new(sample_rate)),
            amplitude: RefCell::new(amplitude_new),
        })
    }
}

impl AmplitudeOvertonesProvider for AmplitudeConstOvertones {
    fn next_chunk(&self, samples: usize) -> SoundResult<()> {
        self.timer.jump_by_time(samples)
    }

    fn apply(&self, overtone: usize, samples: &mut [SampleCalc]) -> SoundResult<()> {
        let amplitude = self.amplitude.borrow();
        if overtone >= amplitude.len() {
            for item in samples.iter_mut() {
                *item = 0.0;
            }
        } else {
            for item in samples.iter_mut() {
                *item *= amplitude[overtone];
            }
        }
        Ok(())
    }
}

impl HasTimer for AmplitudeConstOvertones {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        try!(self.timer.set_timing(timing));
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timer.get_timing()
    }

    fn restart(&self) {
        self.timer.restart();
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        self.timer.apply_parent_timing(parent_timing)
    }
}

impl AmplitudeOvertonesJoinable for AmplitudeConstOvertones {
    fn set_amplitude_start(&self, amplitude: &[SampleCalc]) -> SoundResult<()> {
        let mut self_amplitude = self.amplitude.borrow_mut();
        // checking the input data
        if amplitude.len() > self_amplitude.len() {
            return Err(Error::OvertoneCountInvalid);
        }
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in amplitude {
            if (*amplitude_check < 0.0) || (*amplitude_check > 1.0) {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if (amplitude_sum == 0.0) || (amplitude_sum > 1.0) {
            return Err(Error::AmplitudeInvalid);
        };
        // Copying input amplitudes and filling the rest with zero.
        let (amp_data, amp_empty) = self_amplitude.split_at_mut(amplitude.len());
        for (item, amplitude) in amp_data.iter_mut().zip(amplitude) {
            *item = *amplitude;
        }
        for item in amp_empty.iter_mut() {
            *item = 0.0;
        }
        Ok(())
    }

    fn get_amplitudes(&self, result: &mut [SampleCalc]) -> SoundResult<()> {
        let amplitude = self.amplitude.borrow();
        // checking the input data
        if result.len() < amplitude.len() {
            return Err(Error::OvertoneCountInvalid);
        }
        // Copying amplitudes and filling the rest with zero.
        let (result_data, result_empty) = result.split_at_mut(amplitude.len());
        for (item, amplitude) in result_data.iter_mut().zip(amplitude.iter()) {
            *item = *amplitude;
        }
        for item in result_empty.iter_mut() {
            *item = 0.0;
        }
        Ok(())
    }
}

/// Amplitude is decaying exponentially, also for overtones
/// [Exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
/// index: 0 = fundamental tone, 1.. = overtones.
#[derive(Debug, Clone)]
pub struct AmplitudeDecayExpOvertones {
    timer: Timer,
    sample_time: SampleCalc,
    amplitude_init: Vec<SampleCalc>, // initial amplitudes
    multiplier: Vec<SampleCalc>,
    amplitude: RefCell<Vec<SampleCalc>>,
}

impl AmplitudeDecayExpOvertones {
    /// custom constructor
    /// It normalizes the amplitudes, so the sum of the starting amplitudes will be 1.0.
    /// `half_life` is the time required to reduce the amplitude to it's half.
    /// `overtone_count` is independent of the size of `amplitude` and `half_life` too.
    pub fn new(sample_rate: SampleCalc,
               overtone_count: usize,
               amplitude: &[SampleCalc],
               half_life: &[SampleCalc])
               -> SoundResult<AmplitudeDecayExpOvertones> {
        let sample_time = try!(get_sample_time(sample_rate));
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in amplitude.iter().take(overtone_count + 1) {
            if *amplitude_check < 0.0 {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum == 0.0 {
            return Err(Error::AmplitudeInvalid);
        };
        // fundamental tone is included in size
        let mut amplitude_new = vec![0.0; overtone_count + 1];
        // normalization
        for (item, amplitude_old) in amplitude_new.iter_mut().zip(amplitude) {
            *item = amplitude_old / amplitude_sum;
        }
        for item in half_life {
            if *item <= 0.0 {
                return Err(Error::AmplitudeRateInvalid);
            }
        }
        let mut multiplier = vec![0.0; overtone_count + 1]; // fundamental tone is included in size
        let half: SampleCalc = 0.5;
        for (item, hl) in multiplier.iter_mut().zip(half_life) {
            *item = half.powf(sample_time / hl);
        }
        Ok(AmplitudeDecayExpOvertones {
            timer: try!(Timer::new(sample_rate)),
            sample_time: sample_time,
            amplitude_init: amplitude_new.clone(),
            multiplier: multiplier,
            amplitude: RefCell::new(amplitude_new),
        })
    }
}

impl AmplitudeOvertonesProvider for AmplitudeDecayExpOvertones {
    fn next_chunk(&self, samples: usize) -> SoundResult<()> {
        self.timer.jump_by_time(samples)
    }

    fn apply(&self, overtone: usize, samples: &mut [SampleCalc]) -> SoundResult<()> {
        let mut amplitude = self.amplitude.borrow_mut();
        if (overtone >= amplitude.len()) || (overtone >= self.multiplier.len()) {
            for item in samples.iter_mut() {
                *item = 0.0;
            }
            return Ok(());
        };
        let mut amplitude_overtone = &mut amplitude[overtone];
        for item in samples.iter_mut() {
            *amplitude_overtone *= self.multiplier[overtone];
            *item *= *amplitude_overtone;
        }
        Ok(())
    }
}

impl HasTimer for AmplitudeDecayExpOvertones {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        try!(self.timer.set_timing(timing));
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timer.get_timing()
    }

    fn restart(&self) {
        self.timer.restart();
        for (amplitude, amplitude_init) in self.amplitude
            .borrow_mut()
            .iter_mut()
            .zip(self.amplitude_init.iter()) {
            *amplitude = *amplitude_init;
        }
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        self.timer.apply_parent_timing(parent_timing)
    }
}

impl AmplitudeOvertonesJoinable for AmplitudeDecayExpOvertones {
    fn set_amplitude_start(&self, amplitude: &[SampleCalc]) -> SoundResult<()> {
        let mut self_amplitude = self.amplitude.borrow_mut();
        // checking the input data
        if amplitude.len() > self_amplitude.len() {
            return Err(Error::OvertoneCountInvalid);
        }
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in amplitude {
            if (*amplitude_check < 0.0) || (*amplitude_check > 1.0) {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if (amplitude_sum == 0.0) || (amplitude_sum > 1.0) {
            return Err(Error::AmplitudeInvalid);
        };
        // Copying input amplitudes and filling the rest with zero.
        let (amp_data, amp_empty) = self_amplitude.split_at_mut(amplitude.len());
        for (item, amplitude) in amp_data.iter_mut().zip(amplitude) {
            *item = *amplitude;
        }
        for item in amp_empty.iter_mut() {
            *item = 0.0;
        }
        Ok(())
    }

    fn get_amplitudes(&self, result: &mut [SampleCalc]) -> SoundResult<()> {
        let amplitude = self.amplitude.borrow();
        // checking the input data
        if result.len() < amplitude.len() {
            return Err(Error::OvertoneCountInvalid);
        }
        // Copying amplitudes and filling the rest with zero.
        let (result_data, result_empty) = result.split_at_mut(amplitude.len());
        for (item, amplitude) in result_data.iter_mut().zip(amplitude.iter()) {
            *item = *amplitude;
        }
        for item in result_empty.iter_mut() {
            *item = 0.0;
        }
        Ok(())
    }
}

/// A sequence of amplitude functions with overtones.
#[derive(Clone)]
pub struct AmplitudeOvertonesSequence {
    timer: Timer,
    amplitudes: Vec<Rc<AmplitudeOvertonesJoinable>>,
    amplitude_index: Cell<usize>,
}

impl AmplitudeOvertonesSequence {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc) -> SoundResult<AmplitudeOvertonesSequence> {
        Ok(AmplitudeOvertonesSequence {
            timer: try!(Timer::new(sample_rate)),
            amplitudes: Vec::new(),
            amplitude_index: Cell::new(0),
        })
    }
}

impl HasTimer for AmplitudeOvertonesSequence {
    fn set_timing(&self, timing: TimingOption) -> SoundResult<()> {
        try!(self.timer.set_timing(timing));
        self.restart();
        Ok(())
    }

    fn get_timing(&self) -> TimingOption {
        self.timer.get_timing()
    }

    fn restart(&self) {
        self.timer.restart();
    }

    fn apply_parent_timing(&self, parent_timing: TimingOption) -> SoundResult<()> {
        self.timer.apply_parent_timing(parent_timing)
    }
}

impl AmplitudeOvertonesProvider for AmplitudeOvertonesSequence {
    fn next_chunk(&self, samples: usize) -> SoundResult<()> {
        if self.amplitudes.is_empty() {
            return Err(Error::SequenceEmpty);
        }
        // let amplitude_act =
        //    try!(self.amplitudes.get(self.amplitude_index.get()).ok_or(Error::ItemInvalid));
        //        let buffer_len: usize;
        // match amplitude_act.get_timer().step_time(samples) {
        // Ok(()) => {
        // buffer_len = samples;
        // }
        // Err(Error::ItemsCompleted(completed)) => {
        // buffer_len = completed;
        // }
        // Err(e) => return Err(e),
        // }
        //
        self.timer.jump_by_time(samples)
    }

    fn apply(&self, _overtone: usize, _samples: &mut [SampleCalc]) -> SoundResult<()> {
        if self.amplitudes.is_empty() {
            return Err(Error::SequenceEmpty);
        }


        // TODO
        Ok(())
    }


    // fn restart(&self) {
    // self.amplitude_index.set(0);
    // self.timer.restart();
    // }
}
