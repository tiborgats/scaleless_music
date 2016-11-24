# Examples
Both of them are under construction, will be changed later.

## `instrument1`
Build with PortAudio backend:

`cargo run --example instrument1 --features "be-portaudio"`

Build with SDL2 backend:

`cargo run --example instrument1 --features "be-sdl2"`



The keys from <kbd>Q</kbd> to <kbd>O</kbd> changes the frequency to be higher, the keys from <kbd>A</kbd> to <kbd>L</kbd> changes the frequency to be lower. Other keys play the previous frequency. To quit press <kbd>Esc</kbd>.




## `instrument_overtone`
Build with PortAudio backend:

`cargo run --example instrument_overtone --features "be-portaudio"`

Build with SDL2 backend:

`cargo run --example instrument_overtone --features "be-sdl2"`



The keys from <kbd>Q</kbd> to <kbd>P</kbd> produces half wave resonances, the keys from <kbd>A</kbd> to <kbd>L</kbd> makes full wave resonances. To quit press <kbd>Esc</kbd>.