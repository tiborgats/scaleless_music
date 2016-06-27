use sound::*;
use std::rc::Rc;
use std::cell::RefCell;
// use rayon::prelude::*;

/// A sinusoidal wave generator, with variable frequency.
#[derive(Debug, Copy, Clone)]
pub struct Wave {
    sample_time: SampleCalc,
    /// The interval is used for transposition of the input frequencies
    interval: Interval,
    overtone: SampleCalc,
    frequency_multiplier: SampleCalc,
    /// The phase value is always kept close to zero for maximizing the floating point precision.
    phase: SampleCalc,
}

impl Wave {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc, overtone: usize) -> SoundResult<Wave> {
        let sample_time = try!(get_sample_time(sample_rate));
        Ok(Wave {
            sample_time: sample_time,
            interval: INTERVAL_UNISON,
            overtone: overtone as SampleCalc,
            frequency_multiplier: (overtone as SampleCalc + 1.0) * PI2 * sample_time,
            phase: 0.0,
        })
    }

    /// Gets the next samples of the wave.
    pub fn get(&mut self,
               base_frequency: &[SampleCalc],
               result: &mut [SampleCalc])
               -> SoundResult<()> {
        for (item, frequency) in result.iter_mut().zip(base_frequency) {
            self.phase += frequency * self.frequency_multiplier;
            *item = (self.phase).sin();
        }
        self.phase %= PI2;
        Ok(())
    }

    /// Sets a new frequency interval.
    pub fn set_interval(&mut self, interval: Interval) {
        self.interval = interval;
        self.frequency_multiplier = (self.overtone + 1.0) * PI2 * self.sample_time *
                                    interval.get_ratio();
    }

    /// Sets a new phase value.
    pub fn set_phase(&mut self, phase: SampleCalc) {
        self.phase = phase % PI2;
    }
}

/// A tone with optional overtones and amplitude modulation.
/// Some examples: https://youtu.be/VRAXK4QKJ1Q?t=25s
#[derive(Clone)]
pub struct Timbre {
    // sample_time: SampleCalc,
    /// The interval is used for transposition of the input frequencies
    interval: Interval,
    waves: RefCell<Vec<Wave>>,
    amplitude_overtones: Rc<AmplitudeOvertonesProvider>,
    wave_buffer: RefCell<Vec<SampleCalc>>,
    overtone_max: usize,
}

impl Timbre {
    /// Custom constructor
    pub fn new(sample_rate: SampleCalc,
               buffer_size: usize,
               amplitude_overtones: Rc<AmplitudeOvertonesProvider>,
               overtone_max: usize)
               -> SoundResult<Timbre> {
        let mut wave_vec = Vec::with_capacity(overtone_max + 1);
        for overtone in 0..overtone_max {
            wave_vec.push(try!(Wave::new(sample_rate, overtone)));
        }
        Ok(Timbre {
            interval: INTERVAL_UNISON,
            waves: RefCell::new(wave_vec),
            amplitude_overtones: amplitude_overtones,
            wave_buffer: RefCell::new(vec![0.0; buffer_size]),
            overtone_max: overtone_max,
        })
    }

    /// Sets a new frequency interval.
    pub fn set_interval(&mut self, interval: Interval) {
        self.interval = interval;
        for wave in self.waves.borrow_mut().iter_mut() {
            wave.set_interval(interval);
        }
    }

    /// Set a new amplitude function
    pub fn set_amplitude(&mut self,
                         amplitude_overtones: Rc<AmplitudeOvertonesProvider>)
                         -> &mut Timbre {
        self.amplitude_overtones = amplitude_overtones;
        self
    }
}

