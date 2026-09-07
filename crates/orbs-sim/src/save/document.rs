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
///
/// **4 since the menagerie**, for exactly the same reason: `RngStream::COUNT`
/// went 9 → 10 with `Menagerie`. `ChantSave` beside it is an ordinary addition
/// and would not have needed one. That two of the last two bumps were both
/// stream counts is worth noticing — **a new domain almost always brings a
/// stream, and a stream is always a format change**, so the bump belongs in the
/// same commit as the variant rather than being discovered by the first player
/// whose tower would not open.
///
/// **5 since `bide until` was withdrawn**, and this one is neither a stream nor
/// a field — it is the *language*. A spell is stored as the player's own text
/// and recompiled at cast, so a saved `bide until` does not fail to load: it
/// loads perfectly and compiles to a line the orb cannot read, and a tower whose
/// bound solver quietly started faulting is the format-2 failure again in a
/// different costume. `RunningSave::biding` beside it is an ordinary additive
/// field and would not have needed one.
///
/// **So the rule is wider than "a new stream or a changed struct".** Anything
/// that changes what stored *content* means belongs here too — and content is
/// most of what this game saves.
///
/// **Still 5 after §8.1's audit**, and that is a decision rather than an
/// oversight. [`ProgressSave::cooling`] is the same shape as the three additions
/// this doc has already excused: no stream, no changed meaning, and an absent
/// table reads honestly as *nothing is cooling*, which is true of every save
/// written before the rationing existed. The wider rule asks what stored content
/// **means**, and no stored byte changed meaning — what changed is that a live
/// `verify` can now answer *wait*, which is behaviour, and behaviour moves with
/// the build. Note this is **not** the `bide until` case: that one loaded
/// perfectly and then compiled to a line the orb could not read, where a saved
/// spell containing `verify` still compiles, still runs, and is merely sometimes
/// told to wait — at `Role::Cost`, so it latches no fault either.
///
/// **6 since the siege**, and it is a *stream count* again — the fourth of the
/// last five bumps to be one. `RngStream::COUNT` went 10 → 11 with `Siege`, and
/// [`Save::from_toml`] validates `[rng].positions` against it, so a format-5 save
/// is already unreadable; the bump is what makes it say *behind* rather than
/// *malformed*. `NodeSave::siege` and `ProgressSave::cooling` beside it are
/// ordinary additions and would not have needed one.
///
/// (This said `SiegeSave` and `progress.escrow`, and **neither exists**. A siege
/// is serialised as the component itself — see [`NodeSave::siege`] — and escrow
/// is computed at settle time and never stored. Two invented names in the one
/// doc a future bump is guaranteed to be read against.)
/// **7 since the besieging army was renamed**, and this one is neither a stream
/// nor a field — it is a **node's path**. `/tower/bailey/host` became
/// `/tower/bailey/enemy`, and a document addresses every node by path (see
/// `save::node`), so a format-6 save carries a row naming a place this build
/// does not have.
///
/// **`host` had to go because the game is a computer terminal.** §9b's remote
/// hosts are a planned content type — *"trees, verbs, infiltration"* — so the
/// word would have meant *a machine you break into* and *the army at your wall*
/// in the same vocabulary. §5.1 already calls it **the enemy** seventeen times.
///
/// It is migrated rather than refused, and this is the first *content* rename
/// the migration handles: a path is a string, and rewriting one is exact.
///
/// **7 → 8 is a running siege gaining a resource it was fought without.**
/// `Siege::quintessence` is `#[serde(default)]`, so a format-7 document *loads* —
/// and opens the fight with nothing to pledge for the rest of its length, which
/// is this doc's own line above: *"loads wrongly rather than merely
/// incompletely"*. The migration fills it, so the bump is what turns a silently
/// crippled siege into a correct one.
///
/// **No stream moved.** `RngStream::COUNT` is unchanged — quintessence is
/// integer arithmetic over integrity and the ley line, and takes no draw. What
/// *does* change is how many draws a round takes, since a refused pledge is a die
/// that never rolls; that makes old *replays* diverge and is a property of the
/// game rather than of the format.
///
/// **9 → 10 is the tower learning to be shut.** Three fields join `[progress]`
/// — `tally`, `reached`, `opened` — and none of them needs a migration step,
/// because each defaults to the honest reading of a document that never had it:
/// nothing counted, nothing reached, and *everything open* (`opened` is an
/// `Option` for exactly that). What the bump buys is the version gate: a
/// format-10 document read by a format-9 build would drop `opened` in silence
/// and load a sealed tower as an open one, which is the failure the gate exists
/// for. No stream moved; a tally is a function of the submissions.
pub const FORMAT: u32 = 11;

