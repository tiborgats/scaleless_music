use sound::*;
use std::rc::Rc;
use std::cell::RefCell;
// use rayon::prelude::*;
// use std::fmt::Debug;

/// A tone with optional overtones and amplitude modulation.
#[derive(Clone)]
pub struct Note {
    sample_rate: SampleCalc,
    //    frequency_function: Rc<FrequencyFunction>,
    //    frequency_buffer: RefCell<Vec<SampleCalc>>,
    amplitude_function: Rc<AmplitudeFunctionOvertones>,
    amplitude_buffer: RefCell<Vec<SampleCalc>>,
    wave_buffer: RefCell<Vec<SampleCalc>>,
    overtone_max: usize,
}

impl Note {
    /// Custom constructor
    pub fn new(sample_rate: SampleCalc,
               buffer_size: usize,
               amplitude_function: Rc<AmplitudeFunctionOvertones>,
               overtone_max: usize)
               -> SoundResult<Note> {
        Ok(Note {
            sample_rate: sample_rate,
            //            frequency_function: frequency_function,
            //            frequency_buffer: RefCell::new(vec![0.0; buffer_size]),
            amplitude_function: amplitude_function,
            amplitude_buffer: RefCell::new(vec![0.0; buffer_size]),
            wave_buffer: RefCell::new(vec![0.0; buffer_size]),
            overtone_max: overtone_max,
        })
    }
    /// Set a new amplitude function
    pub fn set_amplitude(&mut self,
                         amplitude_function: Rc<AmplitudeFunctionOvertones>)
                         -> &mut Note {
        self.amplitude_function = amplitude_function;
        self
    }
}

impl SoundStructure for Note {
    fn get(&self,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc],
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        let buffer_size = self.wave_buffer.borrow().len();
        // if self.amplitude_buffer.borrow().len() != buffer_size {
        // return Err(Error::BufferSize);
        // }
        if base_frequency.len() != buffer_size {
            return Err(Error::BufferSize);
        }
        if result.len() != buffer_size {
            return Err(Error::BufferSize);
        }
        let time_sample: SampleCalc = 1.0 / self.sample_rate;
        // let frequency_b = self.frequency_buffer.borrow_mut();
        for item in result.iter_mut() {
            *item = 0.0;
        }
        for overtone in 0..self.overtone_max {
            let freq_multiplier = (overtone as SampleCalc + 1.0) * PI2;
            //            for sample_idx in 0..buffer_size {
            // self.frequency_buffer.borrow_mut().get_mut(sample_idx).unwrap() =
            // base_frequency.get(sample_idx).unwrap();
            // }
            try!(self.amplitude_function.get(time_start,
                                             overtone,
                                             &mut self.amplitude_buffer.borrow_mut()));

            for (((index, item), frequency), amplitude) in result.iter_mut()
                .enumerate()
                .zip(base_frequency)
                .zip(self.amplitude_buffer.borrow().iter()) {
                    let time = (index as SampleCalc * time_sample) + time_start;
                    *item += (time * frequency * freq_multiplier).sin() * *amplitude;
                }
/*            self.wave_buffer
                .borrow_mut()
                .par_iter_mut()
                .zip(base_frequency.par_iter())
                .enumerate()
                .for_each(|(i, (w, f))| {
                    let time = (i as SampleCalc * time_sample) + time_start;
                    *w = (time * f * freq_multiplier).sin();
                });*/
/*            for ((item, wave), amplitude) in result.iter_mut()
                .zip(self.wave_buffer.borrow().iter())
                .zip(self.amplitude_buffer.borrow().iter()) {
                *item += *wave * *amplitude;
            }*/
        }
        Ok(())
    }
}

/// Channel structure used for mixing sound structures.
struct MixerChannel {
    /// The interval of the channel's frequency relative to the mixer's input frequency.
    interval: Interval,
    /// Sound structure.
    sound: Rc<SoundStructure>,
    volume_relative: SampleCalc,
    volume_normalized: SampleCalc,
    frequency_buffer: RefCell<Vec<SampleCalc>>,
    wave_buffer: RefCell<Vec<SampleCalc>>,
}

/// Mixes sound channels (structures).
#[allow(dead_code)]
pub struct Mixer {
    sample_rate: SampleCalc,
    buffer_size: usize,
    channels: RefCell<Vec<MixerChannel>>,
}

