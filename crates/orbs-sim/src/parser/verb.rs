//! The canonical command set.
//!
//! §6.1: one short word each, ≤7 characters, arcane register. Experts type the
//! canonical form all day, so it has to be short — and arcane, because whichever
//! register is canonical is the one players absorb.

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
    /// The *quality* a recipe produces, not a thing on a shelf. What you carry
    /// is a [`Reagent`](Self::Reagent).
    Essence,
    /// Crafting stock: an ingredient, a part-made material, a byproduct, or fuel.
    ///
    /// One kind rather than four. A husk is only litter until a recipe wants it,
    /// so sorting ingredients from waste would encode a judgement the recipes
    /// keep changing.
    Reagent,
    /// A vessel holding a finished brew. `siphon retort`.
    Vessel,
    /// A scroll, which is finished work you spend. `wield gleaning-scroll`.
    ///
    /// Its own kind, not [`Essence`](Self::Essence) — otherwise `wield clarity`
    /// resolves and finds nothing to do. Not [`Reagent`](Self::Reagent) either:
    /// a recipe consumes those, a player spends this.
    Scroll,
    /// A script. `invoke night_watch`.
    Script,
    /// A count of ticks. `meditate 30`.
    Count,
    /// A name the player is coining, not one the world holds. `scribe morning`.
    ///
    /// Free text like [`Pattern`](Self::Pattern), because the spell does not
    /// exist yet and cannot resolve against the scene. `bind` and `invoke` keep
    /// `Script` — they name a spell that is there. Not [`Any`](Self::Any), or
    /// `scribe sage` would quietly name a spell after a reagent.
    Name,
    /// What a room reports about its own state — `passage`, `wall`, `marks`.
    ///
    /// The vocabulary a solver's `if` names; `tower::maze::readings` is the one
    /// list. No slot asks for a `Sense`, so one can never fill a `Reagent` by
    /// accident, while [`Any`](Self::Any) still finds it — which is what lets
    /// `spell::compile` resolve a condition before the way has been walked.
    Sense,
    /// Anything with text in it — `peruse orb.log`, `peruse night_watch.spell`.
    ///
    /// A *slot* kind, never a noun's own: nothing in the tower *is* a readable.
    /// Not [`Any`](Self::Any), which would let `peruse sage` report a zero-line
    /// read of a reagent.
    Readable,
    /// Anything you can pick up — stock, a potion, a scroll.
    ///
    /// A slot kind, like [`Readable`](Self::Readable). `move`'s first slot was
    /// `Reagent` and a finished potion is an `Essence`, so `move clarity to
    /// dispensary` silently failed to fill a required slot until the arsenal
    /// made it matter. Not [`Any`](Self::Any), or `move laboratory to arsenal`
    /// resolves.
    Portable,
    /// Anything you can set going — an instrument, or a scroll.
    ///
    /// A slot kind. `empty` keeps [`Place`](Self::Place) rather than sharing
    /// this, or `empty gleaning-scroll` parses and the executor cannot answer.
    Workable,
    /// Anything that can be running — an instrument, or a spell.
    ///
    /// A slot kind. `stop` took a [`Place`](Self::Place), which left an invoked
    /// spell unstoppable: `repeat` with no count ran for ever and `stop <spell>`
    /// did not resolve.
    Stoppable,
    /// A verb's own name, so `recall grind` can be asked about.
    ///
    /// Reachable from [`Subject`](Self::Subject) and nothing else — deliberately
    /// *not* from [`Any`](Self::Any), which would leak 27 canonicals into Tab
    /// (`purge grind` offered), into `spell::compile` (`if the dispensary has
    /// grind` compiling to a branch that never takes), and into the numbered
    /// prompt's ordering.
    Command,
    /// Anything nameable — `verify` and `purge` accept any surface.
    Any,
    /// What the manual can answer on: a topic, or a command.
    ///
    /// A slot kind. Lets `recall` reach both `recall brewing` and `recall grind`
    /// without widening [`Any`](Self::Any).
    Subject,
}

