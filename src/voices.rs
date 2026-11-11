//! Three voices, each a different answer to the same two questions:
//! does this span move, and what leads from here to there?

use crate::form::{mix, unit, Form, Time};
use crate::tonal::{consonant, fifths, flow, Chord, Chords};
use crate::unfold::{tie, unfold, Point, Voice};

/// A span of time in quarter notes, the scale on which the voices feel duration.
fn quarters(span: Time) -> f64 {
    span as f64 / 2.0
}

/// How readily a span divides: more so the longer it lasts and the higher the energy.
fn density(form: &Form, lo: Time, hi: Time, (rest, drive): (f64, f64)) -> f64 {
    (rest + drive * form.energy(lo)) * quarters(hi - lo).powf(0.7)
}

/// The value at `t` on the straight line from `from` at `lo` to `to` at `hi`.
fn between(from: f64, to: f64, (lo, t, hi): (Time, Time, Time)) -> f64 {
    from + (to - from) * (t - lo) as f64 / (hi - lo) as f64
}

/// The roots of the chords.
pub struct Harmony<'a> {
    pub form: &'a Form,
    /// The tonal centre of the piece: degree 0 for major, 5 for minor.
    pub home: i32,
}

impl Harmony<'_> {
    const DENSITY: (f64, f64) = (0.0, 0.08);
    const GRAVITY: f64 = 0.3; // the appeal a chord keeps, per fifth from the centre, at the head of a section

    /// The tonal centre of the section sounding at `t`. A statement is at home; a
    /// contrast settles on another consonant triad of the scale, one of its nearest keys.
    pub fn centre(&self, t: Time) -> i32 {
        let away: Vec<i32> = (0..7).filter(|x| consonant(*x) && *x != self.home).collect();
        match self.form.role(t, 1) {
            0 => self.home,
            _ => away[(self.form.idea(t, 1) % away.len() as u64) as usize],
        }
    }

    /// The chords of the piece, unfolded from the tonic back to the tonic.
    /// A chord arriving at the head of more than a phrase is led into by a true dominant.
    pub fn chords(&self) -> Chords {
        let (form, mut roots) = (self.form, vec![]);
        unfold(self, form, (0, self.home), (form.end(), self.home), &mut roots);
        let mut roots = tie(roots, |t| t % form.bar() == 0);
        roots.push((form.end(), form.end() + form.bar(), self.home));
        let chord = |i: usize| match roots.get(i + 1) {
            Some(&(t, _, next)) if form.stride(t) > form.spans[form.spans.len() - 2] => Chord::leading_into(roots[i].2, next),
            _ => Chord { root: roots[i].2, leading: false },
        };
        Chords((0..roots.len()).map(|i| (roots[i].0, chord(i))).collect())
    }
}

impl Voice for Harmony<'_> {
    type Tone = i32;

    /// Every bar has its chord; within the bar the harmony quickens with the energy.
    fn moves(&self, lo: Time, hi: Time, _: i32) -> bool {
        let form = self.form;
        hi - lo > form.bar() || form.whim(lo, hi, lo, 0x380 + (hi - lo) as u64) < density(form, lo, hi, Self::DENSITY)
    }

    /// A chord is drawn along by its neighbours, the nearer the stronger, and by
    /// where it is going more than by where it has been. The longer the span it
    /// opens, the harder it is pulled towards the centre of its section.
    fn lead(&self, (lo, a): Point<i32>, t: Time, (hi, b): Point<i32>) -> i32 {
        let form = self.form;
        let centre = self.centre(t);
        let weight = (form.stride(t) as f64 / form.spans[1] as f64).powi(2);
        let pull = |from, to, distance: Time| flow(from, to).powf((form.bar() as f64 / distance as f64).powi(2));
        let appeal = |x: i32| {
            let whim = 1.0 - 0.5 * form.caprice * (1.0 - form.whim(lo, hi, t, 0x300 + x as u64));
            let gravity = Self::GRAVITY.powf(fifths(x, centre) as f64 * weight);
            pull(a, x, t - lo).sqrt() * pull(x, b, hi - t) * gravity * if consonant(x) { whim } else { 0.3 * whim }
        };
        (0..7).max_by(|&x, &y| appeal(x).total_cmp(&appeal(y))).unwrap()
    }
}

/// The singing line, in scale degrees.
pub struct Melody<'a> {
    pub form: &'a Form,
    pub chords: &'a Chords,
    /// The degree the line calls home when the music is at rest.
    pub register: i32,
}

