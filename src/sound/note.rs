use sound::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Musical note.
#[derive(Clone)]
pub struct Note {
    /// The interval of the note's frequency relative to the input frequency buffer.
    interval: Interval,
    /// Time for rhythm syncronization. It usually represents the time needed (from start) for
    /// reaching the initial peak value of the amplitude.
    onset_time: SampleCalc,
    /// Time between onset and release. In ADSR it involves decay.
    sustain: NoteValue,
    /// Time between the onset and the next note's onset.
    duration: NoteValue,
    /// It is used for the syncronization of some effects (e.g. vibrato, tremolo).
    tempo: Tempo,
    /// Sound structure.
    sound: Rc<SoundStructure>,
    volume_relative: SampleCalc,
    volume_normalized: SampleCalc,
    frequency_buffer: RefCell<Vec<SampleCalc>>,
    wave_buffer: RefCell<Vec<SampleCalc>>,
}








/// Sequence of musical notes.
#[doc(hidden)]
#[derive(Clone)]
pub struct NoteSequence {
    buffer_size: usize,
    notes: RefCell<Vec<Note>>,
}

impl NoteSequence {
    /// custom constructor
    pub fn new(buffer_size: usize) -> SoundResult<NoteSequence> {
        Ok(NoteSequence {
            buffer_size: buffer_size,
            notes: RefCell::new(Vec::new()),
        })
    }

    /// Add a new note to the sequence.
    pub fn add(&self,
               // interval: Interval,
               // sound: Rc<SoundStructure>,
               duration: SampleCalc,
               volume: SampleCalc)
               -> SoundResult<&NoteSequence> {
        if duration <= 0.0 {
            return Err(Error::PeriodInvalid);
        }
        if volume < 0.0 {
            return Err(Error::AmplitudeInvalid);
        }
        // let note = Note {
        // interval: interval,
        // onset_time: 0.0,
        // duration: duration,
        // sound: sound,
        // volume_relative: volume,
        // volume_normalized: 0.0,
        // frequency_buffer: RefCell::new(vec![1.0; self.buffer_size]),
        // wave_buffer: RefCell::new(vec![0.0; self.buffer_size]),
        // };
        // self.notes.borrow_mut().push(note);
        self.normalize();
        Ok(self)
    }

    /// Generates the normalized volumes for the channels. Only normalizes if the sum of volumes
    /// is greater than 1.0
    fn normalize(&self) {
        let mut volume_sum: SampleCalc = 0.0;
        for note in self.notes.borrow().iter() {
            volume_sum += note.volume_relative;
        }
        let volume_multiplier = if volume_sum < 1.0 {
            1.0
        } else {
            1.0 / volume_sum
        };
        for note in self.notes.borrow_mut().iter_mut() {
            note.volume_normalized = note.volume_relative * volume_multiplier;
        }
    }
}