impl SoundStructure for Timbre {
    // TODO: filtering out frequencies from the calculations which are out of range
    fn get(&self,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc],
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        let mut wave_buffer = self.wave_buffer.borrow_mut();
        let buffer_size = wave_buffer.len();
        if base_frequency.len() != buffer_size {
            return Err(Error::BufferSize);
        }
        if result.len() != buffer_size {
            return Err(Error::BufferSize);
        }
        for item in result.iter_mut() {
            *item = 0.0;
        }
        for (overtone, wave) in self.waves.borrow_mut().iter_mut().enumerate() {
            try!(wave.get(base_frequency, &mut wave_buffer));
            try!(self.amplitude_overtones
                .apply(time_start, overtone, &mut wave_buffer));
            for (item, wave) in result.iter_mut()
                .zip(wave_buffer.iter()) {
                *item += *wave;
            }
        }
        Ok(())
    }
}

/// Channel structure used for mixing sound structures.
#[derive(Clone)]
struct MixerChannel {
    /// The interval of the channel's frequency relative to the mixer's input frequency.
    interval: Interval,
    /// Sound structure.
    sound: Rc<SoundStructure>,
    volume_relative: SampleCalc,
    volume_normalized: SampleCalc,
    frequency_buffer: Vec<SampleCalc>,
    wave_buffer: Vec<SampleCalc>,
}

/// Mixes sound channels (structures).
#[derive(Clone)]
pub struct Mixer {
    buffer_size: usize,
    channels: RefCell<Vec<MixerChannel>>,
}

impl Mixer {
    /// custom constructor
    pub fn new(buffer_size: usize) -> SoundResult<Mixer> {
        Ok(Mixer {
            buffer_size: buffer_size,
            channels: RefCell::new(Vec::new()),
        })
    }

    /// Add a new channel to the mixer.
    pub fn add(&self,
               interval: Interval,
               sound: Rc<SoundStructure>,
               volume: SampleCalc)
               -> SoundResult<&Mixer> {
        if volume < 0.0 {
            return Err(Error::AmplitudeInvalid);
        }
        let channel = MixerChannel {
            interval: interval,
            sound: sound,
            volume_relative: volume,
            volume_normalized: 0.0,
            frequency_buffer: vec![1.0; self.buffer_size],
            wave_buffer: vec![0.0; self.buffer_size],
        };
        self.channels.borrow_mut().push(channel);
        self.normalize();
        Ok(self)
    }

    /// Generates the normalized volumes for the channels. Only normalizes if the sum of volumes
    /// is greater than 1.0
    fn normalize(&self) {
        let mut volume_sum: SampleCalc = 0.0;
        let mut channels = self.channels.borrow_mut();
        for channel in channels.iter() {
            volume_sum += channel.volume_relative;
        }
        let volume_multiplier = if volume_sum < 1.0 {
            1.0
        } else {
            1.0 / volume_sum
        };
        for channel in channels.iter_mut() {
            channel.volume_normalized = channel.volume_relative * volume_multiplier;
        }
    }

    /// Sets a new interval for the channel, relative to the base frequency of the mixer.
    pub fn set_interval(&self, channel: usize, interval: Interval) -> SoundResult<()> {
        if let Some(ch) = self.channels.borrow_mut().get_mut(channel) {
            ch.interval = interval;
        } else {
            return Err(Error::ChannelInvalid);
        }
        Ok(())
    }

    /// Sets the relative volume of the channel.
    pub fn set_volume(&self, channel: usize, volume: SampleCalc) -> SoundResult<()> {
        if let Some(ch) = self.channels.borrow_mut().get_mut(channel) {
            if volume < 0.0 {
                return Err(Error::AmplitudeInvalid);
            }
            ch.volume_relative = volume;
            self.normalize();
        } else {
            return Err(Error::ChannelInvalid);
        }
        Ok(())
    }
}

impl SoundStructure for Mixer {
    fn get(&self,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc],
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if base_frequency.len() != result.len() {
            return Err(Error::BufferSize);
        }
        for item in result.iter_mut() {
            *item = 0.0;
        }
        for channel in self.channels.borrow_mut().iter_mut() {
            try!(channel.interval
                .transpose(base_frequency, &mut channel.frequency_buffer));
            try!(channel.sound.get(time_start,
                                   &channel.frequency_buffer,
                                   &mut channel.wave_buffer));
            for (item, wave) in result.iter_mut().zip(channel.wave_buffer.iter()) {
                *item += *wave * channel.volume_normalized;
            }
        }
        Ok(())
    }
}

