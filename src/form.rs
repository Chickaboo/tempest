//! Time: a hierarchy of beats, a tree of ideas, and one curve of energy.

use crate::unfold::{unfold, Point, Voice};
use std::f64::consts::PI;

/// Musical time, counted in eighth notes.
pub type Time = i32;

/// Eighth notes gather into beats, and beats into longer beats, until one lasts this long: a bar.
const BAR: Time = 6;

/// Bars gather into ideas, and ideas into larger ideas, until one lasts this long: the piece.
const LENGTH: Time = 150;

/// SplitMix64: a stateless hash, and the piece's only source of chance.
pub fn mix(seed: u64, salt: u64) -> u64 {
    let mut z = seed ^ salt.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A hash as a number in `[0, 1)`.
pub fn unit(seed: u64) -> f64 {
    (seed >> 11) as f64 / (1u64 << 53) as f64
}

/// A trait of a piece: a number between `low` and `high` that belongs to its seed.
pub fn temper(seed: u64, salt: u64, low: f64, high: f64) -> f64 {
    low + (high - low) * unit(mix(seed, salt))
}

/// The shape of an idea, unfolded like everything else. It opens with a statement
/// and a statement follows it; each part between two others says something new —
/// most likely where its neighbours are alike — or echoes one of them, more
/// readily the one heard first.
struct Shape(u64);

impl Voice for Shape {
    type Tone = u64;

    fn moves(&self, _: Time, _: Time, _: u64) -> bool {
        true
    }

    fn lead(&self, (_, a): Point<u64>, t: Time, (_, b): Point<u64>) -> u64 {
        let (whim, fresh) = (unit(mix(self.0, t as u64)), if a == b { 0.8 } else { 0.2 });
        if whim < fresh { t as u64 } else if whim < 0.75 + 0.25 * fresh { a.min(b) } else { a.max(b) }
    }
}

pub struct Form {
    pub seed: u64,
    /// The length of an idea at each depth of the tree, from the piece down to the bar.
    pub spans: Vec<Time>,
    /// How the ideas at each depth are built from smaller ones: parts that share a
    /// number share an identity, and 0 is the statement.
    pub shapes: Vec<Vec<u64>>,
    /// Every metric unit, finest to coarsest: eighth, beat, …, bar, …, piece.
    grid: Vec<Time>,
    /// The piece's character: the level its energy rises from, how far it rises,
    /// and how much chance is allowed to matter against the pulls.
    pub tide: f64,
    pub swell: f64,
    pub caprice: f64,
}

impl Form {
    /// One hierarchy of time, every level gathering a few of the level below:
    /// eighths into beats and beats into a bar, then bars into ideas and ideas into the piece.
    pub fn new(seed: u64) -> Self {
        let (tide, swell, caprice) = (temper(seed, 30, -0.6, 0.3), temper(seed, 31, 0.5, 1.1), temper(seed, 32, 0.5, 1.5));
        let mut form = Form { seed, spans: vec![], shapes: vec![], grid: vec![1], tide, swell, caprice };
        while form.end() < BAR {
            form.grow(2 + (mix(seed, 20 + form.grid.len() as u64) % 2) as Time);
        }
        form.spans = vec![form.end()];
        while form.end() < LENGTH {
            let depth = form.shapes.len() as u64;
            let parts = 2 + (mix(seed, 7 + depth) % 3) as Time;
            let mut frame = Form { spans: vec![], shapes: vec![], grid: vec![1], ..form };
            frame.grow(parts);
            let mut shape = vec![];
            unfold(&Shape(mix(seed, 10 + depth)), &frame, (0, 0), (parts, 0), &mut shape);
            form.grow(parts);
            form.spans.insert(0, form.end());
            form.shapes.insert(0, shape.iter().map(|part| part.2).collect());
        }
        // Every idea is bound for a statement. For the piece itself it must be played.
        form.shapes[0].push(0);
        form.grid.push(form.end() + form.spans[1]);
        form.spans[0] = form.end();
        form
    }

    /// Gathers `n` of the longest unit so far into a longer one; four are two pairs.
    fn grow(&mut self, n: Time) {
        for n in if n == 4 { vec![2, 2] } else { vec![n] } {
            self.grid.push(self.end() * n);
        }
    }

    pub fn bar(&self) -> Time {
        self.spans[self.shapes.len()]
    }

    /// The time signature: eighths over 8 where the beat divides in three, quarters over 4 otherwise.
    pub fn signature(&self) -> (Time, Time) {
        if self.grid[1] == 3 { (self.bar(), 8) } else { (self.bar() / 2, 4) }
    }

    pub fn end(&self) -> Time {
        self.grid[self.grid.len() - 1]
    }

    /// The longest metric unit that begins at `t`: how strong a beat it is.
    pub fn stride(&self, t: Time) -> Time {
        *self.grid.iter().rev().find(|unit| t % *unit == 0).unwrap()
    }

    /// The strongest beats strictly inside a span: where it naturally divides.
    pub fn cuts(&self, lo: Time, hi: Time) -> Vec<Time> {
        let inside = |unit: &Time| ((lo / unit + 1) * unit..hi).step_by(*unit as usize).collect::<Vec<_>>();
        self.grid.iter().rev().map(inside).find(|cuts| !cuts.is_empty()).unwrap_or_default()
    }

    /// The part its parent gives the idea sounding at `t`, `depth` levels down:
    /// 0 for a statement and all its returns, anything else for a contrast.
    pub fn role(&self, t: Time, depth: usize) -> u64 {
        let shape = &self.shapes[depth - 1];
        shape[(t / self.spans[depth]) as usize % shape.len()]
    }

    /// The identity of the idea sounding at `t`, `depth` levels down the tree.
    /// Echoes and returns share an identity, and so everything decided from it.
    pub fn idea(&self, t: Time, depth: usize) -> u64 {
        (1..=depth).fold(self.seed, |seed, d| mix(seed, self.role(t, d)))
    }

    /// A number in `[0, 1)` that is a pure function of the smallest idea spanning
    /// `lo..hi` and of where `t` falls within it — so wherever an idea returns,
    /// every choice made inside it returns too.
    pub fn whim(&self, lo: Time, hi: Time, t: Time, salt: u64) -> f64 {
        let spanned_by = |depth: &usize| lo / self.spans[*depth] == (hi - 1) / self.spans[*depth];
        let depth = (1..self.spans.len()).take_while(spanned_by).last().unwrap_or(0);
        unit(mix(mix(self.idea(lo, depth), (t % self.spans[depth]) as u64), salt))
    }

    /// A swell that rises and falls through the piece, through each section and
    /// through each phrase: the shape of a breath, at three scales. Where it crests,
    /// how much the whole outweighs its parts, the level it rises from and how far
    /// it rises are the piece's own — a still piece and a stormy one differ only here.
    pub fn energy(&self, t: Time) -> f64 {
        let (crest, whole) = (temper(self.seed, 5, 1.0, 2.0), temper(self.seed, 6, 0.4, 0.6));
        let arch = |span: &Time| (PI * ((t % span) as f64 / *span as f64).powf(crest)).sin();
        let shares = [whole, 0.6 * (1.0 - whole), 0.4 * (1.0 - whole)];
        self.tide + self.swell * shares.iter().zip(&self.spans).map(|(share, span)| share * arch(span)).sum::<f64>()
    }
}
