# Tempest: notes for whoever works on this next

Read this before changing anything. It records what the project is for and how to tell whether a change belongs in it.

## The goal

Tempest generates beautiful, well-composed solo piano music by purely algorithmic means. There is no machine learning of any kind and there never will be.

The algorithm is the whole point. It should be elegant at its core, built on a small number of powerful ideas instead of a pile of rules and special cases. It should be as satisfying to read as its music is to hear. The code stays very clean and every line has to count.

## The design in one page

A piece is a single cadence, tonic to tonic, unfolded. One function, `unfold` in `src/unfold.rs`, takes a span of time, the tone it starts on and the tone it is bound for. It either holds the tone, or cuts the span at its strongest beat, chooses a tone there that leads into what follows, and recurses. Tones are chosen last first, so every note is written knowing where it is going.

That function runs once per voice. A voice (`src/voices.rs`) answers two questions: does this span move, and what leads from here to there. Harmony, melody and accompaniment are three answers.

Four ideas make it music:

- Metre is the recursion tree. `Form::cuts` splits a span at its strongest beat, so a tone's metric weight is its structural importance.
- Identity is a seed. `Form::whim` is a hash of which idea a choice belongs to and where in it. Ideas are arranged as statement, echo, contrast, return, so a returning idea brings its choices back with it.
- One curve of energy. `Form::energy` drives how readily spans divide, how high the melody sits, and how loud and fast the piece is played.
- Tension must resolve. A melody tone outside the chord is allowed only if it steps into the next tone, and it cannot be elaborated first.

## How to tell whether a change belongs

Ask these in order. A change that fails one needs a different design, not an exception.

- Does it replace a rule with a pull? Prefer a cost or a weight that makes the right thing likely over a branch that makes it certain. Never restore a hard-coded result because a pull was too weak; strengthen the pull.
- Is it one law for all voices? A case per voice, or a case per metre, is a sign the law is missing.
- Does it add a table of patterns, a template, or a list of special situations? Then it does not belong. The only table of taste is `tonal::flow`, and it is labelled as one.
- Does `unfold` stay as it is? It should.
- Is the code smaller or the same size afterwards? Delete before adding. The composition code (`form`, `tonal`, `unfold`, `voices`, `compose`) is about 320 lines and should stay near that.
- Is the piece still a pure function of its seed? No random number generator, no clock, no state.
- No dependencies.

## How to judge the music

Nobody working on this through a terminal can hear it. Be honest about that.

- `cargo test` composes 200 pieces and checks the invariants the design promises, and the statistics it should keep: cadences, stepwise motion, register, hands.
- Read whole pieces as text, bar by bar: chords, melody, left hand. Check that motifs return, the contrast section climbs, and the ending resolves.
- Then someone has to listen. The constants in `src/voices.rs` were tuned by reading scores, not by ear, and listening outranks every number here.

Measure before and after a change. A change that makes the numbers worse is not kept.

## Known limits

These are known and deliberate for now. Do not bolt them on.

- The left hand does not see the melody, and there is no inner voice. This is the next real design question, and the one most likely to cost the simplicity.
- The form is always statement, echo, contrast, return, 32 bars. Other shapes would need a table of shapes.
- Triads in root position only. Nothing faster than an eighth note. No ornaments.
- The contrast section moves to the relative key, which shares the home key's notes. There is no true modulation.

## House conventions

- The license is GPL-3.0. Do not change it.
- The README is plain prose in simple markdown: short paragraphs, few sections, no tables.
- Never amend or rewrite a commit that has been pushed. Make a new commit.
- Do not push. The owner pushes.
