//! The canonical command set.
//!
//! §6.1: one short word each, arcane register — experts type the canonical form
//! all day, and whichever register is canonical is the one players absorb.

/// What a command slot expects to be filled with.
///
/// Lets the parser resolve `clarity` against essences rather than every noun in
/// the tower, and reject a plausible argument in the wrong category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NounKind {
    /// A location in the tower. `attend /tower/laboratory`.
    Place,
    /// A readable file. `peruse feed.log`.
    File,
    /// Free text matched against file contents. `sift march feed.log`.
    Pattern,
    /// A manual topic. `recall brewing`.
    Topic,
    /// An essence that can be brewed. `recall clarity`.
    ///
    /// The quality a recipe produces, not a thing on a shelf — that is a
    /// [`Reagent`](Self::Reagent).
    Essence,
    /// Crafting stock: an ingredient, a part-made material, a byproduct, or fuel.
    ///
    /// One kind, because a husk is litter only until a recipe wants it.
    Reagent,
    /// A vessel holding a finished brew. `survey retort`.
    Vessel,
    /// A scroll, which is finished work you spend. `wield gleaning-scroll`.
    ///
    /// Not an [`Essence`](Self::Essence), or `wield clarity` resolves to
    /// nothing; not a [`Reagent`](Self::Reagent), which a recipe consumes.
    Scroll,
    /// A script. `invoke night_watch`.
    Script,
    /// A count of ticks. `meditate 30`.
    Count,
    /// A name the player is coining, not one the world holds. `scribe morning`.
    ///
    /// Free text, since the spell does not exist yet. Not [`Any`](Self::Any), or
    /// `scribe sage` would name a spell after a reagent.
    Name,
    /// What a room reports about its own state — `passage`, `wall`, `marks`.
    ///
    /// No slot asks for one, so it never fills a `Reagent` by accident, while
    /// [`Any`](Self::Any) still finds it — which lets `spell::compile` resolve a
    /// condition before the way has been walked.
    Sense,
    /// Anything with text in it — `peruse orb.log`, `peruse night_watch.spell`.
    ///
    /// A slot kind, never a noun's own. Not [`Any`](Self::Any), or `peruse sage`
    /// would read a reagent as zero lines.
    Readable,
    /// Anything you can pick up — stock, a potion, a scroll.
    ///
    /// A slot kind: `move`'s once took only a `Reagent`, so a potion could not be
    /// moved at all. Not [`Any`](Self::Any), or `move laboratory` resolves.
    Portable,
    /// Anything you can set going — an instrument, or a scroll.
    ///
    /// A slot kind. `empty` keeps [`Place`](Self::Place), or `empty
    /// gleaning-scroll` parses and the executor cannot answer.
    Workable,
    /// Anything that can be running — an instrument, a spell, or a supply.
    ///
    /// A slot kind, or a `repeat` with no count could never be stopped.
    Stoppable,
    /// A verb's own name, so `recall grind` can be asked about.
    ///
    /// Reachable from [`Subject`](Self::Subject) only, never [`Any`](Self::Any),
    /// which would leak every canonical into Tab, `spell::compile` and the
    /// numbered prompt.
    Command,
    /// Anything nameable — `verify` and `purge` accept any surface.
    Any,
    /// What the manual can answer on: a topic, or a command.
    ///
    /// A slot kind, so `recall` reaches both without widening [`Any`](Self::Any).
    Subject,
}

/// Which part of the manual a verb belongs under.
///
/// A table, sorted by what a lost player is trying to do. The overview prints
/// in this order: find your way, do the work, teach the orb, ask the orb, and
/// last the two that destroy something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    /// Reaching and reading: where you are, what is here, what it says.
    Getting,
    /// The work of a domain — everything that moves or makes.
    Work,
    /// Writing and running spells (§8).
    Spells,
    /// Asking the orb about itself, and about time.
    Orb,
    /// The two that destroy something. See [`Verb::is_destructive`].
    Careful,
}

impl Group {
    /// Every group, in the order the overview prints them.
    pub const ALL: [Self; 5] = [
        Self::Getting,
        Self::Work,
        Self::Spells,
        Self::Orb,
        Self::Careful,
    ];

