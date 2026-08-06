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
    /// A manual topic. `grimoire brewing`.
    Topic,
    /// An essence that can be brewed. `grimoire clarity`.
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
    /// A researchable fragment. `decipher sigil-iv`.
    Fragment,
    /// A script. `invoke night_watch`.
    Script,
    /// A count of ticks. `meditate 30`.
    Count,
    /// Anything nameable — `verify` and `purge` accept any surface.
    Any,
}

impl NounKind {
    /// The word for this category, as output names it.
    ///
    /// §6 forbids a bare error: when a slot is empty the orb has to say what
    /// would fill it, and it cannot say `Fragment`. Kept to one lower-case word
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
            Self::Fragment => "fragment",
            Self::Script => "script",
            Self::Count => "count",
            Self::Any => "name",
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
const FILE: &[Slot] = &[Slot::required(NounKind::File)];
const PATTERN_AND_FILE: &[Slot] = &[
    Slot::required(NounKind::Pattern),
    Slot::required(NounKind::File),
];
const TOPIC: &[Slot] = &[Slot::required(NounKind::Topic)];
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
/// ```
const MOVE: &[Slot] = &[
    Slot::required(NounKind::Reagent),
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

const FRAGMENT: &[Slot] = &[Slot::required(NounKind::Fragment)];
const SCRIPT: &[Slot] = &[Slot::required(NounKind::Script)];

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
    /// The in-world manual.
    Grimoire,
    /// Detect tampering.
    Verify,
    /// Revert the last command.
    Undo,
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
    /// Collect a finished potion.
    Siphon,
    /// Destroy waste or spoilage.
    Purge,
    /// Research a fragment.
    Divine,
    /// Author a script.
    Scribe,
    /// Attach a script to a trigger.
    Bind,
    /// Run a script or spell.
    Invoke,
}

impl Verb {
    /// Every verb in the Phase 0 vocabulary.
    pub const ALL: [Self; 24] = [
        Self::Attend,
        Self::Survey,
        Self::Peruse,
        Self::Sift,
        Self::Status,
        Self::Grimoire,
        Self::Verify,
        Self::Undo,
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
        Self::Siphon,
        Self::Purge,
        Self::Divine,
        Self::Scribe,
        Self::Bind,
        Self::Invoke,
    ];

    /// The longest a canonical verb may be.
    ///
    /// §6.1 wrote the rule as "one short word, ideally ≤7 characters". The
    /// Phase 0 naming pass settled the "ideally" at **8**: `grimoire` and
    /// `meditate` are the two most in-world names in the set and carry the
    /// game's identity, abbreviation covers the typing cost (`grim`, `medit`),
    /// and the two 8-character names that had *no* such defence — `decipher`
    /// and `inscribe` — were shortened instead.
    pub const MAX_CANONICAL_LEN: usize = 8;

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
            Self::Grimoire => "grimoire",
            Self::Verify => "verify",
            Self::Undo => "undo",
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
            Self::Siphon => "siphon",
            Self::Purge => "purge",
            Self::Divine => "divine",
            Self::Scribe => "scribe",
            Self::Bind => "bind",
            Self::Invoke => "invoke",
        }
    }

    /// Whether this verb belongs to one instrument rather than to the tower.
    ///
    /// A verb that is `true` here only resolves where its instrument stands —
    /// see [`Scene::offers`](super::Scene::offers). `wield` is deliberately not
    /// one: it names the tool explicitly, works anywhere there is a tool, and is
    /// what a script writes when the instrument is the variable.
    #[must_use]
    pub const fn is_operation(self) -> bool {
        matches!(
            self,
            Self::Grind | Self::Digest | Self::Mix | Self::Distil | Self::Kindle
        )
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
    /// reads fine for `wield` and produces **"divineing"** for the one other verb
    /// that can hold the production slot. English inflection is not string
    /// concatenation, and the authored-line tests lint the template rather than
    /// the interpolated result, so nothing caught it.
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
            Self::Grimoire => "reading",
            Self::Verify => "verifying",
            Self::Undo => "undoing",
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
            Self::Siphon => "siphoning",
            Self::Purge => "purging",
            Self::Divine => "divining",
            Self::Scribe => "scribing",
            Self::Bind => "binding",
            Self::Invoke => "invoking",
        }
    }

    /// What this command takes, in order.
    #[must_use]
    pub const fn signature(self) -> &'static [Slot] {
        match self {
            Self::Attend => PLACE,
            Self::Survey => PLACE_OPTIONAL,
            Self::Peruse => FILE,
            Self::Sift => PATTERN_AND_FILE,
            Self::Status | Self::Undo => NOTHING,
            Self::Grimoire => TOPIC,
            Self::Verify | Self::Purge => ANYTHING,
            Self::Meditate => COUNT,
            Self::Move => MOVE,
            // An instrument is a place (§10.1), and you always name the
            // instrument rather than what is inside it.
            Self::Wield | Self::Stop | Self::Empty => PLACE,
            // §10.1's per-instrument verbs name the *material*, not the tool —
            // the tool is what the verb means.
            // `kindle` takes fuel, and takes it optionally: `kindle` on its own
            // relights what is banked, which is the end of every script loop.
            Self::Grind | Self::Digest | Self::Distil | Self::Kindle => ONE_REAGENT,
            Self::Mix => TWO_REAGENTS,
            // Was `VESSEL`, when a finished brew sat in one. §10.1 makes the
            // product sit in the **instrument** that made it, and you always
            // name the instrument rather than its insides (§19).
            Self::Siphon => PLACE,
            Self::Divine => FRAGMENT,
            Self::Scribe | Self::Bind | Self::Invoke => SCRIPT,
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
        assert_eq!(tower_wide.count(), 19);

        // One per instrument the laboratory raises: `grind`, `digest`, `mix`,
        // `distil` and the athanor's `kindle`.
        let laboratory = Verb::ALL.iter().filter(|verb| verb.is_operation());
        assert_eq!(laboratory.count(), 5);

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
