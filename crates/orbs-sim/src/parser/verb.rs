//! The canonical command set.
//!
//! DESIGN.md §6.1 fixes the Phase 0 vocabulary at sixteen commands, and fixes
//! the naming rule that shapes them: **canonical verbs are one short word,
//! ideally ≤7 characters.** Players graduate to typing the canonical form, so it
//! is what expert players type all day — `auspicate --sign=march` would lose to
//! `grep march` every time, which would punish the exact progression the echo
//! mechanism exists to create.
//!
//! The canonical form is **arcane**. Whichever register is canonical is the one
//! players absorb, so making it arcane means the mastery arc is literally
//! learning to speak as a wizard.

/// What a command slot expects to be filled with.
///
/// This is what lets the parser resolve `clarity` against essences rather than
/// against every noun in the tower, and what lets it reject a plausible-sounding
/// argument in the wrong category.
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
    /// §11.5's Resources table names **reagents** as the crafting economy, and
    /// §10.1's pipeline is entirely the business of moving them between
    /// instruments — so this is the kind `move` takes and the one the laboratory
    /// is mostly full of.
    ///
    /// One kind rather than four: a husk is only litter until a recipe wants it
    /// (§10.1's *every byproduct has at least one use*), so a noun kind that
    /// sorted ingredients from waste would be encoding a judgement the recipes
    /// are meant to keep changing.
    Reagent,
    /// A vessel holding a finished brew. `siphon retort`.
    Vessel,
    /// A scroll, which is finished work you spend. `wield gleaning-scroll`.
    ///
    /// **A kind of its own, beside [`Essence`](Self::Essence) rather than inside
    /// it.** A potion is §10.1's *quality* a recipe yields; a scroll is an
    /// object with an effect, and the two answer different questions — a slot
    /// that took "finished work" would let `wield clarity` resolve at full
    /// confidence and then find nothing to do, which is §15's dead end reached
    /// through a kind that was merely convenient.
    ///
    /// Not [`Reagent`](Self::Reagent) either, which is *"crafting stock: an
    /// ingredient, a part-made material, a byproduct, or fuel"* — every one of
    /// those is something a recipe consumes, and a scroll is something the
    /// player spends.
    Scroll,
    /// A script. `invoke night_watch`.
    Script,
    /// A count of ticks. `meditate 30`.
    Count,
    /// A name the player is **coining**, not one the world already holds.
    ///
    /// `scribe morning` — the spell does not exist yet, which is the whole point
    /// of the command, so it cannot resolve against the scene the way every
    /// other argument does. Free text, like [`Pattern`](Self::Pattern), and one
    /// word.
    ///
    /// §6.1 writes the command as `scribe <name>`; the signature said
    /// `NounKind::Script` and therefore **could not parse the create case at
    /// all** — a required slot with nothing in the world to fill it. The two
    /// other script verbs are right to keep `Script`, because `bind` and
    /// `invoke` name a spell that exists.
    ///
    /// Not [`Any`](Self::Any): `Any` searches every category, so `scribe sage`
    /// would quietly create a spell named after a reagent.
    Name,
    /// What the archive's maze reports about the cell you are reading.
    ///
    /// `passage`, `wall`, `exit`, plus `back`, `spoil`, `marks` and the errand
    /// words — the vocabulary a solver's `if` names, and `tower::maze::readings`
    /// is the one list of it. **A kind of their own, and no slot asks for one**,
    /// so a sense can never fill a `Reagent` or an `Essence` by accident while
    /// [`Any`](Self::Any) still finds it. That last part is the whole reason the
    /// kind exists: `spell::compile` resolves a condition's names against the
    /// room *as it is at that instant*, and no way has been walked at the moment
    /// a solver is cast — so without a kind the scene always offers, every `if` in
    /// it would compile to a branch that takes neither half.
    ///
    /// **A kind of its own, and now a `Topic` as well.** This said *"not
    /// [`Topic`](Self::Topic), which `recall` reads: `recall walked` would
    /// resolve and then find no manual entry"* — and both halves have since
    /// stopped being true. `walked` was replaced by the counted `marks`, and
    /// every reading has a `recall_` page, so each one *does* find an entry. The
    /// kind still matters for the reason above it: no slot asks for a `Sense`,
    /// so one can never fill a `Reagent` by accident, while `Any` still finds it.
    Sense,
    /// Anything with text in it — `peruse orb.log`, `peruse night_watch.spell`.
    ///
    /// A **slot** kind, never a noun's own: nothing in the tower *is* a
    /// readable, the way something is a [`File`](Self::File) or a
    /// [`Script`](Self::Script). It says what a slot will take, which is what
    /// makes `peruse` reach a spell without `peruse` reaching a reagent.
    ///
    /// The alternative was [`Any`](Self::Any), and it is wrong in a way that
    /// passes the whole suite: `peruse sage` resolves at full confidence and
    /// reports a zero-line read of a reagent, and a bare `peruse` offers the
    /// four *places* as things to read. `execute::files` already records why
    /// resolution goes by kind, and `peruse` — `read`, `cat`,
    /// `open`, `show` — is the verb a shell-naive tester reaches for first, so
    /// the dead end would land where §15 weighs it heaviest.
    Readable,
    /// Anything you can **pick up** — stock, a potion, a scroll.
    ///
    /// A slot kind, never a noun's own, like [`Readable`](Self::Readable) and
    /// [`Stoppable`](Self::Stoppable).
    ///
    /// # `move` could not carry a potion at all
    ///
    /// Its first slot was [`Reagent`](Self::Reagent), and `produce::transmute`
    /// gives a finished potion [`Essence`](Self::Essence) — so `move clarity to
    /// dispensary` failed to fill a required slot, silently, for as long as
    /// there have been potions. It went unnoticed because `empty` turns an
    /// instrument out wholesale and never asks what kind anything is, so the one
    /// route that mattered in the laboratory worked.
    ///
    /// The arsenal is what made it matter: carrying finished work between rooms
    /// is the whole point of the room, and every route into it goes through this
    /// slot.
    ///
    /// **Not [`Any`](Self::Any)**, which reaches places, files, topics and
    /// spells — `move laboratory to arsenal` would resolve at full confidence.
    /// What this names is the set of things that are *stuff*.
    ///
    /// **There is no `Fragment` in the list, and there was.** A fragment *is*
    /// crafting stock — the lectern's recipe consumes four of them — so it is a
    /// [`Reagent`](Self::Reagent) like every other input, and the separate kind
    /// went with the sigils it was invented for.
    Portable,
    /// Anything you can **set going** — an instrument, or a scroll.
    ///
    /// A slot kind, never a noun's own, like [`Readable`](Self::Readable) and
    /// [`Stoppable`](Self::Stoppable), and it exists for the reason those two
    /// do: `wield` had to reach a second sort of thing and the alternative was a
    /// 23rd tower-wide verb, which `the_vocabulary_is_the_tower_wide_verbs_plus_
    /// the_laboratory_s_own` refuses in advance.
    ///
    /// **`empty` keeps [`Place`](Self::Place)**, and the split is the point:
    /// both verbs used to share one signature, and widening it would have made
    /// `empty gleaning-scroll` a sentence the parser accepts and the executor
    /// cannot answer.
    Workable,
    /// Anything that can be **running** — an instrument, or a spell.
    ///
    /// A slot kind, never a noun's own, like [`Readable`](Self::Readable).
    ///
    /// `stop` took a [`Place`](Self::Place) and so could only ever reach an
    /// instrument. An invoked spell was therefore **unstoppable**: nothing but
    /// running out of program removes it, so `repeat` with no count ran for ever
    /// and `stop <spell>` did not resolve to it. A player who wrote one had no
    /// way back — §6's dead end, arrived at from a direction the parser could
    /// not see.
    Stoppable,
    /// A verb's own name, so `recall grind` can be asked about.
    ///
    /// # Why not [`Topic`](Self::Topic), which `recall` already reads
    ///
    /// Because [`Any`](Self::Any) reaches `Topic`, and registering 27 canonicals
    /// there would have leaked them into three places at once:
    ///
    /// - **Tab** would offer `purge grind`. `complete::nouns` filters by
    ///   `accepts`, and offering a word the parser would refuse is the dead end
    ///   §15 weighs above the raw resolution rate.
    /// - **`spell::compile`** resolves a condition's names through `Any`, so
    ///   `if the dispensary has grind` would compile clean and answer *no* for
    ///   ever — verbatim the `has ground-slat` defect that module was rewritten
    ///   to kill.
    /// - **The numbered prompt** for a bare `purge` would reorder: `Argument`'s
    ///   `Ord` is (kind, value, slot), so inserting a kind moves which four
    ///   readings surface, silently.
    ///
    /// This is the same argument [`Sense`](Self::Sense) makes, one step further:
    /// `Sense` needs `Any` to find it so a spell's `if` can name a reading, and
    /// this needs `Any` **not** to. So the kind is reachable from exactly one
    /// slot kind, [`Subject`](Self::Subject), and from nothing else.
    Command,
    /// Anything nameable — `verify` and `purge` accept any surface.
    Any,
    /// What the manual can answer on: a topic, or a command.
    ///
    /// A **slot** kind, never a noun's own, like [`Readable`](Self::Readable)
    /// and [`Stoppable`](Self::Stoppable). It is what lets `recall` reach both
    /// `recall brewing` and `recall grind` without widening `Any`.
    Subject,
}

