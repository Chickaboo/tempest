//! The whole algorithm: a span of music is either held, or cut at its strongest
//! beats and unfolded on either side — each new tone chosen, last first, to lead
//! into the one that follows it.
//!
//! Start with a single span from the opening tonic to the final one and this
//! writes a piece. Harmony, melody and accompaniment are the same recursion over
//! three different kinds of tone.

use crate::form::{Form, Time};

/// A tone sounding from a moment in time.
pub type Point<T> = (Time, T);

/// A tone held from one moment to another.
pub type Span<T> = (Time, Time, T);

pub trait Voice {
    type Tone: Copy + PartialEq;

    /// Whether `tone`, sounding over `lo..hi`, is elaborated rather than held.
    fn moves(&self, lo: Time, hi: Time, tone: Self::Tone) -> bool;

    /// The tone to sound at `t` on the way from `from` to `to`.
    fn lead(&self, from: Point<Self::Tone>, t: Time, to: Point<Self::Tone>) -> Self::Tone;
}

/// Unfolds the span that starts on `from` and is bound for `to`.
pub fn unfold<V: Voice>(voice: &V, form: &Form, from: Point<V::Tone>, to: Point<V::Tone>, out: &mut Vec<Span<V::Tone>>) {
    let cuts = if voice.moves(from.0, to.0, from.1) { form.cuts(from.0, to.0) } else { vec![] };
    let mut path = vec![to];
    for &t in cuts.iter().rev() {
        path.push((t, voice.lead(from, t, path[path.len() - 1])));
    }
    path.push(from);
    path.reverse();
    match path.len() {
        2 => out.push((from.0, to.0, from.1)),
        _ => path.windows(2).for_each(|step| unfold(voice, form, step[0], step[1], out)),
    }
}

/// Joins a tone to its repetition, except across a barline, where it is struck again.
pub fn tie<T: PartialEq>(spans: Vec<Span<T>>, bar: Time) -> Vec<Span<T>> {
    let mut tied: Vec<Span<T>> = Vec::with_capacity(spans.len());
    for span in spans {
        match tied.last_mut() {
            Some(last) if last.2 == span.2 && span.0 % bar != 0 => last.1 = span.1,
            _ => tied.push(span),
        }
    }
    tied
}