/// Bring an older document up to [`FORMAT`], or say why it cannot be.
///
/// # What is expressible and what is not
///
/// A save is refused when the new model cannot state what the old one meant.
/// That is a real category — the ward rework changed what a *scored reading*
/// means, so a format-1 or -2 document is genuinely unreadable — and it is a
/// much smaller category than *"the number went up"*:
///
/// | Bump | What changed | Migratable |
/// |---|---|---|
/// | 1 → 2 | The lens's scoring model | **No.** Old readings mean nothing now |
/// | 2 → 3 | `RngStream::COUNT` 8 → 9 — **and the `battlements/` → `sanctum/` rename, in the same commit** | **No.** A format-2 save names a room and four fixtures this build does not have, and `adopt` addresses nodes by path |
/// | 3 → 4 | `RngStream::COUNT` 9 → 10 (`Menagerie`) | Yes — pad |
/// | 4 → 5 | `bide until` withdrawn from the language | Yes, with a caveat below |
/// | 5 → 6 | `RngStream::COUNT` 10 → 11 (`Siege`) | Yes — pad |
/// | 6 → 7 | `/tower/bailey/host` → `/tower/bailey/enemy` — a **node's path**, the first *content* rename | Yes — rewrite the path |
/// | 7 → 8 | `Siege::quintessence`, a resource a running siege was fought without | Yes — fill the pool |
/// | 8 → 9 | `RngStream::COUNT` 11 → 12 (`Forge`), **and** quintessence moving from the siege to the tower | Yes — pad, and lift the pool |
/// | 9 → 10 | `[progress]` gains `tally`, `reached` and `opened` — the mastery lines and the sealed tower | Yes — every field defaults to the honest reading; `opened` absent is *everything open* |
/// | 10 → 11 | `[world]` gains `length`, and `[progress]` gains `stores` — how long the game was begun to be, and what the arsenal is stocked in | Yes — `length` absent is the curve as authored, `stores` absent is *full* |
///
/// **Two of these rows were once missing, and `migrate` performed both.** A
/// table that stops short of the function beneath it is worse than no table: it
/// is the one place a future bump is read for the shape of the thing, and it
/// described a `migrate` that has not existed since `0.8.13`.
///
/// **Padding a stream is exact, not approximate.** `Rngs::restore` derives each
/// stream from the master seed and then winds it forward by the stored position,
/// so a stream at nought is precisely what a world that had never drawn from it
/// would hold. A tower that gains the siege's dice gets the same dice it would
/// have had if the format had always been 6.
///
/// **The language bump is the one with a caveat, and it is survivable.** A saved
/// spell containing `bide until` loads perfectly and compiles to a line the orb
/// cannot read — which is a *fault the player can see and fix*, in a file whose
/// text is theirs and is never rewritten (§19). Refusing the whole tower for one
/// bad line in one spell is the larger loss, and §8's taxonomy is *"scripts
/// always log and never halt"* rather than *"a bad line voids the world"*.
fn migrate(mut save: Save) -> Result<Save, super::SaveError> {
    /// The oldest format whose meaning survives into the current model.
    ///
    /// **3, not 2, and the difference is a whole room.** The 2 → 3 bump landed
    /// in the same commit as the `battlements/` → `sanctum/` rename (`de29565`,
    /// v0.4.2), which also renamed every fixture in it —
    /// `rampart`/`barbican`/`bastion`/`redoubt` became
    /// `wellspring`/`conduit`/`barrier`. A format-2 document therefore carries
    /// `/tower/battlements/...` rows, and `adopt::apply` addresses nodes by
    /// path: accepting one padded the streams, stamped it current, and **dropped
    /// the entire sanctum without a word** — which is the *"serde drops what it
    /// no longer knows"* failure the version gate exists to prevent, arriving
    /// through the thing added to prevent it.
    ///
    /// The rename is as string-rewritable as `host` → `enemy` was, so this could
    /// migrate. It does not, because there is a second, unwritable half: the old
    /// `rampart` is the *bailey's* fixture name now, so a format-2 path would
    /// have to be disambiguated by its parent, and a v0.4.1 save is a
    /// developer's own from before two reworks. Refusing says so; the previous
    /// behaviour said nothing.
    const OLDEST: u32 = 3;

    if save.world.format < OLDEST {
        return Err(super::SaveError::Behind {
            found: save.world.format,
            understood: FORMAT,
        });
    }

    // **Pad, never truncate.** A document from a build with *more* streams than
    // this one is `Ahead` and was refused above, so anything reaching here is
    // short or exact. Extra entries would be a malformed file rather than an old
    // one, and the length check below still catches them.
    while save.rng.positions.len() < crate::RngStream::COUNT {
        save.rng.positions.push("0".to_owned());
    }

    // **The bailey's second band was `host` until format 7.** A node is
    // addressed by path, so the rename is a string rewrite and nothing more —
    // which makes it the first *content* change this function migrates rather
    // than refuses, and the demonstration that the category is real.
    if save.world.format < 7 {
        const WAS: &str = "/tower/bailey/host";
        const NOW: &str = "/tower/bailey/enemy";
        for node in &mut save.nodes {
            if node.path == WAS {
                node.path = NOW.to_owned();
            }
        }
    }

    // **A siege fought before quintessence existed opens with a full pool.**
    // `#[serde(default)]` would give it nought, and a fight you cannot pledge in
    // is not the fight that was saved. Full rather than scaled by the tower's
    // integrity on purpose: the pool is granted once, at `defend`, from the world
    // as it stood *then* — and that world is not in the document. Generous is the
    // honest way to be wrong here, and it lasts one siege.
    if save.world.format < 8 {
        for node in &mut save.nodes {
            if let Some(siege) = node.siege.as_mut()
                && siege.quintessence.is_none_or(|held| held == 0)
            {
                siege.quintessence = Some(crate::tower::QUINTESSENCE_BASE);
            }
        }
    }

    // **8 → 9: the pool came up to the tower.** A siege used to carry its own
    // quintessence and spend it privately; the forge spends the same resource,
    // so it is a world resource now (§11.5's own shape, restored — §19).
    //
    // A format-8 document holds the pool on whichever siege was in flight, and a
    // player mid-fight must not lose it. Lifting the largest is exact in
    // practice — there is one bailey — and honest in the pathological case.
    //
    // **A save with no siege in it gets the ceiling, not nought.** This is
    // `ProgressSave::integrity`'s rule and the reason it exists: a bare `u32`
    // defaults to zero and reads as a ruined tower, and here it would read as a
    // returning player whose forge is inert until the trickle catches up. The
    // ceiling cannot be computed from the document alone, so `None` travels and
    // `restore` fills it from the world that is about to be built.
    if save.world.format < 9 {
        let carried = save
            .nodes
            .iter_mut()
            .filter_map(|node| node.siege.as_mut())
            .filter_map(|siege| siege.quintessence.take())
            .max();
        save.progress.quintessence = carried;
    }

    save.world.format = FORMAT;
    Ok(save)
}

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
    /// Whether this tower began sealed — a laboratory and nothing else.
    ///
    /// **Part of the recorded start**, beside the seed: a sealed and an open
    /// tower with identical seed and submissions diverge at the first
    /// `attend archive`, so a replay has to know which it is rebuilding. Absent
    /// is *open*, which is what every tower was before a tower could be shut.
    #[serde(default)]
    pub sealed: bool,
    /// How long this game was begun to be.
    ///
    /// **Part of the recorded start, for `sealed`'s reason and more sharply.**
    /// The length decides every threshold on both tracks, so a save that lost it
    /// would load into a world whose stations sit somewhere else entirely —
    /// crossings re-announced, rooms opened that should not be, concentration
    /// jumping. A replay has to know which curve it is rebuilding.
    ///
    /// Absent is [`Length::Baseline`](crate::content::Length::Baseline), the
    /// curve exactly as authored, which is what every tower had before a game
    /// could have a length. **This is what the `FORMAT` 11 bump buys**: an older
    /// build would drop the field in silence and re-derive a long game's curve at
    /// baseline, which is *loading wrongly* rather than incompletely.
    #[serde(default)]
    pub length: crate::content::Length,
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
/// while the window is open"* and puts accrual in Phase 11a. Nothing here
/// advances the world. But a departure time cannot be recovered after the fact,
/// so a save written today without one is a save Phase 11a can never ask how long
/// the orb was dark. The stamp is cheap; the retrofit is not.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Away {
    /// The tick the save was written at. Always equal to [`WorldSave::tick`]
    /// today, and separate from it because Phase 11a's catch-up advances one and
    /// not the other.
    #[serde(default)]
    pub tick: u64,
    /// Seconds since the Unix epoch, or `0` if the platform would not say.
    #[serde(default)]
    pub unix: u64,
}