    /// The prose key naming this group.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Getting => "man_group_getting",
            Self::Work => "man_group_work",
            Self::Spells => "man_group_spells",
            Self::Orb => "man_group_orb",
            Self::Careful => "man_group_careful",
        }
    }
}

impl NounKind {
    /// The kinds a [`Noun`](super::Noun) in the scene can actually be.
    ///
    /// Not the free-text kinds or the slot-only sets [`accepts`](Self::accepts)
    /// names. `Command` is here because `tower::scene_at` registers every verb
    /// word as one. One list, so `content::Phrasings` is not a second copy.
    pub const NAMEABLE: [Self; 10] = [
        Self::Place,
        Self::File,
        Self::Topic,
        Self::Essence,
        Self::Reagent,
        Self::Vessel,
        Self::Scroll,
        Self::Script,
        Self::Sense,
        Self::Command,
    ];

    /// The word for this category, as output names it.
    ///
    /// When a slot is empty the orb says what would fill it (§6), and it cannot
    /// say `Portable`. One lower-case word, to drop into a content file's
    /// sentence.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Place => "place",
            Self::File => "file",
            Self::Pattern => "pattern",
            Self::Topic => "topic",
            Self::Essence => "essence",
            Self::Reagent => "reagent",
            Self::Vessel => "vessel",
            Self::Scroll => "scroll",
            Self::Script => "script",
            Self::Count => "count",
            Self::Name => "name",
            Self::Sense => "reading",
            // The slot kinds ask for what the orb would say — "which file?",
            // not "which readable?" — and their commonest answer.
            Self::Readable => "file",
            Self::Stoppable => "place",
            Self::Workable => "place",
            Self::Portable => "reagent",
            Self::Any => "name",
            Self::Command => "command",
            Self::Subject => "topic",
        }
    }

    /// Whether a noun of kind `noun` may fill a slot wanting `self`.
    ///
    /// One definition for the matcher, the numbered prompt and Tab, which each
    /// had a copy.
    #[must_use]
    pub const fn accepts(self, noun: Self) -> bool {
        match self {
            // Everything except a command — see [`Command`](Self::Command).
            Self::Any => !matches!(noun, Self::Command),
            Self::Readable => matches!(noun, Self::File | Self::Script),
            // An instrument, a spell, or a supply the arsenal is drawing (§19,
            // *Supply*).
            Self::Stoppable => matches!(
                noun,
                Self::Place | Self::Script | Self::Essence | Self::Scroll
            ),
            Self::Workable => matches!(noun, Self::Place | Self::Scroll),
            Self::Portable => matches!(noun, Self::Reagent | Self::Essence | Self::Scroll),
            Self::Subject => matches!(noun, Self::Topic | Self::Command),
            // `as u8` because `PartialEq::eq` is not const.
            _ => self as u8 == noun as u8,
        }
    }
}

/// One argument position in a command's signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    /// What may fill it.
    pub kind: NounKind,
    /// Whether the command is incomplete without it.
    pub required: bool,
}

impl Slot {
    const fn required(kind: NounKind) -> Self {
        Self {
            kind,
            required: true,
        }
    }

    const fn optional(kind: NounKind) -> Self {
        Self {
            kind,
            required: false,
        }
    }
}

// Named constants, because a slice built from `const fn` calls is a temporary
// and is not promoted to `'static`.
const NOTHING: &[Slot] = &[];
const PLACE: &[Slot] = &[Slot::required(NounKind::Place)];
const PLACE_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Place)];
// `sift` does not take this yet: searching a spell is a different feature from
// reading one.
const READABLE: &[Slot] = &[Slot::required(NounKind::Readable)];
/// `stop` reaches an instrument, a running spell **or** an open supply.
const STOPPABLE: &[Slot] = &[Slot::required(NounKind::Stoppable)];
/// `wield` reaches an instrument **or** a scroll's supply.
const WORKABLE: &[Slot] = &[Slot::required(NounKind::Workable)];
const PATTERN_AND_FILE: &[Slot] = &[
    Slot::required(NounKind::Pattern),
    Slot::required(NounKind::File),
];
/// `recall [topic]` — optional, `survey`'s shape: bare and argumented are one
/// act at two scopes, and a required slot made bare `help` a numbered prompt.
const TOPIC_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Subject)];
const DEMON_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Essence)];
const ANYTHING: &[Slot] = &[Slot::required(NounKind::Any)];
/// `verify`'s: bare is the audit — see its `signature` arm.
const ANYTHING_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Any)];
const COUNT: &[Slot] = &[Slot::required(NounKind::Count)];

