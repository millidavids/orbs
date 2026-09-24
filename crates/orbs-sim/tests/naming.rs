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
    // Standing in the laboratory with every per-instrument verb in scope, so the
    // guards below measure the worst case — every word the game has, live at
    // once. Without them this file would be measuring the scoping rule instead.
    //
    // Built from `Verb::anchor`, not `is_operation`: the latter is the
    // production-slot question and left `research`, `follow` and `wander`
    // unresolvable, which is exactly that failure.
    let mut scene = Verb::ALL
        .into_iter()
        .filter_map(Verb::anchor)
        .fold(Scene::new(), Scene::offering)
        .with(NounKind::Place, "/tower/laboratory")
        .with(NounKind::File, "feed.log")
        .with(NounKind::Topic, "brewing")
        .with(NounKind::Essence, "clarity")
        .with(NounKind::Reagent, "sage")
        // `alembic` became an instrument — a place — with §10.1, so the vessel
        // fixture is `retort`, which stayed one when `crucible` was removed.
        .with(NounKind::Vessel, "retort")
        .with(NounKind::Scroll, "gleaning-scroll")
        .with(NounKind::Script, "night_watch")
        .with(NounKind::Any, "sludge");

    // Every verb word, both ways, because `tower::scene_at` does both. The
    // fixture did neither, so `recall grind` never filled its `Subject` slot and
    // passed only because a reading that explained nothing resolved as the bare
    // verb. That is `Incomplete` now (§19), which made the gap visible.
    for (word, _) in single_words() {
        scene = scene.with(NounKind::Command, word);
    }
    scene.knowing(single_words().map(|(word, _)| word.to_owned()))
}

/// An argument that satisfies `verb`'s signature.
const fn sample_argument(verb: Verb) -> &'static str {
    let Some(slot) = verb.signature().first() else {
        return "";
    };
    match slot.kind {
        NounKind::Place => "/tower/laboratory",
        NounKind::File => "feed.log",
        NounKind::Pattern => "march feed.log",
        NounKind::Topic => "brewing",
        NounKind::Essence => "clarity",
        NounKind::Reagent => "sage",
        NounKind::Vessel => "retort",
        NounKind::Scroll => "gleaning-scroll",
        NounKind::Script => "night_watch",
        NounKind::Count => "30",
        // Free text the player is coining, so any word will do — and a word that
        // is *not* in the scene, so this measures the naming rather than a
        // lucky match against something the fixture happens to hold.
        NounKind::Name => "morning",
        // A reading of the archive's maze. Never a slot's kind, so this arm is
        // unreachable and says so rather than inventing an untested sample.
        NounKind::Sense => "passage",
        // A verb's own name, reachable only from `Subject` — which is what
        // `recall` takes, so the sample is a command.
        NounKind::Command | NounKind::Subject => "grind",
        // Slot kinds, never a noun's own, so each sample is a noun that fills
        // one — and the ordinary reading in each case, leaving spells and
        // scrolls to the tests that are about them.
        NounKind::Readable => "feed.log",
        NounKind::Stoppable => "/tower/laboratory",
        NounKind::Workable => "/tower/laboratory",
        NounKind::Portable => "sage",
        NounKind::Any => "sludge",
    }
}

/// Single-word synonyms, with the verb that owns them.
///
/// The sim's own accessor, which this file used to duplicate. Not a test helper:
/// `tower::scene_at` registers this list as `NounKind::Command` and hands it to
/// `Scene::knowing`, so a word in a player's way here is in their way in game.
use orbs_sim::parser::single_words;

