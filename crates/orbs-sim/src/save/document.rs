//! What a save *is*: a plain-TOML description of one tower at one tick.
//!
//! §13 pins the format — *"plain text/TOML — on-theme and debuggable; invites
//! external editing, which is accepted"* — and §15 makes readable-and-editable a
//! commercial decision rather than a nicety: *"hand-editing a TOML file only
//! affects the person doing it, and readable saves are both on-theme and a
//! debugging asset."*
//!
//! So the shape below is chosen for a person reading it, not for a serialiser.
//! Two consequences worth knowing before adding a field:
//!
//! - **Nodes are addressed by path, never by `NodeId`.** An id is a counter in
//!   spawn order, so it is stable only while `build.rs`'s tables are — and
//!   `tower::node` already says an `Entity` is *"meaningless across a save"*.
//!   A path is what the player types, survives a phase that adds a domain, and
//!   makes `subject = "/tower/laboratory/dispensary/sage"` mean something.
//! - **Everything absent is default.** A node with nothing happening to it is
//!   one line, so a save is mostly skimmable and the interesting rows stand out.
//!
//! The whole document is one `struct` tree with `serde` derives and no manual
//! `Serialize` anywhere. Enums are the exception and [`super::naming`] says why.

use serde::{Deserialize, Serialize};

use super::node::NodeSave;

/// The format this build writes, and the only one it reads.
///
/// Bumped when a change would make an older save load *wrongly* rather than
/// merely incompletely — a field that changed meaning, not a field that was
/// added. Additions are handled by `#[serde(default)]` and need no bump.
///
/// A save from a **newer** format is refused rather than misread: half-loading a
/// world whose rules have moved is worse than saying so.
///
/// # One change that is not obviously a format change, and is
///
/// `rng::derive_stream_seed`. A save carries word *positions*, not keys, so the
/// keys are recomputed on load — and a different mixing function winds different
/// streams to the recorded positions. Nothing compares and nothing errors; the
/// world just diverges from the session that wrote it. Touch that function and
/// bump this.
/// **2 since the lens rework**, which is the first bump this constant has had.
/// `WardSave` lost seven fields — `held`, `best`, `sigil_marks`, `socket_marks`,
/// `tried`, `settled`, `touched` — and `shift` changed vocabulary from
/// gained/held/lost to closer/level/further. Serde ignores what it no longer
/// knows, so a format-1 save loaded silently into the redesigned ward: a reading
/// whose recorded answers were scored by an exchange-and-ratchet codemaker,
/// resumed against one that neither exchanges nor ratchets. `WardSave`'s own doc
/// said the two shapes *"count as different formats"* and nothing enforced it.
///
/// **3 since the sanctum**, and this one is a *stream count* rather than a
/// field. `RngStream::COUNT` went 8 → 9 and [`Save::from_toml`] validates
/// `[rng].positions` against it, so a format-2 save is already unreadable — the
/// bump is what makes it say *behind* instead of *malformed*, which is the
/// difference between "this save is from an older build" and "this file is
/// corrupt". `CourseSave` and `progress.integrity` are ordinary additions and
/// would not have needed one.
pub const FORMAT: u32 = 3;

/// One tower, at one tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Save {
    /// The clock, the seed, and what wrote this.
    pub world: WorldSave,
    /// When the player left. Filled in by the frontend, never by the sim.
    #[serde(default)]
    pub away: Away,
    /// Where each random stream stood.
    pub rng: RngSave,
    /// What the player has earned, found and chosen.
    pub progress: ProgressSave,
    /// The tree, in the order a walk from the root meets it.
    #[serde(default, rename = "node")]
    pub nodes: Vec<NodeSave>,
    /// The tail of the record stream — see [`super`] for why it is bounded.
    #[serde(default, rename = "record")]
    pub records: Vec<RecordSave>,
}

/// The clock, the seed, and the build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSave {
    /// [`FORMAT`]. Read before anything else.
    pub format: u32,
    /// The version of the game that wrote this.
    ///
    /// **Informational only** — nothing branches on it. It is here because the
    /// first question anyone asks of a save that will not open is *which build
    /// made it*, and a save that cannot answer that makes a bug report a guess.
    pub built_by: String,
    /// The master seed every random stream is derived from.
    pub seed: u64,
    /// Ticks elapsed since this world began.
    pub tick: u64,
    /// How many records have ever been pushed, across the world's whole life.
    ///
    /// **Not a length.** A running spell's cursor is a position in this
    /// sequence, so it has to survive the stream being truncated — which is
    /// exactly what carrying only a tail of it does. See `Records::sequence`.
    pub sequence: u64,
}

