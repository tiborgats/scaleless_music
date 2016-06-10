/// Error messages.
pub mod errors;
/// Frequency interval.
pub mod interval;
/// Fuctions which output frequency changes.
pub mod frequency;
/// Fuctions which output amplitude changes.
pub mod amplitude;
/// Fuctions which output complete waveforms.
pub mod wave;
/// [`PortAudio`](https://github.com/RustAudio/rust-portaudio) backend for sound playback.
#[cfg(feature = "be-portaudio")]
pub mod backend_portaudio;

/// [`libsoundio`](https://github.com/klingtnet/rsoundio) backend for sound playback.
#[cfg(feature = "be-rsoundio")]
pub mod backend_rsoundio;

pub use self::errors::Error;
pub use self::interval::Interval;
pub use self::frequency::*;
pub use self::amplitude::*;
pub use self::wave::*;

/// Precision of the finally produced samples.
pub type SampleOutput = f32;
/// Precision of calculations. Changing it to `f64` can slow down some calculations 4 times.
pub type SampleCalc = f32;

/// Sample count for calculations. It affects both latency and computation performance.
/// Latency perception for musical instruments: over ~12ms is already disturbing for some players.
pub const BUFFER_SIZE_DEFAULT: usize = 512;

/// = 3 Hz, the lowest feelable frequency. Tones below it will not be calculated. The hearable
/// lowest is 12 Hz.
/// See: [hearing range](https://en.wikipedia.org/wiki/Hearing_range#Humans)
pub const TONE_FREQUENCY_MIN: SampleCalc = 3.0;

/// = 28 kHz, the highest hearable (feelable) frequency. Overtones above this frequency are
/// filtered out from calculations.
/// See: [hearing range](https://en.wikipedia.org/wiki/Hearing_range#Humans)
pub const TONE_FREQUENCY_MAX: SampleCalc = 28000.0;

/// = 192 kHz, as Humans can discern time differences of
/// [5 microseconds](http://boson.physics.sc.edu/~kunchur/papers/gradual.pdf).
/// Humans can hear < 1° difference in the location of the sound source, when it is in front of
/// them. See also:
/// [interaural time difference](https://en.wikipedia.org/wiki/Interaural_time_difference)
pub const SAMPLE_RATE_DEFAULT: u32 = 192_000;

/// = π x 2
// pub const PI2: SampleCalc = ::std::f64::consts::PI * 2.0;
pub const PI2: SampleCalc = ::std::f32::consts::PI * 2.0;

/// Sound sample generator for output (playback). It can also take real-time input (commands),
/// thus musical instruments can be realized with it.
pub trait SoundGenerator<T: 'static> {
/// Get the next `sample_count` amount of samples, put them in `result`
    fn get_samples(&mut self, sample_count: usize, result: &mut Vec<SampleCalc>);
/// Send a message to the `SoundGenerator`.
    fn process_command(&mut self, _command: T);
}
/// A sound component. Can be a simple wave or a complex structure of waves.
pub trait SoundStructure {
    /// Returns the calculated samples in the `result` buffer.
    fn get(&self,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc], // &Vec<SampleCalc>,
           result: &mut [SampleCalc])
           -> SoundResult<()>;
}

/// Calculates the period of one sample for the given sample rate.
/// Sample rates below 1.0 are considered to be invalid.
pub fn get_sample_time(sample_rate: SampleCalc) -> SoundResult<SampleCalc> {
    if sample_rate < 1.0 {
        Err(Error::SampleRateInvalid)
    } else {
        Ok((1.0 / sample_rate))
    }
}


/// Return type for the sound module functions.
pub type SoundResult<T> = Result<T, Error>;
