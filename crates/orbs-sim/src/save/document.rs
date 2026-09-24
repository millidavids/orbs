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
pub const FORMAT: u32 = 14;

/// Bring an older document up to [`FORMAT`], or say why it cannot be.
///
/// A save is refused only when the new model cannot state what the old one
/// meant — the ward rework changed what a *scored reading* means — not merely
/// because the number went up. Keep the table level with the function: two rows
/// were once missing for migrations `migrate` had been performing since
/// `0.8.13`.
///
/// | Bump | What changed | Migratable |
/// |---|---|---|
/// | 1 → 2 | The lens's scoring model | No — old readings mean nothing now |
/// | 2 → 3 | `RngStream::COUNT` 8 → 9, and the `battlements/` → `sanctum/` rename in the same commit | No — a format-2 save names a room and four fixtures this build lacks, and `adopt` addresses nodes by path |
/// | 3 → 4 | `RngStream::COUNT` 9 → 10 (`Menagerie`) | Yes — pad |
/// | 4 → 5 | `bide until` withdrawn from the language | Yes, with the caveat below |
/// | 5 → 6 | `RngStream::COUNT` 10 → 11 (`Siege`) | Yes — pad |
/// | 6 → 7 | `/tower/bailey/host` → `/tower/bailey/enemy`, the first *content* rename | Yes — rewrite the path |
/// | 7 → 8 | `Siege::quintessence`, a resource a running siege was fought without | Yes — fill the pool |
/// | 8 → 9 | `RngStream::COUNT` 11 → 12 (`Forge`), and quintessence moving from the siege to the tower | Yes — pad, and lift the pool |
/// | 9 → 10 | `[progress]` gains `tally`, `reached` and `opened` | Yes — every field defaults to the honest reading; `opened` absent is *everything open* |
/// | 10 → 11 | `[world]` gains `length`, `[progress]` gains `stores` | Yes — `length` absent is the curve as authored, `stores` absent is *full* |
/// | 11 → 12 | The menagerie's chant replaced by the circle: four syllable nodes and their readings, gone | Yes — drop the nodes; a figure mid-song loads as a circle with no beast |
/// | 12 → 13 | `[progress] opened` gains `circle`, whose absence draws lesser beasts | Yes — an open tower gains the key; a sealed one catches up on its own |
/// | 13 → 14 | A waiting beast gains `turned`, its turned wires | Yes — absent is none turned, which is what it was |
///
/// Padding a stream is exact: `Rngs::restore` winds a stream forward from the
/// master seed by the stored position, so nought is precisely a world that
/// never drew from it.
///
/// The language bump's caveat is survivable. A saved `bide until` compiles to a
/// line the orb cannot read — a fault the player can see and fix, in a file
/// whose text is theirs (§19) — and §8's taxonomy is *"scripts always log and
/// never halt"*.
fn migrate(mut save: Save) -> Result<Save, super::SaveError> {
    /// The oldest format whose meaning survives into the current model.
    ///
    /// 3, not 2: the 2 → 3 bump shipped with the `battlements/` → `sanctum/`
    /// rename (`de29565`, v0.4.2), and `adopt::apply` addresses nodes by path,
    /// so accepting a format-2 document dropped the entire sanctum without a
    /// word. Not a string rewrite like `host` → `enemy` either — the old
    /// `rampart` is the *bailey's* fixture name now, so a format-2 path would
    /// have to be disambiguated by its parent.
    const OLDEST: u32 = 3;

    if save.world.format < OLDEST {
        return Err(super::SaveError::Behind {
            found: save.world.format,
            understood: FORMAT,
        });
    }

    // Pad, never truncate: a document with more streams is `Ahead` and was
    // refused above, so anything here is short or exact.
    while save.rng.positions.len() < crate::RngStream::COUNT {
        save.rng.positions.push("0".to_owned());
    }

    // The bailey's second band was `host` until format 7; a node is addressed
    // by path, so the rename is a string rewrite and nothing more.
    if save.world.format < 7 {
        const WAS: &str = "/tower/bailey/host";
        const NOW: &str = "/tower/bailey/enemy";
        for node in &mut save.nodes {
            if node.path == WAS {
                node.path = NOW.to_owned();
            }
        }
    }

    // A siege fought before quintessence existed opens with a full pool: nought
    // is not the fight that was saved, and the world it would be scaled against
    // is not in the document. Generous is the honest way to be wrong, and it
    // lasts one siege.
    if save.world.format < 8 {
        for node in &mut save.nodes {
            if let Some(siege) = node.siege.as_mut()
                && siege.quintessence.is_none_or(|held| held == 0)
            {
                siege.quintessence = Some(crate::tower::QUINTESSENCE_BASE);
            }
        }
    }

    // 8 → 9: the pool came up to the tower, the forge spending what a siege
    // used to spend privately (§19). The largest is lifted — exact in practice,
    // there being one bailey. A save with no siege gets the ceiling rather than
    // nought, which would read as an inert forge; the ceiling is not in the
    // document, so `None` travels and `restore` fills it.
    if save.world.format < 9 {
        let carried = save
            .nodes
            .iter_mut()
            .filter_map(|node| node.siege.as_mut())
            .filter_map(|siege| siege.quintessence.take())
            .max();
        save.progress.quintessence = carried;
    }

    // 11 → 12: the chant's nodes go, by prefix rather than path — a syllable's
    // readings are children and an orphan would be restored at the root. The
    // circle's children go too: a leftover `remaining` makes `if the circle is
    // empty` false for ever, and they are republished on the next `summon`.
    if save.world.format < 12 {
        const ROOM: &str = "/tower/menagerie/";
        const SYLLABLES: [&str; 4] = ["skyward", "earthward", "leftward", "rightward"];
        let chant = |path: &str| {
            let Some(rest) = path.strip_prefix(ROOM) else {
                return false;
            };
            let head = rest.split('/').next().unwrap_or(rest);
            SYLLABLES.contains(&head) || rest.starts_with("circle/")
        };
        save.nodes.retain(|node| !chant(&node.path));
    }

    // 12 → 13: an open tower gains the key it could not have named. A sealed
    // one is left to `mastery::caught_up`.
    if save.world.format < 13
        && !save.world.sealed
        && let Some(opened) = save.progress.opened.as_mut()
        && !opened.iter().any(|key| key == crate::tower::opened::CIRCLE)
    {
        opened.push(crate::tower::opened::CIRCLE.to_owned());
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
    /// Part of the recorded start, beside the seed: a sealed and an open tower
    /// with the same seed diverge at the first `attend archive`, so a replay
    /// has to know which it is rebuilding. Absent is *open*.
    #[serde(default)]
    pub sealed: bool,
    /// How long this game was begun to be.
    ///
    /// Part of the recorded start, for `sealed`'s reason and more sharply: the
    /// length decides every threshold on both tracks, so a save that lost it
    /// loads into a world whose stations sit elsewhere.
    ///
    /// Absent is [`Length::Baseline`](crate::content::Length::Baseline), the
    /// curve as authored — and what the `FORMAT` 11 bump buys, an older build
    /// having dropped the field in silence.
    #[serde(default)]
    pub length: crate::content::Length,
}

/// When the player left, and by which clock.
///
/// The sim never fills this in — a wall-clock reading inside a sim record would
/// make two runs of one seed differ (§19) — so `orbs-shell` stamps `unix` on
/// the way out. It exists before offline progression does because a departure
/// time cannot be recovered after the fact.
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
    /// How the tower's walls stand (§11.5's Integrity).
    ///
    /// `Option`, not a bare `u32`: nought would come back as a tower worn to
    /// nothing rather than one nobody had measured. `None` restores to whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<u32>,
    /// What the tower holds of §11.5's mana.
    ///
    /// `Option` for `integrity`'s reason: nought would load every returning
    /// player with an inert forge and a siege that cannot pledge. `None`
    /// restores to the ceiling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quintessence: Option<u32>,
    /// Which `verify` surfaces are cooling, and the tick each frees at.
    ///
    /// It travels, because a cooldown that reset on load is one a player clears
    /// by quitting (§19). Absent is *nothing is cooling*.
    ///
    /// Named pairs rather than a slot per surface: TOML has no null, so a
    /// positional `Vec<Option<u64>>` does not serialise at all, and naming each
    /// surface keeps a later one from renumbering the rest.
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
    /// Absent is not *nothing made*: a standing is how many were made inside a
    /// window, so an empty map reads as *every store is out*. `restore` stamps
    /// an absent map at the loaded tick and the tower opens with full stores.
    /// No `FORMAT` bump — an older document still loads, just generously.
    ///
    /// `Option`, not a bare map: only `None` tells *written before stores
    /// existed* from *has genuinely made nothing*, and stamping both cost a
    /// fresh save its round-trip idempotence.
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
    /// `Option`, for `integrity`'s reason: an empty list would seal every room
    /// but the laboratory on a tower that stood in all seven. `None` restores
    /// to everything open, because that is the tower it was.
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
    /// The line, not the resolved `Intent`s `Choices` holds, which would pin
    /// the format against every parser change. The tie-break draws no
    /// randomness (§19), so re-asking on the way in reproduces the same list.
    /// Without this, a save taken with a prompt open came back with no answer
    /// that resolved.
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
/// A save carries records because a `.log` is not a file — it is `Scrollback`
/// filtered by domain — so dropping the stream empties every log in the tower,
/// and §8.1's poisoned-log tell, which re-emits existing lines, would answer
/// `verify` with *tampered* over nothing at all.
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
        // increases a save can simply be padded to. See [`migrate`].
        let save = migrate(save)?;
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
