music [![Build Status](https://travis-ci.org/tiborgats/music.svg?branch=master)](https://travis-ci.org/tiborgats/music)
=====
It is an experimental project for making scaleless music.

# Concept
The aim of this software is to create music according to the following perfectionist principles:
* Music should only contain [harmonic](https://en.wikipedia.org/wiki/Harmony) intervals between notes. (Not any out of tune notes.)
* All of the harmonic interval variations should be possible to be created. (Including harmonies which are missing from the [chromatic scale](https://en.wikipedia.org/wiki/Chromatic_scale))

The realization of these conditions is mathematically impossible with the using of any kind of musical [scale](https://en.wikipedia.org/wiki/Scale_(music)). In this new "scaleless" concept, the frequency of the notes are not chosen from a fixed set (or scale), instead they are calculated dynamically in relation of the previous notes.

Counterexample: the popular [equal temperament](https://en.wikipedia.org/wiki/Equal_temperament) contains only one type of harmonic interval: the octave, all the other frequency intervals are disharmonic ones in it (multiplies of ¹²√2). For a more detailed understandig of the problem of equal temperament and musical scales in general, you can read about the [just intonation](https://en.wikipedia.org/wiki/Just_intonation) approach.

## Sound synthesis
Basically an [additive synthesis](https://en.wikipedia.org/wiki/Additive_synthesis) is used, with an additional rule:
* Frequency can be time-varying, but the intervals must remain harmonic.

Note: this kind of synthesis is very resource hungry. So, for real-time sound generation smaller sample rate (eg. 48kHz) and lower number of overtones are desirable (to prevent buffer underrun). This can change later, after speed optimization of the algorithm.

Another planned option will be the [sample-based synthesis](https://en.wikipedia.org/wiki/Sample-based_synthesis). But it must be used carefully:
* It can contain noise and disharmonic intervals.
* It can contain echoes, effects of the recording space (which interfere the space we want to add to it).

Henceforward, I plan to create a tool for analyzing recorded samples, finding closest mathematical representation, and building harmonic sound structures with similar output. This way we can eliminate noise and we have the option to use precise 3D spacial effects.

# Installation
**music** is built using cargo, so just type `cargo build` at the root of the **music** repository. Currently the project was tested only under Linux. If for some reason the building of [rust-portaudio](https://github.com/RustAudio/rust-portaudio) fails, you can check it's [README](https://github.com/RustAudio/rust-portaudio/blob/master/README.md) for further instructions.

# Development, plans
:construction: It is in very early stage yet!

Tasks:
- [ ] documentation
- [ ] basic effects, building blocks of music structure
	- [x] note
	- [ ] amplitude functions
		- [x] [exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
		- [ ] [tremolo](https://en.wikipedia.org/wiki/Tremolo) - as sinusoidal variation of amplitude
		- [ ] [equal-loudness contour](https://en.wikipedia.org/wiki/Equal-loudness_contour)
	- [ ] frequency functions
		- [ ] [vibrato](https://en.wikipedia.org/wiki/Vibrato)
		- [ ] linearly changing (ascending or descending) pitch
	- [ ] sequences of notes, rhythm functions
	- [ ] polyphony
	- [ ] smooth start and end of sine waves, which are physically possible (no zero time jump in amplitude, or infinite acceleration of speaker membrane)
- [ ] speed optimization of the playback routine, benchmark application
	- [ ] parallel processing
	- [ ] GPU (I did some benchmarks using the [ArrayFire](https://github.com/arrayfire/arrayfire-rust) library, it is very promising, but there is a long latency due to the buffer copying from GPU to CPU memory)
	- [ ] fast `.sin()` algorithm (using lookup table) - but I am not sure if it is necessary at all, as GPU is fast enough
- [ ] proper error handling: `.unwrap()` is not acceptable
- [ ] handling some low-delay input devices
	- [ ] midi keyboard
	- [ ] developing a custom input device, where keys represent intervals instead of frequencies
- [ ] file format, parser
- [ ] GUI
- [ ] 3D audio space, instrument location effects
- [x] [rust-portaudio](https://github.com/RustAudio/rust-portaudio) backend
- [ ] [rsoundio](https://github.com/klingtnet/rsoundio) backend
- [ ] support for Windows, OS X, Android, iOS
- [ ] editor, with a correct, user friendly visual representation of music structure (not sheet music)
- [ ] converter from chromatic scale (eg. midi file) formats to scaleless structure
- [ ] a software for finding the closest mathematical representation of sound samples (from real instruments)
- [ ] a lot of other stuff