/// `move <reagent> [from <source>] to <destination>` — §10.1's pipeline.
///
/// The source is optional, needed only when two instruments hold the same
/// thing. `from` and `to` are filler ([`normalise`](super::normalise)), so the
/// slots fill positionally:
///
/// ```text
/// move sage to mortar_and_pestle              -> reagent, _, destination
/// move husks from alembic to dispensary       -> reagent, source, destination
/// move clarity to arsenal                     -> essence, _, destination
/// ```
const MOVE: &[Slot] = &[
    Slot::required(NounKind::Portable),
    Slot::optional(NounKind::Place),
    Slot::required(NounKind::Place),
];
/// What §10.1's per-instrument verbs take — optional, so bare `grind` starts a
/// mortar already charged and these verbs keep saving keystrokes.
const ONE_REAGENT: &[Slot] = &[Slot::optional(NounKind::Reagent)];

/// Two, for the one instrument that combines. `and` is filler, so the slots
/// fill positionally, as `move`'s do.
const TWO_REAGENTS: &[Slot] = &[
    Slot::optional(NounKind::Reagent),
    Slot::optional(NounKind::Reagent),
];

/// One of the archive's four readings — a `Place`, the only kind a question's
/// place half resolves against.
const WAY: &[Slot] = &[Slot::required(NounKind::Place)];

/// `dial <socket> [<sigil>]` — which dial, and what to turn it to.
///
/// Both `Place`, since a new kind leaks into Tab and `compile::fix`; the cost
/// is that `dial laboratory nitre` parses and is refused. The sigil is
/// optional: a variable-free script cannot name one it has not tried, so bare
/// steps it (`Ward::advance`).
const SOCKET_AND_SIGIL: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::optional(NounKind::Place),
];

/// `haul <from> <to>` — both required, since a spell naming two stations knows
/// both; the direction is what it works out, which is why `potency` is
/// published.
const STATION_AND_STATION: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::required(NounKind::Place),
];
/// `limn <glyph> [<humour>]` — `SOCKET_AND_SIGIL`'s shape and reason.
const GLYPH_AND_HUMOUR: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::optional(NounKind::Place),
];
/// `pledge <die> [to] <area>` — `to` is §6 filler.
const DIE_AND_AREA: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::required(NounKind::Place),
];
/// `imbue <tool> [with] <charm>` — `with` is §6 filler. The tool is a place
/// anywhere in the tower, this domain's one widening of §7.
const TOOL_AND_CHARM: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::required(NounKind::Place),
];
/// `snap` takes one column of the lattice.
const COLUMN: &[Slot] = &[Slot::required(NounKind::Place)];
const SCRIPT: &[Slot] = &[Slot::required(NounKind::Script)];
// `scribe` coins a name — see `NounKind::Name`.
const SPELL_NAME: &[Slot] = &[Slot::required(NounKind::Name)];

