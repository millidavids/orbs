//! How people say the things the orb does (§6, §19 *the augury*).
//!
//! Authored in `content/phrasings.toml`, not here (rule 6). This module knows
//! the *shape* of a template and nothing about which ones exist.
//!
//! # Not loaded by a `Sim`, deliberately
//!
//! Every other content file reaches a decision the world makes. This one
//! reaches none: it is the corpus a reader is trained on and the fixture the
//! parser is measured against, and a running tower needs neither. So there is
//! no `Phrasings` resource and nothing installs one — `builtin()` is called by
//! the bench and, later, by the trainer.
//!
//! Keeping it out of the world is what stops it becoming a *second* vocabulary:
//! §19 records what two expressions of one rule cost, and a phrase table the
//! parser consulted at run time would be exactly that.
//!
//! # Templates, not sentences
//!
//! A `{slot}` names a [`NounKind`] and expands over everything of that kind the
//! scene holds, so one authored line becomes as many examples as there are
//! reagents. That is what makes a corpus large enough to train on something one
//! person can write and keep true — and because the expansion knows where it
//! substituted, **the argument spans come out of generation rather than out of
//! hand-annotation**.
//!
//! # `say` and `holdout` are not interchangeable
//!
//! `say` is the corpus. `holdout` is never taught to anything and is the only
//! thing worth measuring on: a grammar built from `say` matches `say` perfectly
//! and has proved nothing.

use serde::Deserialize;

use crate::parser::NounKind;

/// The compiled-in default, so the bench needs no filesystem.
const BUILTIN: &str = include_str!("../../content/phrasings.toml");

/// The spell language's phrasings, in the same format.
const SPELLINGS: &str = include_str!("../../content/spellings.toml");

/// The file's name, for an error a writer can act on.
const SPELL_FILE: &str = "spellings.toml";

/// How many examples one template may contribute to a training corpus.
///
/// **A number about balance, not about size.** See
/// [`Phrasings::corpus_capped`] for why an uncapped expansion made `move` 47%
/// of the corpus; this is the value the trainer and
/// `no_single_verb_owns_the_capped_corpus` agree on, so raising it cannot
/// quietly reintroduce the skew.
pub const CORPUS_CAP: usize = 24;

/// The file's name, for an error a writer can act on.
const FILE: &str = "phrasings.toml";

/// One canonical command and the ways people ask for it.
#[derive(Debug, Clone, Deserialize)]
pub struct Phrasing {
    /// The command, in the form the orb echoes. May carry slots.
    pub canonical: String,
    /// Phrasings the corpus is built from.
    #[serde(default)]
    pub say: Vec<String>,
    /// Phrasings nothing is ever taught, and everything is measured on.
    #[serde(default)]
    pub holdout: Vec<String>,
}

/// Sentences that are not commands at all.
///
/// **No `canonical`, and that is the point of them.** A reader shown only
/// commands has never seen a sentence it should refuse and answers one anyway —
/// `what should i do next` came back as `recall`, which is the dead end §15
/// weighs heaviest.
///
/// They carry slots like anything else, deliberately: a refusal must not be
/// learnable as *"a sentence with no game words in it"*, because the hard ones
/// are exactly the sentences that name a reagent and still ask for nothing.
#[derive(Debug, Clone, Deserialize)]
pub struct Refusal {
    /// Phrasings the reader is taught to refuse.
    #[serde(default)]
    pub say: Vec<String>,
    /// Phrasings it is measured on and never taught.
    #[serde(default)]
    pub holdout: Vec<String>,
}

/// Every authored template.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Phrasings {
    /// One per canonical command.
    #[serde(default, rename = "entry")]
    entries: Vec<Phrasing>,
    /// Sentences that ask for nothing.
    #[serde(default, rename = "refusal")]
    refusals: Vec<Refusal>,
    /// Words a player might choose as a **name**, for a `{name}` slot.
    ///
    /// Free text, so it cannot come from a content table: `let best be north`
    /// binds a word the tower has never heard of, and that is the point of it.
    /// **Deliberately kept out of the reader's vocabulary** — a name a player
    /// invents reaches the reader as a hash bucket, so the ones it learns from
    /// have to arrive the same way.
    #[serde(default)]
    names: Vec<String>,
    /// ...and the names only the holdout expands over, so what the reader is
    /// measured on is a name it has never seen.
    #[serde(default)]
    holdout_names: Vec<String>,
    /// The sets `for each` walks, for a `{group}` slot.
    ///
    /// A closed list, unlike [`names`](Self::names), and held to the tower by
    /// `the_sets_the_corpus_teaches_are_the_sets_the_tower_raises`: a domain
    /// that declares a set the corpus never names is a set the reader can never
    /// offer, which is the gap `every_verb_has_a_template` closes for verbs.
    #[serde(default)]
    groups: Vec<String>,
}

/// Where a slot's value came from, so a generated example carries its spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// What kind of noun filled it.
    pub kind: NounKind,
    /// Where it sits in the generated line, in bytes.
    pub at: core::ops::Range<usize>,
}

