/// Fuctions which output frequency changes.
pub mod frequency;
/// Fuctions which output amplitude changes.
pub mod amplitude;
/// Fuctions which output complete waveforms.
pub mod wave;
/// [`PortAudio`](https://github.com/RustAudio/rust-portaudio) backend for sound playback
#[cfg(feature = "be-portaudio")]
pub mod backend_portaudio;

/// [`libsoundio`](https://github.com/klingtnet/rsoundio) backend for sound playback
#[cfg(feature = "be-rsoundio")]
pub mod backend_rsoundio;

use std::{error, fmt};

/// Precision of the finally produced samples
pub type SampleOutput = f32;
/// Precision of calculations
pub type SampleCalc = f64;

/// Sample count for calculations. It affects both latency and computation performance.
/// Latency perception for musical instruments: over ~12ms is already disturbing for some players.
pub const BUFFER_SIZE: usize = 1024;
/// The lowest hearable (feelable) frequency. Tones below it will not be calculated.
pub const TONE_FREQUENCY_MIN: SampleCalc = 5.0;
/// The highest hearable (feelable) frequency. Overtones above this frequency are filtered out
/// from calculations.
pub const TONE_FREQUENCY_MAX: SampleCalc = 24000.0;

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
           sample_count: usize,
           time_start: SampleCalc,
           base_frequency: &[SampleCalc], // &Vec<SampleCalc>,
           result: &mut Vec<SampleCalc>)
           -> SoundResult<()>;
}

#[cfg(feature = "be-portaudio")]
use sound::backend_portaudio::*;
#[cfg(feature = "be-rsoundio")]
use sound::backend_rsoundio::*;

/// Error types of the sound module
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Sound output backend error.
    Backend(BackendError),
    /// Invalid buffer size for the given sample count
    BufferSize,
    /// Denominator cannot be 0 (division by zero error)
    DenominatorInvalid,
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
            BufferSize => "incorrect buffer size",
            DenominatorInvalid => "invalid denominator",
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

/// Return type of the sound module functions
pub type SoundResult<T> = Result<T, Error>;