/// A canonical command.
///
/// Not `#[non_exhaustive]`, so a new verb forces every match — echo, help, the
/// balance harness — to account for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verb {
    /// Move to a place.
    Attend,
    /// List what is here.
    Survey,
    /// Read a file.
    Peruse,
    /// Filter for matches.
    Sift,
    /// Tower overview — the boot report.
    Status,
    /// The in-world manual — what the orb remembers about a subject.
    ///
    /// Not `grimoire`, the book you write in (§19).
    Recall,
    /// Detect tampering.
    Verify,
    /// Revert the last command.
    Undo,
    /// Read back through what the orb has said.
    ///
    /// A word for `PageUp`, so the key is taught. Not `recollect`: `rec` is
    /// `recall`'s.
    Unfurl,
    /// Put the orb down and leave.
    ///
    /// A word for `F10`, and what the editor and the weave say for *close this*.
    Quit,
    /// Open the orb's menu — not `quit`, which leaves (§19).
    Menu,
    /// Fast-forward the clock.
    Meditate,
    /// Carry a reagent from one place to another.
    Move,
    /// Set an instrument working on what is in it.
    Wield,
    /// Charge the mortar and pestle and set it going.
    Grind,
    /// Charge the water bath and set it going.
    Digest,
    /// Charge the flask and rod with two things and set it going.
    Mix,
    /// Charge the alembic and set it going.
    Distil,
    /// Feed the athanor and light it.
    Kindle,
    /// Turn everything an instrument holds out into the store.
    Empty,
    /// Cancel a working instrument, refunding what it holds, or close a supply.
    Stop,
    // `Siphon` is retired (§19): the per-instrument verbs reach into an idle
    // instrument themselves.
    /// Destroy what a tool holds, where `empty` shelves it.
    Purge,
    /// Research a fragment.
    Research,
    /// Author a script.
    Scribe,
    /// Attach a script to a trigger.
    Bind,
    /// Run a script or spell.
    Invoke,
    /// Move the archive's reading one cell through a maze (§10, `tower::maze`).
    ///
    /// Not `step` (750 against `stop`) or `tread` (800 against `read`).
    Follow,
    /// Look at what the work has bought (§11.5).
    ///
    /// A tower-wide word earned as `unfurl` did: a track nobody can look at is a
    /// track nobody is on. Not `ascend`, 667 against `attend`.
    Weave,
    /// Give the arrow keys the archive's stacks (§10, §19).
    ///
    /// The surface's only way in. Every obvious word collided: `thread`,
    /// `stride`, `delve`, `trace`, `pace`.
    Wander,
    /// Press the aperture against a far orb's ward, opening a reading if none is.
    ///
    /// Two acts in one word, as `grind sage` is. Not `scry`, whose `scr` reaches
    /// `scribe`.
    Probe,
    /// Turn one dial of the aperture.
    ///
    /// Free; only [`Probe`](Self::Probe) takes a tick. A player and a spell use
    /// the same word and differ only in which readings they consult. Not `seat`,
    /// whose `sea` reaches `search`.
    Dial,
    /// Draw a fresh course of wards up out of the wellspring (§10, `sanctum/`).
    ///
    /// Free like [`Probe`](Self::Probe); the hauling is the work, and a
    /// neglected tower musters more (see `tower::erosion`). Not `fortify`,
    /// `restore` or `mend`.
    Muster,
    /// Carry the topmost ward from one station to another.
    ///
    /// Directional, and it refuses, so a spell has `potency` to read (§19). A
    /// wrong haul is an ordinary refusal. Not `shift` or `heave`.
    Haul,
    /// Draw a beast up at the circle, or call the waiting one in (§10).
    ///
    /// Two acts in one word, like [`Probe`](Self::Probe). Free: a balked call
    /// costs only the call. Not `call`, 750 against `wall`.
    Summon,
    /// Let the enemy arrive, and stand to meet it. Takes nothing: one rampart.
    Defend,
    /// Open or raise a demon's supply on the wall (§19, *Supply*).
    Deploy,
    /// Open or raise a potion's supply on the wall. A scroll uses `wield` (§19).
    Quaff,
    /// Put one of your dice behind part of the wall.
    ///
    /// The domain's decision: three dice against four areas, so every round
    /// leaves something dark, and the range shows before you commit (§5.1).
    Pledge,
    /// Open a lattice to bind a charm onto a tool — §10's Enchanting.
    ///
    /// It names a tool in another room, this domain's one widening of §7.
    Imbue,
    /// Flip one column's glyph on the lattice's working row. Not `temper`, 625
    /// against `tampered`.
    Snap,
    /// Let the lattice cascade, and bind the charm if it lights.
    ///
    /// The domain's operation, holding the production slot so a charm competes
    /// with making things (§10). Not `settle`, 834 against `mettle`.
    Anneal,
    /// Spend standing so that fewer come up the road — renown's first purchase.
    ///
    /// At exactly [`Verb::MAX_CANONICAL_LEN`].
    Petition,
    /// End your turn and let one round resolve — the only thing in the domain
    /// that advances the world (§5.0).
    Hold,
    /// Limn one of the circle's glyphs; bare, step it round the six (§10).
    ///
    /// The lens's `dial`, optional for its reason. Free: holding the beast is
    /// the work. Not `etch`, `carve`, `inscribe` or `engrave`.
    Limn,
    /// Put a name in the satchel (§8, `tower::satchel`).
    ///
    /// The push half; `pull` is a [`SpellWord`](super::SpellWord) because it
    /// binds a name. Unanchored, since every domain has a satchel. `que` also
    /// offers `quench`, accepted; `stow` and `stash` collide worse.
    Queue,
}

