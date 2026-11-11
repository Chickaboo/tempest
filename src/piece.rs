//! A piece: one seed, unfolded three times.

use crate::form::{mix, unit, Form, Time};
use crate::tonal::{degree_near, Chord};
use crate::unfold::{tie, unfold, Span};
use crate::voices::{Bass, Harmony, Melody};

/// A time signature, and how its bar divides down to the eighth note.
pub struct Meter {
    pub signature: (u8, u8),
    pub divisions: &'static [Time],
}

const METERS: [Meter; 5] = [
    Meter { signature: (4, 4), divisions: &[2, 2, 2] },
    Meter { signature: (3, 4), divisions: &[3, 2] },
    Meter { signature: (6, 8), divisions: &[2, 3] },
    Meter { signature: (9, 8), divisions: &[3, 3] },
    Meter { signature: (12, 8), divisions: &[2, 2, 3] },
];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Hand {
    Left,
    Right,
}

#[derive(PartialEq, Debug)]
pub struct Note {
    pub start: Time,
    pub end: Time,
    pub pitch: i32,
    pub hand: Hand,
}

pub struct Piece {
    pub form: Form,
    pub meter: &'static Meter,
    /// The pitch class of degree 0.
    pub key: i32,
    /// The tonal centre: degree 0 (major) or 5 (minor).
    pub home: i32,
    /// Eighth notes per minute, before rubato.
    pub pace: f64,
    /// One chord per bar, the last being the final tonic.
    pub chords: Vec<Chord>,
    pub notes: Vec<Note>,
}

pub fn compose(seed: u64) -> Piece {
    let choose = |salt, n| (mix(seed, salt) % n) as usize;
    let meter = &METERS[choose(1, METERS.len() as u64)];
    let (key, home) = (choose(2, 12) as i32, [0, 5][choose(3, 2)]);
    let pace = 112.0 + 48.0 * unit(mix(seed, 4));

    let form = Form::new(seed, meter.divisions);
    let (bar, end) = (form.bar(), form.end());
    let close = end + bar;

    let chords = Harmony { form: &form, home }.chords();
    let chord = |t: Time| chords[(t / bar) as usize];

    // Melody: from a tone of the first chord to the root of the last.
    let melody = Melody { form: &form, chords: &chords, register: degree_near(67 - key) };
    let (first, last) = (melody.settle(0, false), melody.settle(end, true));
    let mut line = vec![];
    unfold(&melody, &form, (0, first), (end, last), &mut line);
    line.push((end, close, last));

    // Accompaniment: from root to root, where the whole ladder is left to ring.
    let bass = Bass { form: &form, chords: &chords, floor: degree_near(34 - key) };
    let mut figure = vec![];
    unfold(&bass, &form, (0, 0), (end, 0), &mut figure);
    figure.extend((0..Bass::LADDER.len() as i32).map(|rung| (end, close, rung)));

    let sing = |&(start, end, degree): &Span<i32>| Note { start, end, pitch: key + chord(start).pitch(degree), hand: Hand::Right };
    let accompany = |&(start, end, rung): &Span<i32>| {
        let (chord, degree) = bass.sound(start, rung);
        Note { start, end, pitch: key + chord.pitch(degree), hand: Hand::Left }
    };
    let notes = tie(line, bar).iter().map(sing).chain(tie(figure, bar).iter().map(accompany)).collect();
    Piece { form, meter, key, home, pace, chords, notes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tonal::relative;

    fn pieces() -> impl Iterator<Item = Piece> {
        (0..200).map(compose)
    }

    fn melody(piece: &Piece) -> Vec<&Note> {
        piece.notes.iter().filter(|note| note.hand == Hand::Right).collect()
    }

    fn consonant(piece: &Piece, note: &Note) -> bool {
        let chord = piece.chords[(note.start / piece.form.bar()) as usize];
        (0..3).any(|i| (piece.key + chord.pitch(chord.root + 2 * i) - note.pitch) % 12 == 0)
    }

    #[test]
    fn a_seed_is_a_piece() {
        assert_eq!(compose(42).notes, compose(42).notes);
        assert_ne!(compose(42).notes, compose(43).notes);
    }

    #[test]
    fn every_dissonance_resolves_by_step() {
        for piece in pieces() {
            for pair in melody(&piece).windows(2) {
                let step = (pair[1].pitch - pair[0].pitch).abs();
                assert!(consonant(&piece, pair[0]) || (1..=3).contains(&step), "seed {} at {}", piece.form.seed, pair[0].start);
            }
        }
    }

    #[test]
    fn every_bar_is_founded_on_its_root() {
        for piece in pieces() {
            let bar = piece.form.bar();
            for (i, chord) in piece.chords.iter().enumerate() {
                let downbeat = |note: &&Note| note.hand == Hand::Left && note.start == i as Time * bar;
                let lowest = piece.notes.iter().filter(downbeat).map(|note| note.pitch).min().unwrap();
                assert_eq!((lowest - piece.key - chord.pitch(chord.root)) % 12, 0);
            }
        }
    }

    #[test]
    fn every_piece_comes_home() {
        for piece in pieces() {
            let (first, last) = (piece.chords[0], piece.chords[piece.chords.len() - 1]);
            assert!(first == last && last.root == piece.home);
            let close = melody(&piece).pop().unwrap();
            assert_eq!((close.pitch - piece.key - last.pitch(last.root)) % 12, 0);
            assert!(piece.notes.iter().all(|note| (21..=108).contains(&note.pitch) && note.start < note.end));
        }
    }

    /// Floors, not targets: what the music should keep doing however the algorithm changes.
    #[test]
    fn the_music_keeps_its_promises() {
        let (mut cadences, mut steps, mut leaps, mut crossings, mut accompaniment, mut centred) = (0, 0, 0, 0, 0, 0);
        for piece in pieces() {
            let penultimate = piece.chords[piece.chords.len() - 2];
            cadences += (penultimate.root == (piece.home + 4) % 7 && penultimate.quality().is_empty()) as usize;
            let opens = |section: usize| piece.chords[8 * section].root == [piece.home, piece.home, relative(piece.home), piece.home][section];
            centred += (0..4).filter(|&section| opens(section)).count();
            let line = melody(&piece);
            for pair in line.windows(2) {
                let step = (pair[1].pitch - pair[0].pitch).abs() <= 2;
                steps += step as usize;
                leaps += !step as usize;
            }
            assert!(line.iter().all(|note| (55..=96).contains(&note.pitch)), "seed {} leaves its register", piece.form.seed);
            for low in piece.notes.iter().filter(|note| note.hand == Hand::Left) {
                let under = |high: &&Note| high.start <= low.start && low.start < high.end && high.pitch <= low.pitch;
                crossings += line.iter().any(under) as usize;
                accompaniment += 1;
            }
        }
        let share = |part: usize, whole: usize| part as f64 / whole as f64;
        println!("cadences {:.3}  steps {:.3}  crossings {:.4}  centred {:.3}", share(cadences, 200), share(steps, steps + leaps), share(crossings, accompaniment), share(centred, 800));
        assert!(share(cadences, 200) >= 0.9, "pieces should end dominant to tonic");
        assert!(share(steps, steps + leaps) >= 0.5, "the melody should mostly move by step");
        assert!(share(centred, 800) >= 0.95, "sections should open on their tonal centre");
        assert!(share(crossings, accompaniment) < 0.02, "the left hand should stay under the melody");
    }
}