/// Which part of the manual a verb belongs under.
///
/// A table, not a derivation: `is_operation` is about scope and `transmutes`
/// about the pipeline, where a lost player wants sorting by what they are
/// trying to do.
///
/// Order here is the order the overview prints: find your way about, do the
/// work, teach the orb, ask the orb, and last the two that destroy something.
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
    /// Not every variant: `Pattern`, `Count` and `Name` are free text and never
    /// touch the world; `Readable`, `Portable`, `Workable`, `Stoppable`,
    /// `Subject` and `Any` are sets a slot accepts, named by
    /// [`accepts`](Self::accepts) and never carried by a noun. One list, so
    /// `content::Phrasings` is not a second copy.
    ///
    /// `Command` is here because `tower::scene_at` registers every verb word as
    /// one — `recall` takes a `Subject`, which is a topic *or* a command.
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
    /// §6 forbids a bare error: when a slot is empty the orb has to say what
    /// would fill it, and it cannot say `Portable`. Kept to one lower-case word
    /// so it drops into a sentence a content file composes later (§12) without
    /// the file having to case-fold or re-word it.
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
            // What the orb asks for, not what the type is called. "Which
            // readable?" is not a sentence; a spell is a file you read.
            Self::Readable => "file",
            // What the orb asks for, not what the type is called: you stop a
            // thing that is working, and both an instrument and a spell are.
            Self::Stoppable => "place",
            // Same rule again. An instrument is the overwhelmingly commoner
            // answer, and "which workable?" is not a sentence anyone says.
            Self::Workable => "place",
            // And again: the laboratory is mostly full of reagents, so that is
            // what "move which reagent?" should ask for even though the slot
            // will also take a potion or a scroll.
            Self::Portable => "reagent",
            Self::Any => "name",
            Self::Command => "command",
            // What the orb asks for, not what the type is called: "which
            // subject?" is the question, and a command is one of the answers.
            Self::Subject => "topic",
        }
    }

    /// Whether a noun of kind `noun` may fill a slot wanting `self`.
    ///
    /// One definition, because three surfaces ask it — `Scene::best_match`,
    /// `resolve::fillers` and `complete::nouns`. They had a copy each, which is
    /// three chances for the matcher, the numbered prompt and Tab to disagree
    /// about what a command takes.
    #[must_use]
    pub const fn accepts(self, noun: Self) -> bool {
        match self {
            // Everything except a command. `Any` is `verify` and `purge`,
            // and a verb's own name is not a surface either can act on — see
            // [`Command`](Self::Command) for the three places that leak through
            // if it is.
            Self::Any => !matches!(noun, Self::Command),
            Self::Readable => matches!(noun, Self::File | Self::Script),
            Self::Stoppable => matches!(noun, Self::Place | Self::Script),
            Self::Workable => matches!(noun, Self::Place | Self::Scroll),
            Self::Portable => matches!(noun, Self::Reagent | Self::Essence | Self::Scroll),
            Self::Subject => matches!(noun, Self::Topic | Self::Command),
            // `as u8` because `PartialEq::eq` is not const and a fieldless enum
            // casts cleanly. Writing the other ten arms out would be a table
            // that says only "equal" eleven times.
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

// Signatures are named constants rather than inline slice literals: a `&[...]`
// built from `const fn` calls is a temporary and does not get promoted to
// `'static`, so it cannot be returned from `signature()`.
const NOTHING: &[Slot] = &[];
const PLACE: &[Slot] = &[Slot::required(NounKind::Place)];
const PLACE_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Place)];
// `peruse` takes anything with text in it, so a `.spell` reads back like a log.
// `sift` deliberately does **not** yet: its second slot is what it searches, and
// searching a spell is a different feature from reading one.
const READABLE: &[Slot] = &[Slot::required(NounKind::Readable)];
/// `stop` reaches an instrument **or** a running spell.
const STOPPABLE: &[Slot] = &[Slot::required(NounKind::Stoppable)];
/// `wield` reaches an instrument **or** a scroll you spend.
const WORKABLE: &[Slot] = &[Slot::required(NounKind::Workable)];
const PATTERN_AND_FILE: &[Slot] = &[
    Slot::required(NounKind::Pattern),
    Slot::required(NounKind::File),
];
/// `recall [topic]` — optional, and it is `survey`'s shape rather than a
/// weakening.
///
/// A required slot with fillers can never yield an argument-less intent, so
/// bare `recall` — and `help`, `man`, `?` — opened a numbered prompt offering
/// four arbitrary manual subjects. §6 forbids a bare error, and that was it
/// failing at the one command whose job is answering the question.
///
/// §19 declined the same for bare `follow`, where bare and argumented are
/// different acts. Here they are the same act at two scopes, like `survey`.
const TOPIC_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Subject)];
const ANYTHING: &[Slot] = &[Slot::required(NounKind::Any)];
/// `verify`'s, and the second optional slot in the game — see its `signature`
/// arm for why bare is the audit rather than a refusal.
const ANYTHING_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Any)];
const COUNT: &[Slot] = &[Slot::required(NounKind::Count)];
// `VESSEL` was `siphon`'s signature, when a finished brew sat in one. §10.1 puts
// the product in the instrument that made it, so no verb takes a vessel today.
// `NounKind::Vessel` itself stays — `retort` is one, and the kind is what a
// potion container will be when potions need somewhere to sit.

