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

use std::{error, fmt};

/// Precision of the finally produced samples.
pub type SampleOutput = f32;
/// Precision of calculations. Changing it to `f64` can slow down some calculations 4 times.
pub type SampleCalc = f32;
/// = π x 2
// pub const PI2: SampleCalc = ::std::f64::consts::PI * 2.0;
pub const PI2: SampleCalc = ::std::f32::consts::PI * 2.0;


/// Sample count for calculations. It affects both latency and computation performance.
/// Latency perception for musical instruments: over ~12ms is already disturbing for some players.
pub const BUFFER_SIZE: usize = 512;
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

#[cfg(feature = "be-portaudio")]
use sound::backend_portaudio::*;
#[cfg(feature = "be-rsoundio")]
use sound::backend_rsoundio::*;

/// Error types of the sound module.
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Sound output backend error.
    Backend(BackendError),
    /// Invalid sample rate
    SampleRateInvalid,
    /// Invalid buffer size for the given sample count
    BufferSize,
    /// Numerator cannot be 0, because frequencies can not be 0
    NumeratorInvalid,
    /// Denominator cannot be 0 (division by zero error)
    DenominatorInvalid,
    /// The frequency is below the hearing range
    FrequencyTooLow,
    /// The frequency exceeds the hearing range
    FrequencyTooHigh,
    /// This frequency function is a source, it can not use an input frequency buffer
    FrequencySource,
    /// Amplitude cannot be negative
    AmplitudeInvalid,
    /// Amplitude change rate is out of the range allowed for the given function
    AmplitudeRateInvalid,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        f.write_str(self.description())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            Backend(ref err) => err.description(),
            SampleRateInvalid => "invalid sample rate",
            BufferSize => "incorrect buffer size",
            NumeratorInvalid => "invalid numerator",
            DenominatorInvalid => "invalid denominator",
            FrequencyTooLow => "frequency is below the hearing range",
            FrequencyTooHigh => "frequency exceeds the hearing range",
            FrequencySource => "input frequency buffer can not be used",
            AmplitudeInvalid => "invalid amplitude",
            AmplitudeRateInvalid => "invalid amplitude decay rate",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        use self::Error::*;
        match *self {
            Backend(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<BackendError> for Error {
    fn from(e: BackendError) -> Self {
        Error::Backend(e)
    }
}

/// Return type for the sound module functions.
pub type SoundResult<T> = Result<T, Error>;
