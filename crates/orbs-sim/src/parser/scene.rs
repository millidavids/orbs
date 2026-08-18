//! What currently exists, and can therefore be named.
//!
//! DESIGN.md §6, step 2: fuzzy match against known vocabulary **and entities
//! that currently exist**. That second half is what separates this from a
//! command parser — `decoct clarity` resolves because clarity is a researched
//! essence, and stops resolving the moment it is not.
//!
//! The scene is passed in rather than read from the ECS world directly, so the
//! parser stays testable against a handful of nouns instead of requiring a fully
//! built tower. The Phase 0 domains will populate it from real entities.

use bevy_ecs::prelude::*;

use super::fuzzy::{self, MIN_SIMILARITY};
use super::verb::{NounKind, Verb};

/// Something the player can refer to by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noun {
    /// Canonical name — `/tower/laboratory`, `feed.log`, `clarity`.
    pub name: String,
    /// What category it belongs to.
    pub kind: NounKind,
}

/// How well a phrase named something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounMatch {
    /// The canonical name that was matched.
    pub name: String,
    /// Its category.
    pub kind: NounKind,
    /// Similarity, on [`fuzzy::EXACT`]'s scale.
    pub score: u32,
}

/// The nameable surface of the world at the moment of a parse.
///
/// A `Resource` because it *is* world state: what the player can refer to is
/// what the tower currently contains, which is why resolution is grounded in the
/// world rather than in a fixed command table. It is never a `Component` — a
/// type cannot derive both as of Bevy 0.19.
#[derive(Resource, Debug, Default, Clone)]
pub struct Scene {
    nouns: Vec<Noun>,
    operations: Vec<Verb>,
    known: Vec<String>,
}

impl Scene {
    /// An empty scene.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a nameable entity.
    #[must_use]
    pub fn with(mut self, kind: NounKind, name: &str) -> Self {
        self.nouns.push(Noun {
            name: name.to_owned(),
            kind,
        });
        self
    }

    /// Offer a per-instrument verb, because its instrument is here.
    ///
    /// **§7's rule, applied to verbs.** *"You can only name what is where you
    /// are"* has always governed nouns; an instrument's own verb is the same
    /// claim about the same thing said the other way round. `mix` means the
    /// flask and rod, and there is no flask and rod in the archive — so the word
    /// should not resolve there any more than `flask_and_rod` itself does.
    ///
    /// This is what keeps the vocabulary from growing without bound as §10's
    /// five further domains land. Each coins the verbs its own tools need, and
    /// none of them costs the others a possible misreading: the parser never
    /// considers a warding verb while you are brewing.
    #[must_use]
    pub fn offering(mut self, verb: Verb) -> Self {
        self.operations.push(verb);
        self
    }

    /// Teach the scene every substance the laboratory has a **word** for,
    /// whether or not any is here.
    ///
    /// # What this buys, and why fuzzy matching needed a brake
    ///
    /// Fuzzy matching exists for *typos*, and a typo is by definition not a
    /// word. `ground-sage` and `ground-salt` differ by two characters in eleven,
    /// which scores 819 against a 600 threshold — so with no ground-sage on the
    /// shelf, `digest ground-sage` resolved to `digest ground-salt`, echoed it,
    /// and moved the salt into the bath, which could then do nothing with it.
    /// One candidate scored, so it won outright and ran at `Confidence::Clear`:
    /// a silent wrong action, which §6 ranks below a refusal.
    ///
    /// The rule this installs: **a phrase that is itself a known substance only
    /// ever matches exactly.** Typos still fuzz — `ground-slat` is not a word,
    /// so it still reaches `ground-salt` — and abbreviations still prefix, since
    /// `ground-sa` is not a word either and remains an honest tie between the
    /// two. What stops is one real name being read as a different real name.
    ///
    /// It is the live-parser half of a rule `spell::compile` already applied:
    /// `fix` falls back to the recipe vocabulary precisely so a spell can tell
    /// *"there is none here"* from *"you have mistyped something"*. The prompt
    /// could not, and §19 records that the two halves disagreeing is how the
    /// `has ground-slat` defect survived in the first place.
    #[must_use]
    pub fn knowing(mut self, names: impl IntoIterator<Item = String>) -> Self {
        self.known.extend(names);
        self
    }

    /// Whether `phrase` is a substance the laboratory has a word for.
    fn is_known(&self, phrase: &str) -> bool {
        self.known
            .iter()
            .any(|word| word.eq_ignore_ascii_case(phrase))
    }

