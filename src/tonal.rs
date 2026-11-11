//! Pitch: one diatonic collection, the triads that live in it, and how they pull on each other.
//!
//! A tone is a scale degree counted from the bottom of the keyboard, seven to the octave.
//! Major and minor are the same collection heard from a different centre (degree 0 or 5).

const SCALE: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Semitones from degree 0 up to `degree`.
pub fn semitones(degree: i32) -> i32 {
    12 * degree.div_euclid(7) + SCALE[degree.rem_euclid(7) as usize]
}

/// The degree whose pitch lies closest to `semitones`.
pub fn degree_near(semitones: i32) -> i32 {
    (semitones as f64 * 7.0 / 12.0).round() as i32
}

/// The size in semitones of the interval `steps` degrees above `root`.
fn interval(root: i32, steps: i32) -> i32 {
    semitones(root + steps) - semitones(root)
}

/// A triad's fifth must be perfect for it to stand on its own.
pub fn consonant(root: i32) -> bool {
    interval(root, 4) == 7
}

/// How strongly one root wants to move to another, by the interval it rises:
/// falling a fifth is the engine of tonal harmony; standing still is no motion at all.
pub fn flow(from: i32, to: i32) -> f64 {
    [0.2, 0.5, 0.1, 1.0, 0.35, 0.35, 0.15][(to - from).rem_euclid(7) as usize]
}

#[derive(Clone, Copy, PartialEq)]
pub struct Chord {
    pub root: i32,
    /// Whether the third is raised into a leading tone, making the chord a dominant.
    pub leading: bool,
}

impl Chord {
    /// A minor triad that falls a fifth into a consonance can become its dominant.
    pub fn leading_into(root: i32, next: i32) -> Chord {
        let dominant = (next - root).rem_euclid(7) == 3 && interval(root, 2) == 3 && consonant(root) && consonant(next);
        Chord { root, leading: dominant }
    }

    pub fn contains(self, degree: i32) -> bool {
        [0, 2, 4].contains(&(degree - self.root).rem_euclid(7))
    }

    /// The pitch of a degree as this chord colours it.
    pub fn pitch(self, degree: i32) -> i32 {
        semitones(degree) + (self.leading && (degree - self.root).rem_euclid(7) == 2) as i32
    }

    pub fn quality(self) -> &'static str {
        match (interval(self.root, 2) + self.leading as i32, interval(self.root, 4)) {
            (4, _) => "",
            (_, 7) => "m",
            _ => "°",
        }
    }
}