// TODO: `FadeOutLinear` and `FadeInLinear` replaced by `FadeLinear`, ProgressOption shall be used
// too here.
// https://en.wikipedia.org/wiki/Fade_(audio_engineering)#Crossfading
/// Mixes two sound structures. While one fades out, another fades in.
#[doc(hidden)]
#[allow(dead_code)]
pub struct Crossfader {
    duration: SampleCalc,
    sound_fade_out: Rc<SoundStructure>,
    sound_fade_in: Rc<SoundStructure>,
    interval: Interval,
    amplitude_fade_out: FadeLinear,
    amplitude_fade_in: FadeLinear,
    frequency_buffer_in: RefCell<Vec<SampleCalc>>, // only used when interval is not unison
    wave_fade_out_buffer: RefCell<Vec<SampleCalc>>,
    wave_fade_in_buffer: RefCell<Vec<SampleCalc>>,
}

impl Crossfader {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc,
               buffer_size: usize,
               duration: SampleCalc,
               sound_fade_out: Rc<SoundStructure>,
               sound_fade_in: Rc<SoundStructure>)
               -> SoundResult<Crossfader> {
        let amplitude_fade_out = try!(FadeLinear::new_with_time(sample_rate, duration, 0.0));
        try!(amplitude_fade_out.set_amplitude_start(1.0));
        let amplitude_fade_in = try!(FadeLinear::new_with_time(sample_rate, duration, 1.0));
        Ok(Crossfader {
            duration: duration,
            interval: try!(Interval::new(1, 1)),
            sound_fade_out: sound_fade_out,
            sound_fade_in: sound_fade_in,
            amplitude_fade_out: amplitude_fade_out,
            amplitude_fade_in: amplitude_fade_in,
            frequency_buffer_in: RefCell::new(vec![0.0; buffer_size]),
            wave_fade_out_buffer: RefCell::new(vec![0.0; buffer_size]),
            wave_fade_in_buffer: RefCell::new(vec![0.0; buffer_size]),
        })
    }

    /// Sets an interval for the fading in sound (relative to the fading out one).
    pub fn set_interval(&mut self, interval: Interval) -> &mut Crossfader {
        self.interval = interval;
        self
    }

    /// Returns the duration of the crossfade.
    pub fn get_duration(&self) -> SampleCalc {
        self.duration
    }
}

// impl SoundStructure for Crossfader {
// fn get(&self,
// time_start: SampleCalc,
// base_frequency: &[SampleCalc],
// result: &mut [SampleCalc])
// -> SoundResult<()> {
// if base_frequency.len() != result.len() {
// return Err(Error::BufferSize);
// }
// try!(self.sound_fade_out.get(time_start,
// base_frequency,
// &mut self.wave_fade_out_buffer.borrow_mut()));
// if self.interval.is_unison() {
// try!(self.sound_fade_in.get(time_start,
// base_frequency,
// &mut self.wave_fade_in_buffer.borrow_mut()));
//
// } else {
// try!(self.interval
// .transpose(base_frequency, &mut self.frequency_buffer_in.borrow_mut()));
// try!(self.sound_fade_in.get(time_start,
// &self.frequency_buffer_in.borrow(),
// &mut self.wave_fade_in_buffer.borrow_mut()));
// }
// try!(self.amplitude_fade_out
// .apply(time_start, &mut self.wave_fade_out_buffer.borrow_mut()));
// try!(self.amplitude_fade_in
// .apply(time_start, &mut self.wave_fade_in_buffer.borrow_mut()));
// for ((item, sample_out), sample_in) in result.iter_mut()
// .zip(self.wave_fade_out_buffer.borrow().iter())
// .zip(self.wave_fade_in_buffer.borrow().iter()) {
// item = *sample_out + *sample_in;
// }
// Ok(())
// }
// }
