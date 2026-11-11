//! Performance: the same energy that shaped the notes shapes how they are played —
//! louder and quicker as it rises, softer and slower as it falls.

use crate::form::{mix, unit};
use crate::midi::{Track, TICKS_PER_QUARTER};
use crate::piece::{Hand, Piece};

const EIGHTH: u32 = TICKS_PER_QUARTER / 2;

pub fn perform(piece: &Piece) -> Vec<Track> {
    let form = &piece.form;
    let (end, close) = (form.end() as u32, (form.end() + form.bar()) as u32);
    let fermata = close * EIGHTH + 2 * TICKS_PER_QUARTER;

    let mut conductor = Track::named("Tempest");
    let (beats, unit_note) = form.signature();
    conductor.meta(0, 0x58, &[beats as u8, unit_note.trailing_zeros() as u8, 24, 8]);
    for t in 0..close {
        conductor.tempo(t * EIGHTH, piece.pace / 2.0 * (0.82 + 0.26 * form.energy(t as i32)));
    }

    let (mut right, mut left) = (Track::named("Right hand"), Track::named("Left hand"));
    for note in &piece.notes {
        let energy = form.energy(note.start);
        let accent = (form.stride(note.start).min(form.bar()) as f64).log2();
        let touch = 6.0 * unit(mix(form.seed, (note.start * 128 + note.pitch) as u64)) - 3.0;
        let (track, velocity) = match note.hand {
            Hand::Right => (&mut right, 44.0 + 44.0 * energy + 2.5 * accent + 0.25 * (note.pitch - 60) as f64),
            Hand::Left => (&mut left, 28.0 + 30.0 * energy + 2.0 * accent - 0.2 * (note.pitch - 36) as f64),
        };
        // The final chord is rolled from the bottom up, and held.
        let (start, stop) = (note.start as u32 * EIGHTH, note.end as u32 * EIGHTH - 10);
        let (start, stop) = if note.start as u32 == end { (start + 5 * (note.pitch as u32 - 21), fermata) } else { (start, stop) };
        track.note(start, stop, note.pitch as u8, (velocity + touch).clamp(1.0, 127.0) as u8);
    }

    // The pedal is changed with every chord, and held under the last.
    for &(start, _) in &piece.chords.0 {
        left.pedal(start as u32 * EIGHTH, false);
        left.pedal(start as u32 * EIGHTH + 24, true);
    }
    left.pedal(fermata, false);
    vec![conductor, right, left]
}