/// One generated example: what a player might type, and what it means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    /// The phrasing, with every slot filled.
    pub said: String,
    /// The canonical command it stands for, with the same fillings.
    pub canonical: String,
    /// Where each filled slot landed in [`said`](Self::said).
    pub spans: Vec<Span>,
}

impl Phrasings {
    /// The templates compiled into the binary.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_builtin_file_parses`.
    #[must_use]
    pub fn builtin() -> Self {
        super::load::builtin(FILE, BUILTIN)
    }

    /// How people say the things a **spell** does.
    ///
    /// # The same format, and deliberately the same type
    ///
    /// A spell line and a prompt line are different output spaces — `if` is not
    /// a verb and `grind` is not a spell word — but the *shape* of the question
    /// is identical: a canonical form, the ways people write it, and a holdout
    /// nothing is taught. Two formats would mean two expansions, two span
    /// derivations and two holdout rules, which is the defect §19 records more
    /// often than any other.
    ///
    /// # Control flow only, and that is not an omission
    ///
    /// A spell's *command* lines are the prompt's vocabulary, so they are
    /// already written down in `phrasings.toml` and a reader for spells trains
    /// its command class from there. Copying two thousand say-lines into a
    /// second file would be the same corpus twice, free to drift.
    ///
    /// # Panics
    ///
    /// If the built-in file is malformed — a build-time authoring error, covered
    /// by `the_spellings_file_parses`.
    #[must_use]
    pub fn spellings() -> Self {
        super::load::builtin(SPELL_FILE, SPELLINGS)
    }

    /// Parse a phrasings file.
    ///
    /// # Errors
    ///
    /// [`ContentError`](super::ContentError) if the text is not valid TOML of
    /// the expected shape, or if an entry has a `canonical` with nothing to say
    /// for it — which parses perfectly and contributes nothing, so it is a typo
    /// rather than an intention.
    pub fn parse(text: &str) -> Result<Self, super::ContentError> {
        let parsed: Self = super::load::parse(FILE, text)?;
        for entry in &parsed.entries {
            if entry.say.is_empty() {
                return Err(super::ContentError::new(
                    FILE,
                    format!("`{}` has no phrasings to say it", entry.canonical),
                ));
            }
        }
        Ok(parsed)
    }

    /// The templates, in the order they were written.
    #[must_use]
    pub fn entries(&self) -> &[Phrasing] {
        &self.entries
    }

    /// The sentences that ask for nothing.
    #[must_use]
    pub fn refusals(&self) -> &[Refusal] {
        &self.refusals
    }

    /// The sets a spell can walk, as this file teaches them.
    #[must_use]
    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    /// Every refusal `say` template, expanded — what a reader is taught to
    /// refuse.
    #[must_use]
    pub fn refused(&self, scene: &crate::parser::Scene) -> Vec<String> {
        self.expand_refusals(scene, |refusal| &refusal.say, false)
    }

    /// Every refusal `holdout` template, expanded.
    ///
    /// **Scored the opposite way round from a command's holdout.** Here the
    /// number worth reporting is how many the reader *refuses*; there it is how
    /// many it does not.
    #[must_use]
    pub fn refused_holdout(&self, scene: &crate::parser::Scene) -> Vec<String> {
        self.expand_refusals(scene, |refusal| &refusal.holdout, true)
    }

    fn expand_refusals(
        &self,
        scene: &crate::parser::Scene,
        pick: impl Fn(&Refusal) -> &Vec<String>,
        held: bool,
    ) -> Vec<String> {
        let fillers = self.fillers(scene, held);
        let mut out = Vec::new();
        for refusal in &self.refusals {
            for template in pick(refusal) {
                // The canonical is unused for a refusal; `fill` wants one, so it
                // is given the template back and the result thrown away.
                out.extend(
                    fill(template, template, &fillers)
                        .into_iter()
                        .map(|example| example.said),
                );
            }
        }
        out
    }

    /// Every `say` template, expanded over `scene`.
    ///
    /// The corpus. See [`holdout`](Self::holdout) for what to measure on.
    #[must_use]
    pub fn corpus(&self, scene: &crate::parser::Scene) -> Vec<Example> {
        self.expand(scene, |entry| &entry.say, false)
    }