/// `move <reagent> [from <source>] to <destination>` — §10.1's pipeline.
///
/// The middle slot is **optional** because the source is usually obvious: you are
/// standing in the laboratory and there is only one lot of sage. It earns its
/// place when there is not — two instruments each holding `husks` — where naming
/// the source is what tells them apart. `from` and `to` are filler
/// ([`normalise`](super::normalise)), so both phrasings reduce to bare words and
/// the same three slots serve them:
///
/// ```text
/// move sage to mortar_and_pestle              -> reagent, _, destination
/// move husks from alembic to dispensary       -> reagent, source, destination
/// move clarity to arsenal                     -> essence, _, destination
/// ```
///
/// The first slot is [`Portable`](NounKind::Portable) rather than `Reagent`
/// because a finished potion is an `Essence` and could not be picked up at
/// all — see that kind for how long that had been true and why nothing noticed.
const MOVE: &[Slot] = &[
    Slot::required(NounKind::Portable),
    Slot::optional(NounKind::Place),
    Slot::required(NounKind::Place),
];
/// What §10.1's per-instrument verbs take.
///
/// Optional, so bare `grind` still starts a mortar that is already charged.
/// These verbs exist to save keystrokes; a compulsory reagent would make bare
/// `grind` lose to bare `wield mortar_and_pestle`.
const ONE_REAGENT: &[Slot] = &[Slot::optional(NounKind::Reagent)];

/// Two, for the one instrument that combines.
///
/// `mix sage-tincture and ground-salt` fills both. The `and` is dropped by
/// [`normalise`](super::normalise) exactly as `to` and `from` already are, so the
/// slots fill **positionally** — the same mechanism `move sage to
/// mortar_and_pestle` has always used, rather than a second one bolted on.
const TWO_REAGENTS: &[Slot] = &[
    Slot::optional(NounKind::Reagent),
    Slot::optional(NounKind::Reagent),
];

/// One of the archive's four readings. A `Place`, because that is the only kind
/// the place half of a spell's question resolves against.
const WAY: &[Slot] = &[Slot::required(NounKind::Place)];

/// `seat <socket> <sigil>` — which dial, and what to turn it to.
///
/// Both `Place`, no new `NounKind`: sockets and sigils are `Role::Reading`
/// fixtures like the archive's bearings, so they resolve against the kind `WAY`
/// uses. A new kind leaks into Tab, `compile::fix` and the bare-argument prompt.
/// The cost is that `seat laboratory nitre` parses and the handler refuses it.
///
/// The sigil is optional, and bare means *try something else here* — a
/// variable-free script cannot name a sigil it has not tried, which forced
/// twenty-four ladder rungs per socket. `Ward::advance` has the rest.
const SOCKET_AND_SIGIL: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::optional(NounKind::Place),
];

/// `haul <from> <to>` — which station the ward leaves, and which it arrives at.
///
/// Both required, both `Place`, on `SOCKET_AND_SIGIL`'s reasoning.
///
/// Neither is optional, unlike `dial`: a spell that knows which two stations it
/// means knows both, and between any two there is exactly one legal move. The
/// *direction* is what it must work out, which is why `potency` is published.
const STATION_AND_STATION: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::required(NounKind::Place),
];
/// `limn <glyph> [<humour>]` — which glyph, and optionally what to limn it with.
///
/// `SOCKET_AND_SIGIL`'s shape and its reason: glyphs and humours are both
/// `Role::Reading` places, and the humour is optional because a variable-free
/// spell cannot name one it has not tried — bare, the circle steps the glyph.
const GLYPH_AND_HUMOUR: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::optional(NounKind::Place),
];
/// `pledge` takes a die and an area — the lens's socket-and-sigil shape.
///
/// `to` between them is §6 filler, so `pledge d20 to buckler` and `pledge d20
/// buckler` are the same command and the longer one is what a person types.
const DIE_AND_AREA: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::required(NounKind::Place),
];
/// `imbue` takes a tool and a charm — `pledge`'s shape, one room over.
///
/// `with` between them is §6 filler, so `imbue mortar_and_pestle with hurried`
/// and `imbue mortar_and_pestle hurried` are the same command and the longer one
/// is what a person types.
///
/// Both `Place`. A charm is a `Role::Reading` fixture in the forge; the tool is
/// a place anywhere in the tower, which is this domain's one widening of §7.
const TOOL_AND_CHARM: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::required(NounKind::Place),
];
/// `snap` takes one column of the lattice.
const COLUMN: &[Slot] = &[Slot::required(NounKind::Place)];
const SCRIPT: &[Slot] = &[Slot::required(NounKind::Script)];
// `scribe` coins a name rather than naming something that exists — see
// `NounKind::Name`. `bind` and `invoke` keep `SCRIPT`, because a spell they name
// has to be there already.
const SPELL_NAME: &[Slot] = &[Slot::required(NounKind::Name)];