#[test]
fn a_phrase_that_leads_with_another_verbs_word_is_pinned() {
    // `single_words` is all the collision check walks, so a multi-word synonym
    // can open with a word another verb owns outright. The phrase is clean; what
    // the player sees half way through typing it is not covered.
    //
    // It matters most for `quit`, which refuses `leave` and `exit` because an
    // ambiguity prompt must never offer ending the session beside a verb people
    // type constantly — picking wrong there cannot be typed back (§19).
    let owned: Vec<(&str, Verb)> = single_words().collect();
    let mut leading = Vec::new();
    for entry in SYNONYMS.iter().filter(|entry| entry.words.len() > 1) {
        let first = entry.words[0];
        for (word, verb) in &owned {
            if *word == first && *verb != entry.verb {
                leading.push((first, entry.verb, *verb));
            }
        }
    }

    // Amending this list forces someone to check that a player typing the lead
    // word alone gets an answer they can live with.
    assert_eq!(
        leading,
        [
            // `look for` (sift) opens with `look`, which `survey` owns. The
            // benign shape and the reason the others are tolerated too: bare
            // `look` is `survey`, which is what anyone typing it means, and the
            // second word turns it into a search.
            ("look", Verb::Sift, Verb::Survey),
            // `open the menu` opens with `open`, which `peruse` owns. Bare
            // `open` asks what to read — a refusal naming files, not an offer to
            // leave the tower.
            ("open", Verb::Menu, Verb::Peruse),
            // `put it down` opens with `put`, which `dial` owns and predates
            // `quit` entirely, so half way through this is a lens refusal.
            ("put", Verb::Quit, Verb::Dial),
            // `stop playing` opens with `stop`, a canonical destructive verb —
            // the collision §19 would care about most. `stop athanor` damps the
            // fire, bare `stop` asks what to stop, and only the whole phrase
            // ends the session.
            ("stop", Verb::Quit, Verb::Stop),
        ],
        "a multi-word synonym now opens with a word another verb owns; check \
         what typing that word alone answers before pinning it",
    );
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
    // Except across a domain boundary, where the two are never candidates at
    // once unless you are in the laboratory, and there the instrument in front
    // of you breaks the tie (`resolve::DOMAIN_BONUS`).
    //
    // `grind`/`bind` sit at exactly `MIN_SIMILARITY` — two edits in a
    // five-letter word, further apart than the tolerated `find`/`bind` at 750,
    // and they take different argument kinds.
    const ACROSS_DOMAINS: [(&str, &str); 1] = [("grind", "bind")];

    for a in Verb::ALL {
        for b in Verb::ALL {
            if a >= b {
                continue;
            }
            let pair = (a.canonical(), b.canonical());
            if ACROSS_DOMAINS.contains(&pair) {
                assert_ne!(
                    a.is_operation(),
                    b.is_operation(),
                    "{} and {} are both in scope together — the exemption does not apply",
                    a.canonical(),
                    b.canonical()
                );
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
        // No exceptions any more: `gri` was the one shared prefix, between
        // `grimoire` and `grind`, and renaming the manual to `recall` (§19)
        // deleted the clash rather than mitigating it.
        assert_eq!(
            hits.len(),
            1,
            "the abbreviation {prefix:?} reaches {hits:?}"
        );
    }
}

#[test]
fn rec_is_pinned_as_a_prefix_before_anything_else_wants_it() {
    // The forward risk `recall` carries is not edit distance — it scores at most
    // 500 against everything. But `rec` prefix-matches it at 900, and a future
    // `recipe`, `record` or `recover` would rebuild the `dec`-reaches-three
    // defect one word at a time with nothing complaining. So the prefix is
    // claimed here rather than discovered later.
    let owners: Vec<&str> = Verb::ALL
        .iter()
        .map(|verb| verb.canonical())
        .filter(|name| name.starts_with("rec"))
        .collect();
    assert_eq!(
        owners,
        ["recall"],
        "`rec` is spoken for; a second verb wanting it needs a different word",
    );
}

/// The invariant the whole pass rests on: every phrase the vocabulary claims
/// must, given a fitting argument, reach the verb that claims it. A word
/// outranked by another verb's word is a wrong command with `Clear` confidence,
/// not a near miss.
///
/// It replaces a check that compared the synonym list against a set built from
/// the same list, so it was always true — and passed while `find` resolved to
/// `bind` and `write` to `meditate`.
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
            // `light` (wield) vs `list` (survey). Unclaimed, `light athanor`
            // silently ran `survey athanor` and read as the fire being
            // unrelightable; claimed, the exact match wins and the collision
            // costs a prompt on a typo.
            ("list", "light"),
            ("cat", "cast"),
            // `grind` (the mortar) against `find` (sift) and `bind` (scripts).
            // Both across a domain boundary: `grind` is a laboratory-only word
            // (§7) and the other two go everywhere, so the three are candidates
            // together in one room, where the instrument settles the tie
            // (`resolve::DOMAIN_BONUS`). Two edits each, different argument
            // kinds — the case domain scoping was added for.
            ("find", "grind"),
            ("find", "bind"),
            // `search` (sift) vs `research`, at 750 — the highest this list
            // tolerates, and the score that got `step` rejected against `stop`.
            // Kept because the whole spelling behaves: `serch` is 834 to sift
            // and 625 to research, `reserch` is 875 to research and 572 to
            // search. They diverge at the front, where a fuzzy match is decided;
            // `step`/`stop` are both four letters and do not.
            ("search", "research"),
            // `audit` (verify) vs `quit`. This one could not be dodged — `quit`
            // is canonical, so unlike `leave` and `exit` there is no alternative
            // spelling. Tolerated because both are claimed, they take different
            // argument shapes, and they are two edits apart.
            ("audit", "quit"),
            ("audit", "edit"),
            // `("wait", "write")` left this set: §8 needed `wait` as the
            // smallest control structure, so it was released to the spell
            // vocabulary (§19).
            ("decoct", "decant"),
            ("decoct", "decode"),
            // `("make", "take")` left with `siphon` (§19) — `empty` inherited
            // `collect`, `decant` and `pour` and deliberately not `take`, so the
            // collision is gone rather than moved. Two entries have left this
            // list and none has joined it.
            ("grind", "bind"),
        ],
        "the set of tolerated synonym collisions changed"
    );
}

