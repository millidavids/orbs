//! What a save *is*: a plain-TOML description of one tower at one tick.
//!
//! §13 pins TOML and §15 makes it readable and editable, so the shape is chosen
//! for a person reading it. Two consequences before adding a field:
//!
//! - Nodes are addressed by path, never by `NodeId` — an id is a spawn-order
//!   counter, meaningless across a save; a path is what the player types.
//! - Everything absent is default, so a save stays skimmable.
//!
//! One `struct` tree with `serde` derives. Enums are the exception and
//! [`super::naming`] says why.

use serde::{Deserialize, Serialize};

use super::node::NodeSave;

/// The format this build writes, and the only one it reads.
///
/// Bumped when a change would make an older save load *wrongly* rather than
/// merely incompletely; additions are `#[serde(default)]` and need no bump. A
/// newer format is refused rather than misread.
///
/// `rng::derive_stream_seed` is a format change that does not look like one: a
/// save stores word *positions*, so a different mixing function winds the
/// streams elsewhere and nothing errors — the world just diverges.
///
/// The bumps so far:
///
/// - 2, the lens rework: `WardSave` lost seven fields and `shift` changed
///   vocabulary, so a format-1 save resumed against a differently scored ward.
/// - 3, the sanctum: `RngStream::COUNT` 8 → 9. [`Save::from_toml`] already
///   rejects the wrong count, so the bump only buys *behind* over *malformed*.
/// - 4, the menagerie: `RngStream::COUNT` 9 → 10. A stream is always a format
///   change, so the bump belongs in the same commit as the variant.
/// - 5, `bide until` withdrawn — the *language*, neither stream nor field. A
///   spell is stored as the player's text and recompiled at cast, so anything
///   that changes what stored *content* means belongs here too.
/// - Still 5 after §8.1's audit: [`ProgressSave::cooling`] has no stream and no
///   changed meaning, and absent reads honestly as *nothing is cooling*.
/// - 6, the siege: `RngStream::COUNT` 10 → 11, unreadable to format 5 for the
///   reason 3 was. A siege is serialised as the component itself, see
///   [`NodeSave::siege`].
/// - 7, the besieging army renamed: `/tower/bailey/host` →
///   `/tower/bailey/enemy`, since §9b's remote hosts would have shared the
///   word. Migrated rather than refused — a path is a string.
/// - 8, a running siege gaining a resource it was fought without.
///   `Siege::quintessence` would default to nothing to pledge with, so the
///   migration fills it. Old *replays* diverge — a refused pledge never rolls.
/// - 10, the tower learning to be shut: `tally`, `reached` and `opened` join
///   `[progress]`. Without the bump an older build drops `opened` in silence
///   and loads a sealed tower as an open one.
/// - 12, the menagerie becoming a logic puzzle (§19). Chant nodes this build
///   does not raise would be restored as orphans, with a `remaining` that never
///   empties; the migration drops them. `RngStream::Menagerie` keeps its index.
/// - 13, `circle`, a key where absent means *shut*. An open tower's document
///   lists every key it had and not this one, and would draw lesser beasts for
///   ever, so the migration adds it; a sealed one is left to
///   `mastery::caught_up`. `BeastSave`'s wiring became optional here too.
/// - 14, a beast's turned wires. Absence is the honest reading, so `migrate`
///   does nothing and the bump buys only the version gate.
/// - 15, calm-layer log drift removed. Every poisoned log in an older document
///   is that drift, which no round announced, so the migration clears them all.
/// - 16, a siege gaining `fortified` and the `Stilled` outcome, and stillness
///   distilled from `stilling-draught`. An older build would drop the field in
///   silence; the migration renames a quiet-draught already in the alembic, or
///   a brew in flight would land on no recipe.
/// - 17, troops leaving the arsenal for a standing garrison (§19, *Supply*).
///   An older build would drop `garrison` in silence; the migration turns held
///   troops into bodies, as a deploy would have, and drops the nodes.
/// - 18, standing supplies (§19, *Supply*). An older build would drop open
///   supplies and a siege's freshness in silence; absent is none open. A
///   siege's `fortified` stops lasting the whole siege, so `migrate` keeps a
///   room an older document already bought.
/// - 19, `demon:<name>` keys, whose absence means the circle cannot bind one.
///   An open tower's document lists every key it had and none of these, so the
///   migration adds them; a sealed one is left to `mastery::caught_up` — 13's
///   `circle`, again.
pub const FORMAT: u32 = 19;

/// Whether a count is nought, so the file stays quiet about it.
#[allow(clippy::trivially_copy_pass_by_ref)] // serde hands a reference
const fn is_nought(count: &u32) -> bool {
    *count == 0
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
    /// Informational only — nothing branches on it. Without it, the first
    /// question asked of a save that will not open has no answer.
    pub built_by: String,
    /// The master seed every random stream is derived from.
    pub seed: u64,
    /// Ticks elapsed since this world began.
    pub tick: u64,
    /// How many records have ever been pushed, across the world's whole life.
    ///
    /// Not a length: a running spell's cursor is a position in this sequence
    /// and has to survive the stream being truncated to a tail.
    pub sequence: u64,
    /// Whether this tower began sealed — a laboratory and nothing else.
    ///
    /// Part of the recorded start, since sealed and open diverge. Absent is
    /// *open*.
    #[serde(default)]
    pub sealed: bool,
    /// How long this game was begun to be.
    ///
    /// Part of the recorded start: the length decides every threshold. Absent
    /// is [`Length::Baseline`](crate::content::Length::Baseline).
    #[serde(default)]
    pub length: crate::content::Length,
}

