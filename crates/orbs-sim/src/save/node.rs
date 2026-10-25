//! One node of the tower, and everything that can be true of it.
//!
//! Every field past `path` and `kind` is skipped when absent, so the node
//! halfway through a run catches the eye (§15). The nameless root is not here:
//! a restore raises the tower before it adopts ([`mod@super::restore`]).

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
    /// It travels because `spell::advance` orders running spells by it;
    /// re-issued ids would swap which spell took the slot first.
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
    /// Carried, as [`reading`](Self::reading) is, or `for each way` walks
    /// nothing after a load.
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
    /// Carried, since a trained reader may not read alike on another machine.
    /// Only those lines, keyed by text, so a hand-edited line compiles as
    /// written. Absent is a spell that is its own reading.
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

    // --- the intervals: absolute start and end ticks, never countdowns (§8),
    // so `meditate 300` behaves like three hundred watched ticks.
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
    /// What the window divides a run by. No `FORMAT` bump, on `renown`'s rule:
    /// absent is a half, which is all a window was before it said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quickened_by: Option<u64>,
    /// The forge's enchantments, in the order they were laid.
    ///
    /// A `Vec`: empty and absent both mean *nothing charmed*, so an older
    /// document reads back honestly with no format bump.
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
    /// An older document's `chant` is not read; `document::migrate` removes
    /// its nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beast: Option<BeastSave>,
    /// The bailey's siege.
    ///
    /// The component itself, since a `Siege` *is* its state, where a `Course`
    /// is computed from its raising. An ended siege travels too, for the
    /// postmortem.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub siege: Option<tower::Siege>,
    /// The forge's open lattice.
    ///
    /// The component itself, as `siege` is; the tool as a path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<tower::lattice::Binding>,
    /// What a rewritten spell actually said (§8.1's script-text surface).
    ///
    /// Or the corruption travels and the truth does not, and `purge` repairs
    /// nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewritten: Option<Vec<String>>,
    /// How much a retimed spell's clock is dragged by.
    ///
    /// [`rewritten`](Self::rewritten)'s rule, or `verify` reports tampering
    /// with nothing behind it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retimed: Option<u64>,
    /// What is waiting in this room's satchel, oldest first.
    ///
    /// A list, since a queue has order and duplicates. Empty is `None`.
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
    /// What a quickening or a charm divided it by. No `FORMAT` bump, on
    /// `quickened_by`'s rule: absent is unhurried.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hurried_by: Option<u64>,
}

/// An interval, as §8 asks for one: a start and a completion tick.
///
/// Both absolute, though `Burning` and `Quickened` hold a budget, so one name
/// never means a tick in one row and a duration in the next (§15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanSave {
    /// The tick it began.
    pub started: u64,
    /// The tick it ends on. A budget is `ends - started`.
    pub ends: u64,
}

/// One enchantment on a tool: what it does, and the span it runs for.
///
/// Named, as `Cooling` is, by `charm::Kind::word`, so a new charm renumbers
/// nothing. A start and a completion tick, like every interval here.
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
/// A picture, not 759 tables (§15): the walls one character a square, the
/// marks sparse.
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
    /// One string, which `toml` renders as a `"""` block; a list would be one
    /// line.
    pub walls: String,
    /// `[square, times walked]`, for the squares that have been walked at all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<(usize, u8)>,
}

/// A ward, mid-solve.
///
/// See `tower::ward`.
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
/// Three lists of numbers: a station is an index, so `naming.rs` has no table.
/// The height travels, since a course is raised at one; `Course::from_save`
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
/// What the beast *is*, never an index into the circuits, which would mean
/// another beast the day the list grew; `Beast::restored` checks it. The
/// length of `temper` says which circle.
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
/// Not the `Program`, which is rebuilt from the text at every cast, but a
/// [`fingerprint`](Self::fingerprint) of that text — see `save::restore`.
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
    /// One integer per block, so `loops = [2, -2, -4]` reads beside its `pc`.
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
    /// `waiting_since`'s pair: with one alone, `bide 3600` reloads into
    /// another hour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biding: Option<u32>,
    /// Lines whose bad name has already been complained about.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub said: Vec<usize>,
    /// What each name the spell has bound stands for.
    ///
    /// A map, so the file reads `best = "north"` (§15), ordered by key so one
    /// seed writes one document.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: BTreeMap<String, String>,
    /// Which part the spell is inside, if it is inside one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// The callers waiting on it, outermost first.
    ///
    /// Defaulted: an empty stack is the ordinary case.
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
    /// Only when there is more than one: the flat fields above *are* the
    /// first, so an ordinary spell's save is unchanged by forking.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strands: Vec<StrandSave>,
}

/// One forked cursor of a spell, as a save writes it.
///
/// [`RunningSave`]'s per-position half. `seen` is per cursor, or one cursor
/// satisfies another's `wait`.
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
    /// Defaulted, as `stack` is; an older save's one shared store is
    /// [`RunningSave::vars`].
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: BTreeMap<String, String>,
}
