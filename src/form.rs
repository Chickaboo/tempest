//! Time: a hierarchy of beats, a tree of ideas, and one curve of energy.

use std::f64::consts::PI;

/// Musical time, counted in eighth notes.
pub type Time = i32;

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

pub struct Form {
    pub seed: u64,
    /// The length of an idea at each depth of the tree, from the piece down to the bar.
    pub spans: Vec<Time>,
    /// How the ideas at each depth are built from smaller ones: which parts state
    /// something (0), and which echo, contrast or return.
    pub shapes: Vec<Vec<u64>>,
    /// Every metric unit, finest to coarsest: eighth, beat, …, bar, …, piece.
    grid: Vec<Time>,
}

impl Form {
    /// `meter` lists how a bar divides, coarsest first: `[2, 2, 2]` is 4/4, `[2, 3]` is 6/8.
    pub fn new(seed: u64, meter: &[Time]) -> Self {
        let mut form = Form { seed, spans: vec![1], shapes: vec![], grid: vec![1] };
        meter.iter().rev().for_each(|n| form.grow(*n));
        form.spans = vec![form.end()];
        for shape in [vec![0, 1], vec![0, 0, 1, 0], vec![0, 0, 1, 0]] {
            form.grow(shape.len() as Time);
            form.spans.insert(0, form.end());
            form.shapes.insert(0, shape);
        }
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

    pub fn end(&self) -> Time {
        self.grid[self.grid.len() - 1]
    }

    /// How many levels of the metric hierarchy `t` is a downbeat of.
    pub fn weight(&self, t: Time) -> usize {
        self.grid.iter().take_while(|unit| t % *unit == 0).count()
    }

    /// The longest metric unit that begins at `t`.
    pub fn stride(&self, t: Time) -> Time {
        self.grid[self.weight(t) - 1]
    }

    /// The strongest beats strictly inside a span: where it naturally divides.
    pub fn cuts(&self, lo: Time, hi: Time) -> Vec<Time> {
        let inside = |unit: &Time| ((lo / unit + 1) * unit..hi).step_by(*unit as usize).collect::<Vec<_>>();
        self.grid.iter().rev().map(inside).find(|cuts| !cuts.is_empty()).unwrap_or_default()
    }

    /// The part its parent gives the idea sounding at `t`, `depth` levels down:
    /// 0 for a statement and all its returns, 1 for the contrast.
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
    /// and how much the whole outweighs its parts, is the piece's own.
    pub fn energy(&self, t: Time) -> f64 {
        let (crest, whole) = (1.0 + unit(mix(self.seed, 5)), 0.4 + 0.2 * unit(mix(self.seed, 6)));
        let arch = |span: &Time| (PI * ((t % span) as f64 / *span as f64).powf(crest)).sin();
        let shares = [whole, 0.6 * (1.0 - whole), 0.4 * (1.0 - whole)];
        shares.iter().zip(&self.spans).map(|(share, span)| share * arch(span)).sum()
    }
}