/// When the player left, and by which clock.
///
/// `orbs-shell` stamps it, since the sim reads no wall clock (§19).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Away {
    /// The tick the save was written at. Equal to [`WorldSave::tick`] today;
    /// separate because Phase 11a's catch-up advances one and not the other.
    #[serde(default)]
    pub tick: u64,
    /// Seconds since the Unix epoch, or `0` if the platform would not say.
    #[serde(default)]
    pub unix: u64,
}

/// Where each of the streams stood. `RngStream::COUNT` is the number, and it
/// moves once a domain — so not "the eight", which this said while there were
/// nine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RngSave {
    /// One word position per stream, in `RngStream::index` order.
    ///
    /// Decimal strings because they are `u128` and TOML integers are `i64`;
    /// truncating would put every stream in the wrong place.
    pub positions: Vec<String>,
}

/// What the player has earned, found and chosen.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressSave {
    /// The wizard's name. A save outranks the environment (`session::Wizard`):
    /// a frontend seeds this from `USER` only when building a new world.
    #[serde(default)]
    pub wizard: String,
    /// Experience, which only ever rises. Concentration is derived from it and
    /// is deliberately not here.
    #[serde(default)]
    pub experience: u64,
    /// What the tower is known for — the one number that can fall.
    ///
    /// A bare `u64` and no `FORMAT` bump: nought is the honest reading of a
    /// document written before renown existed.
    #[serde(default)]
    pub renown: u64,
    /// How many foes have been bought off the next siege to arrive.
    ///
    /// No `FORMAT` bump, on `renown`'s rule. It is the *allowance*, not a
    /// setting, so a player who paid and then saved keeps what they paid for.
    #[serde(default)]
    pub petitioned: u32,
    /// How far the next herb has grown toward flowering (§19, *Supply*).
    ///
    /// No `FORMAT` bump, on `renown`'s rule: nought is the honest reading of a
    /// document written while verdant was one scroll a herb.
    #[serde(default, skip_serializing_if = "is_nought")]
    pub cultivated: u32,
    /// The tower's standing garrison, and the reserve a siege is holding back
    /// (§19, *Supply*).
    ///
    /// `Option` for `integrity`'s reason: `None` restores the floor, which is
    /// what a tower with no garrison of its own always fought with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub garrison: Option<GarrisonSave>,
    /// Every supply the arsenal is drawing (§19, *Supply*). Absent is none.
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "supply")]
    pub supplies: Vec<SupplySave>,
    /// How the tower's walls stand (§11.5's Integrity).
    ///
    /// `Option`, not a bare `u32`: nought would come back as a tower worn to
    /// nothing rather than one nobody had measured. `None` restores to whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<u32>,
    /// What the tower holds of §11.5's mana.
    ///
    /// `Option` for `integrity`'s reason; `None` restores to the ceiling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quintessence: Option<u32>,
    /// Which `verify` surfaces are cooling, and the tick each frees at.
    ///
    /// It travels, or quitting clears it (§19). Named pairs, since TOML has no
    /// null. Absent is *nothing is cooling*.
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
    /// Absent is *nothing counted*, so an older save re-earns its stations.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub tally: std::collections::BTreeMap<String, u32>,
    /// When the tower last made each thing, lately — its stores, as a rate.
    ///
    /// `None` is an older document, which `restore` opens with full stores;
    /// an empty map has genuinely made nothing.
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
    /// `Option`, for `integrity`'s reason; `None` restores everything open.
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
    /// The line, not the `Intent`s — see `Choices`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked: Option<String>,
    /// One entry per domain the rail has something to say about.
    #[serde(default, rename = "mark")]
    pub marks: Vec<MarkSave>,
}

/// The standing garrison: bodies at home, and those waiting out a siege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GarrisonSave {
    /// Bodies standing at home. Nought while they are on the wall.
    pub standing: u32,
    /// Bodies a hold brought mid-siege, joining when it ends.
    #[serde(default)]
    pub reserve: u32,
}

/// One open supply: what it is, where it draws, and how far this draw went.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplySave {
    /// The item.
    pub name: String,
    /// `siege` or `calm`.
    pub scope: String,
    /// The rate chosen, slowest being nought.
    pub chosen: usize,
    /// The rate the last draw ran at.
    pub running: usize,
    /// The tick it opened.
    pub since: u64,
    /// How many the current draw took.
    #[serde(default)]
    pub drawn: u32,
    /// What the current draw applied.
    #[serde(default)]
    pub applied: u32,
    /// Stopped this round, kept so opening it again draws no second ration.
    /// No `FORMAT` bump: absent is open, which is all a supply was before.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stopped: bool,
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
/// Carried, since every `.log` is the stream filtered by domain.
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
    /// Not optional for an eldritch record: `RecordBuilder::finish` asserts one
    /// is present, so a restore that dropped it trips that assertion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spoken: Option<String>,
    /// Its fields, in order, as `[name, type, value]`.
    ///
    /// Three elements because a count right-aligns in a table and a tick is a
    /// point in world time; stored all as text, a brew's log redraws ragged.
    #[serde(default)]
    pub fields: Vec<(String, String, String)>,
}

impl Save {
    /// Render as TOML, for a file and for a person.
    ///
    /// # Errors
    ///
    /// If the document cannot be represented — a bug here rather than anything
    /// a world could reach, every field being a plain scalar or a sequence.
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
        // Older is migrated where migration is honest and refused where it is
        // not. Refusing only the future is half a version gate — serde drops
        // fields it no longer knows in silence — and refusing every older
        // format is the other half, most bumps being `RngStream::COUNT`
        // increases a save can simply be padded to. See `migrate`.
        let save = super::migrate::migrate(save)?;
        // Checked rather than clamped: a short list leaves streams at word
        // zero, the silent rewind `Rngs::positions` exists to prevent, and §15
        // invites hand-editing so this is a file a person can really produce.
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
