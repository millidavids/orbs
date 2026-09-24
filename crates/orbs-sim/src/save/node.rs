//! One node of the tower, and everything that can be true of it.
//!
//! Every field past `path` and `kind` is optional and skipped when it is not
//! there, so a shelf with three reagents on it is four short rows and the
//! instrument that is halfway through a run is the one that catches the eye.
//! That is §15's readable save doing its job rather than being claimed.
//!
//! The filesystem root is not here. It is nameless, so `path` would have to be
//! optional for exactly one row — and it need not be, because a restore raises
//! the tower before it adopts. See [`mod@super::restore`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::tower;

/// One line of a spell a reader made something else of, beside what it made.
///
/// Keyed by the text, never by position: a load puts a reading back only on a
/// line that still says what was read, so a hand-edited line compiles as written
/// and every other line keeps its reading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadSave {
    /// The line as the player wrote it.
    pub written: String,
    /// What the orb reads it as, which is what compiles in its place.
    pub read: String,
}

/// A node, by path, with whatever is true of it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSave {
    /// `/tower/laboratory/mortar_and_pestle`. How a node is addressed, here and
    /// in every reference from another node.
    pub path: String,
    /// The node's `NodeId`. Data, not an address — nothing is looked up by it,
    /// [`path`](Self::path) is the identity.
    ///
    /// It travels because the sim reads a `NodeId` as an ordering key:
    /// `spell::advance` sorts running spells by it so two spells interleave the
    /// same way on every run from a seed. Ids are issued in spawn order, so a
    /// world that re-issued them would give two player-written spells their ids
    /// in walk order and swap which took the production slot first.
    pub id: u64,
    /// What kind of noun it is, in the save's own words — see `save::naming`.
    pub kind: String,

    // --- the markers. Each is a unit component in the world.
    /// Undestroyable (§7's guard).
    #[serde(default, skip_serializing_if = "not")]
    pub protected: bool,
    /// Furniture in a room rather than a place to stand.
    #[serde(default, skip_serializing_if = "not")]
    pub fixture: bool,
    /// The athanor.
    #[serde(default, skip_serializing_if = "not")]
    pub heat_source: bool,
    /// A shelf.
    #[serde(default, skip_serializing_if = "not")]
    pub store: bool,
    /// The arsenal, which is reachable from every room.
    #[serde(default, skip_serializing_if = "not")]
    pub keep: bool,
    /// A compass bearing, a socket, or a sigil.
    #[serde(default, skip_serializing_if = "not")]
    pub reading: bool,
    /// The set a spell's `for each` walks this node as one of — `way`, `socket`.
    ///
    /// Carried, exactly as [`reading`](Self::reading) is: both are declared by
    /// `build`'s tables and re-inserted by `adopt`, so a restore that dropped
    /// them leaves a tower whose ways answer `survey` and whose `for each way`
    /// walks nothing. A save says what a node *is*.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group: String,
    /// What a recipe was for, as opposed to what it left behind.
    #[serde(default, skip_serializing_if = "not")]
    pub product: bool,
    /// A tampered surface (§8.1).
    #[serde(default, skip_serializing_if = "not")]
    pub poisoned: bool,
    /// A `.log`, and therefore poisonable.
    #[serde(default, skip_serializing_if = "not")]
    pub log: bool,

    // --- the small carried values.
    /// The instrument's own verb — `grind`, `digest`, `mix`, `distil`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// `endless`, or a count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stock: Option<String>,
    /// A spell's lines, exactly as the player typed them. The orb never rewrites
    /// a spell (§19), and a save is not the place to start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held: Option<Vec<String>>,
    /// The lines a reader made something else of, each beside the text it read
    /// — what compiles in their place.
    ///
    /// Carried, not re-derived: `analyse` is deterministic and re-read on load,
    /// but once a trained reader did part of it, a machine with no weights — or
    /// different ones — would build a different program from the same file.
    ///
    /// Only those lines, found by their text. It was every line by position: a
    /// spell no reader touched was written out twice, and a hand-edited `held`
    /// compiled the reading of whatever line used to be there. Absent in a save
    /// written before readings existed, which is a spell that is its own reading
    /// throughout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<Vec<ReadSave>>,
    /// Which domain a spell was written for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Fuel damped before it burned through.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banked: Option<u64>,
    /// What a spent burn will leave behind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ash: Option<Vec<String>>,
    /// The spell that asked for the run in flight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bidden: Option<String>,
    /// Lines whose bad name a held spell has already complained about.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound: Option<Vec<usize>>,

    // --- the intervals. Absolute start and end ticks, never countdowns:
    // §8 wants in-flight actions serialisable *with start and completion
    // ticks*, and comparing against the clock is what makes `meditate 300`
    // inside one `step` behave like three hundred watched ticks.
    /// Work in flight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working: Option<WorkingSave>,
    /// A scour in flight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triaging: Option<SpanSave>,
    /// The fire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burning: Option<SpanSave>,
    /// A quickening window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quickened: Option<SpanSave>,
    /// The forge's enchantments, in the order they were laid.
    ///
    /// A `Vec` rather than an `Option<Vec>`: empty and absent both mean *nothing
    /// charmed*, and `skip_serializing_if` keeps the file quiet either way. No
    /// format bump needed for that reason (the `cooling` precedent) — a document
    /// written before charms existed reads back honestly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub charms: Vec<CharmSave>,
    /// A reagent claiming a name that is not its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub substituted: Option<SubstitutedSave>,

    // --- the two big ones, and the interpreter.
    /// The archive's labyrinth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maze: Option<MazeSave>,
    /// The lens's ward.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ward: Option<WardSave>,
    /// The sanctum's course.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub course: Option<CourseSave>,
    /// The beast waiting at the menagerie's circle.
    ///
    /// A format-11 document's `chant` is simply not read — `NodeSave` allows
    /// unknown fields, so a figure mid-song loads as a circle with no beast,
    /// which is a `summon` away from fine. Its *nodes* are another matter, and
    /// `document::migrate` removes them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beast: Option<BeastSave>,
    /// The bailey's siege.
    ///
    /// The component itself, not a derived shape, unlike the sanctum's course: a
    /// `Course` stores what it was raised at because the rest is computed, where
    /// a `Siege` *is* its state, so a second shape would be a copy that can
    /// disagree. A siege that has ended still travels, so a save taken after
    /// `settle` reopens on the same postmortem.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub siege: Option<tower::Siege>,
    /// The forge's open lattice.
    ///
    /// The component itself, which is `siege`'s decision for `siege`'s reason: a
    /// binding *is* its state. The tool travels as a path, because an entity id
    /// means nothing across a save.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<tower::lattice::Binding>,
    /// What a rewritten spell actually said (§8.1's script-text surface).
    ///
    /// Without this the corruption travels and the truth does not, which is
    /// worse than saving neither: the spell reloads corrupt and still
    /// `Poisoned`, and `purge` clears the mark, reports `cleansed` and repairs
    /// nothing — the player's words gone with no path back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewritten: Option<Vec<String>>,
    /// How much a retimed spell's clock is dragged by.
    ///
    /// Same rule as [`rewritten`](Self::rewritten), and the mirror failure: the
    /// drag would vanish on load while `Poisoned` stayed, leaving `verify`
    /// reporting a permanent tampering with nothing behind it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retimed: Option<u64>,
    /// What is waiting in this room's satchel, oldest first.
    ///
    /// A list rather than a table, because it is a queue: children plus `Stock`
    /// collapses duplicates and has no order, and a satchel holding `heed` twice
    /// with one of them first is the whole point. Empty is `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub satchel: Option<Vec<String>>,
    /// A spell part-way through running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub running: Option<RunningSave>,
}

