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

use orbs_sim::parser::{
    MIN_SIMILARITY, Mode, NounKind, Register, SYNONYMS, Scene, Verb, resolve, similarity,
};

/// One noun of every kind, so any verb can be given a fitting argument.
fn scene() -> Scene {
    Scene::new()
        .with(NounKind::Place, "/tower/alembic")
        .with(NounKind::File, "feed.log")
        .with(NounKind::Topic, "brewing")
        .with(NounKind::Essence, "clarity")
        .with(NounKind::Vessel, "alembic")
        .with(NounKind::Fragment, "sigil-iv")
        .with(NounKind::Script, "night_watch")
        .with(NounKind::Any, "sludge")
}

/// An argument that satisfies `verb`'s signature.
fn sample_argument(verb: Verb) -> &'static str {
    let Some(slot) = verb.signature().first() else {
        return "";
    };
    match slot.kind {
        NounKind::Place => "/tower/alembic",
        NounKind::File => "feed.log",
        NounKind::Pattern => "march feed.log",
        NounKind::Topic => "brewing",
        NounKind::Essence => "clarity",
        NounKind::Vessel => "alembic",
        NounKind::Fragment => "sigil-iv",
        NounKind::Script => "night_watch",
        NounKind::Count => "30",
        NounKind::Any => "sludge",
    }
}

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

/// Canonical abbreviations must name one verb.
///
/// `dec` used to prefix three *canonical* names — `decoct`, `decant`,
/// `decipher` — so the natural shorthand for an expert meant three different
/// things. Synonyms are a separate question, pinned by
/// `ambiguous_synonym_prefixes_are_known`.
#[test]
fn three_character_canonical_prefixes_name_at_most_one_verb() {
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

/// **The invariant the whole pass rests on.**
///
/// Every phrase the vocabulary claims must, when typed with an argument that
/// fits, reach the verb that claims it. A word outranked by some *other* verb's
/// word is not a near miss — it is a wrong command with `Clear` confidence.
///
/// This replaces a check that compared the synonym list against a set built from
/// the same synonym list, and was therefore always true. It passed while `find`
/// resolved to `bind`, `take` and `decode` resolved to `decoct`, and `write`
/// resolved to `meditate`.
#[test]
fn every_phrase_reaches_the_verb_that_claims_it() {
    let scene = scene();
    let mut wrong = Vec::new();

    for entry in SYNONYMS {
        let phrase = entry.words.join(" ");
        let argument = sample_argument(entry.verb);
        let input = format!("{phrase} {argument}");

        let resolution = resolve(input.trim(), &scene, Mode::Siege);
        let reached = resolution.intent().map(|intent| intent.verb);
        if reached != Some(entry.verb) {
            wrong.push(format!(
                "{input:?} claims {} but reached {:?}",
                entry.verb.canonical(),
                reached.map(Verb::canonical)
            ));
        }
    }

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Colliding spellings must both be claimed, and the set is pinned.
///
/// An exact match outranks every approximate one, so a collision between two
/// words that each *own* a verb costs a prompt on a typo. Releasing one is worse
/// than the collision: an unclaimed `decant` resolved to `decoct` — silently
/// brewing when the player meant to collect — which is exactly the defect that
/// justified renaming a canonical verb.
#[test]
fn the_tolerated_collision_set_is_pinned() {
    let mut collisions = Vec::new();
    for (word, verb) in single_words() {
        for (other, other_verb) in single_words() {
            if verb < other_verb && similarity(word, other) >= MIN_SIMILARITY {
                collisions.push((word, other));
            }
        }
    }

    // A new entry that adds a collision has to amend this list, which forces
    // someone to check that both spellings are claimed by the right verbs.
    assert_eq!(
        collisions,
        [
            ("cat", "cast"),
            ("find", "bind"),
            ("audit", "edit"),
            ("wait", "write"),
            ("decoct", "decant"),
            ("decoct", "decode"),
            ("make", "take"),
        ],
        "the set of tolerated synonym collisions changed"
    );
}

/// Prefixes shared across *synonyms* are ambiguous on purpose, and pinned.
///
/// Keeping the pre-rename words claimed is what stops them resolving to the
/// wrong verb, and the cost is that `dec` still reaches three verbs. That is
/// correct behaviour — three legitimate words begin with it, so a prompt is the
/// right answer — but it is not what the canonical rule guarantees, and the
/// distinction is worth a test rather than a sentence.
#[test]
fn ambiguous_synonym_prefixes_are_known() {
    let mut ambiguous = Vec::new();
    for (word, _) in single_words() {
        if word.len() < 3 {
            continue;
        }
        let prefix = &word[..3];
        let verbs: std::collections::BTreeSet<&str> = single_words()
            .into_iter()
            .filter(|(other, _)| other.starts_with(prefix))
            .map(|(_, verb)| verb.canonical())
            .collect();
        if verbs.len() > 1 && !ambiguous.iter().any(|(p, _): &(&str, _)| *p == prefix) {
            ambiguous.push((prefix, verbs.into_iter().collect::<Vec<_>>()));
        }
    }
    ambiguous.sort_unstable();

    // `aut`: automate (bind) vs author (scribe).
    // `dec`: decoct vs decant (siphon) vs decipher/decode (divine).
    // `ins`: inscribe (scribe) vs inspect (verify).
    // Each prompts, which is the right answer — the abbreviation genuinely is
    // ambiguous. What must never happen is one of them resolving silently, and
    // `every_phrase_reaches_the_verb_that_claims_it` is what guards that.
    assert_eq!(
        ambiguous,
        [
            ("aut", vec!["bind", "scribe"]),
            ("dec", vec!["decoct", "divine", "siphon"]),
            ("ins", vec!["scribe", "verify"]),
        ],
        "the set of ambiguous synonym prefixes changed"
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
