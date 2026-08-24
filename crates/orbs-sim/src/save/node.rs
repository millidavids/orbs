//! One node of the tower, and everything that can be true of it.
//!
//! Every field past `path` and `kind` is optional and skipped when it is not
//! there, so a shelf with three reagents on it is four short rows and the
//! instrument that is halfway through a run is the one that catches the eye.
//! That is §15's readable save doing its job rather than being claimed.
//!
//! **The filesystem root is not here.** It is nameless (`build.rs` spawns it as
//! `(NodeId, Protected)` with no `Name`, so `path_of` contributes nothing for
//! it), which would make `path` optional for exactly one row in the document.
//! It does not need to be: a restore raises the tower before it adopts, so the
//! root always exists already. See [`super::restore`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A node, by path, with whatever is true of it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSave {
    /// `/tower/laboratory/mortar_and_pestle`. **How a node is addressed** — by
    /// this and nothing else, here and in every reference from another node.
    pub path: String,
    /// The node's `NodeId`.
    ///
    /// # Data, not an address — and the difference is the whole point
    ///
    /// Nothing is looked up by this; [`path`](Self::path) is the identity. It
    /// travels because a `NodeId` is **read by the sim as an ordering key**:
    /// `spell::advance` sorts running spells by it so that *"two spells'
    /// instructions must interleave the same way on every run from a seed"*.
    ///
    /// Ids are issued by a counter in spawn order, so a restored world that
    /// re-issued them would hand two player-written spells their ids in walk
    /// order rather than the order they were written — and the two would take
    /// the production slot in the opposite order, from the same seed, with
    /// nothing on screen to explain it. Carrying the number costs one field and
    /// closes that.
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
    /// **Carried, exactly as [`reading`](Self::reading) is**, and for the reason
    /// that field is: both are declared by `build`'s fixture tables and both are
    /// re-inserted by `adopt`, so a restore that dropped them would leave a
    /// tower whose ways answer `survey` and whose `for each way` walks nothing.
    /// A save says what a node *is*; deriving half of that on load and reading
    /// the other half is how the two halves come to disagree.
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
    /// A spell's lines, **exactly as the player typed them**. The orb never
    /// rewrites a spell (§19), and a save is not the place to start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held: Option<Vec<String>>,
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

/// An interval, as §8 asks for one: **a start and a completion tick.**
///
/// # Why both are absolute, when two of the three are not in the world
///
/// `Triaging` holds a start and an end; `Burning` and `Quickened` hold a start
/// and a *budget*. One `SpanSave` used to carry whichever the component had, in
/// a field called `ticks` — so a save read `[node.triaging] started = 100, ticks
/// = 104` beside `[node.burning] started = 2, ticks = 600`, and the same field
/// name meant a tick in one row and a duration two rows down.
///
/// Both round-tripped correctly. It was a trap for the reader, not the code —
/// and §15 makes hand-editability a stated criterion, so a reader being able to
/// tell what a number means is the criterion rather than a nicety. §8's own
/// words are *"first-class serialisable entities **with start and completion
/// ticks**"*, and that is now what the file says in every case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanSave {
    /// The tick it began.
    pub started: u64,
    /// The tick it ends on. A budget is `ends - started`.
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
/// # Why a picture and not 759 tables
///
/// `SPAN_X × SPAN_Y` is 33 × 23, so an array-of-tables of `Square` *is* the save
/// file — several hundred lines of `{ wall = true, marks = 0 }` around the
/// dozen rows anyone would want to read. §15 makes readable-and-editable a
/// stated criterion, and this is the one field large enough to decide whether
/// that criterion is met.
///
/// So the walls are the map as it is drawn on screen, one character per square,
/// and the marks are sparse because almost every square has none.
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
    /// **One string with newlines in it, not a list of them**, because `toml`
    /// renders that as a `"""` block and renders a list on a single line. The
    /// difference is the whole argument for this representation: a save is meant
    /// to be *read*, and a labyrinth on one line is not a labyrinth.
    pub walls: String,
    /// `[square, times walked]`, for the squares that have been walked at all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<(usize, u8)>,
}

/// A ward, mid-solve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardSave {
    /// The answer. Sigil indices, one per socket.
    pub code: Vec<usize>,
    /// What the aperture is holding now.
    pub aperture: Vec<usize>,
    /// What the last press held — what a refused press snaps back to.
    pub held: Vec<usize>,
    /// How many of the last press were right.
    pub aligned: u32,
    /// How many were the right sigil in the wrong socket.
    pub astray: u32,
    /// The best `aligned` any press has reached.
    pub best: u32,
    /// Whether the aperture has been pressed at all.
    pub pressed: bool,
    /// Presses spent.
    pub spent: u32,
    /// Whether the last press gained, held or lost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shift: Option<String>,
    /// How many times each sigil has been dialled.
    pub sigil_marks: Vec<u32>,
    /// How many times each socket has been dialled.
    pub socket_marks: Vec<u32>,
    /// Which sigils each socket has already tried, as one string of `.`/`x` per
    /// socket — a grid, drawn the way the board draws one.
    pub tried: Vec<String>,
    /// Sockets proved right and therefore locked.
    pub settled: Vec<bool>,
    /// Sockets dialled since the last press.
    pub touched: Vec<bool>,
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

/// A spell part-way through, which §8 requires a save to carry.
///
/// The compiled `Program` is **not** here: it is a derived view rebuilt from the
/// spell's own text at every cast, and `tower::spell::program` is explicit that
/// the text is the single source of truth. What is here instead is a
/// [`fingerprint`](Self::fingerprint) of the text it was compiled *from* — see
/// `save::restore` for why re-deriving it blind would not be safe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningSave {
    /// The spell being run, **as a path**.
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
    /// **One integer per block rather than a tagged table**, which is the call
    /// this format made when there were two shapes and is worth restating now
    /// there are three: a `loops = [2, -2, -4]` row is legible beside the `pc`
    /// it belongs to, where three tables of one field each would bury it.
    pub loops: Vec<i64>,
    /// How far into the record stream this run has read.
    pub seen: u64,
    /// How deep a nested invocation is.
    pub depth: u8,
    /// Whether it is running without the player in the room.
    pub unattended: bool,
    /// Where it is running, **as a path**.
    pub at: String,
    /// When the current instruction first blocked, if it has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiting_since: Option<u64>,
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
    /// A fingerprint of the spell text this program was compiled from.
    pub fingerprint: u64,
}