    /// Whether a verb's fixture stands here.
    ///
    /// Always true for a verb with no fixture — the core vocabulary goes
    /// everywhere, because `attend`, `survey` and `peruse` are how you *reach* a
    /// domain and gating them would lock the key inside the door.
    ///
    /// **It asks [`Verb::anchor`], and it used to ask `Verb::is_operation`.** That
    /// is the production-slot question, so every verb that took no slot was offered
    /// in every room — `help` in the laboratory listed `research`, `follow` and
    /// `wander`, none of which can do anything there. §19 records the two-word
    /// version of that as a debt waiting on a mechanism; `anchor` is the mechanism.
    #[must_use]
    pub fn offers(&self, verb: Verb) -> bool {
        verb.anchor()
            .is_none_or(|anchor| self.operations.contains(&anchor))
    }

    /// Everything in the scene.
    #[must_use]
    pub fn nouns(&self) -> &[Noun] {
        &self.nouns
    }

    /// The best thing `words` could be naming, within `kind`.
    ///
    /// [`NounKind::Any`] searches every category, which is what makes `verify`
    /// and `purge` work across all four sabotage surfaces (§8.1).
    ///
    /// Both the whole phrase and each individual word are tried, so
    /// `castle gates` reaches `gates` when nothing is called "castle gates".
    #[must_use]
    pub fn best_match(&self, kind: NounKind, words: &[&str]) -> Option<NounMatch> {
        // Ties resolve to whichever noun was registered first, so the result
        // never depends on iteration luck — which `candidates` preserves by
        // sorting stably.
        self.candidates(kind, words).into_iter().next()
    }

    /// Everything `words` could be naming, best first.
    ///
    /// # Why the runner-up is worth keeping
    ///
    /// [`best_match`](Self::best_match) answers the prompt's question — *what
    /// did they most likely mean* — and the player is standing there to see the
    /// echo if it guessed wrong. A **spell** resolves its names with nobody
    /// watching, so it needs the question this answers instead: *was there
    /// anything else nearly as close?* With both products on the shelf, `ground`
    /// is an equally good prefix of `ground-sage` and `ground-salt`, and picking
    /// the first-registered one would be a coin flip deciding what a laboratory
    /// does. See `spell::compile`.
    ///
    /// One entry per noun, at its best-scoring reading — the joined phrase and
    /// each individual word are all tried, so `castle gates` reaches `gates`
    /// when nothing is called "castle gates".
    #[must_use]
    pub fn candidates(&self, kind: NounKind, words: &[&str]) -> Vec<NounMatch> {
        if words.is_empty() {
            return Vec::new();
        }

        let joined = words.join(" ");
        let mut phrases: Vec<&str> = vec![joined.as_str()];
        if words.len() > 1 {
            phrases.extend_from_slice(words);
        }

        let mut found: Vec<NounMatch> = Vec::new();
        for noun in &self.nouns {
            if !kind.accepts(noun.kind) {
                continue;
            }
            let Some(score) = phrases
                .iter()
                .map(|phrase| {
                    let score = score_against(noun, phrase);
                    // A word only ever matches itself. See `knowing`: fuzzing a
                    // real substance into a *different* real substance is how
                    // `digest ground-sage` came to digest ground-salt.
                    if score < fuzzy::EXACT && self.is_known(phrase) {
                        0
                    } else {
                        score
                    }
                })
                .filter(|score| *score >= MIN_SIMILARITY)
                .max()
            else {
                continue;
            };
            found.push(NounMatch {
                name: noun.name.clone(),
                kind: noun.kind,
                score,
            });
        }
        // **Stable, so equal scores keep registration order** — the tie-break
        // `best_match` has always had, and the fact `compile` detects a tie by.
        found.sort_by_key(|found| std::cmp::Reverse(found.score));
        found
    }
}

