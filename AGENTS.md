# Tempest: notes for whoever works on this next

Read this before changing anything. It records what the project is for and how to tell whether a change belongs in it.

## The goal

Tempest generates beautiful, well-composed solo piano music by purely algorithmic means. There is no machine learning of any kind and there never will be.

The algorithm is the whole point. It should be elegant at its core, built on a small number of powerful ideas instead of a pile of rules and special cases. It should be as satisfying to read as its music is to hear. The code stays very clean and every line has to count.

## The design in one page

A piece is a single cadence, tonic to tonic, unfolded. One function, `unfold` in `src/unfold.rs`, takes a span of time, the tone it starts on and the tone it is bound for. It either holds the tone, or cuts the span at its strongest beat, chooses a tone there that leads into what follows, and recurses. Tones are chosen last first, so every note is written knowing where it is going.

That function runs once per voice. Harmony goes first; the other two answer its chords, not the barlines. Before any of them it runs over the form itself: a fourth voice, `Shape` in `src/form.rs`, decides which parts of each idea echo and which contrast. A voice (`src/voices.rs`) answers two questions: does this span move, and what leads from here to there. Harmony, melody and accompaniment are three answers.

Four ideas make it music:

- Metre is the recursion tree. Time is one hierarchy from the eighth note to the piece, each level gathering a few of the level below; the seed chooses the divisions. `Form::cuts` splits a span at its strongest beat, so a tone's metric weight is its structural importance.
- Identity is a seed. `Form::whim` is a hash of which idea a choice belongs to and where in it. Parts of an idea that echo each other share an identity, so a returning idea brings its choices back with it.
- One curve of energy. `Form::energy` drives how readily spans divide in every voice, harmony included, how high the melody sits, and how loud and fast the piece is played. Its shape comes from the seed, and so does the piece's character: `tide` (the level energy rises from), `swell` (how far it rises) and `caprice` (how much chance matters against the pulls). A style is a point in those three numbers, never a preset.
- Tension must resolve. A melody tone outside the chord is allowed only if it steps into the next tone, and it cannot be elaborated first.

## How to tell whether a change belongs

Ask these in order. A change that fails one needs a different design, not an exception.

- Does it replace a rule with a pull? Prefer a cost or a weight that makes the right thing likely over a branch that makes it certain. Never restore a hard-coded result because a pull was too weak; strengthen the pull.
- Is it one law for all voices? A case per voice, or a case per metre, is a sign the law is missing.
- Does it add a table of patterns, a template, or a list of special situations? Then it does not belong. The only table of taste is `tonal::flow`, and it is labelled as one.
- Does `unfold` stay as it is? It should.
- Is the code smaller or the same size afterwards? Delete before adding. The composition code (`form`, `tonal`, `unfold`, `voices`, `compose`) is about 375 lines and should stay near that.
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
- Triads in root position only. Nothing faster than an eighth note. No ornaments.
- A contrasting section settles on another consonant chord of the same scale. There is no true modulation.
- The melody never rests. The pedal clears with every chord, so the only audible rest is a melody that does not strike at a chord's start. Withholding that tone leaves any dissonance before it unresolved, which breaks the fourth idea; a version that checks for this is a pile of conditions. It needs a better idea, not a patch.
- The voice salts in `src/voices.rs` (`0x100`, `0x300`, `0x380`) are bare numbers. They only need to differ.

## Things learned the hard way

- A piece must end with its statement. When the top-level shape was free to end on a contrast, pieces finished in another key and reached the tonic only on the last chord; the cadence numbers showed it before anyone listened. Inner ideas are free because a statement really does follow them.
- Runs of the same idea are the main risk of generated shapes. Three echoes in a row gave 24 bars of two alternating chords. `Shape::lead` now prefers the idea heard first when it echoes. If pieces sound repetitive, that pull is the place to look, along with how likely a new part is.
- Gravity towards the centre must grow with the square of the span a chord opens. Linear growth collapsed the harmony into tonic and dominant.
- The bar has to be between about six and twelve eighth notes. Shorter and motifs have no room; longer and one pedal blurs. `BAR` in `src/form.rs` sets it, the same way `LENGTH` sets the piece.
- `Form::stride` must find the longest unit that begins at a moment by looking at every unit. The appended return makes the top of the grid uneven.

- Character costs almost nothing if it goes through energy. A still piece and a stormy one differ only in tide and swell, because every voice already follows the curve. Do not add per-style behaviour; add or widen a trait.
- A low tide must not lower the melody's register, or still pieces sink into the left hand.
- Without chords, still pieces are bare two-part writing. A held accompaniment span sounds the rungs it had time to climb, which is the same figure not unfolded.

## House conventions

- The license is GPL-3.0. Do not change it.
- The README is plain prose in simple markdown: short paragraphs, few sections, no tables.
- Never amend or rewrite a commit that has been pushed. Make a new commit.
- Do not push. The owner pushes.