    /// Every `say` template, expanded over `scene`, **no template contributing
    /// more than `cap` examples**.
    ///
    /// # Why a cap at all
    ///
    /// A template's expansion is the *product* of its slot cardinalities, so a
    /// two-slot shape is quadratic where a one-slot shape is linear and a bare
    /// one is a single example. `move {reagent} {place}` was **47% of the whole
    /// corpus** on that arithmetic alone — not because moving things is 47% of
    /// what anyone says, but because it is the widest signature in `Verb::ALL`
    /// (§19).
    ///
    /// That is a bias the reader learns as a prior: `can you take me over to the
    /// lectern` came back `move lectern` rather than `attend lectern`, and the
    /// slot-to-verb conditioning built to fix it was answering the wrong
    /// question. Ten examples of a shape teach the shape; four hundred teach the
    /// shape and a prior.
    ///
    /// # Thinned rather than truncated
    ///
    /// Taking the first `cap` would take every expansion of the *first* noun and
    /// none of the rest, so the tagger would see one reagent in two hundred
    /// sentences. This strides the cross-product instead, and offsets the stride
    /// by the template's position so neighbouring templates do not all land on
    /// the same nouns. Deterministic either way — no RNG reaches a corpus.
    #[must_use]
    pub fn corpus_capped(&self, scene: &crate::parser::Scene, cap: usize) -> Vec<Example> {
        self.by_entry(scene, cap, |entry| &entry.say, false)
            .into_iter()
            .map(|(_, example)| example)
            .collect()
    }

    /// [`corpus_capped`](Self::corpus_capped), each example carrying **which
    /// entry it came from**.
    ///
    /// # The spell register's classes are entries, not words
    ///
    /// At the prompt a canonical command names its own class: the head of `grind
    /// sage` is `grind`, and `Verb::ALL` has a row for it. A spell statement does
    /// not. `if {place} is idle`, `if {place} is empty` and `if {place} has
    /// {reagent}` are all `SpellWord::If`, so a class over the twelve words could
    /// never tell an assembler which of the three shapes to build — and the first
    /// two carry the same number of slots, so nothing about the tagging
    /// separates them either.
    ///
    /// The **template** is the class, and the index here is its row. That makes
    /// the reading assemblable by construction: the shape is known, its slots are
    /// filled from the spans, and no shape has to be inferred back out of a
    /// string.
    #[must_use]
    pub fn corpus_by_entry(
        &self,
        scene: &crate::parser::Scene,
        cap: usize,
    ) -> Vec<(usize, Example)> {
        self.by_entry(scene, cap, |entry| &entry.say, false)
    }

    /// [`holdout`](Self::holdout), each example carrying which entry it came
    /// from. Uncapped, because a holdout is measured rather than learned from.
    #[must_use]
    pub fn holdout_by_entry(&self, scene: &crate::parser::Scene) -> Vec<(usize, Example)> {
        self.by_entry(scene, 0, |entry| &entry.holdout, true)
    }

    /// The shared walk: every template of every entry, expanded, thinned to
    /// `cap` (0 for no cap), tagged with the entry's index.
    fn by_entry(
        &self,
        scene: &crate::parser::Scene,
        cap: usize,
        pick: impl Fn(&Phrasing) -> &Vec<String>,
        held: bool,
    ) -> Vec<(usize, Example)> {
        let fillers = self.fillers(scene, held);
        let mut out = Vec::new();
        for (nth, (at, entry, template)) in self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(at, entry)| {
                pick(entry)
                    .iter()
                    .map(move |template| (at, entry, template))
            })
            .enumerate()
        {
            out.extend(
                thin(fill(template, &entry.canonical, &fillers), cap, nth)
                    .into_iter()
                    .map(|example| (at, example)),
            );
        }
        out
    }

    /// Every `holdout` template, expanded over `scene`.
    ///
    /// **Never taught to anything.** A grammar built from the corpus matches the
    /// corpus by construction, and a model trained on it does nearly as well;
    /// what either does on a phrasing it has never seen is the only number worth
    /// reporting.
    #[must_use]
    pub fn holdout(&self, scene: &crate::parser::Scene) -> Vec<Example> {
        self.expand(scene, |entry| &entry.holdout, true)
    }

    fn expand(
        &self,
        scene: &crate::parser::Scene,
        pick: impl Fn(&Phrasing) -> &Vec<String>,
        held: bool,
    ) -> Vec<Example> {
        let fillers = self.fillers(scene, held);
        let mut out = Vec::new();
        for entry in &self.entries {
            for template in pick(entry) {
                out.extend(fill(template, &entry.canonical, &fillers));
            }
        }
        out
    }

    /// Where this file's slots get their values: the scene for a noun, this
    /// file's own lists for free text.
    ///
    /// `held` picks the holdout's names where the file gives any, so a name the
    /// reader has never seen is what it is measured on.
    fn fillers<'a>(&'a self, scene: &'a crate::parser::Scene, held: bool) -> Fillers<'a> {
        let names = if held && !self.holdout_names.is_empty() {
            &self.holdout_names
        } else {
            &self.names
        };
        Fillers {
            scene,
            names,
            groups: &self.groups,
        }
    }
}

