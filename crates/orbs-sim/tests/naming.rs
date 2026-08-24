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
    // **Standing in the laboratory**, with every per-instrument verb in scope.
    // Those verbs only resolve where their instrument is (§7,
    // `Scene::offers`), so a scene without them would have this whole file
    // measuring the scoping rule instead of the naming.
    //
    // The naming guards below are therefore the *worst case*: every word the
    // game has, all live at once. Out in the archive the laboratory's four are
    // not candidates at all, so a collision pinned here is narrower in play than
    // it looks on the page.
    // **Every *anchor*, and it was every operation.** `Scene::offers` asks
    // `Verb::anchor` now rather than `is_operation` — the production-slot question
    // — so a scene built from the latter left `research`, `follow` and `wander`
    // unresolvable and had this file measuring the scoping rule after all, which
    // is the one thing the paragraph above says it must not do.
    Verb::ALL
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
        .with(NounKind::Any, "sludge")
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
        // A reading of the archive's maze. Never a slot's kind — no verb asks
        // for one, which is the whole reason the kind exists (see `NounKind`) —
        // so this arm is unreachable and says so rather than inventing a sample
        // that would go untested.
        NounKind::Sense => "passage",
        // A verb's own name, reachable only from `Subject` — which is what
        // `recall` takes, so the sample is a command.
        NounKind::Command | NounKind::Subject => "grind",
        // A slot kind, never a noun's own, so the sample is a noun that *fills*
        // one. The log, not the spell: this exercises the ordinary reading and
        // leaves `peruse night_watch` to the tests that are about spells.
        NounKind::Readable => "feed.log",
        // A slot kind, never a noun's own. The place, not the spell: this
        // exercises the ordinary `stop <instrument>` and leaves calling a spell
        // off to the tests that are about spells.
        NounKind::Stoppable => "/tower/laboratory",
        // The same again for `wield`: the instrument is the ordinary reading,
        // and spending a scroll is left to the tests that are about scrolls.
        NounKind::Workable => "/tower/laboratory",
        // What `move` takes. The reagent, not the potion: this exercises §10.1's
        // own loop, and carrying finished work is what `tests/arsenal.rs` is
        // about.
        NounKind::Portable => "sage",
        NounKind::Any => "sludge",
    }
}

/// Single-word synonyms, with the verb that owns them.
///
/// **The sim's own accessor**, which this file used to duplicate. It is not a
/// test helper any more: `tower::scene_at` registers exactly this list as
/// `NounKind::Command` *and* hands it to `Scene::knowing`, so a word that is in
/// a player's way here is in their way in the game for the same reason.
use orbs_sim::parser::single_words;

#[test]
fn a_phrase_that_leads_with_another_verbs_word_is_pinned() {
    // **`single_words` is all the collision check walks**, so a multi-word
    // synonym has always been able to open with a word another verb owns
    // outright — and `quit`'s own comment leans on that: *"plain English
    // arriving as a **phrase** reaches the verb with no collision at all."*
    // That is true of the *phrase*, and says nothing about what the player sees
    // half way through typing it.
    //
    // It matters most for `quit`, which §19 is careful about for exactly this
    // reason: it refuses `leave` and `exit` because an ambiguity prompt must
    // never offer ending the session beside a verb people type constantly, and
    // "picking wrong there cannot be typed back".
    let owned: Vec<(&str, Verb)> = single_words();
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
            // `put it down` (quit) opens with `put`, which `dial` owns as its
            // plain synonym. **The phrase is not the hazard.** Bare `put`
            // resolves to `dial` and always has — it predates `quit` entirely —
            // so a player half way through this sees a lens refusal, not an
            // offer to end the session.
            ("put", Verb::Quit, Verb::Dial),
            // `stop playing` (quit) opens with `stop`, which is a canonical verb
            // and a destructive one. **Checked rather than assumed**, because
            // this is the collision §19 would care about most: `stop athanor`
            // damps the fire, `stop stacks` closes the maze, and bare `stop`
            // asks what to stop. None of them offers to end the session, and the
            // session is only ended by the whole phrase.
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
    // **Except across a domain boundary**, where the two are never candidates at
    // the same time unless you are standing in the laboratory — and there the
    // instrument in front of you breaks the tie (`resolve::DOMAIN_BONUS`).
    //
    // `grind` and `bind` sit at exactly `MIN_SIMILARITY`, which is *two* edits in
    // a five-letter word: further apart than the tolerated `find`/`bind` at 750,
    // and the two take different argument kinds on top. `grind` is the word the
    // mortar answers to; a collision this weak does not buy renaming it.
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
        // **No exceptions any more.** `gri` was the one shared prefix, between
        // `grimoire` and `grind`, and it needed a paragraph arguing the clash
        // was survivable because the two were words in the same room. Renaming
        // the manual to `recall` (§19) deleted the clash rather than mitigating
        // it, and the exception went with it — an exemption that outlives its
        // cause is how a guard quietly stops guarding.
        assert_eq!(
            hits.len(),
            1,
            "the abbreviation {prefix:?} reaches {hits:?}"
        );
    }
}

