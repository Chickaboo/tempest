//! A piece: one seed, unfolded voice by voice.

use crate::form::{mix, temper, Form, Time};
use crate::tonal::{degree_near, Chords};
use crate::unfold::{tie, unfold, Span};
use crate::voices::{Bass, Harmony, Melody};

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
    /// The pitch class of degree 0.
    pub key: i32,
    /// The tonal centre: degree 0 (major) or 5 (minor).
    pub home: i32,
    /// Eighth notes per minute, before rubato.
    pub pace: f64,
    /// The harmony, the last chord being the final tonic.
    pub chords: Chords,
    pub notes: Vec<Note>,
}

pub fn compose(seed: u64) -> Piece {
    let choose = |salt, n| (mix(seed, salt) % n) as usize;
    let (key, home) = (choose(2, 12) as i32, [0, 5][choose(3, 2)]);

    let form = Form::new(seed);
    let pace = temper(seed, 4, 112.0, 160.0);
    let (bar, end) = (form.bar(), form.end());
    let close = end + bar;

    let chords = Harmony { form: &form, home }.chords();

    // Melody: from a tone of the first chord to the root of the last.
    let melody = Melody { form: &form, chords: &chords, register: degree_near(67 - key) };
    let (first, last) = (melody.settle(0, false), melody.settle(end, true));
    let mut line = vec![];
    unfold(&melody, &form, (0, first), (end, last), &mut line);
    line.push((end, close, last));

    // Accompaniment: from root to root. A span that is held sounds, at once, the
    // rungs it had time to climb: a long chord is an arpeggio that did not unfold.
    let bass = Bass { form: &form, chords: &chords, floor: degree_near(34 - key) };
    let mut figure = vec![];
    unfold(&bass, &form, (0, 0), (end, 0), &mut figure);
    figure.push((end, close, 0));

    let sing = |&(start, end, degree): &Span<i32>| Note { start, end, pitch: key + chords.at(start).1.pitch(degree), hand: Hand::Right };
    let strike = |start, end, rung| {
        let (chord, degree) = bass.sound(start, rung);
        Note { start, end, pitch: key + chord.pitch(degree), hand: Hand::Left }
    };
    let accompany = |&(start, end, rung): &Span<i32>| {
        let reach = (Bass::REACH * (end - start) as f64 / bar as f64) as i32;
        (rung..=(rung + reach).min(Bass::LADDER.len() as i32 - 1)).map(move |rung| strike(start, end, rung))
    };
    let struck = |t: Time| chords.begins(t);
    let notes = tie(line, struck).iter().map(sing).chain(tie(figure, struck).iter().flat_map(accompany)).collect();
    Piece { form, key, home, pace, chords, notes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn pieces() -> impl Iterator<Item = Piece> {
        (0..200).map(compose)
    }

    fn melody(piece: &Piece) -> Vec<&Note> {
        piece.notes.iter().filter(|note| note.hand == Hand::Right).collect()
    }

    fn consonant(piece: &Piece, note: &Note) -> bool {
        let chord = piece.chords.at(note.start).1;
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
    fn every_chord_is_founded_on_its_root() {
        for piece in pieces() {
            for &(start, chord) in &piece.chords.0 {
                let sounded = |note: &&Note| note.hand == Hand::Left && note.start == start;
                let lowest = piece.notes.iter().filter(sounded).map(|note| note.pitch).min().unwrap();
                assert_eq!((lowest - piece.key - chord.pitch(chord.root)) % 12, 0);
            }
        }
    }

    #[test]
    fn every_piece_comes_home() {
        for piece in pieces() {
            let (first, last) = (piece.chords.at(0).1, piece.chords.at(piece.form.end()).1);
            assert!(first == last && last.root == piece.home);
            let close = melody(&piece).pop().unwrap();
            assert_eq!((close.pitch - piece.key - last.pitch(last.root)) % 12, 0);
            assert!(piece.notes.iter().all(|note| (21..=108).contains(&note.pitch) && note.start < note.end));
        }
    }

    /// Floors, not targets: what the music should keep doing however the algorithm changes.
    #[test]
    fn the_music_keeps_its_promises() {
        let (mut cadences, mut steps, mut leaps, mut crossings, mut accompaniment) = (0, 0, 0, 0, 0);
        let (mut centred, mut sections, mut lengths, mut busiest, mut stillest) = (0, 0, HashSet::new(), 0.0f64, 9.0f64);
        for piece in pieces() {
            let (form, harmony) = (&piece.form, Harmony { form: &piece.form, home: piece.home });
            lengths.insert(form.end() / form.bar());
            for start in (0..form.end()).step_by(form.spans[1] as usize) {
                centred += (piece.chords.at(start).1.root == harmony.centre(start)) as usize;
                sections += 1;
            }
            let penultimate = piece.chords.at(piece.form.end() - 1).1;
            cadences += (penultimate.root == (piece.home + 4) % 7 && penultimate.quality().is_empty()) as usize;
            let line = melody(&piece);
            let busyness = line.len() as f64 / form.end() as f64;
            (busiest, stillest) = (busiest.max(busyness), stillest.min(busyness));
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
        let measured = (share(cadences, 200), share(steps, steps + leaps), share(crossings, accompaniment), share(centred, sections));
        println!("cadences {:.3}  steps {:.3}  crossings {:.4}  centred {:.3}  lengths {}", measured.0, measured.1, measured.2, measured.3, lengths.len());
        println!("melody notes per eighth: stillest {stillest:.3}  busiest {busiest:.3}");
        assert!(share(cadences, 200) >= 0.9, "pieces should end dominant to tonic");
        assert!(share(steps, steps + leaps) >= 0.5, "the melody should mostly move by step");
        assert!(share(centred, sections) >= 0.95, "sections should open on their tonal centre");
        assert!(share(crossings, accompaniment) < 0.02, "the left hand should stay under the melody");
        assert!(lengths.len() >= 5, "pieces should differ in form");
        assert!(busiest >= 4.0 * stillest, "pieces should differ in character");
    }
}