/// Prefixes shared across *synonyms* are ambiguous on purpose, and pinned.
///
/// Keeping the pre-rename words claimed stops them resolving to the wrong verb,
/// and the cost is that `dec` still reaches three. A prompt is the right answer
/// there, but it is not what the canonical rule guarantees — hence a test rather
/// than a sentence.
#[test]
fn ambiguous_synonym_prefixes_are_known() {
    let mut ambiguous = Vec::new();
    for (word, _) in single_words() {
        if word.len() < 3 {
            continue;
        }
        let prefix = &word[..3];
        let verbs: std::collections::BTreeSet<&str> = single_words()
            .filter(|(other, _)| other.starts_with(prefix))
            .map(|(_, verb)| verb.canonical())
            .collect();
        if verbs.len() > 1 && !ambiguous.iter().any(|(p, _): &(&str, _)| *p == prefix) {
            ambiguous.push((prefix, verbs.into_iter().collect::<Vec<_>>()));
        }
    }
    ambiguous.sort_unstable();

    // `aut`: automate (bind) vs author (scribe).
    // `dec`: decoct (now recall) vs decant (now empty) vs decipher/decode (divine).
    // `ins`: inscribe (scribe) vs inspect (verify).
    // `tra`: transfer/transport (move) vs translate (divine).
    // Each prompts, which is right — the abbreviation genuinely is ambiguous.
    // What must never happen is one resolving silently, and
    // `every_phrase_reaches_the_verb_that_claims_it` guards that.
    //
    // `gri` left this set when the manual was renamed to `recall` (§19), which
    // is the direction it should move in. `pro` and `que` joined, with the lens
    // and the satchel: in both the full word is canonical and exact, so only a
    // deliberate abbreviation prompts, and the pair is as far apart as two words
    // in this game get. What separates them from §19's `leave`/`exit` refusal is
    // what sits on the other side — there it was *ending the session*, and a
    // prompt is only unfair when one of its answers is expensive.
    //
    // The canonical prefix is what actually binds:
    // `three_character_canonical_prefixes_name_at_most_one_verb` has no
    // exemptions, which is why `scry` is not a verb (`scr` reaches `scribe`) and
    // why `seat` became `dial` (`sea` reaches `search`). The alternatives to
    // `queue` lose there: `stow` shares `sto` with `stop` and scores 750 against
    // it, `stash` shares `sta` with `status`.
    assert_eq!(
        ambiguous,
        [
            ("aut", vec!["bind", "scribe"]),
            ("dec", vec!["empty", "recall", "research"]),
            ("ins", vec!["scribe", "verify"]),
            ("pro", vec!["probe", "weave"]),
            ("que", vec!["queue", "stop"]),
            // `res` is `research`'s own prefix and `rest` wins it: four letters
            // to eight, so coverage puts `meditate` ahead at 962 to 906. Right
            // way round — `rest` is a whole word, `res` is a part-typed
            // abbreviation, and `rese` already separates them.
            ("res", vec!["meditate", "research"]),
            ("tra", vec!["move", "research"]),
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
    // `decant` must not *release* it either, since unclaimed it lands on
    // `decoct`. Every anchor offered, as in `scene()` above: a scene that scoped
    // `research` out would make this measure the scoping rule instead.
    let scene = Verb::ALL
        .into_iter()
        .filter_map(Verb::anchor)
        .fold(Scene::new(), Scene::offering)
        // `siphon` takes a place now (§10.1): the product sits in the instrument
        // that made it, not in a vessel.
        .with(NounKind::Place, "/tower/laboratory/alembic")
        .with(NounKind::Vessel, "retort")
        .with(NounKind::Topic, "clarity")
        .with(NounKind::Scroll, "gleaning-scroll")
        .with(NounKind::Script, "night_watch");

    for (input, expected) in [
        // Echoed as a leaf: the full path clipped the destination off a
        // three-argument `move` at the 80×22 floor, and §7 says players say the
        // leaf anyway.
        //
        // `decant` outlived the verb it was a synonym for — `siphon` retired
        // (§19) and `empty` inherited its words, because a released word does
        // not stop resolving, it resolves to whatever is nearest (§6.1), and the
        // nearest here were `purge` and `stop`.
        ("decant alembic", "empty alembic"),
        // `divine` takes no argument now: it opens the stacks rather than
        // consuming a fragment (§10, §19). The *word* is what this is about.
        ("decipher", "research"),
        // `divine` was the canonical until the archive got its name right, and
        // is kept because a word the game taught is a word it owes an answer to.
        ("divine", "research"),
        ("research", "research"),
        ("inscribe night_watch", "scribe night_watch"),
        ("scribe night_watch", "scribe night_watch"),
        // Retired in Phase 1 (§19) and still claimed, pointed at the recipe.
        ("decoct clarity", "recall clarity"),
        ("brew clarity", "recall clarity"),
    ] {
        let echo = resolve(input, &scene, Mode::Calm)
            .intent()
            .map_or_else(|| "<unresolved>".to_owned(), orbs_sim::parser::Intent::echo);
        assert_eq!(echo, expected, "{input:?}");
    }
}

/// The readings a bare command offers, in the order the prompt shows them.
fn offered(sim: &orbs_sim::Sim) -> Vec<String> {
    let choices = sim.choices();
    (1..=choices.len())
        .filter_map(|at| choices.pick(at))
        .map(orbs_sim::parser::Intent::echo)
        .collect()
}

#[test]
fn a_bare_anything_verb_offers_the_same_four_readings() {
    // Pinned before the noun space moves, not after. A bare `purge` prompts with
    // every noun in the room sorted by `Argument`'s derived `Ord`, so adding a
    // noun kind reorders it and adding a noun changes which four surface — with
    // nothing on screen saying so.
    //
    // It has moved once, which is what the pin is for: `east` was the fourth
    // reading until the archive gained a `cabinet`, and `/tower/archive/cabinet`
    // sorts first. A fixture added two rooms away changed what a bare `purge`
    // offers.
    //
    // `purge` alone now. This pinned `verify` too, until §8.1's audit made a
    // bare `verify` a legal command, so it no longer prompts. One verb with a
    // required `Any` slot is all it takes to notice the noun space moving.
    let verb = "purge";
    let mut sim = orbs_sim::Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit(verb);
    sim.step();

    assert_eq!(
        offered(&sim),
        [
            format!("{verb} grimoire"),
            format!("{verb} tower"),
            format!("{verb} archive"),
            format!("{verb} cabinet"),
        ],
        "the readings a bare `{verb}` offers moved",
    );
}

#[test]
fn a_verb_name_is_a_subject_and_nothing_else() {
    // Why `NounKind::Command` exists: `recall grind` has to resolve, and
    // registering 27 canonicals as `Topic` would leak them into everything
    // `NounKind::Any` reaches. So it is reachable from exactly one slot kind.
    use orbs_sim::parser::NounKind;
    assert!(
        !NounKind::Any.accepts(NounKind::Command),
        "`verify` and `purge` can reach a verb's own name",
    );
    assert!(NounKind::Subject.accepts(NounKind::Command));
    assert!(NounKind::Subject.accepts(NounKind::Topic));
    for slot in [
        NounKind::Readable,
        NounKind::Stoppable,
        NounKind::Place,
        NounKind::Reagent,
        NounKind::Script,
    ] {
        assert!(
            !slot.accepts(NounKind::Command),
            "{slot:?} reaches a command",
        );
    }
}

#[test]
fn a_verb_name_never_reaches_a_destructive_slot() {
    // The end the kind exists to prevent, driven rather than asserted against
    // `accepts`. `purge` takes `NounKind::Any` and a verb word is not a surface
    // it can act on, so `purge grind` must fall through to the numbered prompt —
    // asking *which*, offering the same four readings a bare `purge` does, with
    // no `grind` among them.
    let mut sim = orbs_sim::Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("purge grind");
    sim.step();

    assert_eq!(
        offered(&sim),
        [
            "purge grimoire".to_owned(),
            "purge tower".to_owned(),
            "purge archive".to_owned(),
            // `east` until the archive gained a `cabinet`; see the pin above.
            // What this test claims is that no *verb name* appears here.
            "purge cabinet".to_owned(),
        ],
        "a verb name moved the readings a destructive verb offers",
    );
}

#[test]
fn a_spell_cannot_name_a_verb_as_a_thing() {
    // `compile::fix` resolves a condition's names through `NounKind::Any`, so a
    // verb registered as a `Topic` would make `if the dispensary has grind`
    // compile clean and answer *no* for ever — the `has ground-slat` defect that
    // module was rewritten to kill.
    let mut sim = orbs_sim::Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();

    let reading = sim.read_spell(
        "laboratory",
        &[
            "if the dispensary has grind".to_owned(),
            "survey".to_owned(),
            "end".to_owned(),
        ],
    );
    assert!(
        reading[0].fault.is_some(),
        "a spell named a verb as a thing and was believed: {:?}",
        reading[0],
    );
}

/// Every word a `Sense` noun answers to, across every domain that has them.
///
/// There was no sweep for these at all, which is how `gained`/`held`/`lost`
/// shipped: a reading is a `NounKind::Sense` and `NounKind::Any` reaches one, so
/// `purge grind` fuzzy-matched `gained` at full confidence. Found by hand twice,
/// both times after it shipped.
fn readings() -> Vec<&'static str> {
    let mut out = orbs_sim::tower::maze::readings();
    out.extend(orbs_sim::tower::ward::readings());
    out.extend(orbs_sim::tower::pylon::readings());
    // The bailey's, and it was missed — twelve unswept words, two of which
    // collided with material names: `vigour` an exact 1000 against the potion,
    // `troops` 834 against `troop`. Renamed to `mettle` and `spears`. A domain
    // that adds readings and not a line here is unswept, and the lint reads
    // exactly as green as if it were not.
    out.extend(orbs_sim::tower::siege::readings());
    // The menagerie's, and the chant's were never here — three words a whole
    // phase shipped unswept. The count was `choler` until this sweep found it
    // 667 against `closer`.
    out.extend(orbs_sim::tower::circle::readings());
    // The forge's, missed the same way — found while choosing the circle's
    // count, whose first name was the forge's own `lit`.
    out.extend(orbs_sim::tower::charm::readings());
    out
}

/// Every material name, which a reading must also not collide with.
///
/// The hole all three earlier sweeps had: a `Sense` sits in the way of
/// everything a player can name — verbs, synonyms, spell words and materials.
/// Sweeping only the first three let `vigour` ship against a potion called
/// `vigour`.
fn materials() -> Vec<String> {
    orbs_sim::content::Materials::builtin()
        .names()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[test]
fn no_reading_collides_with_a_material() {
    let mut bad = Vec::new();
    for reading in readings() {
        for material in materials() {
            let score = similarity(reading, &material);
            if score >= MIN_SIMILARITY {
                bad.push(format!("{reading} vs {material}: {score}"));
            }
            // A prefix collision is invisible to a score, and §19 records three
            // menagerie words lost to one.
            if material.len() >= 3 && reading.len() >= 3 && material[..3] == reading[..3] {
                bad.push(format!("{reading} vs {material}: prefix"));
            }
        }
    }
    bad.sort();
    // Pinned rather than asserted empty, which is
    // `the_tolerated_collision_set_is_pinned`'s idiom: these predate the sweep
    // and most are deliberate. A new reading joining the set is the finding.
    let tolerated = [
        // The errand is named after the scroll that sets it, which is the whole
        // point: `wield gleaning-scroll` then `if the stacks has gleaning`.
        "gleaning vs gleaning-scroll: 930",
        "gleaning vs gleaning-scroll: prefix",
        // Two authored words that happen to share three letters. `dreaming` is a
        // secret potion and `gleaning` an archive errand; nothing takes both.
        "gleaning vs dreaming: 625",
        // `potency` is the sanctum's ward magnitude and `potash` a laboratory
        // byproduct. Three shared letters, different rooms, no shared verb.
        "potency vs potash: prefix",
        // The one tolerated collision that shares a room, with its cost written
        // down. `quintessence` is the siege's pool and `quickening-scroll` is
        // spent at the same wall, so `potency`'s "different rooms" argument does
        // not apply. The verb split does: `wield` takes `Workable = Place |
        // Scroll`, which rejects a `Sense`, so `wield qui` still reaches the
        // scroll — pinned by
        // `the_scroll_keeps_its_abbreviation_against_the_reading`.
        //
        // The exposure is `purge` and `verify`, which see both: `qui` was
        // already ambiguous between the two materials, and the reading now wins
        // it 887 to 876. The alternatives are worse — `mana` scores 750 against
        // `many` (inside the comparison grammar it is written for) and `power`
        // 800 against `tower`.
        "quintessence vs quickening-scroll: prefix",
        "quintessence vs quiet-draught: prefix",
    ];
    let unexpected: Vec<&String> = bad
        .iter()
        .filter(|hit| !tolerated.contains(&hit.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "a reading collides with a material the player can name: {unexpected:#?}",
    );
}

/// What makes `quintessence vs quickening-scroll` tolerable: `wield` is
/// `Workable = Place | Scroll` and a reading is a `Sense`. That is a fact about
/// `Verb::accepts` rather than about the two words, so widening the slot to
/// `Any` would silently un-justify the tolerance. A tolerated collision with no
/// pin under it is a decision that quietly expires.
#[test]
fn the_scroll_keeps_its_abbreviation_against_the_reading() {
    let scene = scene()
        .with(NounKind::Scroll, "quickening-scroll")
        .with(NounKind::Sense, "quintessence");

    let resolution = resolve("wield qui", &scene, Mode::Calm);
    let named = resolution.intent().and_then(|intent| {
        intent
            .arguments
            .first()
            .map(|argument| argument.value.clone())
    });
    assert_eq!(
        named.as_deref(),
        Some("quickening-scroll"),
        "`wield qui` stopped reaching the scroll — `Workable` must still reject a Sense",
    );
}

#[test]
fn the_readings_that_score_against_a_typed_word_are_pinned() {
    // A reading is nameable from every room, so it sits in the way of the whole
    // vocabulary. These score above the bar a canonical faces and are answered
    // by the resolver rather than a rename — every verb word is in
    // `Scene::knowing`, so it matches exactly and none is reachable by fuzzing.
    // The two tests below drive that; this keeps the list honest, because a new
    // entry is a word somebody should look at before shipping.
    //
    // It caught the ward's second delta on its first run:
    // `fuller`/`steady`/`thinner` scored 667 against `filter` and `study`.
    let mut collisions = Vec::new();
    for reading in readings() {
        for (word, _) in single_words() {
            if similarity(reading, word) >= MIN_SIMILARITY {
                collisions.push((reading, word));
            }
        }
    }
    collisions.sort_unstable();
    assert_eq!(
        collisions,
        [
            // `edit` (scribe) vs the maze's way out, at 750. `recall edit` used
            // to answer with the way out of a maze.
            ("exit", "edit"),
            // The forge's column bit, which shipped unswept — its readings were
            // missing until the menagerie's count nearly took this word.
            // `light` (kindle) at 600 and `list` (survey) at 750; both verb
            // words, and `a_verb_word_never_fuzzes_into_a_noun` drives both.
            ("lit", "light"),
            ("lit", "list"),
            // `make` (recall) vs a way's walk count, at 600.
            ("marks", "make"),
            // `walk` (follow) vs a way with no way through, at 750 — the worst,
            // because both are words a player uses about the same screen.
            ("wall", "walk"),
        ],
        "the readings scoring against a typed word have changed",
    );
}

#[test]
fn a_verb_word_never_fuzzes_into_a_noun() {
    // §19's `gained` leak, closed as a class. A destructive verb could name a
    // reading from any room, and `walk`, `edit` and `make` all scored high
    // enough to get there by typo — `purge walk` echoed `purge wall`.
    // `Scene::knowing` holds every verb word now, so each falls through to the
    // numbered prompt exactly as `purge grind` does.
    let mut sim = orbs_sim::Sim::new(1);
    sim.submit("attend archive");
    sim.step();

    for (typed, was) in [("walk", "wall"), ("edit", "exit"), ("make", "marks")] {
        sim.submit(&format!("purge {typed}"));
        sim.step();
        assert!(
            !offered(&sim).is_empty(),
            "`purge {typed}` did not ask which",
        );
        assert!(
            !offered(&sim).iter().any(|line| line.contains(was)),
            "`purge {typed}` still reaches `{was}`: {:?}",
            offered(&sim),
        );
    }

    // The forge's `lit`, from the room whose columns carry it, with a lattice
    // open so there is a lit column to reach.
    sim.submit("attend forge");
    sim.step();
    sim.submit("imbue mortar_and_pestle hurried");
    sim.step();
    for typed in ["light", "list"] {
        sim.submit(&format!("purge {typed}"));
        sim.step();
        assert!(
            !offered(&sim).is_empty(),
            "`purge {typed}` did not ask which",
        );
        assert!(
            !offered(&sim).iter().any(|line| line.contains("lit")),
            "`purge {typed}` reaches `lit`: {:?}",
            offered(&sim),
        );
    }
}

#[test]
fn the_manual_answers_the_word_that_was_typed() {
    // The worse half of the same leak: `recall edit` is a player asking about
    // the spell editor, and it explained the maze's way out. Every one-word
    // synonym is a `NounKind::Command` now.
    let mut sim = orbs_sim::Sim::new(1);
    sim.submit("attend archive");
    sim.step();

    for (typed, page) in [
        ("walk", "follow <way>"),
        ("edit", "scribe <name>"),
        ("make", "recall"),
        ("light", "kindle"),
    ] {
        sim.submit(&format!("recall {typed}"));
        sim.step();
        let said: Vec<String> = sim
            .scrollback()
            .records()
            .iter()
            .filter_map(
                |record| match record.field(orbs_render::FieldName::Message) {
                    Some(orbs_render::Value::Text(text)) => Some(text.to_owned()),
                    _ => None,
                },
            )
            .collect();
        assert!(
            said.iter().any(|line| line.starts_with(page)),
            "`recall {typed}` did not reach the {page:?} page",
        );
    }
}

#[test]
fn a_word_the_game_knows_may_still_abbreviate() {
    // The affordance `knowing` must not eat, found by breaking it: `check` is
    // one of `verify`'s words, so once every verb word joined the known set a
    // spell called `check.spell` stopped being reachable by `invoke check`.
    // Prefixing is not fuzzing — `check` *starts* `check.spell`, where `walk`
    // does not start `wall`. See `Scene::candidates`.
    let mut sim = orbs_sim::Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.write_spell("check", &["survey".to_owned()]);
    sim.step();
    sim.submit("invoke check");
    sim.step();

    assert!(
        offered(&sim).is_empty(),
        "`invoke check` asked which instead of finding check.spell: {:?}",
        offered(&sim),
    );
}

#[test]
fn no_two_readings_fuzzy_match_each_other() {
    // Two readings colliding inside one domain is worse than a verb near-miss,
    // which is what rejected `warmer`/`even`/`cooler` for the ward's second
    // delta: `cooler` is 667 against `closer` and `even` 600 against `level`, so
    // a spell's author could not tell which channel they had asked.
    let words = readings();
    let mut collisions = Vec::new();
    for (index, reading) in words.iter().enumerate() {
        for other in words.iter().skip(index + 1) {
            if similarity(reading, other) >= MIN_SIMILARITY {
                collisions.push(format!(
                    "{reading} vs {other} at {}",
                    similarity(reading, other),
                ));
            }
        }
    }
    assert!(
        collisions.is_empty(),
        "two readings are in each other's way: {collisions:?}",
    );
}

/// Every plain-English synonym that is genuinely more than one word.
///
/// Pinned rather than derived because `Synonym::words` is one phrase, pre-split:
/// `&["spy", "peek", "try"]` declares the phrase `spy peek try`, not three
/// synonyms. The lens shipped that way, leaving §6's plain register with no way
/// in — and two tests watched it happen, because one asks whether a `Plain`
/// entry exists and the other drives the declared phrase, which resolves fine.
///
/// So the multi-word ones are listed. A real phrase — `go to`, `get rid of` —
/// belongs here; three words meant to be three entries do not, and adding one
/// fails this test with the phrase printed.
#[test]
fn multi_word_plain_synonyms_are_pinned() {
    let mut found: Vec<String> = SYNONYMS
        .iter()
        .filter(|entry| entry.register == Register::Plain && entry.words.len() > 1)
        .map(|entry| entry.words.join(" "))
        .collect();
    found.sort_unstable();

    let mut expected = [
        "get rid of",
        "go to",
        "how are things",
        "how do i",
        "look around",
        "look for",
        // The menu, said the way someone who has never met a shell would say
        // it. `menu` alone is the arcane register's word for the same verb.
        "open the menu",
        "page back",
        "page up",
        "put it down",
        "stop playing",
        "take it back",
        "what's here",
    ];
    expected.sort_unstable();

    assert_eq!(
        found, expected,
        "a plain synonym is more than one word. if it is a phrase a player says \
         as a unit, pin it here; if it is several synonyms, `syn` them separately \
         — `words` is one phrase, not a list of them",
    );
}

/// §6.1's command table says what the code must do, so the two are checked
/// against each other.
///
/// A design table is a second list, and this one had drifted six rows —
/// `meditate` appeared twice saying opposite things about `wait`, `stop` claimed
/// `damp` where the code says `quench`, and four verbs had gained words the
/// table never heard about. §19 records the same shape three times over, every
/// one found by a person reading two things side by side.
///
/// Only the rows that are there: §6.1 is the *slice* vocabulary and the domains
/// coined fourteen verbs after it. The plain register is not compared either —
/// it is prose, written as English with quotes and capitals. What is held is the
/// canonical name and its shell synonyms, exact words in both places.
#[test]
fn the_slice_table_in_this_document_matches_the_vocabulary() {
    let design = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/DESIGN.md")
        .canonicalize()
        .expect("DESIGN.md is beside the crates");
    let text = std::fs::read_to_string(&design).expect("DESIGN.md is readable");

    let mut rows: Vec<(String, Vec<String>)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for line in text.lines() {
        // A vocabulary row opens with a backticked canonical name and has four
        // cells. Any other table in the document has neither.
        if !line.starts_with("| `") {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 4 {
            continue;
        }
        let Some(name) = cells[0].trim_matches('`').split_whitespace().next() else {
            continue;
        };
        let Some(verb) = Verb::ALL.into_iter().find(|verb| verb.canonical() == name) else {
            // `decoct` and anything else retired. The table says so in its own
            // words and there is no verb left to compare against.
            continue;
        };
        assert!(
            !seen.contains(&name.to_owned()),
            "`{name}` has two rows in the table. one of them is stale, and the \
             two said opposite things about `wait` for a whole phase",
        );
        seen.push(name.to_owned());

        let shell: Vec<String> = if cells[2] == "—" {
            Vec::new()
        } else {
            cells[2]
                .split(',')
                .map(|word| word.trim().trim_matches('`').to_owned())
                .collect()
        };
        rows.push((verb.canonical().to_owned(), shell));
    }
    assert!(
        rows.len() >= 15,
        "only {} vocabulary rows found; the table moved and this stopped \
         reading it",
        rows.len(),
    );

    for (name, documented) in rows {
        let verb = Verb::ALL
            .into_iter()
            .find(|verb| verb.canonical() == name)
            .expect("matched above");
        let mut live: Vec<String> = SYNONYMS
            .iter()
            .filter(|entry| entry.verb == verb && entry.register == Register::Shell)
            .map(|entry| entry.words.join(" "))
            .collect();
        let mut documented = documented;
        live.sort_unstable();
        documented.sort_unstable();
        assert_eq!(
            documented, live,
            "DESIGN.md §6.1 and `SYNONYMS` disagree about `{name}`'s shell words",
        );
    }
}
