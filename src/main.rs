//! tempest — solo piano music unfolded from a single seed.
//!
//!     tempest [SEED] [OUT.mid]

mod form;
mod midi;
mod perform;
mod piece;
mod tonal;
mod unfold;
mod voices;

use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs::File};

const NAMES: [&str; 12] = ["C", "D♭", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B"];

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let now = || SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64 % 1_000_000);
    let seed = args.next().and_then(|arg| arg.parse().ok()).unwrap_or_else(now);
    let path = args.next().unwrap_or_else(|| format!("tempest-{seed}.mid"));

    let piece = piece::compose(seed);
    midi::write(File::create(&path)?, perform::perform(&piece))?;

    let name = |degree: i32| NAMES[(piece.key + tonal::semitones(degree)).rem_euclid(12) as usize];
    let (beats, unit) = piece.meter.signature;
    let mode = if piece.home == 0 { "major" } else { "minor" };
    let mut report = format!("Tempest No. {seed} — {} {mode}, {beats}/{unit}", name(piece.home));
    let bar = piece.form.bar();
    for (start, chord) in &piece.chords.0 {
        let lead = if start % (8 * bar) == 0 { "\n  " } else if start % bar == 0 { "  " } else { " " };
        report += &format!("{lead}{}{}", name(chord.root), chord.quality());
    }
    io::stdout().write_all(format!("{report}\n→ {path}\n").as_bytes())
}