/// Everything the content tables name, as one scene to expand a corpus over.
///
/// # Why not the room the player is standing in
///
/// A corpus is not a session. `{reagent}` should become every reagent the game
/// *has a word for*, not the one that happens to be on the shelf — a reader
/// trained on `grind sage` and nothing else has learned the sage rather than the
/// grinding.
///
/// **The bench was measuring against a scene with one reagent in it**, so every
/// `{reagent}` template expanded once where the content names thirty-three, and
/// the corpus read a factor of thirty smaller than it is. That understated the
/// corpus and flattered nothing — it made the ratio of parameters to examples
/// look far worse than it is.
///
/// # A scene, not a world
///
/// Deliberately built from the *tables* rather than from a `Sim`: a world has a
/// place the player is standing and a shelf with things missing from it, and
/// neither is a fact about the language. Every fixture's verb is offered, so a
/// phrasing is never scored against the room it happened to be expanded in.
#[must_use]
pub fn corpus_scene() -> crate::parser::Scene {
    use crate::parser::{NounKind, Scene, Verb};

    let recipes = crate::content::Recipes::builtin();
    let mut scene = Scene::new();

    for substance in recipes.vocabulary() {
        scene = scene.with(NounKind::Reagent, substance);
    }
    for name in crate::content::Materials::builtin().names() {
        scene = scene.with(NounKind::Reagent, name);
    }
    // Instruments are places you stand at and name — `wield mortar_and_pestle`,
    // `empty alembic`, `stop lectern`.
    for instrument in recipes.instruments() {
        scene = scene.with(NounKind::Place, instrument);
    }
    for room in [
        "/tower/laboratory",
        "/tower/archive",
        "/tower/sanctum",
        "/tower/menagerie",
        "/tower/bailey",
        "/tower/forge",
        "/tower/lens",
    ] {
        scene = scene.with(NounKind::Place, room);
    }
    for file in ["feed.log", "purge_cycle.log", "ward.log"] {
        scene = scene.with(NounKind::File, file);
    }
    for topic in ["brewing", "scripting", "apprentice", "clarity", "warding"] {
        scene = scene.with(NounKind::Topic, topic);
    }
    for essence in ["clarity", "warding"] {
        scene = scene.with(NounKind::Essence, essence);
    }
    for script in ["night_watch", "first_light"] {
        scene = scene.with(NounKind::Script, script);
    }

    Verb::ALL
        .into_iter()
        .filter_map(Verb::anchor)
        .fold(scene, Scene::offering)
}

/// The slot a `{name}` marker names, if it names one.
///
/// **By label, so the file and the parser cannot disagree.** `NounKind::label`
/// is what the orb already prints when it asks for a slot, so a writer copying
/// the word out of a refusal has written a valid marker.
fn kind_of(name: &str) -> Option<NounKind> {
    NounKind::NAMEABLE
        .into_iter()
        .find(|kind| kind.label().eq_ignore_ascii_case(name))
}

/// Slots filled from a list the file gives rather than from the scene.
///
/// **Free text never touches the world**, which is why `NounKind::NAMEABLE`
/// leaves `Name` out: a name a spell binds is a word the tower has no noun for,
/// and a set is a fixture's *kind* rather than a fixture. So these two expand
/// over the phrasings file's own `names` and `groups`, and their spans carry
/// `NounKind::Name` — the kind `scribe <name>` already uses for one word of the
/// player's own.
const FREE_TEXT: [&str; 2] = ["name", "group"];

/// Whether `{label}` is a slot at all.
fn is_marker(label: &str) -> bool {
    kind_of(label).is_some()
        || FREE_TEXT
            .iter()
            .any(|free| free.eq_ignore_ascii_case(label))
}

/// The first `{slot}` in `template`, as its marker and its label.
fn first_slot(template: &str) -> Option<(String, String)> {
    let open = template.find('{')?;
    let close = template[open..].find('}')? + open;
    let label = &template[open + 1..close];
    is_marker(label).then(|| (template[open..=close].to_owned(), label.to_lowercase()))
}

/// Where every slot gets its values.
#[derive(Clone, Copy)]
struct Fillers<'a> {
    scene: &'a crate::parser::Scene,
    names: &'a [String],
    groups: &'a [String],
}

impl Fillers<'_> {
    /// The kind a slot's span carries, and every value it expands over.
    fn values(&self, label: &str) -> (NounKind, Vec<String>) {
        match label {
            "name" => (NounKind::Name, self.names.to_vec()),
            "group" => (NounKind::Name, self.groups.to_vec()),
            _ => {
                let Some(kind) = kind_of(label) else {
                    return (NounKind::Name, Vec::new());
                };
                // A place answers to its leaf (§7) — players say the room, not
                // the path.
                let values = self
                    .scene
                    .nouns()
                    .iter()
                    .filter(|noun| kind.accepts(noun.kind))
                    .map(|noun| crate::parser::leaf(&noun.name).to_owned())
                    .collect();
                (kind, values)
            }
        }
    }
}

/// Expand one template over every noun that could fill its slots.
///
/// One slot deep at a time, recursing, so a template with two slots yields
/// every pairing. Templates here have at most one, but `move {reagent} to
/// {place}` is a shape the file will want and this costs nothing to allow.
/// At most `cap` of `examples`, spread across the whole list.
///
/// `offset` shifts where the stride starts, so two templates of the same shape
/// do not both keep the same nouns. See [`Phrasings::corpus_capped`].
fn thin(examples: Vec<Example>, cap: usize, offset: usize) -> Vec<Example> {
    if cap == 0 || examples.len() <= cap {
        return examples;
    }
    let stride = examples.len().div_ceil(cap);
    examples
        .into_iter()
        .skip(offset % stride)
        .step_by(stride)
        .take(cap)
        .collect()
}