/// `#[serde(skip_serializing_if)]` wants a path, and `!` is not one.
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn not(flag: &bool) -> bool {
    !*flag
}

/// A run in flight: what it is doing, to what, and between which ticks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSave {
    /// The verb that started it.
    pub verb: String,
    /// What it is working on, **as a path**.
    pub subject: String,
    /// The tick it began.
    pub started: u64,
    /// The tick it lands on.
    pub ends: u64,
}

/// An interval, as §8 asks for one: a start and a completion tick.
///
/// Both absolute, though `Burning` and `Quickened` hold a *budget* in the world.
/// One `SpanSave` used to carry whichever the component had in a field called
/// `ticks`, so the same name meant a tick in one row and a duration two rows
/// down. Both round-tripped; it was a trap for the reader, and §15 makes
/// hand-editability a criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanSave {
    /// The tick it began.
    pub started: u64,
    /// The tick it ends on. A budget is `ends - started`.
    pub ends: u64,
}

/// One enchantment on a tool: what it does, and the span it runs for.
///
/// Named rather than positional, which is `Cooling`'s decision one resource
/// over: `[["log", 120]]` says what `[null, 120]` cannot, and adding a sixth
/// charm renumbers nothing in an older save. The word is `charm::Kind::word`,
/// the same one a player types, so there is no second spelling.
///
/// A start and a completion tick, like every other interval in this file — the
/// component holds a *budget*, and the conversion is at the boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharmSave {
    /// Which charm, by the word a player would type.
    pub kind: String,
    /// The tick it was laid.
    pub started: u64,
    /// The tick it lapses on. A budget is `ends - started`.
    pub ends: u64,
}