/// When the player left, and by which clock.
///
/// # Why the sim never fills this in
///
/// §19 is explicit that a wall-clock reading inside a sim record *"would make
/// two runs of one seed differ"*. The sim has no clock but its own tick and must
/// not acquire one, so `orbs-shell` stamps `unix` on the way to the file and
/// reads it back on the way in.
///
/// # Why it exists before offline progression does
///
/// §5 opens *"initially there is no offline progression — the tower ticks only
/// while the window is open"* and puts accrual in Phase 9a. Nothing here
/// advances the world. But a departure time cannot be recovered after the fact,
/// so a save written today without one is a save Phase 9a can never ask how long
/// the orb was dark. The stamp is cheap; the retrofit is not.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Away {
    /// The tick the save was written at. Always equal to [`WorldSave::tick`]
    /// today, and separate from it because Phase 9a's catch-up advances one and
    /// not the other.
    #[serde(default)]
    pub tick: u64,
    /// Seconds since the Unix epoch, or `0` if the platform would not say.
    #[serde(default)]
    pub unix: u64,
}

/// Where each of the eight streams stood.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RngSave {
    /// One word position per stream, in `RngStream::index` order.
    ///
    /// **Decimal strings, because they are `u128` and TOML integers are `i64`.**
    /// A `ChaCha8Rng` counts in 32-bit words over a 64-bit block counter, so the
    /// position genuinely needs the width; truncating it to fit would put every
    /// stream in the wrong place after a long session and nowhere near one after
    /// a short one.
    pub positions: Vec<String>,
}

/// What the player has earned, found and chosen.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressSave {
    /// The wizard's name. **A save outranks the environment** (`session::Wizard`):
    /// a frontend seeds this from `USER` only when building a new world.
    #[serde(default)]
    pub wizard: String,
    /// Experience, which only ever rises. Concentration is derived from it and
    /// is deliberately not here.
    #[serde(default)]
    pub experience: u64,
    /// How the tower's walls stand (§11.5's Integrity).
    ///
    /// **`Option`, not a bare `u32`**, and the difference is the whole tower: a
    /// missing field would default to nought, so a save written before this
    /// existed would come back as a tower worn to nothing rather than as one
    /// nobody had measured. `None` restores to whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<u32>,
    /// Mastery nodes taken, by id.
    #[serde(default)]
    pub taken: Vec<String>,
    /// Secret recipes found.
    #[serde(default)]
    pub learned: Vec<String>,
    /// Solves since the last find, which is what raises the odds of the next.
    #[serde(default)]
    pub fruitless: u32,
    /// Where the player is standing, as a path.
    #[serde(default)]
    pub cwd: Option<String>,
    /// The tonal register everything is currently spoken in.
    #[serde(default)]
    pub register: Option<String>,
    /// The line that raised an open numbered prompt, if one is open.
    ///
    /// **The line, not the readings.** `Choices` holds the parser's resolved
    /// `Intent`s, and putting those in the format would pin it against every
    /// future parser change for a question that survives until the next command.
    /// The parser's tie-break draws no randomness (§19), so re-asking on the way
    /// in reproduces the same numbered list the player was looking at.
    ///
    /// Without this, a save taken with a prompt open came back with the question
    /// still on the transcript and no answer that resolved — §15's dead end,
    /// arriving through the affordance built to remove one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked: Option<String>,
    /// One entry per domain the rail has something to say about.
    #[serde(default, rename = "mark")]
    pub marks: Vec<MarkSave>,
}

/// A domain with something the rail should notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkSave {
    /// The domain's name.
    pub domain: String,
    /// `fault` or `news`.
    pub mark: String,
}

