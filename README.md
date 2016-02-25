music [![Build Status](https://travis-ci.org/tiborgats/music.svg?branch=master)](https://travis-ci.org/tiborgats/music)
=====
It is an experimental project for making scaleless music.

# Concept
The aim of this software is to create music according to the following perfectionist principles:
* Music should only contain [harmonic](https://en.wikipedia.org/wiki/Harmony) intervals between notes. (No noise, no out of tune notes.)
* All of the harmonic interval variations should be possible to create. (Including harmonies which are missing from the [chromatic scale](https://en.wikipedia.org/wiki/Chromatic_scale))

The realization of these conditions is mathematically impossible with any musical [scale](https://en.wikipedia.org/wiki/Scale_(music)), therefore a correct precise music creation software should be scaleless. On "scaleless" I mean that the frequency of the notes are not chosen from a fixed set (= scale), instead they are calculated dynamically in relation of the previous notes.

Counterexample: the popular [equal temperament](https://en.wikipedia.org/wiki/Equal_temperament) contains only one type of harmonic interval: the octave, all the other frequency intervals are disharmonic ones in it (multiplies of ¹²√2). The baby-toy manufacturers even developed it further to achieve the most annoying disharmonies.

See also: [just intonation](https://en.wikipedia.org/wiki/Just_intonation)

## Sound synthesis
Basically an [additive synthesis](https://en.wikipedia.org/wiki/Additive_synthesis) is used, with an additional rule:
* Frequency can be time-varying, but the intervals must remain harmonic.

Note: this kind of synthesis is very resource hungry. So, for real-time sound generation smaller sample rate (eg. 48kHz) and lower number of overtones are desirable (to prevent buffer underrun). This can change later, after speed optimiztion of the algorithm.

[Sample-based synthesis](https://en.wikipedia.org/wiki/Sample-based_synthesis) is not an option. It was ruled out for the following reasons:
* It can contain noise and disharmonic intervals.
* It can contain echoes, effects of the recording space.

Later I plan to create a tool for building harmonic sound structures from recorded samples.

# Installation
**music** is built using cargo, so just type `cargo build` at the root of the **music** repository. Currently only Linux is supported.
You can build the documentation (as soon as I will have one) with `cargo doc`.

# Development
:construction: It is in very early stage yet! Temporarily generates some sounds if you press some keys on the keyboard (Q..P, A..L).
As soon as the basic structure of it will become stable, I will convert it to be a crate and some examples.

Tasks:
- [ ] I need time and money to be able to work on it effectively
- [ ] documentation :book:
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
	- [ ] fast `.sin()` algorithm (using lookup table)
	- [ ] fast `.exp()` algorithm
- [ ] proper error handling: `.unwrap()` is not acceptable
- [ ] handling some low-delay input devices
	- [ ] midi keyboard
	- [ ] developing a custom input device, where keys represent intervals instead of frequencies
- [ ] file format, parser
- [ ] GUI
- [ ] 3D audio space, instrument location effects
- [ ] supporting other sound interfaces (besides [rust-portaudio](https://github.com/RustAudio/rust-portaudio))
- [ ] support for Windows, Android, iOS
- [ ] editor, with a correct, user friendly visual representation of music structure (not sheet music)
- [ ] converter from chromatic scale (eg. midi file) formats to scaleless structure
- [ ] a software for finding the closest mathematical representation of sound samples (from real instruments)
- [ ] a lot of other stuff