/// A lie, and the truth under it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstitutedSave {
    /// The name it really has.
    pub was: String,
    /// The tick the lie landed, which is what `settling` measures against.
    pub since: u64,
}

/// The labyrinth, written as a picture.
///
/// A picture and not 759 tables: `SPAN_X × SPAN_Y` is 33 × 23, so an
/// array-of-tables of `Square` *is* the save file — hundreds of lines of
/// `{ wall = true, marks = 0 }` around the dozen rows anyone wants to read, and
/// §15 makes readable-and-editable a criterion. So the walls are the map as
/// drawn, one character per square, and the marks are sparse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeSave {
    /// Squares per row.
    pub width: usize,
    /// Where the reading stands.
    pub at: usize,
    /// Where the way out is.
    pub exit: usize,
    /// The bearing the reading arrived by, if it has moved at all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub came: Option<String>,
    /// `way` or `glean`.
    pub errand: String,
    /// Squares holding something to gather.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spoils: Vec<usize>,
    /// The maze as a picture: `#` is a wall, `.` is floor, one row per line.
    ///
    /// One string with newlines, not a list of them: `toml` renders that as a
    /// `"""` block and a list on one line, and a labyrinth on one line is not a
    /// labyrinth.
    pub walls: String,
    /// `[square, times walked]`, for the squares that have been walked at all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<(usize, u8)>,
}

/// A ward, mid-solve.
///
/// Seven fields shorter than it was: `held`, `best`, `sigil_marks`,
/// `socket_marks`, `tried`, `settled` and `touched` were all scaffolding under
/// the exchange ambiguity, which repeats deleted. A save written against the old
/// shape cannot be read — `code` and `aperture` are all it shares. See
/// `tower::ward`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardSave {
    /// The answer. Sigil indices, one per socket.
    pub code: Vec<usize>,
    /// What the aperture is holding now — and what the next press will send.
    pub aperture: Vec<usize>,
    /// How many of the last press were right.
    pub aligned: u32,
    /// How many were the right sigil in the wrong socket.
    pub astray: u32,
    /// Whether the aperture has been pressed at all.
    pub pressed: bool,
    /// Presses spent.
    pub spent: u32,
    /// Which way `aligned` moved on the last press.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shift: Option<String>,
    /// Which way `astray` moved on the last press — Mastermind's other number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drift: Option<String>,
    /// Every press and its answer, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<HistorySave>,
}