/// Expand one template, one slot at a time.
///
/// # The **canonical's** slot order, not the phrasing's
///
/// A span's index *is* the argument's position: `Sample`'s tagger writes
/// `Tag::Begin(n)` for `spans[n]`, and a reader reassembles a command by putting
/// slot `n` where the canonical's `n`th argument goes. So the two have to be
/// counted the same way round — and the phrasing is free to say them in any
/// order at all.
///
/// `if there is {reagent} in the {place}` means `if {place} has {reagent}`, and
/// recursing on the *phrasing's* first slot made the reagent span 0 and the place
/// span 1, exactly backwards. Half the templates of a two-slot shape are written
/// that way, so the tagger was taught both orders for one shape and the assembler
/// built `if sage has alembic`: `if {place} has {reagent}` scored **0.0%** on its
/// holdout while every one-slot shape read fine.
///
/// # Substituted left to right, then **renumbered**
///
/// The substitution itself has to walk the phrasing's slots in the order they
/// appear, because a span records a byte range into the sentence being built and
/// filling a later slot first would leave every earlier offset pointing at the
/// pre-substitution string. So the expansion is unchanged and the spans are
/// permuted once at the end, into the order the canonical names its arguments.
/// A slot the canonical does not carry sorts last, which is where a refusal's
/// slots and an argument the command form drops both belong.
fn fill(template: &str, canonical: &str, fillers: &Fillers<'_>) -> Vec<Example> {
    let order = ranked(template, canonical);
    let mut out = expand(template, canonical, fillers);
    for example in &mut out {
        renumber(&mut example.spans, &order);
    }
    out
}

/// For each of `template`'s slots in the order they are substituted, where the
/// canonical names that argument.
fn ranked(template: &str, canonical: &str) -> Vec<usize> {
    let wanted = markers(canonical);
    markers(template)
        .iter()
        .map(|marker| {
            wanted
                .iter()
                .position(|named| named == marker)
                .unwrap_or(usize::MAX)
        })
        .collect()
}

/// Every distinct `{slot}` in `text`, in the order they first appear.
///
/// Distinct, because substitution replaces every occurrence of a marker at once,
/// so a marker written twice is one slot.
fn markers(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}').map(|at| at + open) else {
            break;
        };
        let marker = rest[open..=close].to_owned();
        if is_marker(&rest[open + 1..close]) && !found.contains(&marker) {
            found.push(marker);
        }
        rest = &rest[close + 1..];
    }
    found
}

/// Put `spans` into the order `ranked` gives, stably.
fn renumber(spans: &mut Vec<Span>, order: &[usize]) {
    if spans.len() < 2 || order.len() < spans.len() {
        return;
    }
    let mut at: Vec<usize> = (0..spans.len()).collect();
    at.sort_by_key(|slot| order[*slot]);
    *spans = at.into_iter().map(|slot| spans[slot].clone()).collect();
}

