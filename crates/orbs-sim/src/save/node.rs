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
//! root always exists already. See [`mod@super::restore`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::tower;

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
    /// The forge's enchantments, in the order they were laid.
    ///
    /// A `Vec` rather than an `Option<Vec>`: an empty list and an absent one mean
    /// the same thing — *nothing charmed* — and `skip_serializing_if` keeps the
    /// file quiet either way. **Needs no format bump for that reason**, which is
    /// the `cooling` precedent: a document written before charms existed reads
    /// back honestly rather than wrongly.
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
    /// The menagerie's figure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chant: Option<ChantSave>,
    /// The bailey's siege.
    ///
    /// **The component itself, not a derived shape**, which is where this
    /// differs from the sanctum's course. A `Course` stores only what it was
    /// raised at because everything else about it is computed; a `Siege` *is*
    /// its state — two bands, a telegraphed intent, and what has been staged —
    /// so a second shape would be a copy of the first with a chance to disagree.
    ///
    /// A siege that has ended still travels, deliberately: `settle` leaves the
    /// board up so the last thing that happened is readable, and a save taken
    /// afterwards should reopen on the same postmortem.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub siege: Option<tower::Siege>,
    /// The forge's open lattice.
    ///
    /// **The component itself**, which is `siege`'s decision for `siege`'s
    /// reason: a binding *is* its state — a charm, a tool and a part-worked
    /// puzzle — so a derived shape would be a copy with a chance to disagree.
    /// The tool travels as a **path**, because an entity id means nothing across
    /// a save.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<tower::lattice::Binding>,
    /// What a rewritten spell actually said (§8.1's script-text surface).
    ///
    /// **Without this the corruption travels and the truth does not**, which is
    /// worse than not saving either: the spell reloads corrupt, still
    /// `Poisoned`, and `purge` clears the mark, reports `cleansed` and repairs
    /// nothing — the player's own words gone with no path back. That is exactly
    /// the defect `triage::purge`'s two repair lines exist to prevent,
    /// reintroduced through the save.
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
    /// **A list rather than a table, because it is a queue.** Every other
    /// counted thing here is children plus `Stock`, which collapses duplicates
    /// and has no order; a satchel holding `skyward` twice with one of them
    /// first is the whole point of it. Empty is `None`, so a tower nobody has
    /// queued into writes no rows.
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

/// One enchantment on a tool: what it does, and the span it runs for.
///
/// **Named rather than positional**, which is `Cooling`'s decision one resource
/// over: `[["log", 120]]` says what a bare `[null, 120]` cannot, and it means
/// adding a sixth charm renumbers nothing in a save written before it. The word
/// is `charm::Kind::word` — the same one a player types and a spell reads, so
/// there is no second spelling to keep in step.
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
///
/// **Seven fields shorter than it was, because the ward stopped keeping them.**
/// It carried `held`, `best`, `sigil_marks`, `socket_marks`, `tried`, `settled`
/// and `touched` while the code space was 360 and a dial *exchanged* two sockets
/// — all of them scaffolding under that one ambiguity. Repeats deleted the
/// exchange and the state went with it, so a save written against the old shape
/// cannot be read: `code` and `aperture` are all it shares, and the two count as
/// different formats. See `tower::ward` for the argument in full.
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
/// **Three lists of numbers and nothing else**, which is why `save/naming.rs`
/// gains no table for this domain: a station is an index rather than a named enum,
/// so there is no word to spell one way here and another way in the world. The
/// maze's `way` and `errand` needed that treatment; this does not.
///
/// The height travels even though it is derivable from the three lists, because
/// a course is *raised* at a height and everything about how it reads — the
/// parity, what the meter is against — is a fact about the raising rather than
/// about how many wards happen to be standing. `Course::from_save` recomputes
/// and clamps it anyway, so a hand-edited file cannot make the two disagree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSave {
    /// Each station's wards, bottom first, as magnitudes. Always three lists.
    pub stations: Vec<Vec<usize>>,
    /// How many wards the course was raised with.
    pub height: usize,
    /// Hauls spent.
    pub hauls: u32,
}

/// A figure part-way through being sung.
///
/// **This was documented before it existed.** `save::document`'s `FORMAT` 3 → 4
/// entry called `ChantSave` *"an ordinary addition"* and two fields of
/// `tower::Chant` justify their representation by a save they never reached —
/// `travelled` is held rather than derived from a start tick because *"a chant
/// can be saved mid-approach"*, and `sung` is a sequence partly because *"it is
/// what the save needs"*. Neither was true until now: reloading mid-figure
/// dropped the component and the circle came back empty.
///
/// **The whole chart travels**, unlike the sanctum's course, which stores only
/// where the wards are standing. A figure is drawn once from
/// `RngStream::Menagerie` and cannot be re-rolled on load without moving that
/// stream — `Chant::restored` exists precisely so a restore reads the figure
/// rather than drawing one, which is the ward's rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChantSave {
    /// The figure, in the order it lands, by syllable word.
    ///
    /// **Words, not indices.** A save is a file §15 invites a person to edit,
    /// and `skyward` says what `0` does not — the same call `WardSave` makes for
    /// its sigils. An unreadable word is dropped by `Chant::restored`'s caller
    /// rather than panicking, which is that section's rule for a hand-edited
    /// file.
    pub chart: Vec<String>,
    /// Which syllable is at the aperture.
    pub at: usize,
    /// How each answered syllable went, oldest first: `true` struck.
    pub sung: Vec<bool>,
    /// How far the syllable at the aperture has travelled, out of `PACE`.
    pub travelled: u32,
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
    /// How many ticks the running `bide` was told to spend, if one is running.
    ///
    /// **The pair with `waiting_since`, and it was left out.** A `bide` is the
    /// two together — when it started and how long it is for — so carrying one
    /// and dropping the other made `run::bide` fall to its start arm and stamp a
    /// *fresh* `waiting_since`, restarting the count. `bide 3600` reloaded into
    /// another whole hour.
    ///
    /// It was defensible while `bide until` existed: the delay came off the
    /// world, so re-reading it was *more* correct than restoring a stale number,
    /// and the field was *"always re-derivable"*. The reading form is gone, a
    /// count is a literal, and there is nothing left to re-derive it from.
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
    /// **One hash covers every descent, and it stays that way.** Each one walks
    /// a tree found by name in *this* spell's text, and a spell is contained to
    /// a single `.spell` file by decision (§19) — so one fingerprint answers for
    /// the whole stack, and there is no shape where it would not.
    ///
    /// An `invoke`d spell is a second `Running` with a fingerprint of its own,
    /// which is the same rule seen from the other side.
    pub fingerprint: u64,
    /// Every cursor after the first, for a spell that has forked (§8,
    /// `alongside`).
    ///
    /// **Written only when there is more than one**, and the fields above are
    /// always the first. So a save of an ordinary spell is byte-for-byte what it
    /// was before forking existed — which matters because *every* spell is
    /// ordinary and a format that spent a table on the empty case would put
    /// `[[node.running.strands]]` in every tower anybody ever saves.
    ///
    /// The flat fields are not a duplicate of `strands[0]`: they *are* it, and
    /// `capture` writes them from it so the two cannot drift.
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
    /// Defaulted rather than required, exactly as `stack` is: absent from a save
    /// written before a part took arguments, and from every frame that had
    /// bound nothing when it called. An empty store is the ordinary case, and
    /// an old save reading as one is correct — before parameters there was a
    /// single shared store, and it is [`RunningSave::vars`] that carries it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: BTreeMap<String, String>,
}