impl Mixer {
    /// custom constructor
    pub fn new(sample_rate: SampleCalc, buffer_size: usize) -> SoundResult<Mixer> {
        Ok(Mixer {
            sample_rate: sample_rate,
            buffer_size: buffer_size,
            channels: RefCell::new(Vec::new()),
        })
    }
    /// Add a new channel to the mixer.
    pub fn add(&mut self,
               interval: Interval,
               sound: Rc<SoundStructure>,
               volume: SampleCalc)
               -> SoundResult<&mut Mixer> {
        if volume < 0.0 {
            return Err(Error::AmplitudeInvalid);
        }
        let channel = MixerChannel {
            interval: interval,
            sound: sound,
            volume_relative: volume,
            volume_normalized: 0.0,
            frequency_buffer: RefCell::new(vec![0.0; self.buffer_size]),
            wave_buffer: RefCell::new(vec![0.0; self.buffer_size]),
        };
        self.channels.borrow_mut().push(channel);
        self.normalize();
        Ok(self)
    }
    /// Generates the normalized volumes for the channels. Only normalizes if the sum of volumes
    /// is greater than 1.0
    fn normalize(&self) {
        let mut volume_sum: SampleCalc = 0.0;
        for channel in self.channels.borrow().iter() {
            volume_sum += channel.volume_relative;
        }
        let volume_multiplier = if volume_sum < 1.0 {
            1.0
        } else {
            1.0 / volume_sum
        };
        for channel in self.channels.borrow_mut().iter_mut() {
            channel.volume_normalized = channel.volume_relative * volume_multiplier;
        }
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
        for channel in self.channels.borrow().iter() {
            try!(channel.interval
                .transpose(base_frequency, &mut channel.frequency_buffer.borrow_mut()));
            try!(channel.sound.get(time_start,
                                   &channel.frequency_buffer.borrow(),
                                   &mut channel.wave_buffer.borrow_mut()));
            for (item, wave) in result.iter_mut().zip(channel.wave_buffer.borrow().iter()) {
                *item = *wave * channel.volume_normalized;
            }
        }
        Ok(())
    }
}

// https://en.wikipedia.org/wiki/Fade_(audio_engineering)#Crossfading
/// Mixes two sound structures. While one fades out, another fades in.
#[allow(dead_code)]
pub struct Crossfader {
    sample_rate: SampleCalc,
    buffer_size: usize,
    duration: SampleCalc,
    sound_fade_out: Rc<SoundStructure>,
    sound_fade_in: Rc<SoundStructure>,
    interval: Interval,
    amplitude_fade_out: FadeOutLinear,
    amplitude_fade_in: FadeInLinear,
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
        let amplitude_fade_out = try!(FadeOutLinear::new(sample_rate, duration));
        let amplitude_fade_in = try!(FadeInLinear::new(sample_rate, duration));
        Ok(Crossfader {
            sample_rate: sample_rate,
            buffer_size: buffer_size,
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
}

impl SoundStructure for Crossfader {
    fn get(&self,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc],
           result: &mut [SampleCalc])
           -> SoundResult<()> {
        if base_frequency.len() != result.len() {
            return Err(Error::BufferSize);
        }
        try!(self.sound_fade_out.get(time_start,
                                     base_frequency,
                                     &mut self.wave_fade_out_buffer.borrow_mut()));
        if self.interval.is_unison() {
            try!(self.sound_fade_in.get(time_start,
                                        base_frequency,
                                        &mut self.wave_fade_in_buffer.borrow_mut()));

        } else {
            try!(self.interval
                .transpose(base_frequency, &mut self.frequency_buffer_in.borrow_mut()));
            try!(self.sound_fade_in.get(time_start,
                                        &self.frequency_buffer_in.borrow(),
                                        &mut self.wave_fade_in_buffer.borrow_mut()));
        }
        try!(self.amplitude_fade_out
            .apply(time_start, &mut self.wave_fade_out_buffer.borrow_mut()));
        try!(self.amplitude_fade_in
            .apply(time_start, &mut self.wave_fade_in_buffer.borrow_mut()));
        for ((item, sample_out), sample_in) in result.iter_mut()
            .zip(self.wave_fade_out_buffer.borrow().iter())
            .zip(self.wave_fade_in_buffer.borrow().iter()) {
            *item = *sample_out + *sample_in;
        }
        Ok(())
    }
}
