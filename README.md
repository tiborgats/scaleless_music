music [![Build Status](https://travis-ci.org/tiborgats/music.svg?branch=master)](https://travis-ci.org/tiborgats/music)
=====
It is an experimental project for making scaleless music.
# About
IMHO:
* Music should only contain [harmonic](https://en.wikipedia.org/wiki/Harmony) relations between notes.
* All of the harmonic relation variations should be possible to create.

The realization of these conditions is mathematically impossible with any musical [scale](https://en.wikipedia.org/wiki/Scale_(music)), therefore real music should be scaleless. On "scaleless" I mean the frequency of the notes are not chosen from a fixed set (= scale), instead they are calculated dynamically in relation of the previous notes.

For example: the popular [equal temperament](https://en.wikipedia.org/wiki/Equal_temperament) contains only one type of harmonic relation, the octave, all the other relations are disharmonic ones in it.

See also: [just intonation](https://en.wikipedia.org/wiki/Just_intonation)

# Installation
**music** is built using cargo, so just type `cargo build` at the root of the **music** repository.
You can build the documentation (as soon as I will have one) with `cargo doc`.

# Development
:construction: It is in very early stage yet! Temporarily generates some sounds if you press some keys on the keyboard (Q..P, A..L).
As soon as the basic structure of it will become stable, I will convert it to be a crate and some examples.

Tasks:
- [ ] I need time and money (for living)
- [ ] proper documentation :book:
- [ ] effects, building blocks of music structure
- [ ] handling some low-delay input devices (eg. midi keyboards)
- [ ] file format, parser
- [ ] GUI
- [ ] 3D audio space, instrument location effects
- [ ] supporting other sound interfaces (besides **rust-portaudio**)
- [ ] other stuff
