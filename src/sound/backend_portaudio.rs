use crate::portaudio as pa;
use crate::sound::*;
// use std::thread;
// use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use thiserror::Error;

/// This is a wrapper around the sound output backend
pub struct SoundInterface<T: 'static> {
    sample_rate: u32,
    channel_count: u16,
    stream: pa::Stream<pa::NonBlocking, pa::stream::Output<SampleOutput>>,
    sender: Option<Sender<T>>, // receiver: Option<Receiver<T>,
}

impl<T> SoundInterface<T> {
    /// Creates a new backend for sound playback.
    /// At the moment all channels output the same sound.
    pub fn new(
        sample_rate: u32,
        buffer_size: usize,
        channel_count: u16,
        mut generator: Box<dyn SoundGenerator<Command = T>>,
    ) -> BackendResult<SoundInterface<T>> {
        println!("PortAudio version : {}", pa::version());
        println!("PortAudio version text : {:?}", pa::version_text());
        let pa = pa::PortAudio::new()?;
        println!("host count: {}", pa.host_api_count()?);
        let mut settings = pa.default_output_stream_settings(
            channel_count as i32,
            sample_rate as f64,
            buffer_size as u32,
        )?;
        // we won't output out of range samples so don't bother clipping them.
        settings.flags = pa::stream_flags::CLIP_OFF;

        let mut generator_buffer: Vec<SampleCalc> = vec![0.0; buffer_size];

        let (sender, receiver) = ::std::sync::mpsc::channel();
        // This routine will be called by the PortAudio engine when audio is needed. It may
        // called at interrupt level on some machines so don't do anything that could mess
        // up the system like dynamic resource allocation or IO.
        let callback_fn = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
            if let Ok(command) = receiver.try_recv() {
                generator.process_command(command);
            }
            generator.get_samples(frames, &mut generator_buffer);
            let mut idx = 0;
            for item in generator_buffer.iter().take(frames) {
                for _ in 0..(channel_count as usize) {
                    buffer[idx] = *item; // as SampleOutput;
                    idx += 1;
                }
            }
            //            for output_frame in buffer.chunks_mut(channel_count) {
            //                for channel_sample in output_frame {
            //                    *channel_sample = synthesizer.sample_next();
            // sender.send(time_start).ok();
            pa::Continue
        };

        // Open a non-blocking stream.
        let stream = pa.open_non_blocking_stream(settings, callback_fn)?;
        println!("Stream is created.");

        Ok(SoundInterface {
            sample_rate: sample_rate,
            channel_count: channel_count,
            stream: stream,
            sender: Some(sender),
        })
    }
    /// Starts the sound output stream.
    pub fn start(&mut self) -> BackendResult<()> {
        self.stream.start()?;
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

impl<T> Drop for SoundInterface<T> {
    fn drop(&mut self) {
        if self.stream.is_active() == Ok(true) {
            if let Err(err) = self.stream.stop() {
                println!("PortAudio.stream.stop: {}", &err.to_string());
            }
        }
        if let Err(err) = self.stream.close() {
            println!("PortAudio.stream.close: {}", &err.to_string());
        }
    }
}

/// Return type for the backend functions.
pub type BackendResult<T> = Result<T, BackendError>;

/// Wrapper for the propagation of backend specific errors.
#[derive(Debug, Copy, Clone, Error)]
pub enum BackendError {
    /// Errors of the PortAudio backend.
    #[error("PortAudio backend error: {0}")]
    PortAudio(#[from] pa::Error),
    /// The SoundGenerator is disconnected, could not recieve the command
    #[error("SoundGenerator is disconnected")]
    Disconnected,
}
