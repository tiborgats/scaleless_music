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

//#![warn(missing_docs)]


extern crate portaudio;
#[macro_use]
extern crate lazy_static;
extern crate rayon;

pub mod sound;