/// One press, and what the ward said about it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySave {
    /// What was in the aperture.
    pub figure: Vec<usize>,
    /// Right sigil, right socket.
    pub aligned: u32,
    /// Right sigil, wrong socket.
    pub astray: u32,
}

/// A course of wards, mid-solve.
///
/// Three lists of numbers and nothing else, which is why `save/naming.rs` gains
/// no table for this domain: a station is an index rather than a named enum, so
/// there is no word to spell two ways.
///
/// The height travels though it is derivable, because a course is *raised* at a
/// height and the parity and the meter are facts about the raising rather than
/// about how many wards happen to stand. `Course::from_save` recomputes and
/// clamps it anyway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSave {
    /// Each station's wards, bottom first, as magnitudes. Always three lists.
    pub stations: Vec<Vec<usize>>,
    /// How many wards the course was raised with.
    pub height: usize,
    /// Hauls spent.
    pub hauls: u32,
}

/// A beast waiting at the menagerie's circle, part-limned.
///
/// The wiring, the turned wires and the temper travel, never an index into the
/// circuits a draw picks from: a beast is drawn once from
/// `RngStream::Menagerie` and cannot be re-rolled without moving that stream,
/// and an index would mean a different beast the day the list grew. So the
/// document says what the beast *is*, and `Beast::restored` checks some circuit
/// could have made it.
///
/// A lesser beast is four rows and no wiring, a whole one eight rows and both
/// pairs. The length of `temper` is which circle it is, so there is no third
/// field to disagree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastSave {
    /// The two senses the sunwise glyph is given, by index, lower first. Absent
    /// for a lesser beast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sunwise: Option<[usize; 2]>,
    /// The two senses the widdershins glyph is given. Absent for a lesser beast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widdershins: Option<[usize; 2]>,
    /// Which wires into the outer glyphs are turned, as four `0`s and `1`s —
    /// sunwise's first and second, then widdershins's. Absent when none is,
    /// which is every beast a format-13 document held; `0000` means the same.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turned: Option<String>,
    /// What every row must answer, row one first: `1` lit, `0` dark.
    ///
    /// A string of rows rather than a number: §15 invites a person to edit this
    /// file, and `01110110` reads as a truth table where `110` does not.
    pub temper: String,
    /// How each glyph stands — keystone, sunwise, widdershins — by humour word.
    ///
    /// Words, not indices, the call `WardSave` makes for its sigils. An
    /// unreadable word drops the beast rather than substituting one.
    pub glyphs: Vec<String>,
    /// How many times the beast has been called in.
    pub calls: u32,
    /// What the circle last answered, in `temper`'s shape. Absent before the
    /// first call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
}

