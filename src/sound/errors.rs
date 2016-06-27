use std::{error, fmt};

#[cfg(feature = "be-portaudio")]
use sound::backend_portaudio::*;
#[cfg(feature = "be-rsoundio")]
use sound::backend_rsoundio::*;

/// Return type for the sound module functions.
pub type SoundResult<T> = Result<T, Error>;

/// Error types of the sound module.
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Sound output backend error.
    Backend(BackendError),
    /// Invalid sample rate.
    SampleRateInvalid,
    /// Invalid buffer size for the given sample count.
    BufferSize,
    /// Overtone count does not match the reserved array size.
    OvertoneCountInvalid,
    /// Numerator cannot be 0, because frequencies can not be 0.
    NumeratorInvalid,
    /// Denominator cannot be 0 (division by zero error).
    DenominatorInvalid,
    /// The frequency is below the hearing range.
    FrequencyTooLow,
    /// The frequency exceeds the hearing range.
    FrequencyTooHigh,
    /// Frequency can not be zero or negative.
    FrequencyInvalid,
    /// This frequency function is a source, it can not use an input frequency buffer.
    FrequencySource,
    /// A rate must be positive.
    RateInvalid,
    /// Amplitude cannot be negative.
    AmplitudeInvalid,
    /// Amplitude change time is not positive.
    AmplitudeTimeInvalid,
    /// Amplitude change rate is out of the range allowed for the given function.
    AmplitudeRateInvalid,
    /// A time period must be positive.
    PeriodInvalid,
    /// A time duration must be positive.
    DurationInvalid,
    /// Channel of the given number does not exist.
    ChannelInvalid,
    /// Beats per minute must be positive
    TempoInvalid,
    /// The selected progress option is invalid for this case.
    ProgressInvalid,
    /// Progress is finished.
    ProgressCompleted,
    /// The number of items completed in an unfinished buffer operation.
    ItemsCompleted(usize),
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
            OvertoneCountInvalid => "invalid overtone count",
            NumeratorInvalid => "invalid numerator",
            DenominatorInvalid => "invalid denominator",
            FrequencyTooLow => "frequency is below the hearing range",
            FrequencyTooHigh => "frequency exceeds the hearing range",
            FrequencyInvalid => "frequency can not be zero or negative",
            FrequencySource => "input frequency buffer can not be used",
            RateInvalid => "invalid rate",
            AmplitudeInvalid => "invalid amplitude",
            AmplitudeTimeInvalid => "invalid amplitude change time",
            AmplitudeRateInvalid => "invalid amplitude decay rate",
            PeriodInvalid => "invalid period",
            DurationInvalid => "invalid duration",
            ChannelInvalid => "invalid channel",
            TempoInvalid => "beats per minute must be positive",
            ProgressInvalid => "invalid progress option",
            ProgressCompleted => "progress completed",
            ItemsCompleted(_) => "",
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
