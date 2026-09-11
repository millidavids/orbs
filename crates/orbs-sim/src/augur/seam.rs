//! The one thing every reader is.

/// Something that can read a line the orb could not read itself.
///
/// Implementors answer with the **canonical command** the line meant — the same
/// string [`Intent::echo`](crate::parser::Intent::echo) would produce — or
/// [`None`] to abstain, which hands the line back to §6's ordinary suggestions.
///
/// # Slots carry the player's words, not resolved names
///
/// `smash the sage` answers `grind sage`, and `sage` is whatever the player
/// typed. Binding it to something actually in the room is the matcher's work,
/// and doing it here would mean every reader needed a `Scene`, could go stale
/// between ticks, and could disagree with `parser::resolve` about what a name
/// means. See the module documentation.
///
/// # Abstaining is a first-class answer
///
/// A reader that always answers is a reader that guesses at *"what should I do
/// next"*. §15 weighs the dead-end rate above the raw resolution rate, and a
/// confident wrong command is a worse dead end than an honest *"I do not know
/// that word"* — so [`None`] is expected to be common and costs nothing.
///
/// # `Send + Sync`, because a reader outlives the call
///
/// The Bevy build keeps its reader in a `Resource`, which must be both, and a
/// trained one will hand work to a worker thread and read the answer back
/// (CLAUDE.md rule 8 — a thread and a channel, never an async runtime). Neither
/// is a burden on a table or a grammar.
pub trait Augur: Send + Sync {
    /// The canonical commands `line` might mean, best first. Empty to abstain.
    ///
    /// # Why several, when only one can run
    ///
    /// **Because a reader cannot see the world, and telling slot kinds apart
    /// needs it.** `run {script}` and `run the {place}` are the same sentence:
    /// nothing in *"run `night_watch`"* says `night_watch` is a script, so a
    /// reader answering once has to guess, and answers `wield night_watch` as
    /// often as `invoke night_watch`. Measured on the authored corpus, every
    /// one of the grammar's misreadings was this shape.
    ///
    /// Answering with a few and letting [`Sim::submit_reading`] take the first
    /// that **resolves** hands that decision to `parser::resolve`, which knows
    /// what is in the room and is the only thing qualified to make it. The
    /// reader still never sees the world; it simply stops pretending to know
    /// something it cannot.
    ///
    /// A model with a scored output produces candidates for free, so this shape
    /// costs a grammar a little and a trained reader nothing.
    ///
    /// # Best first, and not many
    ///
    /// The caller tries them in order and stops at the first that resolves, so
    /// ordering is the reader's confidence expressed as a list. Past
    /// [`MAX_READINGS`] they are ignored — a reader offering twenty is
    /// guessing, and §15 weighs the dead-end rate above the raw resolution
    /// rate.
    ///
    /// **Expected to return in the time a keystroke has**, because the echo is
    /// §6's teaching mechanism and *"a terminal that takes a second to answer
    /// reads as broken"*. A reader that cannot promise that answers from a
    /// worker thread and abstains when the answer is not back yet — the
    /// deadline belongs to the implementor, which is the only place that knows
    /// what it is waiting for.
    ///
    /// [`Sim::submit_reading`]: crate::Sim::submit_reading
    fn read(&self, line: &str) -> Vec<String>;
}

/// Something that can read one **line of a spell** the orb could not read.
///
/// [`Augur`]'s sibling, and deliberately not the same trait. Three differences,
/// each of which would be a defect if the two were merged:
///
/// - **The output space is the spell language, not the verb vocabulary.** A
///   spell line can be `if the mortar is idle` or `bide 10`, which mean nothing
///   at the prompt — `parser::analyse` answers `InSpell` for a leading spell
///   word precisely to keep them apart (§19).
/// - **One answer, not a list.** `Augur` offers several because only the room
///   can tell `run {script}` from `run the {place}`. A spell line is read with
///   no room to consult — it is read at *write* time, not at cast — so there is
///   nothing to try them against. Abstaining is the only other answer.
/// - **Nobody is watching.** The prompt echoes what it heard and the player is
///   standing there; a spell is read once and run for hours. §19's *The orb
///   accepts an abbreviation, never a typo, and never a coin flip* is explicit
///   that this is why a spell resolves stricter than the prompt — *"a spell
///   resolves with nobody watching, so it takes the stricter half"* — and it is
///   why an implementor must leave a line alone unless its reading **accounts
///   for every word in it**.
///
/// # Answer `None` far more often than not
///
/// A line that is already canonical, a comment, a blank line, and anything whose
/// reading does not fully parse must all come back [`None`]. The caller keeps
/// the player's text either way — see `Sim::write_spell_reading` — so an
/// abstention costs nothing and a wrong answer costs a spell that runs for
/// hours doing the wrong thing.
pub trait Scrivener: Send + Sync {
    /// The canonical form of `line`, or [`None`] to leave it exactly as written.
    fn read(&self, line: &str) -> Option<String>;

    /// Which reader this is: a number two readers share only if they would
    /// answer every line alike.
    ///
    /// # What a kept reading is keyed by, beside the text
    ///
    /// `Sim::write_spell_reading` keeps a line's reading while its text does not
    /// change, which is what makes an autosave on every pause affordable. Keyed
    /// by the text alone, a reading outlived the reader that made it: switching
    /// the driver to `plain` left every line compiling what the model had made
    /// of it, and a verbatim copy — a restored save, a repaired sabotage — stood
    /// in for a reading no reader ever took. A reading is kept now only for the
    /// reader that made it; see `tower::Read::by`.
    ///
    /// **Required, not defaulted.** A default every implementor shared would be
    /// that defect again, for whichever reader forgot to override it.
    fn identity(&self) -> u64;
}

/// A [`Scrivener`] that leaves every line exactly as written.
///
/// **What a build with no reader uses**, so `Sim::write_spell` is
/// `write_spell_reading` with this rather than a second code path — and the
/// identity case is the one the whole cache rests on: with nothing reading, the
/// lines that compile are the lines the player typed.
pub struct Verbatim;

impl Verbatim {
    /// Its [`Scrivener::identity`]. Nothing it answers depends on anything, so a
    /// constant is the whole truth of it.
    pub const IDENTITY: u64 = 0;
}

impl Scrivener for Verbatim {
    fn read(&self, _line: &str) -> Option<String> {
        None
    }

    fn identity(&self) -> u64 {
        Self::IDENTITY
    }
}

/// How many readings a caller will try before giving up.
///
/// Sized like the numbered prompt's `MAX_PROMPT` and for the same reason: past
/// a handful, offering more is not offering better.
pub const MAX_READINGS: usize = 4;