/// How well one phrase names one noun.
///
/// A [`NounKind::Place`] also answers to its last path segment, so `attend
/// laboratory` reaches `/tower/laboratory` — paths are places (§7), and players say
/// the place, not the path.
fn score_against(noun: &Noun, phrase: &str) -> u32 {
    let direct = fuzzy::similarity(phrase, &noun.name);
    if noun.kind != NounKind::Place {
        return direct;
    }

    let leaf = noun.name.rsplit('/').next().unwrap_or(&noun.name);
    direct.max(fuzzy::similarity(phrase, leaf))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tower() -> Scene {
        Scene::new()
            .with(NounKind::Place, "/tower/laboratory")
            .with(NounKind::Place, "/tower/battlements")
            .with(NounKind::Place, "/tower/archive")
            .with(NounKind::File, "feed.log")
            .with(NounKind::File, "purge_cycle.log")
            .with(NounKind::Essence, "clarity")
            .with(NounKind::Essence, "warding")
            .with(NounKind::Script, "night_watch")
    }

    /// A shelf holding only the salt, in a laboratory that has a word for both.
    fn shelf() -> Scene {
        Scene::new()
            .with(NounKind::Reagent, "ground-salt")
            .knowing(["ground-sage".to_owned(), "ground-salt".to_owned()])
    }

    #[test]
    fn a_real_name_is_never_read_as_a_different_real_name() {
        // The defect this whole rule exists for. `ground-sage` and
        // `ground-salt` are two characters apart in eleven, which scores 819
        // against a threshold of 600 — so with no ground-sage on the shelf,
        // `digest ground-sage` resolved to `digest ground-salt`, echoed it, and
        // moved the salt into the bath. One candidate scored, so it won outright
        // and ran at full confidence: a silent wrong action, which §6 ranks
        // below a refusal.
        assert!(
            fuzzy::is_near("ground-sage", "ground-salt"),
            "the premise is gone: these are no longer close enough to collide",
        );
        assert!(
            shelf()
                .candidates(NounKind::Any, &["ground-sage"])
                .is_empty(),
            "a known name reached a different known name",
        );
    }

    #[test]
    fn a_typo_still_reaches_the_thing_it_misspells() {
        // The other half, and the reason the rule is *"a word only matches
        // itself"* rather than *"turn fuzzy matching off"*. `ground-slat` is not
        // a word, so it is a typo and still resolves.
        let found = shelf().best_match(NounKind::Any, &["ground-slat"]);
        assert_eq!(
            found.map(|found| found.name).as_deref(),
            Some("ground-salt")
        );
    }

    #[test]
    fn an_abbreviation_still_reaches_what_it_abbreviates() {
        // `ground-sa` is not a word either, so prefix matching is untouched —
        // which matters because it is how a player types at all.
        let found = shelf().best_match(NounKind::Any, &["ground-sa"]);
        assert_eq!(
            found.map(|found| found.name).as_deref(),
            Some("ground-salt")
        );
    }

    #[test]
    fn a_known_name_still_matches_itself_when_it_is_here() {
        // The rule tightens fuzzy matching, not exact matching. Stated because
        // "only ever matches exactly" would be a plausible way to break this.
        let found = shelf()
            .with(NounKind::Reagent, "ground-sage")
            .best_match(NounKind::Any, &["ground-sage"]);
        assert_eq!(
            found.map(|found| found.name).as_deref(),
            Some("ground-sage")
        );
    }

    #[test]
    fn an_exact_name_matches() {
        let found = tower()
            .best_match(NounKind::Essence, &["clarity"])
            .expect("clarity exists");
        assert_eq!(found.name, "clarity");
        assert_eq!(found.score, fuzzy::EXACT);
    }

    #[test]
    fn a_place_answers_to_its_last_segment() {
        // §7: paths are places, and players say the place.
        let found = tower()
            .best_match(NounKind::Place, &["laboratory"])
            .expect("laboratory exists");
        assert_eq!(found.name, "/tower/laboratory");
    }

    #[test]
    fn a_full_path_still_matches() {
        let found = tower()
            .best_match(NounKind::Place, &["/tower/battlements"])
            .expect("battlements exists");
        assert_eq!(found.name, "/tower/battlements");
    }

    #[test]
    fn kinds_are_not_crossed() {
        // "clarity" is an essence, not a place. Resolving it as one would let
        // `attend clarity` succeed nonsensically.
        assert!(tower().best_match(NounKind::Place, &["clarity"]).is_none());
        assert!(
            tower()
                .best_match(NounKind::Essence, &["laboratory"])
                .is_none()
        );
    }

    #[test]
    fn any_searches_every_category() {
        // `verify` and `purge` take Any, which is what lets them reach all four
        // sabotage surfaces (§8.1).
        let scene = tower();
        assert!(scene.best_match(NounKind::Any, &["clarity"]).is_some());
        assert!(scene.best_match(NounKind::Any, &["feed.log"]).is_some());
        assert!(scene.best_match(NounKind::Any, &["night_watch"]).is_some());
    }

    #[test]
    fn a_typo_still_finds_the_noun() {
        let found = tower()
            .best_match(NounKind::Essence, &["clarty"])
            .expect("near enough to clarity");
        assert_eq!(found.name, "clarity");
        assert!(found.score < fuzzy::EXACT, "a typo should not score exact");
    }

    #[test]
    fn a_multi_word_phrase_falls_back_to_its_parts() {
        let found = tower()
            .best_match(NounKind::Essence, &["potion", "clarity"])
            .expect("clarity is in there");
        assert_eq!(found.name, "clarity");
    }

    #[test]
    fn nothing_that_does_not_exist_resolves() {
        // The scene is the live world: an essence not yet researched must not
        // resolve, or the parser would promise something the sim cannot do.
        assert!(tower().best_match(NounKind::Essence, &["haste"]).is_none());
        assert!(tower().best_match(NounKind::Any, &["gibberish"]).is_none());
    }

    #[test]
    fn an_empty_phrase_matches_nothing() {
        assert!(tower().best_match(NounKind::Any, &[]).is_none());
    }
}
