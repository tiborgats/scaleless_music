//! Create scaleless music.
//!
//! # Usage
//!
//! Add **scaleless_music** to your project by adding the dependency to your Cargo.toml as follows:
//!
//! ```toml
//! [dependencies]
//! music = { git = "https://github.com/tiborgats/scaleless_music" }
//! ```
//!
//!
//! Building the documentation:
//!
//! ```bash
//! cargo doc --no-deps --features "be-portaudio be-rsoundio"
//! ```
//!
#![doc(html_root_url = "https://tiborgats.github.io/music/")]

#![forbid(bad_style, exceeding_bitshifts, mutable_transmutes, no_mangle_const_items,
    unknown_crate_types, warnings)]
#![deny(deprecated, drop_with_repr_extern, improper_ctypes, missing_docs,
    non_shorthand_field_patterns, overflowing_literals, plugin_as_library,
    private_no_mangle_fns, private_no_mangle_statics, stable_features, unconditional_recursion,
    unknown_lints, unsafe_code, unused, unused_allocation, unused_attributes,
    unused_comparisons, unused_features, unused_parens, while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
    unused_qualifications, unused_results)]

// #![feature(question_mark)]

#[cfg(feature = "be-portaudio")]
extern crate portaudio;
#[cfg(feature = "be-rsoundio")]
extern crate rsoundio;
#[cfg(feature = "be-rsoundio")]
extern crate rb;

extern crate num;

/// Basic sound synthesizer routines.
pub mod sound;
