#[cfg(feature = "be-portaudio")]
use crate::sound::backend_portaudio::*;
#[cfg(feature = "be-rsoundio")]
use crate::sound::backend_rsoundio::*;
#[cfg(feature = "be-sdl2")]
use crate::sound::backend_sdl2::*;

use thiserror::Error;

/// Return type for the sound module functions.
pub type SoundResult<T> = Result<T, Error>;

/// Error types of the sound module.
#[derive(Debug, Clone, Error)]
pub enum Error {
    #[cfg(any(feature = "be-portaudio", feature = "be-rsoundio", feature = "be-sdl2"))]
    /// Sound output backend error.
    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),
    /// Invalid sample rate.
    #[error("Invalid sample rate")]
    SampleRateInvalid,
    /// Invalid buffer size for the given sample count.
    #[error("Incorrect buffer size")]
    BufferSize,
    /// Overtone count does not match the reserved array size.
    #[error("Invalid overtone count")]
    OvertoneCountInvalid,
    /// Numerator cannot be 0, because frequencies can not be 0.
    #[error("Invalid numerator")]
    NumeratorInvalid,
    /// Denominator cannot be 0 (division by zero error).
    #[error("Invalid denominator")]
    DenominatorInvalid,
    /// The frequency is below the hearing range.
    #[error("Frequency is below the hearing range")]
    FrequencyTooLow,
    /// The frequency exceeds the hearing range.
    #[error("Frequency exceeds the hearing range")]
    FrequencyTooHigh,
    /// Frequency can not be zero or negative.
    #[error("Frequency can not be zero or negative")]
    FrequencyInvalid,
    /// This frequency function is a source, it can not use an input frequency buffer.
    #[error("Input frequency buffer can not be used")]
    FrequencySource,
    /// A rate must be positive.
    #[error("Invalid rate")]
    RateInvalid,
    /// Amplitude cannot be negative.
    #[error("Invalid amplitude")]
    AmplitudeInvalid,
    /// Amplitude change time is not positive.
    #[error("Invalid amplitude change time")]
    AmplitudeTimeInvalid,
    /// Amplitude change rate is out of the range allowed for the given function.
    #[error("Invalid amplitude decay rate")]
    AmplitudeRateInvalid,
    /// A time period must be positive.
    #[error("Invalid period")]
    PeriodInvalid,
    /// A time duration must be positive.
    #[error("Invalid duration")]
    DurationInvalid,
    /// Channel of the given number does not exist.
    #[error("Invalid channel")]
    ChannelInvalid,
    /// Beats per minute must be positive.
    #[error("Beats per minute must be positive")]
    TempoInvalid,
    /// Timing option does not match the method.
    #[error("Invalid timing option")]
    TimingInvalid,
    /// The selected progress option is invalid for this case.
    #[error("Invalid progress option")]
    ProgressInvalid,
    /// Progress is finished.
    #[error("Progress completed")]
    ProgressCompleted,
    /// The number of items completed in an unfinished buffer operation.
    #[error("The number of items completed in an unfinished buffer operation: {0}")]
    ItemsCompleted(usize),
    /// The Sequence has no items.
    #[error("Sequence has no items")]
    SequenceEmpty,
    /// Item at the given index does not exist.
    #[error("The item does not exist")]
    ItemInvalid,
    /// Overflow occured during calculations.
    #[error("Overflow")]
    Overflow,
}
