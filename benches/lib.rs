#![feature(test)]

extern crate test;
extern crate music;

use test::Bencher;
use music::sound::*;
use std::rc::Rc;

const BENCH_SAMPLE_RATE: SampleCalc = 192_000.0;
const BENCH_BUFFER_SIZE: usize = 256;
const BENCH_BUFFER_TIME: SampleCalc = BENCH_BUFFER_SIZE as SampleCalc / BENCH_SAMPLE_RATE;

#[bench]
fn math_sin(bencher: &mut Bencher) {
    let mut rad: f32 = 0.0;
    let mut s: f32 = 0.0;

    bencher.iter(|| {
        rad += 0.001;
        s = rad.sin();
        test::black_box(s);
    });
}

// FrequencyConst
#[bench]
fn freqconst(bencher: &mut Bencher) {
    let mut frequency_buffer: Vec<SampleCalc> = vec![440.0; BENCH_BUFFER_SIZE];
    let frequency = FrequencyConst::new(440.0).unwrap();

    bencher.iter(|| {
        frequency.get(0.0, None, &mut frequency_buffer).unwrap();
    });
}

// FrequencyConst + Vibrato
#[bench]
fn freqconst_vibrato(bencher: &mut Bencher) {
    let mut tempo_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut frequency_buffer: Vec<SampleCalc> = vec![440.0; BENCH_BUFFER_SIZE];
    let tempo = Tempo::new(120.0).unwrap();
    tempo.get_beats_per_second(0.0, &mut tempo_buffer);
    let frequency = FrequencyConst::new(440.0).unwrap();
    let mut vibrato = Vibrato::new(BENCH_SAMPLE_RATE, NoteValue::new(1, 4).unwrap(), 1.125)
        .unwrap();

    bencher.iter(|| {
        frequency.get(0.0, None, &mut frequency_buffer).unwrap();
        vibrato.apply(&tempo_buffer, &mut frequency_buffer).unwrap();
    });
}

#[bench]
fn tremolo(bencher: &mut Bencher) {
    let mut tempo_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut amplitude_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let tempo = Tempo::new(120.0).unwrap();
    tempo.get_beats_per_second(0.0, &mut tempo_buffer);
    let mut amplitude_rhythm = Tremolo::new_with_tempo(BENCH_SAMPLE_RATE,
                                                       NOTEVALUE_MAX,
                                                       NoteValue::new(1, 4).unwrap(),
                                                       1.2)
        .unwrap();

    bencher.iter(|| {
        amplitude_rhythm.get_rhythmic(&tempo_buffer, &mut amplitude_buffer).unwrap();
    });
}

// AmplitudeConstOvertones
#[bench]
fn ampconst_overtone(bencher: &mut Bencher) {
    let mut amplitude_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut time: SampleCalc = 0.0;
    let amplitude = {
        let overtones_amplitude: Vec<SampleCalc> = vec![1.0, 0.5];
        AmplitudeConstOvertones::new(overtones_amplitude).unwrap()
    };

    bencher.iter(|| {
        amplitude.get(time, 0, &mut amplitude_buffer).unwrap();
        time += BENCH_BUFFER_TIME;
    });
}

// AmplitudeDecayExpOvertones
#[bench]
fn ampdec_overtone(bencher: &mut Bencher) {
    let mut amplitude_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut time: SampleCalc = 0.0;
    let amplitude = {
        let overtones_amplitude: Vec<SampleCalc> = vec![1.0, 0.5];
        let overtones_dec_rate: Vec<SampleCalc> = vec![-1.0, -2.0];
        AmplitudeDecayExpOvertones::new(BENCH_SAMPLE_RATE, overtones_amplitude, overtones_dec_rate)
            .unwrap()
    };

    bencher.iter(|| {
        amplitude.get(time, 0, &mut amplitude_buffer).unwrap();
        time += BENCH_BUFFER_TIME;
    });
}

