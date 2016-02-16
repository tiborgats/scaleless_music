
pub mod frequency;
pub mod amplitude;
pub mod wave;
pub mod instrument;
pub mod interface;

use std::{error, fmt};
pub use portaudio as pa;
use sound::instrument::*;

pub type SampleOutput = f32;
/// Type definition for the precision of calculations
pub type SampleCalc = f64;

pub const BUFFER_SIZE: usize = 1 * 1024; // sample count (for calculations)
pub const TONE_FREQUENCY_MIN: SampleCalc = 5.0; // lowest hearable (feelable) frequency
pub const TONE_FREQUENCY_MAX: SampleCalc = 24000.0; // highest hearable (feelable) frequency

pub trait SoundGenerator {
//    type GeneratorCommand;
    fn sample_next(&mut self) -> SampleOutput;
//    fn get_samples(&mut self, count: usize) -> &Vec<SampleOutput>;
    fn process_command(&mut self, _command: GeneratorCommand) {}
}



#[derive(Debug, Copy, Clone)]
pub enum Error {
    BufferSize,
    PortAudio(pa::Error),
//    Other,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        //        write!(f, "sound error: {}", self.description())
        // write!(f, "{:?}", self)
        f.write_str(self.description())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            BufferSize => "incorrect buffer size",
            PortAudio(ref err) => err.description(),
//            Other => "other",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl From<pa::Error> for Error {
    fn from(e: pa::Error) -> Self {
        Error::PortAudio(e)
    }
}

pub type SoundResult<T> = Result<T, Error>;
