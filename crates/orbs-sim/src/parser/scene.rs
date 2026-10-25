//! What currently exists, and can therefore be named.
//!
//! §6, step 2: fuzzy match against vocabulary *and* entities that currently
//! exist. Passed in rather than read from the world, so the parser tests
//! against a handful of nouns.

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
    /// How many of the offered words the match actually used — what tells a
    /// command from a sentence with one at the front, which
    /// `Analysis::reads_outright` must.
    pub words: usize,
}

/// The nameable surface of the world at the moment of a parse.
///
/// A `Resource`, because what the player can refer to is what the tower
/// contains.
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

    /// Offer a per-instrument verb, because its instrument is here — §7's rule
    /// for verbs, which keeps the vocabulary bounded per room.
    #[must_use]
    pub fn offering(mut self, verb: Verb) -> Self {
        self.operations.push(verb);
        self
    }

    /// Teach the scene every substance the laboratory has a word for, whether
    /// or not any is here.
    ///
    /// A known word only ever matches exactly, or `digest ground-sage` moves
    /// the salt. Typos still fuzz and abbreviations still prefix; what stops is
    /// one real name read as another (§19).
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
    /// By [`Verb::anchor`]. Always true for a verb with no fixture, or the key
    /// to a domain would be locked inside it.
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
        // Ties resolve to whichever noun was registered first, never to
        // iteration luck; `candidates` preserves that by sorting stably.
        self.candidates(kind, words).into_iter().next()
    }

    /// Everything `words` could be naming, best first, one entry per noun.
    ///
    /// A spell resolves with nobody watching, so `spell::compile` asks whether
    /// a runner-up was as close — `ground` prefixes both `ground-sage` and
    /// `ground-salt`.
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

        // Once per phrase, outside the noun loop: inside, it was
        // O(nouns × known) every command.
        let known: Vec<bool> = phrases.iter().map(|phrase| self.is_known(phrase)).collect();

        let mut found: Vec<NounMatch> = Vec::new();
        for noun in &self.nouns {
            if !kind.accepts(noun.kind) {
                continue;
            }
            // First wins a tie, so the joined phrase outranks one word of it.
            let mut best: Option<(u32, usize)> = None;
            for (index, (phrase, known)) in phrases.iter().zip(&known).enumerate() {
                let score = score_against(noun, phrase);
                // A known word only ever matches itself (see `knowing`) —
                // unless it is abbreviating, or `invoke check` misses
                // `check.spell`.
                let score = if score < fuzzy::EXACT && *known && !abbreviates(noun, phrase) {
                    0
                } else {
                    score
                };
                if score >= MIN_SIMILARITY && best.is_none_or(|(found, _)| score > found) {
                    best = Some((score, index));
                }
            }
            let Some((score, index)) = best else {
                continue;
            };
            found.push(NounMatch {
                name: noun.name.clone(),
                kind: noun.kind,
                score,
                // `phrases[0]` is every word joined; the rest are the words one
                // at a time, so anything but the head explains exactly one.
                words: if index == 0 { words.len() } else { 1 },
            });
        }
        // Stable, so equal scores keep registration order — the tie-break
        // `best_match` has always had, and the fact `compile` detects a tie by.
        found.sort_by_key(|found| std::cmp::Reverse(found.score));
        found
    }
}

/// Whether `phrase` is the start of what this noun is called — the line
/// between abbreviating and mistyping: `check` starts `check.spell` and `walk`
/// does not start `wall`. A [`NounKind::Place`] answers to its leaf as well.
fn abbreviates(noun: &Noun, phrase: &str) -> bool {
    let phrase = phrase.to_lowercase();
    let name = noun.name.to_lowercase();

    // Except §8.1's lie about this very word, which extends the truth by a
    // struck sigil; asked of `tower::sabotage`, so the shape has one copy.
    if name == crate::tower::claimed(&phrase) {
        return false;
    }

    if name.starts_with(&phrase) {
        return true;
    }
    noun.kind == NounKind::Place
        && name
            .rsplit('/')
            .next()
            .is_some_and(|leaf| leaf.starts_with(&phrase))
}

/// How well one phrase names one noun. A [`NounKind::Place`] also answers to
/// its last path segment, since players say the place, not the path (§7).
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
            .with(NounKind::Place, "/tower/sanctum")
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
        // Two characters apart in eleven scores 819, past the 600 threshold: a
        // silent wrong action, which §6 ranks below a refusal.
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
    fn a_known_name_does_not_abbreviate_the_lie_told_about_it() {
        // §8.1's lie extends the truth by a struck sigil, so it reads as an
        // abbreviation and the sabotage amounts to nothing. `tests/tampering.rs`
        // finds it on one seed in four; this finds it every time.
        let lied = Scene::new()
            .with(NounKind::Reagent, &crate::tower::claimed("sage"))
            .knowing(["sage".to_owned()]);

        assert!(
            lied.candidates(NounKind::Any, &["sage"]).is_empty(),
            "a swapped pile answered to the name it had stopped being called",
        );
        assert!(
            !lied.candidates(NounKind::Any, &["sage-"]).is_empty(),
            "the pile stopped answering to the name it now has, which is not \
             sabotage but deletion",
        );
    }

    #[test]
    fn a_known_name_still_matches_itself_when_it_is_here() {
        // The rule tightens fuzzy matching, not exact matching.
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
            .best_match(NounKind::Place, &["/tower/sanctum"])
            .expect("the sanctum exists");
        assert_eq!(found.name, "/tower/sanctum");
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