impl Verb {
    /// Every verb in the Phase 0 vocabulary, and what the phases since have added.
    pub const ALL: [Self; 45] = [
        Self::Attend,
        Self::Survey,
        Self::Peruse,
        Self::Sift,
        Self::Status,
        Self::Recall,
        Self::Verify,
        Self::Undo,
        Self::Unfurl,
        Self::Quit,
        Self::Menu,
        Self::Meditate,
        Self::Move,
        Self::Wield,
        Self::Grind,
        Self::Digest,
        Self::Mix,
        Self::Distil,
        Self::Kindle,
        Self::Empty,
        Self::Stop,
        Self::Purge,
        Self::Research,
        Self::Scribe,
        Self::Bind,
        Self::Invoke,
        // Appended, never inserted: `the_tolerated_collision_set_is_pinned`
        // walks pairs in this order.
        Self::Weave,
        Self::Follow,
        Self::Wander,
        Self::Probe,
        Self::Dial,
        Self::Muster,
        Self::Haul,
        Self::Summon,
        // In the chant's old slot (§19).
        Self::Limn,
        Self::Queue,
        Self::Defend,
        Self::Deploy,
        Self::Quaff,
        Self::Hold,
        Self::Pledge,
        Self::Imbue,
        Self::Snap,
        Self::Anneal,
        Self::Petition,
    ];

    /// The longest a canonical verb may be.
    ///
    /// §6.1's "ideally ≤7" settled at 8, for words that carry the game's voice
    /// and have a short prefix (`medit`, `res`).
    pub const MAX_CANONICAL_LEN: usize = 8;

    /// Which part of the manual this verb is listed under.
    ///
    /// No wildcard, so a new verb is a compile error, not one missing from the
    /// overview.
    #[must_use]
    pub const fn group(self) -> Group {
        match self {
            Self::Attend | Self::Survey | Self::Peruse | Self::Sift | Self::Verify => {
                Group::Getting
            }
            // Every room's together: a player looking for what to do wants one
            // list.
            Self::Move
            | Self::Wield
            | Self::Empty
            | Self::Grind
            | Self::Digest
            | Self::Mix
            | Self::Distil
            | Self::Kindle
            | Self::Research
            | Self::Follow
            | Self::Wander
            // the lens's two
            | Self::Probe
            | Self::Dial
            // the sanctum's two
            | Self::Muster
            | Self::Haul
            // the menagerie's two
            | Self::Summon
            | Self::Limn
            // the bailey's six
            | Self::Defend
            | Self::Petition
            | Self::Deploy
            | Self::Quaff
            | Self::Hold
            | Self::Pledge
            // the forge's three
            | Self::Imbue
            | Self::Snap
            | Self::Anneal
            // and the satchel's push, which changes the world
            | Self::Queue => Group::Work,
            Self::Scribe | Self::Bind | Self::Invoke => Group::Spells,
            Self::Status
            | Self::Recall
            | Self::Undo
            | Self::Unfurl
            | Self::Weave
            | Self::Quit
            | Self::Menu
            | Self::Meditate => Group::Orb,
            // Kept in step with `is_destructive` by a test.
            Self::Purge | Self::Stop => Group::Careful,
        }
    }