// FrequencyConst, Timbre{ AmplitudeDecayExpOvertones with 16 overtones }
#[bench]
fn timbre_freqconst_ampdec_overtones16(bencher: &mut Bencher) {
    let mut generator_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut frequency_buffer: Vec<SampleCalc> = vec![440.0; BENCH_BUFFER_SIZE];
    let mut time: SampleCalc = 0.0;
    let frequency = FrequencyConst::new(440.0).unwrap();
    let amplitude = {
        let overtones_amplitude: Vec<SampleCalc> = vec![10.0, 1.0, 1.0, 0.95, 0.9, 0.9, 0.86,
                                                        0.83, 0.80, 0.78, 0.76, 0.74, 0.73, 0.72,
                                                        0.71, 0.70];
        let overtones_dec_rate: Vec<SampleCalc> = vec![-1.0, -1.4, -1.9, -2.1, -2.4, -3.0, -3.5,
                                                       -3.7, -3.8, -4.0, -4.2, -4.4, -4.8, -5.3,
                                                       -6.1, -7.0];
        AmplitudeDecayExpOvertones::new(BENCH_SAMPLE_RATE, overtones_amplitude, overtones_dec_rate)
            .unwrap()
    };
    let timbre = Timbre::new(BENCH_SAMPLE_RATE, BENCH_BUFFER_SIZE, Rc::new(amplitude), 16).unwrap();

    bencher.iter(|| {
        frequency.get(time, None, &mut frequency_buffer).unwrap();
        timbre.get(time, &frequency_buffer, &mut generator_buffer).unwrap();
        time += BENCH_BUFFER_TIME;
        // test::black_box(&mut generator_buffer);
    });
}

// FrequencyConst, Timbre{ AmplitudeDecayExpOvertones with 4 overtones }
#[bench]
fn timbre_freqconst_ampdec_overtones4(bencher: &mut Bencher) {
    let mut generator_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut frequency_buffer: Vec<SampleCalc> = vec![440.0; BENCH_BUFFER_SIZE];
    let mut time: SampleCalc = 0.0;
    let frequency = FrequencyConst::new(440.0).unwrap();
    let amplitude = {
        let overtones_amplitude: Vec<SampleCalc> = vec![10.0, 1.0, 1.0, 0.95];
        let overtones_dec_rate: Vec<SampleCalc> = vec![-1.0, -1.4, -1.9, -2.1];
        AmplitudeDecayExpOvertones::new(BENCH_SAMPLE_RATE, overtones_amplitude, overtones_dec_rate)
            .unwrap()
    };
    let timbre = Timbre::new(BENCH_SAMPLE_RATE, BENCH_BUFFER_SIZE, Rc::new(amplitude), 4).unwrap();

    bencher.iter(|| {
        frequency.get(time, None, &mut frequency_buffer).unwrap();
        timbre.get(time, &frequency_buffer, &mut generator_buffer).unwrap();
        time += BENCH_BUFFER_TIME;
        // test::black_box(&mut generator_buffer);
    });
}

// FrequencyConst, Mixer{4 x Timbre{ AmplitudeDecayExpOvertones with 4 overtones }}
#[bench]
fn mixer4_timbre_freqconst_ampdec_overtones4(bencher: &mut Bencher) {
    let mut generator_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut frequency_buffer: Vec<SampleCalc> = vec![440.0; BENCH_BUFFER_SIZE];
    let mut time: SampleCalc = 0.0;
    let frequency = FrequencyConst::new(440.0).unwrap();
    let amplitude = Rc::new({
        let overtones_amplitude: Vec<SampleCalc> = vec![10.0, 1.0, 1.0, 0.95];
        let overtones_dec_rate: Vec<SampleCalc> = vec![-1.0, -1.4, -1.9, -2.1];
        AmplitudeDecayExpOvertones::new(BENCH_SAMPLE_RATE, overtones_amplitude, overtones_dec_rate)
            .unwrap()
    });
    let timbre1 = Rc::new(Timbre::new(BENCH_SAMPLE_RATE, BENCH_BUFFER_SIZE, amplitude.clone(), 4)
        .unwrap());
    let mixer = Mixer::new(BENCH_BUFFER_SIZE).unwrap();
    mixer.add(Interval::new(1, 1).unwrap(), timbre1.clone(), 2.0)
        .unwrap()
        .add(Interval::new(1, 2).unwrap(), timbre1.clone(), 3.0)
        .unwrap();

    bencher.iter(|| {
        frequency.get(time, None, &mut frequency_buffer).unwrap();
        mixer.get(time, &frequency_buffer, &mut generator_buffer).unwrap();
        time += BENCH_BUFFER_TIME;
        // test::black_box(&mut generator_buffer);
    });
}