/// One slot deep, recursing over every value that could fill it.
fn expand(template: &str, canonical: &str, fillers: &Fillers<'_>) -> Vec<Example> {
    let Some((marker, label)) = first_slot(template) else {
        // Fully ground. Spans are recovered by walking the canonical form's
        // arguments back through the said line, which is what the caller wants
        // and what a slot-free template has none of.
        return vec![Example {
            said: template.to_owned(),
            canonical: canonical.to_owned(),
            spans: Vec::new(),
        }];
    };

    let (kind, values) = fillers.values(&label);
    let mut out = Vec::new();
    for value in &values {
        let said = template.replace(&marker, value);
        let meant = canonical.replace(&marker, value);
        // **Where the marker was, and to the end of the word.** Where the
        // marker was, because every slot to its left is already filled, so its
        // offset in the template is its offset in the sentence — and searching
        // for the value instead finds `way` inside `always` first. To the end
        // of the word, because `{group}s` says `ways` for a canonical that says
        // `way`, and `Sample`'s tagger asks whether the *whole* word lies in a
        // span: one stopping short tags the word as nothing at all.
        let at = template.find(&marker).map(|start| {
            let end = start + value.len();
            let word = said[end..]
                .find(char::is_whitespace)
                .unwrap_or(said.len() - end);
            start..end + word
        });

        for mut deeper in expand(&said, &meant, fillers) {
            if let Some(at) = at.clone() {
                deeper.spans.insert(0, Span { kind, at });
            }
            out.push(deeper);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Scene, Verb};

    fn scene() -> Scene {
        Verb::ALL.into_iter().filter_map(Verb::anchor).fold(
            Scene::new()
                .with(NounKind::Place, "/tower/laboratory")
                .with(NounKind::Reagent, "sage")
                .with(NounKind::Reagent, "rock-salt"),
            Scene::offering,
        )
    }

    #[test]
    fn every_verb_has_a_template() {
        // **Seventeen of the forty-six had none** (§19), and nothing said so.
        // A verb with no examples cannot be predicted, and the refusal head
        // cannot cover for it either — it is trained on sentences that ask for
        // *nothing*, not on sentences that ask for something unteachable. So
        // `send the wolves to the gate` came back as a confident `move`.
        //
        // A domain that coins a verb and writes no phrasing for it reopens the
        // hole silently, which is what this is here to stop.
        let phrasings = Phrasings::builtin();
        let written: std::collections::HashSet<&str> = phrasings
            .entries()
            .iter()
            .filter_map(|entry| entry.canonical.split_whitespace().next())
            .collect();
        let missing: Vec<&str> = Verb::ALL
            .iter()
            .map(|verb| verb.canonical())
            .filter(|canonical| !written.contains(canonical))
            .collect();
        assert!(
            missing.is_empty(),
            "{} verbs have no phrasing and so cannot be read: {missing:?}",
            missing.len()
        );
    }

    #[test]
    fn a_cap_bounds_what_one_template_contributes() {
        let phrasings = Phrasings::parse(
            r#"
            [[entry]]
            canonical = "grind {reagent}"
            say = ["smash the {reagent}"]
            holdout = ["pound the {reagent}"]
            "#,
        )
        .expect("parses");

        let scene = scene();
        assert_eq!(phrasings.corpus_capped(&scene, 1).len(), 1);
        assert_eq!(phrasings.corpus_capped(&scene, 2).len(), 2);
        // A cap above the cross-product changes nothing.
        assert_eq!(
            phrasings.corpus_capped(&scene, 99).len(),
            phrasings.corpus(&scene).len()
        );
    }

    #[test]
    fn the_cap_thins_rather_than_truncating() {
        // **The distinction the cap lives or dies on.** Truncating would keep
        // every expansion of the first noun and none of the second, so the
        // tagger would meet one reagent over and over.
        let phrasings = Phrasings::parse(
            r#"
            [[entry]]
            canonical = "grind {reagent}"
            say = ["smash the {reagent}", "crush the {reagent}"]
            holdout = ["pound the {reagent}"]
            "#,
        )
        .expect("parses");

        let kept: Vec<String> = phrasings
            .corpus_capped(&scene(), 1)
            .into_iter()
            .map(|example| example.canonical)
            .collect();
        assert_eq!(kept.len(), 2, "one per template");
        assert_ne!(
            kept[0], kept[1],
            "both templates kept the same noun, so the stride is not striding"
        );
    }

    #[test]
    fn no_single_verb_owns_the_capped_corpus() {
        // **`move` was 47% of it** (§19), because it is the only three-slot
        // signature and its expansion is a product where everything else's is a
        // sum. A prior that strong is learned as one, and it cost `attend`.
        use std::collections::HashMap;
        let corpus = Phrasings::builtin().corpus_capped(&corpus_scene(), CORPUS_CAP);
        let mut count: HashMap<&str, usize> = HashMap::new();
        for example in &corpus {
            *count
                .entry(example.canonical.split_whitespace().next().unwrap_or(""))
                .or_default() += 1;
        }
        let (worst, share) = count
            .iter()
            .map(|(verb, n)| (*verb, n * 100 / corpus.len().max(1)))
            .max_by_key(|(_, share)| *share)
            .expect("a corpus");
        println!(
            "{} examples, widest share {worst} at {share}%",
            corpus.len()
        );
        assert!(
            share <= 15,
            "{worst} is {share}% of the corpus; the cap is not holding"
        );
    }

    #[test]
    fn the_spellings_file_parses() {
        let spellings = Phrasings::spellings();
        assert!(!spellings.entries().is_empty());
        // **Its refusals are the gate's, and they read cleanly.** The reason
        // this used to give — *"a reader shown only lines that should be
        // rewritten rewrites everything"* — was never true of them: the gate
        // keeps a line that reads cleanly from the scrivener, and
        // `Corpus::spells` drops what the reader is never shown. What they are
        // for is the measurement's claim that a working line comes back
        // untouched, so a refusal that did *not* read cleanly would be a line
        // the reader is shown and taught on by accident — this lists them.
        let scene = corpus_scene();
        let mut refused = spellings.refused(&scene);
        refused.extend(spellings.refused_holdout(&scene));
        assert!(
            !refused.is_empty(),
            "the gate has nothing to be measured on"
        );
        let shown: Vec<&String> = refused
            .iter()
            .filter(|line| !crate::tower::spell::reads_cleanly(line))
            .collect();
        assert!(
            shown.is_empty(),
            "these refusals reach the reader: {shown:?}"
        );
    }

    #[test]
    fn every_spelling_canonical_is_a_spell_statement() {
        // **The thing that makes this corpus different from the prompt's.** A
        // canonical here must be a line `program::read` recognises as a
        // *statement*, not a command — a `say` line that read back as
        // `Kind::Command` would train the reader to rewrite control flow into a
        // verb, which is the one output the spell language cannot take.
        for entry in Phrasings::spellings().entries() {
            let head = entry
                .canonical
                .split_whitespace()
                .next()
                .expect("a canonical says something");
            assert!(
                crate::parser::spell_word(&entry.canonical).is_some(),
                "{:?} opens on {head:?}, which is no spell word",
                entry.canonical,
            );
        }
    }

    #[test]
    fn a_spelling_never_says_what_it_means() {
        // A `say` line already in canonical form teaches the reader to rewrite
        // what needed no rewriting. The refusals hold the other half of this;
        // together they are what stops a reader that answers everything.
        let spellings = Phrasings::spellings();
        for entry in spellings.entries() {
            for line in entry.say.iter().chain(&entry.holdout) {
                assert_ne!(
                    line.trim(),
                    entry.canonical,
                    "a phrasing of {:?} is the canonical form",
                    entry.canonical,
                );
            }
        }
    }

    #[test]
    fn a_spelling_is_never_a_line_the_language_already_reads() {
        // **The other half of the same rule, and the harder half to see.** A
        // phrasing that is not the canonical form can still be one the spell
        // language parses on its own: `if the {place} is not busy` is heard as
        // `if not {place} is working`, which means what it says and needs no
        // reader at all.
        //
        // Teaching those is worse than wasteful. `Scribe` leaves alone anything
        // `reads_cleanly` accepts, so the model is being taught to rewrite lines
        // it will never be shown — and the measurement counts every one of them
        // as a miss, which is a bench reporting on work that was never asked
        // for. `Copyist::worked` makes the same argument about `crush the sage`.
        let spellings = Phrasings::spellings();
        let scene = corpus_scene();
        let taught = spellings.fillers(&scene, false);
        let held = spellings.fillers(&scene, true);
        let mut already: Vec<String> = Vec::new();
        for entry in spellings.entries() {
            for (templates, fillers) in [(&entry.say, &taught), (&entry.holdout, &held)] {
                for template in templates {
                    if fill(template, &entry.canonical, fillers)
                        .iter()
                        .any(|example| crate::tower::spell::reads_cleanly(&example.said))
                    {
                        already.push(template.clone());
                    }
                }
            }
        }
        assert!(
            already.is_empty(),
            "{} phrasings the language already reads:\n{}",
            already.len(),
            already.join("\n"),
        );
    }

    #[test]
    fn the_builtin_file_parses() {
        let phrasings = Phrasings::builtin();
        assert!(!phrasings.entries().is_empty());
    }

    #[test]
    fn every_entry_says_something_and_holds_something_back() {
        // A template with no holdout contributes to the corpus and to no
        // measurement, which is the one shape that looks like work and is not.
        for entry in Phrasings::builtin().entries() {
            assert!(!entry.say.is_empty(), "{} says nothing", entry.canonical);
            assert!(
                !entry.holdout.is_empty(),
                "{} holds nothing back, so nothing it teaches can be measured",
                entry.canonical
            );
        }
    }

    #[test]
    fn every_slot_names_a_real_kind() {
        // A typo in a marker expands to nothing and silently shrinks the corpus.
        for entry in Phrasings::builtin().entries() {
            for line in entry
                .say
                .iter()
                .chain(&entry.holdout)
                .chain([&entry.canonical])
            {
                let mut rest = line.as_str();
                while let Some(open) = rest.find('{') {
                    let close = rest[open..].find('}').expect("an unclosed slot marker");
                    let name = &rest[open + 1..open + close];
                    assert!(kind_of(name).is_some(), "{line:?} names no kind {name:?}");
                    rest = &rest[open + close..];
                }
            }
        }
    }

    #[test]
    fn a_template_expands_once_per_noun_of_its_kind() {
        let phrasings = Phrasings::parse(
            r#"
            [[entry]]
            canonical = "grind {reagent}"
            say = ["smash the {reagent}"]
            holdout = ["pound the {reagent}"]
            "#,
        )
        .expect("parses");

        let corpus = phrasings.corpus(&scene());
        assert_eq!(corpus.len(), 2, "two reagents, two examples");
        assert!(corpus.iter().any(|e| e.said == "smash the sage"));
        assert!(corpus.iter().any(|e| e.canonical == "grind rock-salt"));
    }

    #[test]
    fn generation_knows_where_it_substituted() {
        // **The reason the corpus is tractable.** Slot labels fall out of the
        // expansion, so nobody annotates 200k lines by hand.
        let phrasings = Phrasings::parse(
            r#"
            [[entry]]
            canonical = "grind {reagent}"
            say = ["smash the {reagent}"]
            holdout = ["pound the {reagent}"]
            "#,
        )
        .expect("parses");

        let sage = phrasings
            .corpus(&scene())
            .into_iter()
            .find(|e| e.said == "smash the sage")
            .expect("expanded");
        assert_eq!(sage.spans.len(), 1);
        assert_eq!(sage.spans[0].kind, NounKind::Reagent);
        assert_eq!(&sage.said[sage.spans[0].at.clone()], "sage");
    }

    #[test]
    fn a_free_text_slot_expands_over_the_files_own_words() {
        // **The fix for `let tool be {place}`.** A name used to be a literal in
        // the shape, so `tool` was the only name the reader could ever offer.
        // It is a slot now, and a slot needs values the world cannot supply.
        let phrasings = Phrasings::parse(
            r#"
            names = ["hammer", "best"]
            holdout_names = ["zorb"]
            groups = ["way", "band"]

            [[entry]]
            canonical = "let {name} be {place}"
            say = ["call the {place} {name}"]
            holdout = ["{name} is the {place}"]

            [[entry]]
            canonical = "for each {group}"
            say = ["work through the {group}s", "always the {group}"]
            "#,
        )
        .expect("parses");
        let taught = phrasings.corpus(&scene());

        let hammer = taught
            .iter()
            .find(|example| example.said == "call the laboratory hammer")
            .expect("a name from the file");
        assert_eq!(hammer.canonical, "let hammer be laboratory");
        assert_eq!(
            hammer
                .spans
                .iter()
                .map(|span| &hammer.said[span.at.clone()])
                .collect::<Vec<_>>(),
            vec!["hammer", "laboratory"],
            "the name is the canonical's first argument, wherever the phrasing puts it",
        );

        // **The holdout's names are its own**, so it measures a name the reader
        // has never seen rather than one it memorised.
        let held = phrasings.holdout(&scene());
        assert!(!held.is_empty());
        assert!(held.iter().all(|example| example.said.starts_with("zorb")));

        // A plural is one word, and the span covers all of it.
        let ways = taught
            .iter()
            .find(|example| example.said == "work through the ways")
            .expect("a set from the file");
        assert_eq!(ways.canonical, "for each way");
        assert_eq!(&ways.said[ways.spans[0].at.clone()], "ways");

        // ...and it is the marker's own position, not the first place the value
        // happens to occur: `way` is inside `always`.
        let always = taught
            .iter()
            .find(|example| example.said == "always the way")
            .expect("expanded");
        assert_eq!(&always.said[always.spans[0].at.clone()], "way");
    }

    #[test]
    fn a_free_text_slot_always_has_something_to_fill_it() {
        // A template saying `{name}` in a file with no `names` expands to
        // nothing, and contributes nothing without saying so — the same failure
        // as an entry with no `say`, one level down.
        for phrasings in [Phrasings::builtin(), Phrasings::spellings()] {
            let says = |marker: &str| {
                phrasings.entries.iter().any(|entry| {
                    core::iter::once(&entry.canonical)
                        .chain(&entry.say)
                        .chain(&entry.holdout)
                        .any(|line| line.contains(marker))
                }) || phrasings.refusals.iter().any(|refusal| {
                    refusal
                        .say
                        .iter()
                        .chain(&refusal.holdout)
                        .any(|line| line.contains(marker))
                })
            };
            assert!(!says("{name}") || !phrasings.names.is_empty());
            assert!(!says("{group}") || !phrasings.groups.is_empty());
        }
    }

    #[test]
    fn a_span_is_numbered_by_the_canonical_and_not_by_the_phrasing() {
        // **The defect that made `if {place} has {reagent}` read 0.0%.** A span's
        // index *is* the argument's position, and half the phrasings of a
        // two-slot shape name the second argument first — so recursing on the
        // phrasing's slots taught the tagger both orders for one shape and the
        // assembler built `if sage has alembic`.
        let phrasings = Phrasings::parse(
            r#"
            [[entry]]
            canonical = "if {place} has {reagent}"
            say = ["if there is {reagent} in the {place}"]
            "#,
        )
        .expect("parses");

        let said = phrasings
            .corpus(&scene())
            .into_iter()
            .find(|example| example.said == "if there is sage in the laboratory")
            .expect("expanded");
        assert_eq!(said.canonical, "if laboratory has sage");
        assert_eq!(
            said.spans
                .iter()
                .map(|span| &said.said[span.at.clone()])
                .collect::<Vec<_>>(),
            vec!["laboratory", "sage"],
            "the spans are in the phrasing's order, not the canonical's",
        );
    }

    #[test]
    fn the_holdout_is_kept_apart_from_the_corpus() {
        let phrasings = Phrasings::builtin();
        let corpus: Vec<String> = phrasings
            .corpus(&scene())
            .into_iter()
            .map(|e| e.said)
            .collect();
        for example in phrasings.holdout(&scene()) {
            assert!(
                !corpus.contains(&example.said),
                "{:?} is in both the corpus and the holdout",
                example.said
            );
        }
    }

    #[test]
    fn an_entry_that_says_nothing_is_refused() {
        let error = Phrasings::parse(
            r#"
            [[entry]]
            canonical = "grind {reagent}"
            "#,
        );
        assert!(error.is_err());
    }
}
