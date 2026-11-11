# Tempest

[Examples](examples) · [The algorithm](src/unfold.rs)

Tempest composes solo piano music from a single number. Give it a seed and it writes a complete piece as a MIDI file, with its own key, metre, tempo, form, themes and harmony.

There is no machine learning in it, no training data and no templates. The music comes from one small recursive function, written in Rust with no dependencies.

## Usage

```
cargo run --release -- 13            # writes tempest-13.mid
cargo run --release -- 13 out.mid    # or choose where it goes
cargo run --release                  # a new piece every time
```

The same seed always gives the same piece. The command prints the key, the metre, the form and character it chose and the chords, so you can follow what you hear. The MIDI file has a track for each hand, with pedal, dynamics and rubato, and opens in any player or notation program.

## How it works

A piece of music is treated as a single cadence, from the tonic back to the tonic, unfolded.

Take a span of time, the tone it starts on and the tone it is headed for. Either hold the tone, or cut the span at its strongest beat, choose a tone for that moment that leads into what follows, and do the same to both halves. That is the whole algorithm, and it is a dozen lines long.

It runs three times over the whole piece: once with chords, once with scale degrees and once with the notes of an arpeggio. The results are the harmony, the melody and the left hand. Before that it runs over the piece's own plan, to decide which parts repeat and which contrast.

Four ideas make it sound like music.

### Metre is the recursion

Time is one hierarchy. Eighth notes gather into beats and beats into a bar, in twos or threes. Bars gather into phrases, phrases into sections and sections into the piece, until it is long enough. The seed chooses every division, so pieces differ in metre, in phrase length and in length.

A span is always cut at its strongest beat first. Because of this, chords change on barlines and stable notes land on strong beats without any rule saying so.

### Identity is a seed

There is no random number generator, only a hash. Every choice depends on which musical idea it belongs to and where in that idea it falls. When an idea comes back, its rhythm and contour come back with it, fitted to the new harmony. This is where motifs and reprises come from.

Which parts of an idea repeat is decided by the same unfolding. A part between two others either says something new, most likely where its neighbours are alike, or echoes one of them. Forms like ABA, AABA and ABACA come out of that without being named, and every piece ends by returning to its opening. A contrasting part settles on a different chord of the scale, one of the nearby keys.

### One curve of energy

A single swell rises and falls across the piece, across each section and across each phrase. It decides how busy the music is, how quickly the chords change, how high the melody sits, and how loud and fast it is played. When an idea returns at higher energy it comes back with more notes in it. Where the swell crests is different for every seed, so each piece has its own arc.

Each piece also has its own character, and it is only three numbers. Its tide is the level the energy rises from and its swell is how far it rises: a low, flat tide gives a still piece of long chords and single held notes, a high one a busy and loud one. Its caprice is how much chance is allowed to matter: a plain piece keeps to the strongest progressions and straight melodic lines, a capricious one wanders.

### Tension must resolve

A melody note outside the chord is only allowed if it steps into the next note, and it cannot be decorated before it does. Passing notes and appoggiaturas come out of that one condition. Harmony follows the pull of falling fifths, strongest toward the chord it is about to reach, so cadences appear on their own. The longer the span a chord opens, the harder it is pulled toward the key's centre, so sections find their tonic without being told to.

The left hand climbs the notes of each chord. When it holds still instead, it sounds at once the notes it would have climbed, so a long chord is an arpeggio that did not unfold.

## Project layout

- `src/unfold.rs` is the algorithm
- `src/form.rs` handles time: metre, the shapes of ideas, the hash and the energy curve
- `src/tonal.rs` handles pitch: one scale, its chords and how they move
- `src/voices.rs` defines the harmony, the melody and the accompaniment
- `src/piece.rs` turns a seed into notes, and holds the tests
- `src/perform.rs` adds dynamics, rubato and pedal
- `src/midi.rs` writes the MIDI file

## Testing

```
cargo test
```

The tests compose 200 pieces and check what the design promises: every dissonance resolves by step, every bar stands on its root, and every piece ends where it began.

## Tools

<img src="https://skillicons.dev/icons?i=rust&theme=dark" alt="Rust" />

## License

Tempest is free software, released under the [GPL-3.0](LICENSE).
