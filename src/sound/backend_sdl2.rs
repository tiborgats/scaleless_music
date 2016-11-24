// Help: http://angrylawyer.github.io/rust-sdl2/sdl2/audio/index.html
//
// Installation instructions: https://github.com/AngryLawyer/rust-sdl2/blob/master/README.md
//

use sdl2::audio::*;
use sound::*;
use std::sync::mpsc::{Sender, Receiver};

/// Struct containing the settings of the callback routine for SDL2
struct Player<T: 'static + Send> {
    channel_count: usize,
    frame_size: usize,
    generator_buffer: Vec<SampleCalc>,
    generator: Box<SoundGenerator<Command = T>>,
    receiver: Receiver<T>,
}

impl<T> Player<T>
    where T: Send
{
    /// Custom constructor.
    fn new(spec: AudioSpec,
           buffer_size: usize,
           generator: Box<SoundGenerator<Command = T>>,
           receiver: Receiver<T>)
           -> Player<T> {
        Player {
            channel_count: spec.channels as usize,
            frame_size: buffer_size,
            generator_buffer: vec![0.0; buffer_size],
            generator: generator,
            receiver: receiver,
        }
    }
}

impl<T> AudioCallback for Player<T>
    where T: Send
{
    type Channel = f32;
    /// Callback routine for SDL2
    fn callback(&mut self, out: &mut [f32]) {
        if let Ok(command) = self.receiver.try_recv() {
            self.generator.process_command(command);
        }
        self.generator.get_samples(self.frame_size, &mut self.generator_buffer);
        let mut idx = 0;
        for item in self.generator_buffer.iter().take(self.frame_size) {
            for _ in 0..(self.channel_count) {
                out[idx] = *item;// as SampleOutput;
                idx += 1;
            }
        }
    }
}

/// This is a wrapper around the sound output backend
pub struct SoundInterface<T: 'static + Send> {
    sample_rate: u32,
    channel_count: u16,
    // sdl_context: ::sdl2::Sdl,
    // sdl_audio_subsystem: ::sdl2::AudioSubsystem,
    sdl_device: AudioDevice<Player<T>>,
    sender: Option<Sender<T>>, // receiver: Option<Receiver<T>,
}

impl<T> SoundInterface<T>
    where T: Send
{
    /// Creates a new backend for sound playback.
    /// At the moment all channels output the same sound.
    pub fn new(sample_rate: u32,
               buffer_size: usize,
               channel_count: u16,
               generator: Box<SoundGenerator<Command = T>>)
               -> BackendResult<SoundInterface<T>> {

        let sdl_context = ::sdl2::init()?;
        let sdl_audio_subsystem = sdl_context.audio()?;

        let desired_spec = AudioSpecDesired {
            freq: Some(sample_rate as i32),
            channels: Some(channel_count as u8),
            samples: Some((buffer_size as u16) * channel_count), // None, // default sample size
        };

        let (sender, receiver) = ::std::sync::mpsc::channel();

        let sdl_device = sdl_audio_subsystem.open_playback(None,
                           &desired_spec,
                           |spec| Player::new(spec, buffer_size, generator, receiver))?;

        println!("Stream is created.");

        Ok(SoundInterface {
            sample_rate: sample_rate,
            channel_count: channel_count,
            // sdl_context: sdl_context,
            // sdl_audio_subsystem: sdl_audio_subsystem,
            sdl_device: sdl_device,
            sender: Some(sender),
        })
    }
    /// Starts the sound output stream.
    pub fn start(&mut self) -> BackendResult<()> {
        self.sdl_device.resume();
        println!("Successfully started the stream.");
        Ok(())
    }
    /// Sends a command to the sound generator.
    pub fn send_command(&mut self, command: T) -> BackendResult<()> {
        if let Some(ref sender) = self.sender {
            match sender.send(command) {
                Ok(_) => Ok(()),
                Err(_) => Err(BackendError::Disconnected),
            }
        } else {
            return Err(BackendError::Disconnected);
        }
    }

    /// Returns the sample rate of the sond output
    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
    /// Returns the channel count of the sond output
    pub fn get_channel_count(&self) -> u16 {
        self.channel_count
    }
}


/// Wrapper for the propagation of backend specific errors.
#[derive(Debug, Clone)]
pub enum BackendError {
    /// Errors of the Sdl backend.
    Sdl(String),
    /// The SoundGenerator is disconnected, could not recieve the command
    Disconnected,
}

use std::{error, fmt};

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        f.write_str(self.description())
    }
}

impl error::Error for BackendError {
    fn description(&self) -> &str {
        use self::BackendError::*;
        match *self {
            Sdl(ref err) => err.as_str(),
            Disconnected => "SoundGenerator is disconnected",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl From<String> for BackendError {
    fn from(e: String) -> Self {
        BackendError::Sdl(e)
    }
}

/// Return type for the backend functions.
pub type BackendResult<T> = Result<T, BackendError>;
