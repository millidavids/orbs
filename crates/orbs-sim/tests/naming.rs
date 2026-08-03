//! The naming rules, enforced.
//!
//! DESIGN.md §15: *"The canonical command set is player-facing API and requires
//! a dedicated in-world naming pass before Phase 1 freezes vocabulary."*
//!
//! The Phase 0 pass ran against the sixteen slice commands and found three
//! defects that only measurement shows: a canonical collision between the two
//! core brewing verbs, a `dec-` prefix shared three ways, and four names over
//! the length rule. Phase 1 adds ~35 more commands, which is exactly when a
//! vocabulary drifts back into collision — so the pass is a test, not an event.

use std::collections::BTreeSet;

use orbs_sim::parser::{
    MIN_SIMILARITY, Mode, NounKind, Register, SYNONYMS, Scene, Verb, resolve, similarity,
};

/// Single-word synonyms, with the verb that owns them.
fn single_words() -> Vec<(&'static str, Verb)> {
    SYNONYMS
        .iter()
        .filter(|entry| entry.words.len() == 1)
        .map(|entry| (entry.words[0], entry.verb))
        .collect()
}

#[test]
fn canonical_names_are_one_short_word() {
    for verb in Verb::ALL {
        let name = verb.canonical();
        assert!(
            name.len() <= Verb::MAX_CANONICAL_LEN,
            "{name} is {} characters; the canonical form is what experts type all day",
            name.len()
        );
        assert!(!name.contains(' '), "{name} is not one word");
        assert!(
            name.chars().all(|c| c.is_ascii_lowercase()),
            "{name} is not plain lowercase ASCII"
        );
    }
}

/// The defect the Phase 0 pass existed to find.
///
/// `decoct` and `decant` sat two edits apart (667) while being the two core
/// verbs of a Phase 0 domain, so a near-typo in a siege — where §6 forbids a
/// blocking prompt and the parser must take its best guess — could brew when
/// the player meant to collect. `decant` became `siphon`.
#[test]
fn no_two_canonical_names_fuzzy_match_each_other() {
    for a in Verb::ALL {
        for b in Verb::ALL {
            if a >= b {
                continue;
            }
            let score = similarity(a.canonical(), b.canonical());
            assert!(
                score < MIN_SIMILARITY,
                "{} and {} collide at {score}: a typo in a siege resolves to a coin flip",
                a.canonical(),
                b.canonical()
            );
        }
    }
}

/// Abbreviations must name one verb.
///
/// `dec` used to prefix `decoct`, `decant`, and `decipher`, so the natural
/// shorthand for the brewing domain meant three different things.
#[test]
fn three_character_prefixes_name_at_most_one_verb() {
    for verb in Verb::ALL {
        let name = verb.canonical();
        if name.len() < 3 {
            continue;
        }
        let prefix = &name[..3];
        let hits: Vec<&str> = Verb::ALL
            .iter()
            .map(|verb| verb.canonical())
            .filter(|other| other.starts_with(prefix))
            .collect();
        assert_eq!(
            hits.len(),
            1,
            "the abbreviation {prefix:?} reaches {hits:?}"
        );
    }
}

/// Synonyms may collide only where both spellings are claimed.
///
/// An exact match always outscores a near one, so a collision between two words
/// that each *own* a verb is a prompt on a typo — acceptable. A collision with a
/// word nothing claims is worse than a collision: `decant` unclaimed would have
/// resolved to `decoct`, silently brewing when the player meant to collect.
#[test]
fn every_colliding_synonym_is_claimed_by_a_verb() {
    let claimed: BTreeSet<&str> = single_words().into_iter().map(|(word, _)| word).collect();
    let mut collisions = Vec::new();

    for (word, verb) in single_words() {
        for (other, other_verb) in single_words() {
            if verb >= other_verb {
                continue;
            }
            if similarity(word, other) >= MIN_SIMILARITY {
                collisions.push((word, other));
                assert!(
                    claimed.contains(word) && claimed.contains(other),
                    "{word} and {other} collide but one is unclaimed"
                );
            }
        }
    }

    // Pinned, not merely bounded: a new entry that adds a collision should have
    // to say so here rather than slipping in under a threshold.
    assert_eq!(
        collisions,
        [("cat", "cast"), ("audit", "edit"), ("decoct", "decant")],
        "the set of tolerated synonym collisions changed"
    );
}

#[test]
fn every_verb_is_reachable_from_plain_english() {
    // Half the Phase 0 gate's testers self-report no shell experience.
    for verb in Verb::ALL {
        assert!(
            SYNONYMS
                .iter()
                .any(|entry| entry.verb == verb && entry.register == Register::Plain),
            "{} has no plain-English synonym",
            verb.canonical()
        );
    }
}

#[test]
fn the_words_the_naming_pass_replaced_still_resolve() {
    // Renaming a player-facing command must not strand the old word — and for
    // `decant` it must not *release* it either, since an unclaimed `decant`
    // lands on `decoct`.
    let scene = Scene::new()
        .with(NounKind::Vessel, "alembic")
        .with(NounKind::Essence, "clarity")
        .with(NounKind::Fragment, "sigil-iv")
        .with(NounKind::Script, "night_watch");

    for (input, expected) in [
        ("decant alembic", "siphon alembic"),
        ("siphon alembic", "siphon alembic"),
        ("decipher sigil-iv", "divine sigil-iv"),
        ("divine sigil-iv", "divine sigil-iv"),
        ("inscribe night_watch", "scribe night_watch"),
        ("scribe night_watch", "scribe night_watch"),
        ("decoct clarity", "decoct clarity"),
    ] {
        let echo = resolve(input, &scene, Mode::Calm)
            .intent()
            .map(|intent| intent.echo())
            .unwrap_or_else(|| "<unresolved>".to_owned());
        assert_eq!(echo, expected, "{input:?}");
    }
}
