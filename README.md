# Tempest

Solo piano music, composed by one recursive function. No machine learning, no
corpus, no templates, no dependencies — about 300 lines of Rust that write the
music and 100 that play it.

```
cargo run --release -- 13            # writes tempest-13.mid
cargo run --release -- 13 out.mid    # or anywhere you like
cargo run --release                  # a new piece every time
```

Each seed is a different piece: its own key, metre, tempo, themes and harmony.
The output is a standard MIDI file (two hands, pedal, tempo rubato); open it in
any player or notation program. A few are in [`examples/`](examples).

## The idea

**A piece of music is a single cadence — tonic to tonic — unfolded.**

Take a span of time, the tone it starts on, and the tone it is bound for. Either
hold the tone, or cut the span at its strongest beat, choose a tone for that
moment *that leads into what follows*, and unfold both halves the same way.
That is the whole algorithm ([`src/unfold.rs`](src/unfold.rs)):

```rust
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
```

It is called three times on the span from the first downbeat to the last —
once with chords as tones, once with scale degrees, once with the rungs of an
arpeggio — and the results are the harmony, the melody and the accompaniment.
Music is written backwards from its goals: every note exists to arrive
somewhere, so every note is chosen knowing where it is going.

Four ideas make that recursion musical.

### 1. Metre is the recursion tree

A span is cut at the strongest beats inside it (`Form::cuts`). The piece divides
into sections, sections into phrases, phrases into bars, bars into beats, beats
into eighths — two or three ways, depending on the metre. Because structure is
decided top-down, a tone's metric weight *is* its structural importance: chords
change on barlines, stable tones land on strong beats, passing tones fall
between them. Nobody has to say so.

### 2. Identity is a seed

There is no random number generator, only a hash. Every choice is a pure
function of *which idea it belongs to* and *where in that idea it falls*
(`Form::whim`). Ideas form a tree — statement, echo, contrast, return, at every
scale — and echoes share their statement's identity. So when an idea comes
back, every decision inside it comes back too: the same rhythm, the same
contour, re-fitted to whatever harmony it now finds itself over. Motif,
sequence and reprise are not features; they are referential transparency.

### 3. One curve of energy

`Form::energy` is a swell that crests two-thirds of the way through the piece,
through each section and through each phrase — a breath at three scales. It
is the only expressive input. A span divides when its whim falls below the
energy, so a returning idea at higher energy keeps every note it had and grows
new ones between them: variation by embellishment, for free. The same curve
lifts the melody's register, and in performance sets loudness and tempo, so
the music gets higher, busier, louder and faster together, and relaxes
together.

### 4. Tension must resolve

Each voice has one rule for choosing a tone ([`src/voices.rs`](src/voices.rs)):

- **Harmony** is drawn by root motion — falling fifths strongest — pulled harder
  by the chord it leads to than the one it leaves, and harder by near chords
  than far ones. Sections open on their tonic (the contrasting section on the
  relative key's); cadences emerge on their own. A minor chord that falls a
  fifth into the head of a phrase has its third raised into a leading tone.
- **Melody** takes the tone nearest a path between its neighbours. A tone
  outside the chord costs more the longer it lasts, and is allowed only if it
  steps into the next tone — and once chosen it cannot be elaborated, so it
  *must* resolve. That one condition yields passing tones, neighbours and
  appoggiaturas without naming any of them.
- **Accompaniment** climbs a ladder of chord tones spaced like the harmonic
  series (root, fifth, tenth…), founded on the root at every barline. Its whims
  belong to the section, so a texture persists until the music moves on.

## Layout

| file | what it holds |
| --- | --- |
| [`src/unfold.rs`](src/unfold.rs) | the algorithm |
| [`src/form.rs`](src/form.rs) | time: metre, the tree of ideas, the hash, energy |
| [`src/tonal.rs`](src/tonal.rs) | pitch: one scale, its triads, root motion |
| [`src/voices.rs`](src/voices.rs) | harmony, melody, accompaniment |
| [`src/piece.rs`](src/piece.rs) | a seed → notes; tests of the invariants |
| [`src/perform.rs`](src/perform.rs) | notes → dynamics, rubato, pedal |
| [`src/midi.rs`](src/midi.rs) | a Standard MIDI File writer |

`cargo test` composes 200 pieces and checks what the design promises: every
dissonance resolves by step, every bar stands on its root, every piece ends
where it began.

## License

[GPL-3.0](LICENSE)
