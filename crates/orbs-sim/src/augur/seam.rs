//! The one thing every reader is.

/// Something that can read a line the orb could not read itself.
///
/// Implementors answer with the canonical command the line meant — the same
/// string [`Intent::echo`](crate::parser::Intent::echo) would produce — or
/// [`None`] to abstain, which hands the line back to §6's ordinary suggestions.
///
/// Slots carry the player's words, not resolved names: `smash the sage` answers
/// `grind sage`. Binding a name to something in the room is the matcher's work,
/// and doing it here would need a `Scene` that goes stale between ticks and can
/// disagree with `parser::resolve`. See the module documentation.
///
/// Abstaining is a first-class answer. §15 weighs the dead-end rate above the
/// raw resolution rate, so a confident wrong command is a worse dead end than
/// [`None`], which is expected to be common.
///
/// `Send + Sync` because the Bevy build keeps its reader in a `Resource`. No
/// burden on a table, a grammar or the trained reader, which answers on the
/// calling thread — see [`read`](Self::read).
pub trait Augur: Send + Sync {
    /// The canonical commands `line` might mean, best first. Empty to abstain.
    ///
    /// Several, because a reader cannot see the world and telling slot kinds
    /// apart needs it: nothing in *"run `night_watch`"* says `night_watch` is a
    /// script, and on the authored corpus every one of the grammar's
    /// misreadings was that shape. [`Sim::submit_reading`] takes the first that
    /// *resolves*, handing the decision to `parser::resolve`, which is the only
    /// thing that knows what is in the room. A scored model produces candidates
    /// for free.
    ///
    /// Order is the reader's confidence. Past [`MAX_READINGS`] they are ignored
    /// — a reader offering twenty is guessing.
    ///
    /// Expected to return in the time a keystroke has, because the echo is §6's
    /// teaching mechanism. The trained reader does, on the calling thread: 436µs
    /// to read a line and 8ms to load in release (`orbs-augury`'s
    /// `tests/loading.rs`), so no worker thread, deadline or indicator. A much
    /// larger model would answer from a thread and a channel (CLAUDE.md rule 8,
    /// never an async runtime) and set its own deadline.
    ///
    /// [`Sim::submit_reading`]: crate::Sim::submit_reading
    fn read(&self, line: &str) -> Vec<String>;
}

/// Something that can read one line of a spell the orb could not read.
///
/// [`Augur`]'s sibling, and deliberately not the same trait:
///
/// - The output space is the spell language, not the verb vocabulary. `bide 10`
///   means nothing at the prompt, and `parser::analyse` answers `InSpell` for a
///   leading spell word to keep the two apart (§19).
/// - One answer, not a list. A spell line is read at *write* time, with no room
///   to try candidates against.
/// - Nobody is watching. A spell is read once and run for hours, so it resolves
///   stricter than the prompt (§19): leave a line alone unless its reading
///   accounts for every word in it.
///
/// Answer [`None`] far more often than not — a canonical line, a comment, a
/// blank line, and anything whose reading does not fully parse. The caller keeps
/// the player's text either way (`Sim::write_spell_reading`), so abstaining
/// costs nothing and a wrong answer costs hours of the wrong thing.
pub trait Scrivener: Send + Sync {
    /// The canonical form of `line`, or [`None`] to leave it exactly as written.
    fn read(&self, line: &str) -> Option<String>;

    /// Which reader this is: a number two readers share only if they would
    /// answer every line alike.
    ///
    /// `Sim::write_spell_reading` keeps a line's reading while its text does not
    /// change. Keyed by the text alone a reading outlived its reader: switching
    /// the driver to `plain` left every line compiling what the model had made
    /// of it. A reading is kept only for the reader that made it; see
    /// `tower::Read::by`.
    ///
    /// Required, not defaulted — a shared default would be that defect again,
    /// for whichever reader forgot to override it.
    fn identity(&self) -> u64;
}

/// A [`Scrivener`] that leaves every line exactly as written.
///
/// What a build with no reader uses, so `Sim::write_spell` is
/// `write_spell_reading` with this rather than a second code path.
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