/// One line of the record stream, as much of it as a save carries.
///
/// # Why a save carries records at all
///
/// Because a `.log` is not a file. `execute::files` reads `Scrollback` and
/// filters it by domain, and `tower::node` says so outright: *"`orb.log` is the
/// whole of it and `laboratory.log` is that same stream filtered by where each
/// line happened. Neither has contents of its own."* Dropping the stream would
/// empty every log in the tower — and §8.1's poisoned-log tell is built by
/// *re-emitting existing lines*, so a restored tower with a `Poisoned` log would
/// answer `verify` with *tampered* and then show a player nothing at all.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSave {
    /// The record's kind, by name.
    pub kind: String,
    /// Its role, by name. Absent means the ordinary one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The register it was spoken in. Absent means plain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub register: Option<String>,
    /// Logged but not drawn — `follow`'s step, and everything else `quiet`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub quiet: bool,
    /// The authored linear variant (§14), if the record was given one.
    ///
    /// **Not optional for an eldritch record.** `RecordBuilder::finish` asserts
    /// that a record whose text may be damaged carries one, so a restore that
    /// dropped this would trip that assertion on the way back in — which is the
    /// accessibility promise catching a save bug, exactly as §14 intends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spoken: Option<String>,
    /// Its fields, in order, as `[name, type, value]`.
    ///
    /// Three elements rather than two because a value is text, a count, or a
    /// tick, and the three are not interchangeable: a count right-aligns in a
    /// table and a tick is a point in world time. A save that stored every value
    /// as text would redraw a brew's log with its numbers ragged.
    #[serde(default)]
    pub fields: Vec<(String, String, String)>,
}

impl Save {
    /// Render as TOML, for a file and for a person.
    ///
    /// # Errors
    ///
    /// If the document cannot be represented — which would be a bug here rather
    /// than anything a world could reach, since every field is a plain scalar,
    /// string, or sequence of them.
    pub fn to_toml(&self) -> Result<String, super::SaveError> {
        toml::to_string(self).map_err(|error| super::SaveError::Unwritable {
            reason: error.to_string(),
        })
    }

    /// Read one back.
    ///
    /// # Errors
    ///
    /// [`SaveError::Malformed`](super::SaveError::Malformed) if it is not the
    /// TOML this writes, or [`SaveError::Ahead`](super::SaveError::Ahead) if it
    /// came from a build whose format has moved on.
    pub fn from_toml(text: &str) -> Result<Self, super::SaveError> {
        let save: Self = toml::from_str(text).map_err(|error| super::SaveError::Malformed {
            reason: error.to_string(),
        })?;
        if save.world.format > FORMAT {
            return Err(super::SaveError::Ahead {
                found: save.world.format,
                understood: FORMAT,
            });
        }
        // **And older, which this did not check.** Refusing only the future is
        // half a version gate: serde drops fields it no longer knows without a
        // word, so a format-1 save opened straight into the redesigned ward and
        // resumed a reading scored under rules that no longer exist. There is no
        // migration to write — the old answers are not expressible in the new
        // model — so refusing is the honest answer, and the same one `Ahead`
        // gives for the same reason.
        //
        // This is a **pre-1.0 project with no shipped audience**: the cost is a
        // developer's own save from before the rework, and §15 already invites
        // deleting one. If that ever stops being true, this arm is where a
        // migration hangs.
        if save.world.format < FORMAT {
            return Err(super::SaveError::Behind {
                found: save.world.format,
                understood: FORMAT,
            });
        }
        // **Checked here rather than clamped on the way in.** A short or mangled
        // list would leave the missing streams at word zero, which is precisely
        // the silent rewind `Rngs::positions` exists to prevent — and §15 invites
        // hand-editing, so this is a file a person can really produce. Refusing
        // is the honest answer: a tower whose randomness quietly restarted is
        // worse than one that would not open.
        if save.rng.positions.len() != crate::RngStream::COUNT {
            return Err(super::SaveError::Malformed {
                reason: format!(
                    "[rng] has {} stream positions, and a world has {}",
                    save.rng.positions.len(),
                    crate::RngStream::COUNT,
                ),
            });
        }
        if let Some(bad) = save
            .rng
            .positions
            .iter()
            .find(|text| text.parse::<u128>().is_err())
        {
            return Err(super::SaveError::Malformed {
                reason: format!("[rng] position {bad:?} is not a number"),
            });
        }
        Ok(save)
    }
}