/// A canonical command.
///
/// Deliberately not `#[non_exhaustive]`, for the reason given on
/// [`crate::RngStream`]: every consumer is in-workspace, and adding a verb
/// should force every match — echo, help, the balance harness — to account for
/// it rather than silently falling through to a default.
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
    /// Was `grimoire` and released (§19): a grimoire is a book of *spells*,
    /// which is what `/grimoire` holds, so one word meant both the reference you
    /// read and the book you write in.
    Recall,
    /// Detect tampering.
    Verify,
    /// Revert the last command.
    Undo,
    /// Read back through what the orb has said.
    ///
    /// A word for a key that already worked. `PageUp` has always scrolled the
    /// transcript, but the border only says `PgDn newest` once you are already
    /// scrolled back — so in a mouseless game the key announced itself only to
    /// players who had found it. Using the word once teaches the key.
    ///
    /// Not `recollect`: `rec` is `recall`'s.
    Unfurl,
    /// Put the orb down and leave.
    ///
    /// `unfurl`'s argument again: `F10` has always left and nothing says so.
    /// Also the word the spell editor and the weave screen use for *close
    /// this*, so it means one thing at three depths. No other verb begins `q`.
    Quit,
    /// Open the orb's menu.
    ///
    /// Was `quit` for one iteration, superseded (§19): leaving the game and
    /// stepping out to a menu are two things, and one word for both made
    /// stopping a two-step operation. `quit` leaves and asks first.
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
    /// Cancel a working instrument, refunding what it holds.
    Stop,
    // `Siphon` was here — *"collect a finished potion"* — and is **retired**
    // (§19). It took the product out of a tool and put it on the laboratory
    // floor, which mattered when §10.1's loop was `move`/`wield`/`siphon` and a
    // stage's output had to be carried by hand.
    //
    // The per-instrument verbs ended that: `digest ground-sage` reaches into an
    // idle instrument and takes what it needs, so the pipeline advances without
    // anything being drawn off first. What was left was a convenience that put
    // things somewhere `empty` puts them better — and the floor and the store
    // are now one place, because `siphon` was the only thing that could ever put
    // a reagent on the floor.
    /// Destroy waste or spoilage.
    ///
    /// The one that is genuinely not `empty`: this **destroys** what a tool
    /// holds rather than shelving it.
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
    /// Not `step`: 750 against `stop`, and a typo that stopped a run instead of
    /// advancing it would cost the whole maze. `tread` is 800 against `read`.
    Follow,
    /// Look at what the work has bought (§11.5).
    ///
    /// The twentieth tower-wide word. §6.1 wants the set smaller, and this earns
    /// a seat the way `unfurl` did: without it `status` prints two numbers with
    /// no sense of what they are for. A track nobody can look at is a track
    /// nobody is on.
    ///
    /// Not `ascend`: 667 against `attend`.
    Weave,
    /// Give the arrow keys the archive's stacks (§10, §19).
    ///
    /// `unfurl`'s argument again: the surface has no other way in. It buys the
    /// *keys* for a maze a player would otherwise walk with a hundred `follow`
    /// lines — the map is on screen either way.
    ///
    /// Every obvious word collided: `thread` 667 against `read`, `stride` 667
    /// against `scribe`, `delve` 600 against `weave`, `trace` 600 against
    /// `twice`, `pace` 750 against `page`; `enter` and `walk` are taken.
    Wander,
    /// `probe` — press the aperture against a far orb's ward, opening a reading
    /// if none is open.
    ///
    /// Two acts in one word, §19's per-instrument idiom: a `scry` and a press,
    /// the way `grind sage` is a `move` and a `wield`. Costs nothing in clarity
    /// because the aperture opens on a fixed figure.
    ///
    /// `scry` was the opener and lost to the prefix rule — `scr` reaches
    /// `scribe`. The domain is still scrying; the word you type is `probe`.
    Probe,
    /// `dial <socket> <sigil>` — turn one dial of the aperture.
    ///
    /// Free, because turning a dial is not work; only [`Probe`](Self::Probe)
    /// takes a tick. **Both channels use this same word**: a player dials what
    /// they have deduced, a spell dials what its ladder reached, and the
    /// difference is entirely in which readings they consult.
    ///
    /// `seat` was the first name and lost to the same rule that took `scry`:
    /// `sea` reaches `sift`'s plain synonym `search`. `dial` is free, and it is
    /// the better word anyway — a ward is a lock, and this is what a lock has.
    Dial,
    /// `muster` — draw a fresh course of wards up out of the wellspring
    /// (§10, `sanctum/`).
    ///
    /// Free and instant like [`Probe`](Self::Probe): drawing a course is not
    /// work, the hauling is. §19 has the pricing.
    ///
    /// How tall a course is, is the whole of what erosion does — a neglected
    /// tower musters more wards, so the same word is a minute's work or a
    /// quarter of an hour. See `tower::erosion`.
    ///
    /// Near misses: `fortify` 914 against `for`, `restore` 935 against `rest`,
    /// `mend` 935 against `mending`.
    Muster,
    /// `haul <from> <to>` — carry the topmost ward from one station to another.
    ///
    /// Directional, and it refuses. A symmetric word would have been
    /// unambiguous — exactly one move between any two stations is legal — and
    /// was declined because it leaves a spell nothing to *read*. That refusal is
    /// what makes `potency` worth publishing (§19).
    ///
    /// A wrong haul is an ordinary refusal, not a fault: the spell is told and
    /// carries on. See `say_failure`.
    ///
    /// `shift` is 800 against `sift`, `heave` 800 against `weave`.
    Haul,
    /// `summon` — draw a beast up at the circle, or call the waiting one in (§10,
    /// `menagerie/`).
    ///
    /// Two acts in one word, like [`Probe`](Self::Probe): opening a reading and
    /// pressing it. Free and instant — a balked call costs the call and nothing
    /// else, so trying is never a resource decision.
    ///
    /// Takes nothing: there is one circle, and naming it would be naming the
    /// only thing there is. `call` is 750 against `wall`.
    Summon,
    /// `defend` — let the enemy arrive, and stand to meet it.
    ///
    /// Takes nothing, like `muster` and `summon`: there is one rampart.
    ///
    /// Clean at 500. `siege` itself scored 600 against `sing` when the menagerie
    /// was a chant, so the domain keeps that name and the verb does not — `sing`
    /// is gone (§19, `0.15.0`) and the split still stands.
    Defend,
    /// `deploy <troop>` — send what the menagerie summoned into the line.
    ///
    /// The arsenal's troops, spent. Swept clean at 500 against `play`.
    Deploy,
    /// `quaff <potion>` — spend a potion on the coming round.
    ///
    /// A potion needed its own word and a scroll did not — spending a scroll is
    /// setting a thing going, which is `wield` (§19). Nothing scores against it.
    Quaff,
    /// `pledge <die> to <area>` — put one of your dice behind part of the wall.
    ///
    /// The domain's central decision: three dice against four areas, so the
    /// board can never be covered and every round leaves something dark. §5.1's
    /// telegraph names the urgent area a round ahead, which is what the turn is
    /// spent on.
    ///
    /// A die is rolled when the round resolves — a `d20` is a gamble, a `d6` a
    /// floor. The board prints the range before you commit (§5.1).
    Pledge,
    /// `imbue` — open a lattice to bind a charm onto a tool.
    ///
    /// §10's Enchanting, and the verb the whole domain hangs on. It names the
    /// tool and the charm; what it opens is the puzzle that binds them.
    ///
    /// It names a tool in another room — this domain's one widening of §7's
    /// *"you can only name what is where you are"*. Cheap, because an
    /// instrument is a `NounKind::Place` and `tower::scene` already registers
    /// every place from everywhere.
    ///
    /// The first sweep missed this domain's own words: `imbue`/`imbued` came
    /// back at 975 by the prefix rule, which is why the reading is `graced`.
    Imbue,
    /// `snap` — flip one column's glyph on the lattice's working row.
    ///
    /// Three columns, eight openings, one of them right. `temper` was the first
    /// choice and scores 625 against `tampered`, a `verify` verdict.
    Snap,
    /// `anneal` — let the lattice cascade, and bind the charm if it lights.
    ///
    /// The domain's only operation, so it holds the tower's one production slot
    /// while it runs — which is what makes maintaining a charm compete with
    /// making things (§10).
    ///
    /// Was `settle`: 834 against `mettle`, a live siege reading, and `set` is
    /// already a `dial` synonym that prefixes it. Chosen before it was swept;
    /// three pinned tables went red at once.
    Anneal,
    /// `petition` — spend standing so that fewer come up the road.
    ///
    /// The first thing renown buys. Fame lengthens the tail of what arrives;
    /// this is how a wizard shortens it again, by letting some of it go.
    ///
    /// It is eight characters, which is exactly [`Verb::MAX_CANONICAL_LEN`] and
    /// the third word in the game to sit on the limit after `meditate` and
    /// `research`. Swept before it was taken: `pet` names nothing else,
    /// `per` is `peruse` and `ple` is `pledge`.
    Petition,
    /// `hold` — end your turn and let one round resolve.
    ///
    /// The only thing in the domain that advances the world, which is what makes
    /// the siege turn-based rather than merely slow: §5.0's *"no per-command
    /// tick cost"* holds because everything else on your turn is free.
    ///
    /// Swept clean at 500 (`halt`, `help`, `odd`).
    Hold,
    /// `limn <glyph> [<humour>]` — limn one of the circle's glyphs; bare, step it
    /// round the six (§10, `menagerie/`).
    ///
    /// The lens's `dial`, one room over, and optional for `dial`'s reason: a
    /// spell cannot name the humour it has not tried, so `limn keystone` asks
    /// the circle for the next one. Three `repeat 6` loops around that search
    /// every circle there is, which is what makes the room scriptable.
    ///
    /// Free and instant: limning is not the work, holding the beast is.
    ///
    /// Replaced `sing` and `chorus` when the menagerie stopped being a rhythm
    /// game (§19). `etch` is 750 against `each`, `carve` 600 against `carry`,
    /// `inscribe` 750 against `scribe`, `engrave` 715 against `engage`.
    Limn,
    /// `queue <name>` — put a name in the satchel (§8, `tower::satchel`).
    ///
    /// The push half of the channel between two spells. The pull half is `pull`,
    /// a [`SpellWord`](super::SpellWord) rather than a verb, because pulling
    /// *binds a name* and only `let` does that. This half changes a node, which
    /// is what verbs do — and `queue heed` at the prompt is the See-it line for
    /// the whole mechanic.
    ///
    /// Unanchored, like `move` and `wield`: every domain has a satchel, so there
    /// is no fixture to scope it to.
    ///
    /// `que` offers both this and `quench`, a live `stop` synonym, and that is
    /// accepted: the full word is exact, and nobody types `quench` often. The
    /// alternatives are worse — `stow` shares `sto` with `stop` and scores 750
    /// against it, `stash` shares `sta` with `status`, and `draw` already means
    /// producing a random thing (`Beast::draw`, `muster`, `summon`).
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
        // Appended, deliberately: `the_tolerated_collision_set_is_pinned` walks
        // pairs in this order, so inserting mid-table reorders the pinned set
        // without changing a score.
        Self::Weave,
        Self::Follow,
        Self::Wander,
        Self::Probe,
        Self::Dial,
        Self::Muster,
        Self::Haul,
        Self::Summon,
        // In `sing`'s place, which it replaced along with `chorus` (§19) — the
        // chant's two words held this slot, and the slot decides pair order.
        Self::Limn,
        Self::Queue,
        // Appended, for the reason the block above gives.
        Self::Defend,
        Self::Deploy,
        Self::Quaff,
        Self::Hold,
        Self::Pledge,
        // The forge's three, appended for the same reason.
        Self::Imbue,
        Self::Snap,
        Self::Anneal,
        // The bailey's sixth. It belongs beside `defend` by anchor and subject,
        // and goes at the end regardless — same pair-order reason.
        Self::Petition,
    ];

    /// The longest a canonical verb may be.
    ///
    /// §6.1 says "ideally ≤7 characters"; the Phase 0 naming pass settled the
    /// "ideally" at 8. `meditate` and `research` are the two that spend the
    /// eighth — both carry the game's voice, both are typed occasionally, and a
    /// prefix (`medit`, `res`) covers the cost. Names with no such defence
    /// (`decipher`, `inscribe`) were shortened instead.
    pub const MAX_CANONICAL_LEN: usize = 8;

    /// Which part of the manual this verb is listed under.
    ///
    /// No wildcard, like every table here: a verb added later is a compile
    /// error rather than one silently missing from the overview.
    #[must_use]
    pub const fn group(self) -> Group {
        match self {
            Self::Attend | Self::Survey | Self::Peruse | Self::Sift | Self::Verify => {
                Group::Getting
            }
            // Everything that moves or makes, every room's together: a player
            // looking for what to *do* wants one list, not one sorted by which
            // room a verb belongs to or whether it costs a tick.
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
            // and the satchel's push — §8's channel, not a room's puzzle, but
            // it changes the world, which is what `Group::Work` collects.
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
            // Kept in step with `is_destructive` by a test, not by trust.
            Self::Purge | Self::Stop => Group::Careful,
        }
    }

    /// What this verb wants after it, as a single word.
    ///
    /// §6 forbids a bare error, and a listing owes the same courtesy: a verb
    /// offered with no hint of what follows it is a word to guess at. Empty for
    /// the verbs that take nothing.
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
    /// A verb that is `true` here only resolves where its instrument stands —
    /// see [`Scene::offers`](super::Scene::offers). `wield` is deliberately not
    /// one: it names the tool explicitly, works anywhere there is a tool, and is
    /// what a script writes when the instrument is the variable.
    ///
    /// The lens's two are here, which stops a third debt: §19 records `follow`
    /// and `wander` as tower-wide words waiting on a mechanism that does not
    /// exist, because a fixture carries one `Operation` and the archive's one
    /// fixture had spent it. The lens has five, so `probe` and `dial` scope
    /// without one. The sanctum's follow the lens for the same reason — a sixth
    /// domain copying the archive would have doubled the tower-wide count that
    /// `the_tolerated_collision_set_is_pinned` defends.
    ///
    /// Not the same question as "does this take the production slot": `dial` is
    /// a scoped operation that schedules nothing. See `spell::block::begins_work`.
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
                // The forge takes the slot, the first domain since brewing to
                // do so: §10 wants maintaining a charm to compete with making
                // things, or idle buff-time is free.
                | Self::Anneal
        )
    }

    /// The fixture verb that must stand in the room for this one to mean anything.
    ///
    /// The mechanism §19 recorded as missing. [`Scene::offers`](super::Scene::offers)
    /// used to ask [`is_operation`](Self::is_operation), the *production slot*
    /// question, so `research`, `follow` and `wander` were offered in every room.
    /// The two are separate now: a verb can be scoped and take no slot (`dial`),
    /// or take the slot and be scoped (`grind`).
    ///
    /// Mostly derived — a verb some `Branch` in `tower::build` declares as its
    /// `operation` is self-anchored, and
    /// `every_self_anchored_verb_is_declared_by_a_fixture` fails the build if
    /// this arm and `BRANCHES` disagree. `follow` and `wander` are spelled out:
    /// they act on the reading *inside* the stacks, and the stacks' one
    /// `Operation` went to `research`.
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
            // The pylon declares `muster`; each of the three stations declares
            // `haul` — a station is a place the player names anyway.
            | Self::Muster
            | Self::Haul
            // The circle declares `summon`.
            | Self::Summon
            // The rampart declares `defend`.
            | Self::Defend
            | Self::Research => Some(self),
            // The bailey's other four anchor to the rampart: a band is something
            // you *read*, not somewhere you stand. `petition` too, though it
            // runs *before* a siege — the anchor names the fixture that offers
            // the word, not a live component, so it resolves with the road
            // empty, which is the only time it is any use.
            Self::Deploy | Self::Quaff | Self::Hold | Self::Pledge | Self::Petition => {
                Some(Self::Defend)
            }
            // The bailey's shape: `imbue` is declared by the lattice and the
            // other two anchor to it, because a column is read, not stood at.
            Self::Imbue => Some(self),
            Self::Snap | Self::Anneal => Some(Self::Imbue),
            // The maze's other two words, anchored to the stacks.
            Self::Follow | Self::Wander => Some(Self::Research),
            // `limn` anchors to the circle, not to a glyph: a glyph is named in
            // an argument, not stood at, and there is one circle.
            Self::Limn => Some(Self::Summon),
            // `wield`, `empty`, `stop` and `move` are deliberately unanchored —
            // they name their target and work wherever one stands, which is
            // what a script writes when the instrument is the variable.
            _ => None,
        }
    }

    /// Whether finishing this turns an instrument's contents into something.
    ///
    /// `divine` holds the production slot too but produces no material, so
    /// `finish` has to tell them apart. It asked `verb == Wield` until §10.1's
    /// per-instrument verbs arrived, at which point a completed `grind` released
    /// the slot and left the sage whole in the mortar. Naming the property, not
    /// the one verb, stops the next one doing the same.
    #[must_use]
    pub const fn transmutes(self) -> bool {
        matches!(
            self,
            // `Kindle` is absent: lighting the athanor takes no Focus slot
            // (§10.1), inserts no `Working`, and produces nothing. `start`
            // returns at the `HeatSource` branch before this is asked.
            Self::Wield | Self::Grind | Self::Digest | Self::Mix | Self::Distil
        )
    }

    /// The `-ing` form, for a refusal that names what holds a slot.
    ///
    /// Spelled out, not `{state}ing` in a template. `prose.toml` appended a
    /// literal `ing` to [`canonical`](Self::canonical) and produced "divineing";
    /// the authored-line lint checks templates, not interpolated results, so
    /// nothing caught it. That verb is `research` now, whose naive form happens
    /// to be right — which is why the table stays: the next `-e` verb brings the
    /// bug back. Beside `canonical` because it is a form of the word, not prose.
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
            // Spelled out, never `{verb}ing` — that rule is why `divine` does
            // not read as `divineing`, and `settle` would read as `settleing`.
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
            // `divine` takes nothing now: it opens the stacks, and there is one
            // lectern to open them at. `wander` joins for `weave`'s reason — it
            // opens a surface, and a surface takes no argument.
            Self::Status
            | Self::Undo
            | Self::Unfurl
            | Self::Quit
            | Self::Menu
            | Self::Weave
            | Self::Research
            | Self::Wander
            // One prism, one lattice, one circle, one pylon, one rampart — so
            // naming it would be naming the only thing there is. What varies is
            // set by another verb (`dial`, `snap`). `hold` ends your turn, so
            // there is nothing to name; `petition` is one foe a word, because
            // `petition 3` would have to say what a partial refusal does when
            // the third is already at the floor.
            | Self::Probe
            | Self::Anneal
            | Self::Summon
            | Self::Defend
            | Self::Petition
            | Self::Hold
            | Self::Muster => NOTHING,
            // What the arsenal holds — `wield`'s shape, not `limn`'s: a troop is
            // something you *have*, not a reading the board publishes. Refusing
            // free text catches `deploy asdfgh` here, not a round later.
            Self::Deploy | Self::Quaff => ANYTHING,
            // A glyph and, optionally, a humour — the socket-and-sigil shape,
            // and bare for `dial`'s reason: the circle steps a glyph a spell
            // cannot name the next humour for.
            Self::Limn => GLYPH_AND_HUMOUR,
            // Anything the room can name, chosen for what it *refuses*:
            // `NounKind::Name` takes free text, so `queue asdfgh` would fail a
            // tick later and a room away. Resolving here also canonicalises an
            // abbreviation, so the satchel holds a word the game knows.
            Self::Queue => ANYTHING,
            // A die and an area, both `Role::Reading` places — socket-and-sigil,
            // for its reason: a spell's condition resolves its place half
            // against `NounKind::Place`, so `if the buckler is empty` needs one.
            Self::Pledge => DIE_AND_AREA,
            Self::Imbue => TOOL_AND_CHARM,
            Self::Snap => COLUMN,
            // A socket and a sigil, both `Role::Reading` places.
            Self::Dial => SOCKET_AND_SIGIL,
            // Two stations, likewise.
            Self::Haul => STATION_AND_STATION,
            // A way, which is a place — see `Role::Reading`.
            Self::Follow => WAY,
            Self::Recall => TOPIC_OPTIONAL,
            // `verify` bare is the audit, so its slot is optional — `recall`'s
            // shape, for §19's reason: bare and argumented are the same act at
            // two scopes. §8.1 writes the wide form `verify --all`, but the
            // parser has no flag syntax and one word is no reason to invent one.
            //
            // `purge` keeps `ANYTHING`: bare, it would scour everything, which
            // §7 protects against rather than prices.
            Self::Verify => ANYTHING_OPTIONAL,
            Self::Purge => ANYTHING,
            Self::Meditate => COUNT,
            Self::Move => MOVE,
            // An instrument is a place (§10.1), and you name it, not its
            // contents. `wield` split from `empty` when scrolls arrived: setting
            // a thing going reaches a scroll too, and one shared signature would
            // have made `empty gleaning-scroll` parse.
            Self::Wield => WORKABLE,
            Self::Empty => PLACE,
            // A spell counts: `stop` reaching only instruments made an invoked
            // spell unstoppable. See `NounKind::Stoppable`.
            Self::Stop => STOPPABLE,
            // §10.1's per-instrument verbs name the *material*, not the tool —
            // the tool is what the verb means.
            // `kindle` takes fuel, and takes it optionally: `kindle` on its own
            // relights what is banked, which is the end of every script loop.
            Self::Grind | Self::Digest | Self::Distil | Self::Kindle => ONE_REAGENT,
            Self::Mix => TWO_REAGENTS,

            Self::Scribe => SPELL_NAME,
            Self::Bind | Self::Invoke => SCRIPT,
        }
    }

    /// Whether the command destroys something and therefore confirms when the
    /// target is not routine (§6, *Undo*).
    ///
    /// `stop` qualifies: §10.1 refunds the inputs but the work done is gone, and
    /// at capacity 1 that was the tower's only slot.
    ///
    /// Nothing reads this yet — §6's confirm-when-not-routine is unbuilt. The
    /// classification is cheap to keep true; the confirming is separate work.
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
        // The eleven ordinary kinds are exact. `as u8` stands in for an `==`
        // that is not const, so this is the test that says the cast is a
        // comparison and not an ordering accident.
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
        // The two slot-only kinds, and the line between them. `Readable` exists
        // because `Any` is too wide for `peruse`: a reagent is nameable and has
        // no text in it.
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
        // `Readable` and `Any` say what a slot takes; no node is ever spawned
        // as one. If that changes, `accepts` starts answering a question about
        // itself — `Readable.accepts(Readable)` is false, so a noun spawned as
        // one would be unreachable by the very slot named after it.
        assert!(!NounKind::Readable.accepts(NounKind::Readable));
        assert!(!NounKind::Readable.accepts(NounKind::Any));
    }

    #[test]
    fn the_vocabulary_is_the_tower_wide_verbs_plus_the_laboratory_s_own() {
        // The vocabulary a player meets in *every* room, which is what §6.1's
        // ceiling is about. `anchor`, not `is_operation`: the first asks which
        // fixture must stand here (the scope question `Scene::offers` asks), the
        // second asks whether the verb spends the production slot. This metric
        // asked the slot question for a while and counted scoped words like
        // `follow` against a ceiling they are nowhere near (§19).
        let tower_wide = Verb::ALL.iter().filter(|verb| verb.anchor().is_none());
        // 22 is a number to defend, not a budget to spend: a new word here needs
        // an argument of the shape §19 records for `unfurl`, `weave`, `quit` and
        // `menu` — each reaches something that was otherwise only a keystroke,
        // or addresses the orb rather than the tower.
        //
        // `follow` and `wander` are the standing debt: they are the archive's
        // words wearing a tower-wide coat, because a fixture carries one
        // `Operation` and the lectern spends its on `research`. Both retire the
        // day a second verb can be scoped to an instrument. A domain verb
        // arriving here is the argument for building that, not for a third debt.
        assert_eq!(tower_wide.count(), 22);

        // One per instrument with a word of its own. This is the number a new
        // domain is meant to move, and the one above is not: a domain that
        // scopes its verbs leaves every other room's vocabulary as it was.
        let scoped = Verb::ALL.iter().filter(|verb| verb.is_operation());
        // Twelve; the forge moved it by one, because of its three verbs only
        // `anneal` takes the production slot.
        assert_eq!(scoped.count(), 12);

        assert!(
            !Verb::ALL.iter().any(|verb| verb.canonical() == "decoct"),
            "decoct is retired (§19)"
        );
    }

    #[test]
    fn canonical_names_obey_the_length_rule() {
        // §6.1: canonical verbs are one short word. The canonical form is what
        // expert players type all day, so length is a real cost.
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
        // Guards against adding a variant and forgetting the table.
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
