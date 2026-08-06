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

    /// Whether a per-instrument verb has its instrument here.
    ///
    /// Always true for a verb that is not one — the core vocabulary goes
    /// everywhere, because `attend`, `survey` and `peruse` are how you *reach* a
    /// domain and gating them would lock the key inside the door.
    #[must_use]
    pub fn offers(&self, verb: Verb) -> bool {
        !verb.is_operation() || self.operations.contains(&verb)
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
        if words.is_empty() {
            return None;
        }

        let joined = words.join(" ");
        let mut candidates: Vec<&str> = vec![joined.as_str()];
        if words.len() > 1 {
            candidates.extend_from_slice(words);
        }

        let mut best: Option<NounMatch> = None;
        for noun in &self.nouns {
            if !kind.accepts(noun.kind) {
                continue;
            }
            for candidate in &candidates {
                let score = score_against(noun, candidate);
                if score < MIN_SIMILARITY {
                    continue;
                }
                // Ties resolve to whichever noun was registered first, so the
                // result never depends on iteration luck.
                if best.as_ref().is_none_or(|found| score > found.score) {
                    best = Some(NounMatch {
                        name: noun.name.clone(),
                        kind: noun.kind,
                        score,
                    });
                }
            }
        }
        best
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