#[test]
fn rec_is_pinned_as_a_prefix_before_anything_else_wants_it() {
    // **The forward risk `recall` actually carries, which is not edit distance.**
    // `recall` scores at most 500 against every other canonical and synonym in
    // the vocabulary — nowhere near `MIN_SIMILARITY` — so no collision exists
    // today. But `rec` prefix-matches `recall` at 900, and §5 has `repair` for
    // nuisances while §11 has recipes and records: a future `recipe`, `record`
    // or `recover` would build the `dec`-reaches-three-verbs defect the naming
    // pass exists to prevent, one word at a time and with nothing complaining.
    //
    // So the prefix is claimed here rather than discovered in Phase 3, where the
    // naming pass runs and five domains' worth of new verbs arrive.
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
            // `light` (wield) vs `list` (survey). Tolerated on purpose, and the
            // reason is the rule this whole test exists for: unclaimed, `light`
            // resolved to `list`, so `light athanor` silently ran `survey
            // athanor` and read as the fire being unrelightable. Claimed, an
            // exact match beats the fuzzy one and the collision costs a prompt
            // on a typo instead of a wrong command with `Clear` confidence.
            ("list", "light"),
            ("cat", "cast"),
            // `grind` (the mortar) against `find` (sift) and `bind` (scripts).
            // Both are **across a domain boundary**: `grind` is a word only in
            // the laboratory (§7), and the other two go everywhere — so the
            // three are candidates together in exactly one room, where the
            // instrument standing in it settles the tie
            // (`resolve::DOMAIN_BONUS`). `find`/`grind` and `grind`/`bind` are
            // both two edits, further apart than the `find`/`bind` above them,
            // and the three take different argument kinds. This is the case
            // domain scoping was added for.
            ("find", "grind"),
            ("find", "bind"),
            // **`search` (sift) vs `research`, at 750 — the highest score this
            // list tolerates**, above `decoct`/`decant`'s 667, and the exact
            // score that got `step` rejected against `stop`. It is kept, and the
            // difference from `step` is what the *whole* spelling does rather
            // than what the pair scores: `step` and `stop` are both four
            // letters, so a typo in either lands nearer the other. Here every
            // spelling a player produces resolves to the verb they meant —
            // `search` and `research` are exact, `serch` is 834 to *sift* and
            // 625 to research, `reserch` is 875 to *research* and 572 to search.
            // The two words diverge at the front, which is where a fuzzy match
            // is decided.
            ("search", "research"),
            // **`audit` (verify) vs `quit`, and this one could not be dodged**:
            // `quit` is the canonical name, so unlike `leave` and `exit` — which
            // `vocabulary.rs` declines for exactly this reason — there is no
            // alternative spelling to reach for.
            //
            // Tolerated because both are claimed, which is `("list", "light")`'s
            // rule: an exact `audit` resolves to `verify` and never reaches the
            // fuzzy path. They also take different argument shapes — `verify`
            // names a thing and `quit` takes nothing — and they are two edits
            // apart, further than most of this list.
            ("audit", "quit"),
            ("audit", "edit"),
            // `("wait", "write")` **left this set**, and the set is one shorter
            // than it was. `wait` was `meditate`'s shell synonym; §8 needed it
            // as the smallest control structure, and one word cannot be both —
            // so it was released to the spell vocabulary (§19). A tolerated
            // collision disappearing is the direction this list should move in.
            ("decoct", "decant"),
            ("decoct", "decode"),
            // `("make", "take")` **left this set** with `siphon` (§19). `take`
            // was one of its plain synonyms and sat one edit from `make`
            // (`recall`); it was survivable only because the two verbs took
            // different argument kinds. `empty` inherited `collect`, `decant`
            // and `pour` and deliberately **not** `take`, so the collision is
            // gone rather than moved. Two entries have now left this list and
            // none has joined it.
            ("grind", "bind"),
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
    // `dec`: decoct (now recall) vs decant (now empty) vs decipher/decode (divine).
    // `ins`: inscribe (scribe) vs inspect (verify).
    // `tra`: transfer/transport (move) vs translate (divine).
    // Each prompts, which is the right answer — the abbreviation genuinely is
    // ambiguous. What must never happen is one of them resolving silently, and
    // `every_phrase_reaches_the_verb_that_claims_it` is what guards that.
    //
    // **`gri` left this set.** It was `grimoire` vs `grind`, and it needed an
    // argument about the mortar being in only one room. Renaming the manual to
    // `recall` (§19) removed the clash outright — the set is one shorter than it
    // was, which is the direction it should move in.
    //
    // **`pro` joined it with the lens**, and it is the mildest entry here:
    // `probe` is a *canonical* and `progress` is one of `weave`'s plain
    // synonyms, so an exact `probe` beats the fuzzy reading outright and the
    // clash costs a prompt only on a genuine abbreviation. The two are also as
    // far apart as two words in this game get — one is the scrying room's work,
    // the other opens the progression screen — so a player who meant either and
    // typed `pro` is being asked a fair question.
    //
    // The canonical `pro` is unshared, which is the rule that actually binds:
    // `three_character_canonical_prefixes_name_at_most_one_verb` has no
    // exemptions, and it is why `scry` is not a verb (`scr` reaches `scribe`)
    // and why `seat` became `dial` (`sea` reaches `sift`'s `search`).
    assert_eq!(
        ambiguous,
        [
            ("aut", vec!["bind", "scribe"]),
            ("dec", vec!["empty", "recall", "research"]),
            ("ins", vec!["scribe", "verify"]),
            ("pro", vec!["probe", "weave"]),
            // **`res` is `research`'s own prefix, and `rest` wins it.** `rest`
            // is `meditate`'s, four letters to `research`'s eight, so the
            // coverage half of the prefix score puts it ahead (962 to 906) and
            // three characters reach *meditate*. That is the right way round:
            // `rest` is a whole word a player means, `res` is an abbreviation
            // they are part-way through, and `rese` already separates them.
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
    // `decant` it must not *release* it either, since an unclaimed `decant`
    // lands on `decoct`.
    // Every anchor offered, for the same reason the shared `scene()` above does it:
    // this test is about a *word* still reaching its verb, and a scene that scoped
    // `research` out would have it measuring the scoping rule instead. `decipher`
    // resolving to nothing is the scope, not the naming.
    let scene = Verb::ALL
        .into_iter()
        .filter_map(Verb::anchor)
        .fold(Scene::new(), Scene::offering)
        // `siphon` takes a **place** now (§10.1): the product sits in the
        // instrument that made it, not in a vessel.
        .with(NounKind::Place, "/tower/laboratory/alembic")
        .with(NounKind::Vessel, "retort")
        .with(NounKind::Topic, "clarity")
        .with(NounKind::Scroll, "gleaning-scroll")
        .with(NounKind::Script, "night_watch");

    for (input, expected) in [
        // Echoed as a **leaf**: the full path clipped the destination off a
        // three-argument `move` at the 80×22 floor, and the leaf is what §7 says
        // players say anyway.
        // **`decant` outlived the verb it was a synonym for.** `siphon` retired
        // (§19) and `empty` inherited its words, because §6.1's rule is that a
        // released word does not stop resolving — it resolves to whatever it is
        // nearest, and the two nearest here are `purge` and `stop`. Somebody who
        // learned `decant` still gets the thing that takes stuff out of a tool.
        ("decant alembic", "empty alembic"),
        // `divine` takes no argument now: it opens the stacks
        // rather than consuming a fragment (§10, §19). The *word* is what this
        // test is about, and `decipher` still reaches it.
        ("decipher", "research"),
        // **`divine` was the canonical until the archive got its name right.**
        // A room of shelves and readings is somewhere you look things up, not
        // somewhere you guess; `divine` is kept because a word the game taught
        // is a word it owes an answer to.
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
    // **Pinned before the noun space moves, not after.** `verify` and `purge`
    // take `NounKind::Any`, so their numbered prompt is every noun in the room
    // sorted by `Argument`'s derived `Ord` — (kind, value, slot). Adding a noun
    // *kind* reorders it, and adding nouns to an existing kind changes which
    // four surface, with nothing on screen saying so.
    //
    // The manual is about to want `recall <verb>` to resolve, and the obvious
    // way to get it — registering all 27 canonicals as `NounKind::Topic` — would
    // move this list. That is why it is written down first: `Topic` is reachable
    // from `Any`, so a change made for the manual would silently land here.
    //
    // **It has moved once, and this is what the pin is for.** `east` was the
    // fourth reading until the archive gained a `cabinet` — somewhere to turn the
    // lectern out into, without which its `dust` was trapped in the instrument
    // that made it. Places sort by full path, and `/tower/archive/cabinet` comes
    // before `/tower/archive/east`, so a fixture added for a reason two rooms
    // away changed what a bare `purge` offers. Nothing on screen would have said
    // so; this did.
    //
    // Both readings are still places the orb refuses to unmake — the cabinet is a
    // `Store` and therefore `Protected`, exactly as the dispensary is — so what
    // changed is which four are listed and not what answering one does.
    for verb in ["purge", "verify"] {
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
}

#[test]
fn a_verb_name_is_a_subject_and_nothing_else() {
    // **The whole reason `NounKind::Command` exists.** `recall grind` has to
    // resolve, and the obvious way — registering 27 canonicals as `Topic` —
    // leaks them into everything `NounKind::Any` reaches. So the kind is
    // reachable from exactly one slot kind and from no other.
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
            // `east` until the archive gained a `cabinet`; places sort by full
            // path and `cabinet` comes first. See the pin above, which is where
            // the reasoning lives — what this test claims is that a *verb name*
            // does not appear here, and it does not.
            "purge cabinet".to_owned(),
        ],
        "a verb name moved the readings a destructive verb offers",
    );
}

#[test]
fn a_spell_cannot_name_a_verb_as_a_thing() {
    // **`compile::fix` resolves a condition's names through `NounKind::Any`**, so
    // a verb registered as a `Topic` would make `if the dispensary has grind`
    // compile clean and answer *no* for ever — which is verbatim the
    // `has ground-slat` defect that module was rewritten to kill. It has to be
    // refused instead.
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
/// **There was no sweep for these at all**, which is how `gained`/`held`/`lost`
/// shipped: a reading is a `NounKind::Sense` and `NounKind::Any` reaches one, so
/// `purge` and `verify` resolve against them from every room in the tower —
/// `purge grind` fuzzy-matched `gained` at full confidence and answered *"there
/// is no gained within reach"*. That was found by hand, twice, and both times
/// after it had shipped.
fn readings() -> Vec<&'static str> {
    let mut out = orbs_sim::tower::maze::readings();
    out.extend(orbs_sim::tower::ward::readings());
    out
}

#[test]
fn the_readings_that_score_against_a_typed_word_are_pinned() {
    // A reading is nameable from every room, so it sits in the way of the whole
    // vocabulary rather than of one domain's. Three score above the bar a
    // canonical faces, and **all three are answered by the resolver rather than
    // by a rename** — every verb word is now in `Scene::knowing`, so it can only
    // ever match exactly and none of these is reachable by fuzzing. The two
    // tests below drive that; this one keeps the list honest, because a *fourth*
    // is a word somebody should look at before shipping it.
    //
    // It caught the ward's second delta on its first run:
    // `fuller`/`steady`/`thinner` scored 667 against `filter` and `study`, and is
    // `richer`/`unchanged`/`poorer`.
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
            // `make` (recall) vs a way's walk count, at 600.
            ("marks", "make"),
            // `walk` (follow) vs a way with no way through, at 750 — the worst
            // of the three, because both are words a player uses about the same
            // screen.
            ("wall", "walk"),
        ],
        "the readings scoring against a typed word have changed",
    );
}