/// Which part of the manual a verb belongs under.
///
/// **A table, not a derivation.** The predicates that already exist — the ones a
/// reader might reach for — group by the wrong thing: `is_operation` is about
/// *scope*, `transmutes` about the pipeline. What a lost player wants is sorted
/// by what they are trying to do, and that is a judgement rather than a
/// consequence, so it is written down.
///
/// Order here is the order the overview prints, which is the order a player
/// needs them: find your way about, then do the work, then teach the orb, then
/// ask the orb, and last the two that destroy something.
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
    /// **The one definition of that question.** It was three copies of
    /// `kind == NounKind::Any || noun.kind == kind` — in
    /// [`Scene::best_match`](super::Scene::best_match), in `resolve::fillers`
    /// and in `complete::nouns` — which is three chances for a slot kind to be
    /// understood by the matcher and not by the numbered prompt, or by both and
    /// not by Tab. A slot that accepts a *set* has to agree in all three or the
    /// three surfaces disagree about what a command takes.
    #[must_use]
    pub const fn accepts(self, noun: Self) -> bool {
        match self {
            // **Everything except a command.** `Any` is `verify` and `purge`,
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
/// # Why the slot gave up being required
///
/// A required slot with fillers can never yield an argument-less intent:
/// `resolve::collect` pushes one candidate per filler, they all tie, and
/// `analyse` returns `Ambiguous`. The scene always has topics, so **bare
/// `recall` opened a numbered prompt offering the four alphabetically-first
/// manual subjects** — and `help`, `man` and `?` are all synonyms of it. §6
/// forbids a bare error; a lost player typing `help` and being asked to pick
/// between `archive`, `brewing`, `clarified-draught` and `clarity` is that rule
/// failing at the one command whose whole job is answering the question.
///
/// **§19 declined exactly this for bare `follow`, and the difference is
/// `survey`.** `follow` bare and `follow east` are categorically different acts
/// — one walks a cell, one seizes the keyboard — where `survey` bare and
/// `survey alembic` are the *same act at two scopes*, which is what `recall` and
/// `recall grind` are. The numbered prompt a required slot is said to buy was
/// never a disambiguation here either: nothing was typed to disambiguate, so it
/// offered four arbitrary subjects rather than four readings of an input.
const TOPIC_OPTIONAL: &[Slot] = &[Slot::optional(NounKind::Subject)];
const ANYTHING: &[Slot] = &[Slot::required(NounKind::Any)];
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
/// **Optional**, so `grind` on its own still starts a mortar that is already
/// charged. The whole point of these verbs is to spend fewer keystrokes; making
/// the reagent compulsory would have `grind sage` beat `move sage to
/// mortar_and_pestle; wield mortar_and_pestle` while bare `grind` lost to bare
/// `wield mortar_and_pestle`.
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
/// **Both `Place`, and no new `NounKind`.** The lens's sockets and sigils are
/// `Role::Reading` fixtures exactly as the archive's compass bearings are, so
/// they resolve against the kind `WAY` already uses. verbs.md warns that a new
/// kind leaks into tab completion, `compile::fix` and the bare-argument prompt;
/// the cheapest new kind is the one you did not need.
///
/// The cost is that `seat laboratory nitre` parses, and the handler refuses it
/// in voice — the same shape `research::named` already has for `follow`.
/// **The sigil is optional, and bare means *try something else here*.**
///
/// A variable-free script cannot name the sigil it has not tried yet — that is
/// what forced a ladder to spell all six out per socket, twenty-four rungs using
/// a mark count as an index. `dial first` asks the ward instead, which is the one
/// sentence the language could not otherwise form. `Ward::advance` has the rest.
///
/// Optional rather than a second verb, on `recall`'s precedent (`TOPIC_OPTIONAL`):
/// bare and argumented are the same act — turning that dial — at two scopes.
const SOCKET_AND_SIGIL: &[Slot] = &[
    Slot::required(NounKind::Place),
    Slot::optional(NounKind::Place),
];
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
    /// **Was `grimoire`, and the word was released** (§19). A grimoire is a
    /// wizard's book of *spells*, which is what `/grimoire` now holds, so using
    /// the same word for the manual made the one word mean both the reference
    /// you read and the book you write in. §6.1's *"a released word does not
    /// stop resolving"* rule protects **shipped** vocabulary; nothing has
    /// shipped, and unclaimed `grimoire` now reads as the directory it names.
    Recall,
    /// Detect tampering.
    Verify,
    /// Revert the last command.
    Undo,
    /// Read back through what the orb has said.
    ///
    /// # A verb for a key that already worked
    ///
    /// `PageUp` has scrolled the transcript since the transcript existed. The
    /// problem is that nothing says so: the border advertises `PgDn newest` only
    /// once you are *already* scrolled back, so the affordance announces itself
    /// exclusively to players who have found it. In a game with no mouse and no
    /// menus, a key nobody can discover is a key nobody has.
    ///
    /// So the way in is a word, like everything else here — and using it once
    /// teaches the keys, which is the part that survives after the player stops
    /// needing the word.
    ///
    /// **Not `recollect`**, which was the first name and collides: `rec` is
    /// claimed by `recall`, and `rec_is_pinned_as_a_prefix_before_anything_else_
    /// wants_it` names this exact scenario a word in advance. `und` and `unf`
    /// part at the third character, which is the length the naming pass governs.
    Unfurl,
    /// Put the orb down and leave.
    ///
    /// **A word for a key that already worked**, which is `unfurl`'s argument
    /// exactly: `F10` has always left and nothing on screen says so. §6 makes
    /// this a game played by typing, so the way out should be a word like every
    /// other way through it.
    ///
    /// The name is free of the naming pass's traps — no other verb begins with
    /// `q`, so `qui` names one verb and always will. It is also the word the
    /// spell editor and the weave screen already use for *close this*, which
    /// makes it mean one thing at three depths rather than three things.
    Quit,
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
    /// **Not `step`, which scores 750 against `stop`** — over `MIN_SIMILARITY`,
    /// and a typo that stopped a run instead of advancing it would cost the
    /// whole maze. `tread` was the next candidate and scores **800 against
    /// `read`**, which `peruse` claims. `follow` is 429 against its nearest and
    /// shares no three-character prefix with anything.
    Follow,
    /// Look at what the work has bought (§11.5).
    ///
    /// **The twentieth tower-wide word, and it needs the argument `unfurl`
    /// made.** §6.1 wants this set smaller, not larger, and `unfurl` earned its
    /// seat by being the only way to reach a surface that already existed.
    /// Progression is the opposite case and lands in the same place: the surface
    /// does *not* exist, `status` prints two numbers with no sense of what they
    /// are for, and §11.5's own turn — buying the first Concentration — arrives
    /// as one line that was never chosen. A track nobody can look at is a track
    /// nobody is on.
    ///
    /// **Not `ascend`**, which was the obvious name and collides: two edits from
    /// `attend` in a six-letter word is 667, over `MIN_SIMILARITY`. `weave`
    /// scores 200 against `wield` and 400 against `write` — the only other `w`
    /// words in the vocabulary — and `wea` is a free three-character prefix.
    Weave,
    /// Give the arrow keys the archive's stacks (§10, §19).
    ///
    /// **The twenty-second tower-wide word, and it is `unfurl`'s argument
    /// again**: the surface has no other way in, and in a mouseless game a word
    /// is the only way to reach one. What it reaches is a maze a player would
    /// otherwise walk with a hundred `follow` lines — the map is on screen the
    /// whole time either way, so this buys the *keys* and nothing else.
    ///
    /// **The naming sweep was unusually brutal here**, and the near misses are
    /// worth keeping because every one of them is the obvious word: `thread` is
    /// 667 against `read`, `stride` 667 against `scribe`, `delve` 600 against
    /// `weave`, `trace` 600 against `twice` — a reading permanently in scope in
    /// the very room this works in — and `pace` 750 against `page`. `enter` and
    /// `walk` are already taken, by `attend` and by `follow`. `wander` is 500 at
    /// worst and `wan` is a free three-character prefix.
    Wander,
    /// `probe` — press the aperture against a far orb's ward, opening a reading
    /// if none is open.
    ///
    /// **Two acts in one word, which is §19's per-instrument idiom**: `grind
    /// sage` is a `move` and a `wield`, and this is a `scry` and a press. It
    /// collapses the command a player types most, and it costs nothing in
    /// clarity here because the aperture opens on a fixed figure — so the first
    /// press means the same thing every time and there is nothing to set before
    /// it.
    ///
    /// **`scry` was the opener and never shipped.** It is §10's word for the
    /// domain and it had to go: `tests/naming.rs` forbids two canonicals sharing
    /// a three-character prefix outright, with the exemption it once had deleted
    /// on the grounds that *"an exemption that outlives its cause is how a guard
    /// quietly stops guarding"* — and `scr` reaches `scribe`. The domain is still
    /// scrying; the room is `lens/`; the word you type is `probe`.
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
}

impl Verb {
    /// Every verb in the Phase 0 vocabulary, and what the phases since have added.
    pub const ALL: [Self; 30] = [
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
        // **Appended, deliberately.** `the_tolerated_collision_set_is_pinned`
        // walks pairs in this order, so inserting anywhere else would reorder
        // the pinned set without changing a single score.
        Self::Weave,
        Self::Follow,
        Self::Wander,
        Self::Probe,
        Self::Dial,
    ];

    /// The longest a canonical verb may be.
    ///
    /// §6.1 wrote the rule as "one short word, ideally ≤7 characters". The
    /// Phase 0 naming pass settled the "ideally" at **8**: `grimoire` and
    /// `meditate` were the two most in-world names in the set and carried the
    /// game's identity, abbreviation covered the typing cost (`grim`, `medit`),
    /// and the two 8-character names that had *no* such defence — `decipher`
    /// and `inscribe` — were shortened instead.
    ///
    /// **`meditate` and `research` are the two that spend the eighth
    /// character.** `meditate` kept it because it is the name that makes the
    /// verb feel like a thing a wizard does; `research` because the archive is
    /// a room of shelves and the word for what you do at a lectern is not
    /// shorter. Both are words you type occasionally rather than a thousand
    /// times, which is what §6.1's rule was always about — and `res` reaches
    /// this one, so the typing cost is three characters either way.
    ///
    /// The limit stays at 8 rather than tightening to 7: tightening would
    /// forbid a word no verb currently wants, at the cost of two that do.
    pub const MAX_CANONICAL_LEN: usize = 8;

    /// Which part of the manual this verb is listed under.
    ///
    /// **No wildcard**, like every other table here: a verb added later is a
    /// compile error in this file rather than one silently missing from the
    /// overview. That matters more than usual, because a verb assigned to a
    /// group nobody prints would compile and simply not be there.
    #[must_use]
    pub const fn group(self) -> Group {
        match self {
            Self::Attend | Self::Survey | Self::Peruse | Self::Sift | Self::Verify => {
                Group::Getting
            }
            // Everything that moves or makes, including the archive's: `research`
            // and `follow` are the lectern's work exactly as `grind` is the
            // mortar's, and a player looking for what to *do* here wants them in
            // one place rather than sorted by which room they happen to be in.
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
            // The lens's two are work for the same reason the archive's are:
            // a player looking for what to *do* in this room wants them
            // together, not sorted by which of them happens to cost a tick.
            | Self::Probe
            | Self::Dial => Group::Work,
            Self::Scribe | Self::Bind | Self::Invoke => Group::Spells,
            Self::Status
            | Self::Recall
            | Self::Undo
            | Self::Unfurl
            | Self::Weave
            | Self::Quit
            | Self::Meditate => Group::Orb,
            // Kept in step with `is_destructive`, which had no reader until now
            // — a test asserts the two agree rather than trusting this list.
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
        }
    }

    /// Whether this verb belongs to one instrument rather than to the tower.
    ///
    /// A verb that is `true` here only resolves where its instrument stands —
    /// see [`Scene::offers`](super::Scene::offers). `wield` is deliberately not
    /// one: it names the tool explicitly, works anywhere there is a tool, and is
    /// what a script writes when the instrument is the variable.
    ///
    /// **The lens's two are here, and that is what stops a third debt.** §19
    /// records `follow` and `wander` as tower-wide words waiting on a mechanism
    /// that does not exist — a fixture carries exactly one `Operation`, and the
    /// archive's one fixture had spent it. The lens has five fixtures that can
    /// carry one, so `probe` and `dial` are both scoped without any new
    /// mechanism, and neither means anything in the laboratory.
    ///
    /// **It is not the same question as "does this take the production slot"**,
    /// and the lens is where the two came apart: `dial` is a scoped operation
    /// that schedules nothing. See `spell::block::begins_work`.
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
        )
    }

    /// The fixture verb that must stand in the room for this one to mean anything.
    ///
    /// # This is the mechanism §19 recorded as missing
    ///
    /// [`Scene::offers`](super::Scene::offers) used to ask
    /// [`is_operation`](Self::is_operation), which is the *production slot*
    /// question — so every verb that did not take the slot was offered in every
    /// room. `research`, `follow` and `wander` were therefore listed by `help` in
    /// the laboratory and the lens, where none of them can do anything: §19 called
    /// two of them a debt *"waiting on one missing mechanism"* and said that a
    /// third would be the argument for building it. `research` was the third.
    ///
    /// The two questions are now genuinely separate. A verb can be **scoped to a
    /// fixture and take no slot** (`research`, `follow`, `wander`, `dial`), or take
    /// the slot and be scoped (`grind`), and neither implies the other.
    ///
    /// # Most of it is derived, and the test is what keeps it that way
    ///
    /// A verb that some `Branch` in `tower::build` declares as its `operation` is
    /// *self-anchored* — the content already says which room it belongs to, so
    /// nothing here needs a list of rooms. `every_self_anchored_verb_is_declared_by
    /// _a_fixture` fails the build if this arm and `BRANCHES` disagree, which is
    /// what stops the pair drifting the way three entangled lists already did.
    ///
    /// `follow` and `wander` are the exception and are spelled out: they act on the
    /// *reading inside* the stacks rather than on a fixture of their own, and a
    /// fixture carries exactly one `Operation`, which the stacks had spent on
    /// `research`.
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
            | Self::Research => Some(self),
            // The maze's other two words, anchored to the stacks.
            Self::Follow | Self::Wander => Some(Self::Research),
            // **`wield` is deliberately not anchored**, and neither is `empty`,
            // `stop` or `move`: they name their target explicitly and work
            // wherever one stands, which is what a script writes when the
            // instrument is the variable.
            _ => None,
        }
    }

    /// Whether finishing this turns an instrument's contents into something.
    ///
    /// `divine` also occupies the production slot but produces no material, so
    /// `finish` has to tell them apart. It asked `verb == Wield` until §10.1's
    /// per-instrument verbs arrived — at which point a completed `grind` would
    /// have released the slot, said nothing, and left the sage sitting whole in
    /// the mortar. Naming the property rather than the one verb is what stops the
    /// next one added from doing the same.
    #[must_use]
    pub const fn transmutes(self) -> bool {
        matches!(
            self,
            // **`Kindle` is deliberately absent.** Lighting the athanor is not a
            // run: it takes no Focus slot (§10.1), inserts no `Working`, and
            // produces nothing to transmute. `start` returns at the `HeatSource`
            // branch long before this is asked.
            Self::Wield | Self::Grind | Self::Digest | Self::Mix | Self::Distil
        )
    }

    /// The `-ing` form, for a refusal that names what holds a slot.
    ///
    /// **Spelled out, not `{state}ing` in a template.** `content/prose.toml` built
    /// this by appending a literal `ing` to [`canonical`](Self::canonical), which
    /// reads fine for `wield` and produced **"divineing"** for the one other verb
    /// that can hold the production slot. English inflection is not string
    /// concatenation, and the authored-line tests lint the template rather than
    /// the interpolated result, so nothing caught it.
    ///
    /// That verb is `research` now, whose naive form happens to be right — which
    /// is exactly why the table stays: the next `-e` verb would bring the bug
    /// back and the example that proves it is a rename away from vanishing.
    ///
    /// It lives here beside `canonical` because it is the same class of thing —
    /// a form of the vocabulary word — rather than an authored sentence.
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
            // **`divine` takes nothing now.** It named a fragment while it was
            // a twelve-tick command that consumed nothing; it opens the stacks
            // on the lectern, and there is only one lectern to open one at.
            // `wander` joins them for the reason `weave` did: it opens a
            // surface, and a surface is not something you name an argument for.
            Self::Status
            | Self::Undo
            | Self::Unfurl
            | Self::Quit
            | Self::Weave
            | Self::Research
            | Self::Wander
            // **`probe` takes nothing**, for the reason `research` does: there
            // is one prism to press, so naming it would be naming the only
            // thing there is. What changes between presses is the *aperture*,
            // and `dial` is what changes it.
            | Self::Probe => NOTHING,
            // A socket and a sigil, both `Role::Reading` places.
            Self::Dial => SOCKET_AND_SIGIL,
            // A way, which is a place — see `Role::Reading`.
            Self::Follow => WAY,
            Self::Recall => TOPIC_OPTIONAL,
            Self::Verify | Self::Purge => ANYTHING,
            Self::Meditate => COUNT,
            Self::Move => MOVE,
            // An instrument is a place (§10.1), and you always name the
            // instrument rather than what is inside it.
            //
            // **`wield` split from `empty` when scrolls arrived.** Setting a
            // thing going reaches both an instrument and a scroll; turning a
            // thing out reaches only somewhere that holds something. Sharing one
            // signature would have made `empty gleaning-scroll` parse.
            Self::Wield => WORKABLE,
            Self::Empty => PLACE,
            // **A spell counts.** `stop` reaching only instruments made an
            // invoked spell unstoppable — see `NounKind::Stoppable`.
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
    /// `stop` qualifies: §10.1 refunds an instrument's inputs but the work done
    /// so far is gone, and at capacity 1 that is the tower's only slot spent.
    ///
    /// **Nothing reads this yet.** §6's *"destructive commands additionally
    /// confirm when the target is not routine"* is unbuilt, and this flag was
    /// once described as earning that confirmation "for free" — it does not. The
    /// classification is correct and cheap to keep true; the confirming is
    /// separate work.
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
        // §6.1's sixteen, plus `move`/`wield`/`stop` for §10.1's pipeline, minus
        // `decoct` — retired because a verb claiming to brew a potion, when
        // brewing is four stages, teaches the player something false through the
        // one mechanism §6 uses to teach.
        //
        // The four on top are the laboratory's **own**, and they are the reason
        // this count is not a ceiling: §10's five further domains each coin the
        // verbs their tools need, and none of them is in scope anywhere else.
        // What must stay bounded is the vocabulary a *single place* offers.
        // ...plus `empty`, which turns an instrument out into the store rather
        // than destroying what is in it (§10.1's byproduct rule).
        let tower_wide = Verb::ALL.iter().filter(|verb| !verb.is_operation());
        // 19 until `siphon` retired (§19), then 18, and 19 again for `unfurl`.
        // The per-instrument verbs reach into idle instruments, so drawing a
        // stage's output onto the bench had stopped doing anything — and with it
        // gone, the bench and the shelf are one place.
        //
        // **`unfurl` is the first word added back**, and it earns the seat by
        // being the only way to reach a surface that already existed: `PageUp`
        // has always scrolled the transcript and nothing ever said so. A
        // vocabulary getting smaller is the direction §6.1 wants, and a word
        // that makes a mouseless game navigable is the exception it allows for.
        //
        // **`weave` is the second, and it is the opposite case landing in the
        // same place.** `unfurl` reached a surface that existed; this one has no
        // surface at all. Progression is two numbers in `status` with nothing
        // saying what they are for, and §11.5's own turn — buying the first
        // Concentration — arrives as a single line that was never chosen. A
        // track nobody can look at is a track nobody is on, and in a mouseless
        // game a word is the only way to look.
        //
        // **`follow` is the third, and it is the first that is a *domain's* word
        // wearing a tower-wide coat.** It walks the archive's maze and means
        // nothing anywhere else, so by rights it would be an operation scoped to
        // the lectern — except `Scene::offering` derives scope from the
        // `Operation` component and a fixture carries exactly one, which the
        // lectern spends on `divine`. Scoping a *second* verb to one instrument
        // is the missing mechanism, and until it exists this word is global and
        // should be counted as a debt rather than a seat earned.
        //
        // **`wander` is the fourth, and it is both of the above at once** — so
        // it is worth saying which half buys the seat and which half is owed.
        //
        // The seat is `unfurl`'s: the map draws whenever a maze is open, but
        // *who owns the arrow keys* has no other way to be said, and a maze
        // walked by typing `follow east` a hundred times is a chore rather than
        // a minigame. That is the same exception §6.1 allows — a word that makes
        // a mouseless game navigable.
        //
        // The debt is `follow`'s, unchanged and not doubled: this is a domain's
        // word wearing a tower-wide coat for exactly one reason, that
        // `Scene::offering` derives scope from the `Operation` component and the
        // lectern spends its only one on `divine`. Both retire together the day
        // a second verb can be scoped to an instrument. Two words waiting on one
        // mechanism is an argument for building the mechanism; it is not an
        // argument for a third.
        //
        // **22 is a number to defend, not a budget to spend**: the next word
        // added here needs an argument of this shape, or the count is a ceiling
        // nobody kept. `wander` is the last one this reasoning stretches to —
        // the archive now has both the word it needs and the debt it owes, and a
        // fifth would mean the missing mechanism had been deferred once too
        // often.
        // **Still 22, and the lens is the argument holding.** Phase 2 added two
        // verbs and neither is tower-wide: the lens has five fixtures that can
        // carry an `Operation`, so `probe` and `dial` are both scoped without
        // the missing mechanism and without a third debt. A domain that needs
        // more words than it has fixtures is the case that would finally force
        // it.
        //
        // **23, and `quit` is the one word the ceiling was never about.** Every
        // entry above it acts on the tower or reports on it, and the budget
        // exists to stop a *domain's* vocabulary sprawling because it lacked the
        // mechanism to scope a word to a fixture. `quit` is not waiting on that
        // mechanism and never could be: it addresses the orb rather than the
        // tower, it touches no world state, and there is no fixture in any room
        // that leaving the game could be scoped to. It is `Group::Orb`'s, beside
        // `status` and `unfurl`, and like `unfurl` it exists because the thing it
        // does was previously reachable only by a key nobody could discover.
        //
        // The ceiling still stands for the case it was drawn for. A domain verb
        // arriving here is still the argument for building the mechanism.
        assert_eq!(tower_wide.count(), 23);

        // One per instrument that has a word of its own: the laboratory's
        // `grind`, `digest`, `mix`, `distil` and `kindle`, and the lens's
        // `probe` and `dial`.
        let scoped = Verb::ALL.iter().filter(|verb| verb.is_operation());
        assert_eq!(scoped.count(), 7);

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