impl Melody<'_> {
    const DENSITY: (f64, f64) = (0.2, 0.5); // how readily a span divides: at rest, and per unit of energy
    const WANDER: f64 = 1.3; //                how far, in degrees, a tone may stray from its path
    const LIFT: f64 = 5.0; //                  how far, in degrees, energy raises the line
    const SPRING: f64 = 0.06; //               the pull back towards the register, per degree squared
    const TENSION: f64 = 0.8; //               the cost of a dissonance, per quarter note it lasts
    const STALE: f64 = 0.4; //                 the cost of repeating a neighbouring tone

    /// The first tone of the chord at `t` at or above the resting register — or, to close, its root.
    pub fn settle(&self, t: Time, close: bool) -> i32 {
        let chord = self.chords.at(t).1;
        let fits = |x: &i32| if close { (x - chord.root).rem_euclid(7) == 0 } else { chord.contains(*x) };
        (self.register..).find(fits).unwrap()
    }
}

impl Voice for Melody<'_> {
    type Tone = i32;

    /// The line answers every change of harmony. Between changes a consonance may be
    /// elaborated; a dissonance is held until it resolves.
    fn moves(&self, lo: Time, hi: Time, tone: i32) -> bool {
        let form = self.form;
        let stirs = || form.whim(lo, hi, lo, 0x100 + (hi - lo) as u64) < density(form, lo, hi, Self::DENSITY);
        self.chords.changes(lo, hi) || self.chords.at(lo).1.contains(tone) && stirs()
    }

    /// The tone nearest a point on the path between its neighbours — a path that rises
    /// where the energy swells, and strays by a whim that belongs to the motif.
    /// A tone outside the chord is welcome only if brief, and only if it resolves by step
    /// before the harmony moves on.
    fn lead(&self, (lo, a): Point<i32>, t: Time, (hi, b): Point<i32>) -> i32 {
        let (form, at) = (self.form, (lo, t, hi));
        let swell = form.energy(t) - between(form.energy(lo), form.energy(hi), at);
        let stray = Self::WANDER * form.caprice * quarters(hi - lo).powf(0.25) * (2.0 * form.whim(lo, hi, t, 0x100) - 1.0);
        let goal = between(a as f64, b as f64, at) + Self::LIFT * swell + stray;
        let home = self.register as f64 + Self::LIFT * form.energy(t).max(0.0);
        let cost = |x: i32| {
            let tension = match (self.chords.at(t).1.contains(x), (x - b).abs()) {
                (true, _) => 0.0,
                (false, 1) if !self.chords.changes(t, hi) => Self::TENSION * quarters(hi - t),
                (false, _) => f64::INFINITY,
            };
            let stale = if x == a || x == b { Self::STALE } else { 0.0 };
            (x as f64 - goal).abs() + Self::SPRING * (x as f64 - home).powi(2) + tension + stale
        };
        let near = goal.round() as i32;
        (near - 7..=near + 7).min_by(|&x, &y| cost(x).total_cmp(&cost(y))).unwrap()
    }
}

/// The accompaniment: rungs of a ladder of chord tones, spaced like the harmonic
/// series — wide at the bottom, close at the top.
pub struct Bass<'a> {
    pub form: &'a Form,
    pub chords: &'a Chords,
    /// The lowest degree a root may take.
    pub floor: i32,
}

impl Bass<'_> {
    pub const LADDER: [i32; 5] = [0, 4, 9, 11, 14]; // root, fifth, tenth, twelfth, fifteenth
    const DENSITY: (f64, f64) = (0.3, 0.8);
    pub const REACH: f64 = 2.6; // how many rungs the figure climbs in half a bar, or a held bar sounds at once
    const STALE: f64 = 0.8; // the cost of repeating a neighbouring rung

    /// A whim that belongs to the section and to a place in the bar, so that every
    /// bar of a section is figured alike.
    fn whim(&self, t: Time, salt: u64) -> f64 {
        unit(mix(mix(self.form.idea(t, 1), (t % self.form.bar()) as u64), salt))
    }

    /// The pitch of a rung of the chord sounding at `t`.
    pub fn pitch(&self, t: Time, rung: i32) -> i32 {
        let chord = self.chords.at(t).1;
        chord.pitch(self.floor + (chord.root - self.floor).rem_euclid(7) + Self::LADDER[rung as usize])
    }
}

impl Voice for Bass<'_> {
    type Tone = i32;

    fn moves(&self, lo: Time, hi: Time, _: i32) -> bool {
        self.chords.changes(lo, hi) || self.whim(lo, (hi - lo) as u64) < density(self.form, lo, hi, Self::DENSITY)
    }

    /// Every chord is founded on its root; between roots the figure arches upward.
    fn lead(&self, (lo, a): Point<i32>, t: Time, (hi, b): Point<i32>) -> i32 {
        let bar = self.form.bar();
        if self.chords.begins(t) {
            return 0;
        }
        let climb = Self::REACH * ((hi - lo) as f64 / bar as f64).sqrt() * (0.4 + 0.6 * self.whim(t, 0));
        let goal = between(a as f64, b as f64, (lo, t, hi)) + climb;
        let cost = |x: i32| (x as f64 - goal).abs() + if x == a || x == b { Self::STALE } else { 0.0 };
        (0..Self::LADDER.len() as i32).min_by(|&x, &y| cost(x).total_cmp(&cost(y))).unwrap()
    }
}
