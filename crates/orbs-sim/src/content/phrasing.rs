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

    /// Every refusal `say` template, expanded — what a reader is taught to
    /// refuse.
    #[must_use]
    pub fn refused(&self, scene: &crate::parser::Scene) -> Vec<String> {
        self.expand_refusals(scene, |refusal| &refusal.say)
    }

    /// Every refusal `holdout` template, expanded.
    ///
    /// **Scored the opposite way round from a command's holdout.** Here the
    /// number worth reporting is how many the reader *refuses*; there it is how
    /// many it does not.
    #[must_use]
    pub fn refused_holdout(&self, scene: &crate::parser::Scene) -> Vec<String> {
        self.expand_refusals(scene, |refusal| &refusal.holdout)
    }

    fn expand_refusals(
        &self,
        scene: &crate::parser::Scene,
        pick: impl Fn(&Refusal) -> &Vec<String>,
    ) -> Vec<String> {
        let mut out = Vec::new();
        for refusal in &self.refusals {
            for template in pick(refusal) {
                // The canonical is unused for a refusal; `fill` wants one, so it
                // is given the template back and the result thrown away.
                out.extend(
                    fill(template, template, scene)
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
        self.expand(scene, |entry| &entry.say)
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
        let mut out = Vec::new();
        for (nth, (entry, template)) in self
            .entries
            .iter()
            .flat_map(|entry| entry.say.iter().map(move |template| (entry, template)))
            .enumerate()
        {
            out.extend(thin(fill(template, &entry.canonical, scene), cap, nth));
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
        self.expand(scene, |entry| &entry.holdout)
    }

    fn expand(
        &self,
        scene: &crate::parser::Scene,
        pick: impl Fn(&Phrasing) -> &Vec<String>,
    ) -> Vec<Example> {
        let mut out = Vec::new();
        for entry in &self.entries {
            for template in pick(entry) {
                out.extend(fill(template, &entry.canonical, scene));
            }
        }
        out
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

/// The first `{slot}` in `template`, as its marker and its kind.
fn first_slot(template: &str) -> Option<(String, NounKind)> {
    let open = template.find('{')?;
    let close = template[open..].find('}')? + open;
    let name = &template[open + 1..close];
    Some((template[open..=close].to_owned(), kind_of(name)?))
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

fn fill(template: &str, canonical: &str, scene: &crate::parser::Scene) -> Vec<Example> {
    let Some((marker, kind)) = first_slot(template) else {
        // Fully ground. Spans are recovered by walking the canonical form's
        // arguments back through the said line, which is what the caller wants
        // and what a slot-free template has none of.
        return vec![Example {
            said: template.to_owned(),
            canonical: canonical.to_owned(),
            spans: Vec::new(),
        }];
    };

    let mut out = Vec::new();
    for noun in scene.nouns() {
        if !kind.accepts(noun.kind) {
            continue;
        }
        // A place answers to its leaf (§7) — players say the room, not the path.
        let value = crate::parser::leaf(&noun.name);
        let said = template.replace(&marker, value);
        let meant = canonical.replace(&marker, value);
        let at = said.find(value).map(|start| start..start + value.len());

        for mut deeper in fill(&said, &meant, scene) {
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