/// Where each of the streams stood.
///
/// **Deliberately not "the eight"**, which is what this said while there were
/// nine. `RngStream::COUNT` is the number and it moves once a domain.
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
    /// What the tower is known for — the one number that can fall.
    ///
    /// **A bare `u64`, not an `Option`, and no `FORMAT` bump.** Nought is the
    /// honest reading of a document written before renown existed: that tower
    /// earned none it can show. The rule is *bumped when an older save loads
    /// wrongly rather than merely incompletely*, and `cooling` is the precedent
    /// — where `opened` needed a bump because absent meant *everything*, which
    /// an older build would have silently dropped.
    #[serde(default)]
    pub renown: u64,
    /// How many foes have been bought off the next siege to arrive.
    ///
    /// **A bare `u32` and no `FORMAT` bump**, on `renown`'s rule above: nought
    /// is the honest reading of a document written before anyone could petition
    /// — that tower has bought nothing off. It is the *allowance*, not a
    /// setting, so a player who paid and then saved keeps what they paid for.
    #[serde(default)]
    pub petitioned: u32,
    /// How the tower's walls stand (§11.5's Integrity).
    ///
    /// **`Option`, not a bare `u32`**, and the difference is the whole tower: a
    /// missing field would default to nought, so a save written before this
    /// existed would come back as a tower worn to nothing rather than as one
    /// nobody had measured. `None` restores to whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<u32>,
    /// What the tower holds of §11.5's mana.
    ///
    /// **`Option` for `integrity`'s reason, and it bites harder here.** A bare
    /// `u32` defaults to nought, so every returning player would load with an
    /// empty pool — an inert forge and a siege that cannot pledge, until the
    /// trickle caught up. `None` restores to the **ceiling**, which is the
    /// generous reading and the honest one: nobody had measured it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quintessence: Option<u32>,
    /// Which `verify` surfaces are cooling, and the tick each frees at.
    ///
    /// **It travels, because a cooldown that reset on load is one a player can
    /// clear by quitting** — the shape §19 calls an exploit that then needs its
    /// own rule. Absent is *nothing is cooling*, which is the honest reading of
    /// a document written before §8.1's rationing existed.
    ///
    /// **Named pairs rather than a slot per surface**, for two reasons. TOML has
    /// no null, so the positional `Vec<Option<u64>>` this was first written as
    /// does not serialise at all — `toml` answers `unsupported None value` — and
    /// naming each surface means the two Phase 8 surfaces cannot renumber the
    /// two that ship. It is also the readable file §15 asks for: `["log", 120]`
    /// says what `[null, 120]` cannot.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cooling: Vec<(String, u64)>,
    /// Ley Line fork nodes taken, by id.
    #[serde(default)]
    pub taken: Vec<String>,
    /// Secret recipes found.
    #[serde(default)]
    pub learned: Vec<String>,
    /// What has been done, counted — what the mastery lines read.
    ///
    /// Absent is *nothing counted*, which a save from before the lines existed
    /// honestly says: its stations are re-earned, and the CHANGELOG says so.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub tally: std::collections::BTreeMap<String, u32>,
    /// When the tower last made each thing, lately — its stores, as a rate.
    ///
    /// **Absent is not *nothing made*, and the difference is the whole feature.**
    /// A store's standing is how many were made inside a window, so an empty map
    /// reads as *every store is out* — which would hand a returning player an
    /// arsenal they cannot spend a single thing from. That is `opened`'s shape
    /// one field down: absent meaning the opposite of the honest reading.
    ///
    /// So `restore` **stamps** an absent map at the loaded tick rather than
    /// taking it at face value, and the tower opens with full stores. No
    /// `FORMAT` bump, because the older document still loads *correctly* — it
    /// loads generously, which is the right way to be wrong about a save written
    /// before the rule existed.
    ///
    /// **`Option`, not a bare map, and the difference is the whole migration** —
    /// `integrity` one field down is the precedent and the reason. A bare map
    /// cannot tell *a document from before stores existed* from *a tower that
    /// has genuinely made nothing*: both are empty, so the stamp fired on both,
    /// a fresh save reloaded into a world with stores it never had, and the
    /// round-trip stopped being idempotent. Four persistence tests said so at
    /// once, twice over — the first fix only moved which of the two was wrong.
    ///
    /// `None` is *written before stores existed*, and is the only thing the
    /// stamp answers. `Some(empty)` is a tower that has made nothing lately, and
    /// is taken at face value.
    #[serde(default)]
    pub stores: Option<std::collections::BTreeMap<String, Vec<u64>>>,
    /// Mastery stations reached, by id.
    ///
    /// Saved rather than derived from `tally`, because reaching is *said* and
    /// a restore must not say it again.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reached: Vec<String>,
    /// What the tower has opened: rooms, recipes, charms, the wall.
    ///
    /// **`Option`, for `integrity`'s reason.** A save written before anything
    /// could be shut says nothing about it, and an empty list would seal every
    /// room but the laboratory on a tower that had been standing in all seven.
    /// `None` restores to everything open, because that is the tower it was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opened: Option<Vec<String>>,
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
        // **Older is migrated where migration is honest, and refused where it is
        // not.** Refusing only the future is half a version gate — serde drops
        // fields it no longer knows without a word, so a format-1 save opened
        // straight into the redesigned ward and resumed a reading scored under
        // rules that no longer exist.
        //
        // This arm used to refuse *every* older format, on the argument that
        // *"the old answers are not expressible in the new model"* and that
        // §15 invites deleting a developer's save. That is true of the ward
        // rework and **false of every bump since**: three of the last four were
        // pure `RngStream::COUNT` increases, and a stream a save has never heard
        // of is exactly a stream at position nought. See [`migrate`].
        let save = migrate(save)?;
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