/// A spell part-way through, which §8 requires a save to carry.
///
/// The compiled `Program` is **not** here: it is a derived view rebuilt from the
/// spell's own text at every cast, and `tower::spell::program` is explicit that
/// the text is the single source of truth. What is here instead is a
/// [`fingerprint`](Self::fingerprint) of the text it was compiled *from* — see
/// `save::restore` for why re-deriving it blind would not be safe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningSave {
    /// The spell being run, as a path.
    pub spell: String,
    /// Where the orb is in the file. A path, not a line: `[2, 1]` is the second
    /// step inside the third, and a save with a position and no loop counts
    /// would resume every enclosing `repeat` from its first turn.
    pub pc: Vec<usize>,
    /// Each open block, outermost first, as one number.
    ///
    /// Nought or more is a `repeat`'s remaining turns. `-1` is a `repeat until`,
    /// which counts nothing. `-2` is a branch of an `if`, which runs once. `-3`
    /// and below is a `for each`, at member `-3 - code` of its set.
    ///
    /// One integer per block rather than a tagged table: `loops = [2, -2, -4]`
    /// is legible beside the `pc` it belongs to, where three tables of one field
    /// each would bury it.
    pub loops: Vec<i64>,
    /// How far into the record stream this run has read.
    pub seen: u64,
    /// How deep a nested invocation is.
    pub depth: u8,
    /// Whether it is running without the player in the room.
    pub unattended: bool,
    /// Where it is running, as a path.
    pub at: String,
    /// When the current instruction first blocked, if it has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiting_since: Option<u64>,
    /// How many ticks the running `bide` was told to spend, if one is running.
    ///
    /// The pair with `waiting_since`, and it was left out: a `bide` is the two
    /// together, so carrying one made `run::bide` fall to its start arm and
    /// stamp a fresh `waiting_since`. `bide 3600` reloaded into another hour.
    /// It was defensible while `bide until` existed and the delay came off the
    /// world; a count is a literal with nothing to re-derive it from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biding: Option<u32>,
    /// Lines whose bad name has already been complained about.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub said: Vec<usize>,
    /// What each name the spell has bound stands for.
    ///
    /// A map, so the file reads `[node.running.vars]` / `best = "north"` — which
    /// §15's hand-editable criterion is what asks for. Ordered by key, because a
    /// document whose rows moved between two runs of one seed would fail the
    /// lockstep test that pins a snapshot as complete.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: BTreeMap<String, String>,
    /// Which part the spell is inside, if it is inside one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// The callers waiting on it, outermost first.
    ///
    /// Defaulted rather than required: it is absent from a save written before
    /// parts existed, and from every save of a spell that never calls one. An
    /// empty stack is the ordinary case.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stack: Vec<DescentSave>,
    /// A fingerprint of the spell text this program was compiled from.
    ///
    /// One hash covers every descent: each walks a tree found by name in *this*
    /// spell's text, and a spell is contained to one `.spell` file (§19). An
    /// `invoke`d spell is a second `Running` with a fingerprint of its own.
    pub fingerprint: u64,
    /// Every cursor after the first, for a spell that has forked (§8,
    /// `alongside`).
    ///
    /// Written only when there is more than one, and the fields above are always
    /// the first — so an ordinary spell's save is byte-for-byte what it was
    /// before forking existed, rather than carrying
    /// `[[node.running.strands]]` in every tower anybody saves. The flat fields
    /// are not a duplicate of `strands[0]`: they *are* it, and `capture` writes
    /// them from it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strands: Vec<StrandSave>,
}

/// One forked cursor of a spell, as a save writes it.
///
/// [`RunningSave`]'s per-position half, and nothing else: which spell it is, how
/// deep it sits and where it runs are facts about the *cast* and stay up there.
/// `seen` is here rather than there, and a review is what moved it — two cursors
/// sharing one record-stream mark makes one satisfy the other's `wait`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrandSave {
    /// Where this cursor is — see [`RunningSave::pc`].
    pub pc: Vec<usize>,
    /// Its open blocks, coded as [`RunningSave::loops`] codes them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loops: Vec<i64>,
    /// How far it has read the record stream.
    #[serde(default)]
    pub seen: u64,
    /// When its current instruction first blocked, if it has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiting_since: Option<u64>,
    /// How long its running `bide` is for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biding: Option<u32>,
    /// What it has bound.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: BTreeMap<String, String>,
    /// Which part it is inside, if it is inside one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// The callers waiting on it, outermost first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stack: Vec<DescentSave>,
}

/// One suspended caller of a part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescentSave {
    /// The part this frame was walking, or absent for the spell's own body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// Where in it — pointing at the call that suspended it.
    pub pc: Vec<usize>,
    /// Its open blocks, coded as [`RunningSave::loops`] codes them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loops: Vec<i64>,
    /// Its bindings, which the part it called cannot see.
    ///
    /// Defaulted rather than required, as `stack` is: absent from a save written
    /// before parts took arguments, and from every frame that had bound nothing.
    /// An old save reading as empty is correct — before parameters there was one
    /// shared store, carried by [`RunningSave::vars`].
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: BTreeMap<String, String>,
}