#[test]
fn a_verb_word_never_fuzzes_into_a_noun() {
    // **§19's `gained` leak, closed as a class.** A reading is a
    // `NounKind::Sense`, which `NounKind::Any` reaches, so a destructive verb
    // could name one from any room in the tower — and `walk`, `edit` and `make`
    // all scored high enough to get there by typo. `purge walk` echoed `purge
    // wall` and answered *"there is no wall within reach"* to a player who typed
    // a word the game taught them.
    //
    // `Scene::knowing` holds every verb word now, so each of these falls through
    // to the numbered prompt exactly as `purge grind` does.
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
}

#[test]
fn the_manual_answers_the_word_that_was_typed() {
    // The other half of the same leak, and the worse one: `recall edit` is a
    // player asking about the spell editor and it explained the *maze's way
    // out*. Every one-word synonym is a `NounKind::Command` now, so a page is
    // reachable by whichever word they know.
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
    // **The affordance `knowing` must not eat, found by breaking it.** `check` is
    // one of `verify`'s words, so once every verb word joined the known set a
    // spell called `check.spell` stopped being reachable by `invoke check` —
    // six tests went red at once, all of them on a player's own file name losing
    // to a word they never typed.
    //
    // Prefixing is not fuzzing: `check` *starts* `check.spell`, where `walk` does
    // not start `wall`. See `Scene::candidates`.
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
    // **Two readings colliding inside one domain is worse than a verb
    // near-miss**, which is what rejected `warmer`/`even`/`cooler` for the ward's
    // second delta: `cooler` scores 667 against `closer` and `even` 600 against
    // `level`, so a spell's author could not tell which channel they had asked.
    // `thicker`/`thinner` fails the same way at 715. They are
    // `richer`/`unchanged`/`poorer`.
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
