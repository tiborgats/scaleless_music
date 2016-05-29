//! Create scaleless music.
//!
//! # Usage
//!
//! Add `music` to your project by adding the dependency to your Cargo.toml as follows:
//!
//! ```toml
//! [dependencies]
//! music = { git = "https://github.com/tiborgats/music" }
//! ```

#![warn(missing_docs)]

#[cfg(feature = "be-portaudio")]
extern crate portaudio;
#[cfg(feature = "be-portaudio")]
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "be-rsoundio")]
extern crate rsoundio;
#[cfg(feature = "be-rsoundio")]
extern crate rb;

extern crate rayon;

/// Contains the basic sound synthesizer routines
pub mod sound;