    /// What this verb wants after it, as a single word, so a listing hints at
    /// what follows (§6). Empty for the verbs that take nothing.
    #[must_use]
    pub const fn signature_label(self) -> &'static str {
        match self.signature() {
            [] => "",
            [slot, ..] => slot.kind.label(),
        }
    }

    /// The arcane name — what the echo shows and what experts type.
    #[must_use]
    pub const fn canonical(self) -> &'static str {
        match self {
            Self::Attend => "attend",
            Self::Survey => "survey",
            Self::Peruse => "peruse",
            Self::Sift => "sift",
            Self::Status => "status",
            Self::Recall => "recall",
            Self::Verify => "verify",
            Self::Undo => "undo",
            Self::Unfurl => "unfurl",
            Self::Quit => "quit",
            Self::Menu => "menu",
            Self::Meditate => "meditate",
            Self::Move => "move",
            Self::Wield => "wield",
            Self::Grind => "grind",
            Self::Digest => "digest",
            Self::Mix => "mix",
            Self::Distil => "distil",
            Self::Kindle => "kindle",
            Self::Empty => "empty",
            Self::Stop => "stop",
            Self::Purge => "purge",
            Self::Research => "research",
            Self::Scribe => "scribe",
            Self::Bind => "bind",
            Self::Invoke => "invoke",
            Self::Weave => "weave",
            Self::Follow => "follow",
            Self::Wander => "wander",
            Self::Probe => "probe",
            Self::Dial => "dial",
            Self::Muster => "muster",
            Self::Haul => "haul",
            Self::Summon => "summon",
            Self::Limn => "limn",
            Self::Queue => "queue",
            Self::Defend => "defend",
            Self::Petition => "petition",
            Self::Deploy => "deploy",
            Self::Quaff => "quaff",
            Self::Hold => "hold",
            Self::Pledge => "pledge",
            Self::Imbue => "imbue",
            Self::Snap => "snap",
            Self::Anneal => "anneal",
        }
    }

    /// Whether this verb belongs to one instrument rather than to the tower.
    ///
    /// Not `wield`, which names its tool and is what a script writes when the
    /// instrument is the variable. Not the production-slot question either:
    /// `dial` is scoped and schedules nothing — see `spell::block::begins_work`.
    #[must_use]
    pub const fn is_operation(self) -> bool {
        matches!(
            self,
            Self::Grind
                | Self::Digest
                | Self::Mix
                | Self::Distil
                | Self::Kindle
                | Self::Probe
                | Self::Dial
                | Self::Muster
                | Self::Haul
                | Self::Summon
                | Self::Limn
                // The forge takes the slot, so a charm competes with making
                // things (§10).
                | Self::Anneal
        )
    }

    /// The fixture verb that must stand in the room for this one to mean anything.
    ///
    /// The scope question [`Scene::offers`](super::Scene::offers) asks. A verb a
    /// `Branch` declares is self-anchored, and
    /// `every_self_anchored_verb_is_declared_by_a_fixture` checks the two agree.
    #[must_use]
    pub const fn anchor(self) -> Option<Self> {
        match self {
            // Declared by a fixture, so the fixture's room is the scope.
            Self::Grind
            | Self::Digest
            | Self::Mix
            | Self::Distil
            | Self::Kindle
            | Self::Probe
            | Self::Dial
            | Self::Muster
            | Self::Haul
            | Self::Summon
            | Self::Defend
            | Self::Research => Some(self),
            // The rampart's: a band is read, not stood in. `petition` too,
            // offered with the road empty, the only time it is any use.
            Self::Deploy | Self::Quaff | Self::Hold | Self::Pledge | Self::Petition => {
                Some(Self::Defend)
            }
            // The lattice's: a column is read, not stood at.
            Self::Imbue => Some(self),
            Self::Snap | Self::Anneal => Some(Self::Imbue),
            // The stacks', whose one `Operation` went to `research`.
            Self::Follow | Self::Wander => Some(Self::Research),
            // The circle's: a glyph is named, not stood at.
            Self::Limn => Some(Self::Summon),
            // `wield`, `empty`, `stop` and `move` name their target and work
            // wherever one stands.
            _ => None,
        }
    }

    /// Whether finishing this turns an instrument's contents into something.
    ///
    /// The property, not one verb named: asking `verb == Wield` once left the
    /// sage whole in the mortar after a `grind`.
    #[must_use]
    pub const fn transmutes(self) -> bool {
        matches!(
            self,
            // Not `Kindle`: lighting the athanor produces nothing (§10.1).
            Self::Wield | Self::Grind | Self::Digest | Self::Mix | Self::Distil
        )
    }

    /// The `-ing` form, for a refusal that names what holds a slot.
    ///
    /// Spelled out, never `{verb}ing`, which produced "divineing" and would
    /// make `settle` "settleing".
    #[must_use]
    pub const fn participle(self) -> &'static str {
        match self {
            Self::Attend => "attending",
            Self::Survey => "surveying",
            Self::Peruse => "perusing",
            Self::Sift => "sifting",
            Self::Status => "checking",
            Self::Recall => "reading",
            Self::Verify => "verifying",
            Self::Undo => "undoing",
            Self::Unfurl => "unfurling",
            Self::Quit => "leaving",
            Self::Menu => "opening the menu",
            Self::Meditate => "meditating",
            Self::Move => "moving",
            Self::Wield => "wielding",
            Self::Grind => "grinding",
            Self::Digest => "digesting",
            Self::Mix => "mixing",
            Self::Distil => "distilling",
            Self::Kindle => "kindling",
            Self::Empty => "emptying",
            Self::Stop => "stopping",
            Self::Purge => "purging",
            Self::Research => "researching",
            Self::Scribe => "scribing",
            Self::Bind => "binding",
            Self::Invoke => "invoking",
            Self::Weave => "weaving",
            Self::Follow => "following",
            Self::Wander => "wandering",
            Self::Probe => "probing",
            Self::Dial => "dialling",
            Self::Muster => "mustering",
            Self::Haul => "hauling",
            Self::Defend => "defending",
            Self::Petition => "petitioning",
            Self::Deploy => "deploying",
            Self::Quaff => "quaffing",
            Self::Hold => "holding",
            Self::Pledge => "pledging",
            Self::Imbue => "imbuing",
            Self::Snap => "snapping",
            Self::Anneal => "annealing",
            Self::Summon => "summoning",
            Self::Limn => "limning",
            Self::Queue => "queueing",
        }
    }

    /// What this command takes, in order.
    #[must_use]
    pub const fn signature(self) -> &'static [Slot] {
        match self {
            Self::Attend => PLACE,
            Self::Survey => PLACE_OPTIONAL,
            Self::Peruse => READABLE,
            Self::Sift => PATTERN_AND_FILE,
            // Surfaces take no argument, and a room's one prism, lattice,
            // circle, pylon or rampart would be naming the only thing there is.
            // `petition` is one foe a word, so a partial refusal never arises.
            Self::Status
            | Self::Undo
            | Self::Unfurl
            | Self::Quit
            | Self::Menu
            | Self::Weave
            | Self::Research
            | Self::Wander
            | Self::Probe
            | Self::Anneal
            | Self::Defend
            | Self::Petition
            | Self::Hold
            | Self::Muster => NOTHING,
            // Bare refills the garrison; named binds that demon (§19, *Supply*).
            // An essence, since a topic slot fuzzed *summon a beast* to `belt`.
            Self::Summon => DEMON_OPTIONAL,
            // What the arsenal holds, refusing `deploy asdfgh` here rather than
            // a round later.
            Self::Deploy | Self::Quaff => ANYTHING,
            Self::Limn => GLYPH_AND_HUMOUR,
            // Anything nameable, for what it refuses: free text would fail a
            // tick later and a room away. It also canonicalises an abbreviation.
            Self::Queue => ANYTHING,
            Self::Pledge => DIE_AND_AREA,
            Self::Imbue => TOOL_AND_CHARM,
            Self::Snap => COLUMN,
            Self::Dial => SOCKET_AND_SIGIL,
            Self::Haul => STATION_AND_STATION,
            Self::Follow => WAY,
            Self::Recall => TOPIC_OPTIONAL,
            // Bare is the audit — §8.1's `verify --all`, without flag syntax.
            // `purge` stays required: bare would scour everything.
            Self::Verify => ANYTHING_OPTIONAL,
            Self::Purge => ANYTHING,
            Self::Meditate => COUNT,
            Self::Move => MOVE,
            Self::Wield => WORKABLE,
            Self::Empty => PLACE,
            Self::Stop => STOPPABLE,
            // §10.1's verbs name the material; the verb means the tool. Bare
            // `kindle` relights what is banked, the end of every script loop.
            Self::Grind | Self::Digest | Self::Distil | Self::Kindle => ONE_REAGENT,
            Self::Mix => TWO_REAGENTS,

            Self::Scribe => SPELL_NAME,
            Self::Bind | Self::Invoke => SCRIPT,
        }
    }

    /// Whether the command destroys something and therefore confirms when the
    /// target is not routine (§6, *Undo*).
    ///
    /// `stop` qualifies: the inputs are refunded but the work done is gone.
    /// Nothing reads this yet — §6's confirming is unbuilt.
    #[must_use]
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::Purge | Self::Stop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slot_kind_accepts_itself_and_nothing_else_by_default() {
        // The test that says the `as u8` cast is a comparison.
        for kind in [
            NounKind::Place,
            NounKind::File,
            NounKind::Reagent,
            NounKind::Script,
            NounKind::Scroll,
        ] {
            assert!(kind.accepts(kind), "{kind:?} did not accept itself");
            for other in [NounKind::Place, NounKind::File, NounKind::Reagent] {
                if (kind as u8) != (other as u8) {
                    assert!(!kind.accepts(other), "{kind:?} accepted a {other:?}");
                }
            }
        }
    }

    #[test]
    fn readable_is_text_and_any_is_everything() {
        assert!(NounKind::Readable.accepts(NounKind::File));
        assert!(NounKind::Readable.accepts(NounKind::Script));
        for opaque in [
            NounKind::Reagent,
            NounKind::Place,
            NounKind::Essence,
            NounKind::Vessel,
            NounKind::Scroll,
        ] {
            assert!(
                !NounKind::Readable.accepts(opaque),
                "a {opaque:?} is not something to read",
            );
            assert!(NounKind::Any.accepts(opaque), "`Any` means any");
        }
    }

    #[test]
    fn nothing_in_the_world_is_ever_a_slot_only_kind() {
        // A noun spawned as one would be unreachable by the slot named after it.
        assert!(!NounKind::Readable.accepts(NounKind::Readable));
        assert!(!NounKind::Readable.accepts(NounKind::Any));
    }

    #[test]
    fn the_vocabulary_is_the_tower_wide_verbs_plus_the_laboratory_s_own() {
        // The words a player meets in every room, §6.1's ceiling — by `anchor`,
        // the scope question.
        let tower_wide = Verb::ALL.iter().filter(|verb| verb.anchor().is_none());
        // A number to defend: a new word here needs `unfurl`'s argument (§19).
        assert_eq!(tower_wide.count(), 22);

        // The number a new domain is meant to move.
        let scoped = Verb::ALL.iter().filter(|verb| verb.is_operation());
        assert_eq!(scoped.count(), 12);

        assert!(
            !Verb::ALL.iter().any(|verb| verb.canonical() == "decoct"),
            "decoct is retired (§19)"
        );
    }

    #[test]
    fn canonical_names_obey_the_length_rule() {
        for verb in Verb::ALL {
            let name = verb.canonical();
            assert!(
                name.len() <= Verb::MAX_CANONICAL_LEN,
                "{name} is {} characters",
                name.len()
            );
            assert!(!name.contains(' '), "{name} is not one word");
        }
    }

    #[test]
    fn canonical_names_are_unique() {
        let mut names: Vec<_> = Verb::ALL.iter().map(|verb| verb.canonical()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "two verbs share a canonical name");
    }

    #[test]
    fn every_verb_appears_in_all() {
        for verb in Verb::ALL {
            assert!(Verb::ALL.contains(&verb));
        }
        assert_eq!(
            Verb::ALL
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            Verb::ALL.len()
        );
    }

    #[test]
    fn commands_taking_no_arguments_have_empty_signatures() {
        assert!(Verb::Status.signature().is_empty());
        assert!(Verb::Undo.signature().is_empty());
    }

    #[test]
    fn sift_takes_a_pattern_then_a_source() {
        let signature = Verb::Sift.signature();
        assert_eq!(signature.len(), 2);
        assert_eq!(signature[0].kind, NounKind::Pattern);
        assert_eq!(signature[1].kind, NounKind::File);
        assert!(signature.iter().all(|slot| slot.required));
    }

    #[test]
    fn survey_works_with_no_argument() {
        let signature = Verb::Survey.signature();
        assert_eq!(signature.len(), 1);
        assert!(!signature[0].required);
    }
}
