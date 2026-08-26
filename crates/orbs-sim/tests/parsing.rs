//! The parser, exercised the way a player would.
//!
//! DESIGN.md §6 is the spec and its worked examples are the fixtures. The Phase 0
//! exit gate measures this system and nothing else, so these tests are written as
//! claims about player experience rather than about functions.

use orbs_sim::parser::{Confidence, Mode, NounKind, Register, Resolution, Scene, Verb, resolve};

/// The slice's world: two starting domains, thin (DESIGN.md §15).
/// A scene standing **nowhere in particular**, offering no fixture's verb.
///
/// Load-bearing for `the_retired_brewing_words_all_still_land_somewhere_deliberate`,
/// which needs `mix` and `distil` out of scope to reach `Resolution::Elsewhere`. Use
/// [`anywhere`] for a test about phrasing rather than about scope.
fn tower() -> Scene {
    Scene::new()
        .with(NounKind::Place, "/tower/laboratory")
        .with(NounKind::Place, "/tower/archive")
        .with(NounKind::Place, "/tower/sanctum")
        .with(NounKind::File, "feed.log")
        .with(NounKind::File, "purge_cycle.log")
        .with(NounKind::Essence, "clarity")
        .with(NounKind::Essence, "warding")
        // Recipes are `Topic` nouns beside their `Essence` (§6.1), which is what
        // lets `make a potion of clarity` answer with the recipe now that
        // `decoct` is retired.
        .with(NounKind::Topic, "clarity")
        .with(NounKind::Topic, "warding")
        .with(NounKind::Reagent, "sage")
        .with(NounKind::Vessel, "retort")
        .with(NounKind::Script, "night_watch")
        .with(NounKind::Scroll, "gleaning-scroll")
        .with(NounKind::Topic, "brewing")
        .with(NounKind::Any, "sludge")
}

/// [`tower`], plus every fixture-anchored verb in scope at once.
///
/// `Scene::offers` asks [`Verb::anchor`], so a bare [`tower`] scopes out
/// `research`, `follow`, `wander` and the five laboratory operations. A test about
/// *phrasing* reaching a verb must not also be measuring which room the verb lives
/// in — that is what `every_verb_is_reachable_from_plain_english` was accidentally
/// doing when `study` came back `Elsewhere { Research }`.
fn anywhere() -> Scene {
    Verb::ALL
        .into_iter()
        .filter_map(Verb::anchor)
        .fold(tower(), Scene::offering)
}

fn echo(input: &str) -> String {
    match resolve(input, &tower(), Mode::Calm) {
        Resolution::Resolved { intent, .. } => intent.echo(),
        other => panic!("{input:?} did not resolve: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Three registers in, one register out (§6)
// ---------------------------------------------------------------------------

#[test]
fn the_designs_worked_examples_resolve() {
    // Verbatim from §6. `make a potion of clarity` now answers with the recipe:
    // §15 chose brewing to gate the parser because a shell-naive tester
    // understands the phrase, which requires it to resolve somewhere *useful* —
    // not that a verb exist to satisfy it. `decoct` is retired (§19).
    assert_eq!(echo("make a potion of clarity"), "recall clarity");
    assert_eq!(echo("grep march feed.log"), "sift march feed.log");
}

#[test]
fn the_retired_brewing_words_all_still_land_somewhere_deliberate() {
    // The Phase 0 naming pass's rule: a released word does not stop resolving,
    // it resolves to whatever it is nearest. These three stay claimed.
    //
    // `mix` and `distil` **left** this set: §10.1 gave them to the flask and the
    // alembic, where they name a real operation rather than a retired one. That
    // is not a release — they are more firmly claimed than before — but they are
    // claimed *by a domain*, so out of the laboratory they resolve to
    // `Elsewhere` rather than to the manual. `there is nothing here to mix with`
    // is a better answer than a recipe nobody asked for.
    for input in ["decoct clarity", "brew clarity", "make clarity"] {
        assert_eq!(echo(input), "recall clarity", "{input:?}");
    }

    // ...and this scene stands nowhere in particular, so the two that left are
    // out of scope entirely. The word is still recognised, which is the whole
    // point of `Resolution::Elsewhere`.
    for input in ["mix clarity", "distil clarity"] {
        assert!(
            matches!(
                resolve(input, &tower(), Mode::Calm),
                Resolution::Elsewhere { .. }
            ),
            "{input:?} should be a verb this place does not answer to"
        );
    }
}

#[test]
fn all_three_registers_reach_the_same_canonical_command() {
    for input in ["survey", "ls", "look"] {
        assert_eq!(echo(input), "survey", "{input:?}");
    }
    for input in ["attend laboratory", "cd laboratory", "go to the laboratory"] {
        assert_eq!(echo(input), "attend laboratory", "{input:?}");
    }
    for input in ["peruse feed.log", "cat feed.log", "read the feed.log"] {
        assert_eq!(echo(input), "peruse feed.log", "{input:?}");
    }
}

// ---------------------------------------------------------------------------
// What a slot accepts — `peruse` reads text, and only text
// ---------------------------------------------------------------------------

#[test]
fn a_spell_reads_back_like_any_other_file() {
    // `peruse` takes `NounKind::Readable`, which is `File` *or* `Script`. Until
    // it did, a `.spell` registered as a `Script` was unreadable: `best_match`
    // filters on the slot's kind, so the only noun kind `peruse` could see was
    // `File`, and the spell you had just written could not be read back.
    assert_eq!(echo("peruse night_watch"), "peruse night_watch");
    assert_eq!(echo("cat night_watch"), "peruse night_watch");
}

#[test]
fn peruse_refuses_things_that_are_not_text() {
    // **The guard, and the reason `Readable` exists rather than `Any`.**
    //
    // Widening `peruse` to `NounKind::Any` also makes a spell readable, passes
    // the whole suite, and is wrong: every noun in the scene becomes something
    // to read. `peruse sage` resolves at full confidence and reports a
    // zero-line read of a reagent — the symptom `execute::files` documents as
    // the reason resolution goes by kind in the first place.
    //
    // It lands where it costs most. `peruse` owns `read`, `cat`, `open` and
    // `show`, so it is the verb a shell-naive tester reaches for earliest, and
    // §15 weighs the dead-end rate above the raw resolution rate.
    for input in ["peruse sage", "read the sage", "open retort", "cat clarity"] {
        let resolution = resolve(input, &tower(), Mode::Calm);
        let read_it = matches!(
            &resolution,
            Resolution::Resolved { intent, .. } if intent.verb == Verb::Peruse,
        );
        assert!(!read_it, "{input:?} was read as a file: {resolution:?}");
    }
}

#[test]
fn a_slot_offers_only_what_could_fill_it() {
    // The matcher, the numbered prompt and Tab completion each asked "does this
    // noun fit this slot?" in their own words, so a slot kind meaning *a set*
    // could be understood by one and not the others. `NounKind::accepts` is now
    // the single answer; this is the surface where a disagreement would show.
    //
    // A bare `peruse` must offer files to read. Under `Any` it offered the
    // three places instead — `1. peruse /tower/laboratory` — which is not a
    // question a player can answer usefully.
    let resolution = resolve("peruse", &tower(), Mode::Calm);
    let offered: Vec<NounKind> = match &resolution {
        Resolution::Resolved { intent, .. } => intent.arguments.iter().map(|a| a.kind).collect(),
        Resolution::Ambiguous { candidates } => candidates
            .iter()
            .flat_map(|candidate| candidate.intent.arguments.iter())
            .map(|a| a.kind)
            .collect(),
        Resolution::Incomplete { filled, .. } => filled.iter().map(|a| a.kind).collect(),
        other => panic!("a bare `peruse` went nowhere useful: {other:?}"),
    };

    assert!(
        !offered.is_empty(),
        "a bare `peruse` named nothing at all, so this test proves nothing: {resolution:?}",
    );
    for kind in offered {
        assert!(
            NounKind::Readable.accepts(kind),
            "a bare `peruse` offered a {kind:?}, which has no text in it",
        );
    }
}

#[test]
fn the_echo_is_always_arcane() {
    // §6: whichever register is canonical is the one players absorb, so a shell
    // native typing `rm` must still be shown `purge`.
    assert_eq!(echo("rm sludge"), "purge sludge");
    assert_eq!(echo("man brewing"), "recall brewing");
    assert_eq!(echo("run night_watch"), "invoke night_watch");
    assert_eq!(echo("cron night_watch"), "bind night_watch");
}

#[test]
fn every_verb_is_reachable_from_plain_english() {
    // Half the Phase 0 gate's testers self-report no shell experience.
    let plain = [
        ("go to laboratory", Verb::Attend),
        ("look", Verb::Survey),
        ("read feed.log", Verb::Peruse),
        ("search march feed.log", Verb::Sift),
        ("how are things", Verb::Status),
        ("explain brewing", Verb::Recall),
        ("inspect night_watch", Verb::Verify),
        ("take it back", Verb::Undo),
        ("rest 30", Verb::Meditate),
        ("transfer sage to laboratory", Verb::Move),
        ("use laboratory", Verb::Wield),
        ("cancel laboratory", Verb::Stop),
        // `collect` was `siphon`'s. It moved to `empty` when `siphon` retired
        // (§19), because a released word does not stop resolving — it resolves
        // to whatever it is nearest, and the two nearest here are `purge` and
        // `stop`, the pair this room can least afford to confuse.
        ("collect laboratory", Verb::Empty),
        ("get rid of sludge", Verb::Purge),
        // `research` takes no argument at all now — it opens the stacks on the
        // one lectern (§19) — so what this pins is the *word*, which is all it
        // ever pinned: `study` has to reach `research` and nothing else.
        ("study", Verb::Research),
        ("author night_watch", Verb::Scribe),
        ("schedule night_watch", Verb::Bind),
        ("cast night_watch", Verb::Invoke),
    ];

    for (input, expected) in plain {
        let resolution = resolve(input, &anywhere(), Mode::Calm);
        let intent = resolution
            .intent()
            .unwrap_or_else(|| panic!("{input:?} did not resolve: {resolution:?}"));
        assert_eq!(intent.verb, expected, "{input:?}");
    }
}

#[test]
fn the_register_the_player_used_is_recorded() {
    // The gate needs to know which dialect newcomers actually reach for.
    let cases = [
        ("survey", Register::Arcane),
        ("ls", Register::Shell),
        ("look", Register::Plain),
    ];
    for (input, expected) in cases {
        let resolution = resolve(input, &tower(), Mode::Calm);
        assert_eq!(
            resolution.intent().expect("resolves").register,
            expected,
            "{input:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Forgiveness
// ---------------------------------------------------------------------------

#[test]
fn typos_resolve() {
    assert_eq!(echo("brew clarty"), "recall clarity");
    assert_eq!(echo("survy"), "survey");
    assert_eq!(echo("invok night_watch"), "invoke night_watch");
}

#[test]
fn abbreviations_resolve() {
    assert_eq!(echo("sur"), "survey");
    // Was `grim brewing`, when the manual was called `grimoire`. That
    // abbreviation now reaches **`grind`** — correctly, and `Elsewhere` says so
    // rather than guessing — which is the clash the rename removed.
    assert_eq!(echo("reca brewing"), "recall brewing");
}

#[test]
fn filler_is_ignored() {
    assert_eq!(echo("please go to the laboratory"), "attend laboratory");
    assert_eq!(echo("brew me a potion of warding"), "recall warding");
}

#[test]
fn a_place_can_be_named_by_its_leaf_or_its_path() {
    // Both forms are accepted, and **both echo the leaf** — the echo teaches one
    // canonical form, and §7's is the one players say. The full path clipped the
    // destination off a three-argument `move` at the 80×22 floor.
    assert_eq!(echo("attend laboratory"), "attend laboratory");
    assert_eq!(echo("attend /tower/laboratory"), "attend laboratory");
}

// ---------------------------------------------------------------------------
// Disambiguation (§6)
// ---------------------------------------------------------------------------

#[test]
fn a_missing_argument_becomes_a_numbered_prompt() {
    // §6's worked example: a bare verb yields a numbered list of what could
    // fill it, rather than an error.
    //
    // **The fixture was a bare `brew`**, a `recall` synonym, until `recall`'s
    // slot became optional so that `help` could list the vocabulary rather than
    // ask a lost player to pick between four arbitrary subjects (§19,
    // `TOPIC_OPTIONAL`). `purge` is the fixture now — a required
    // `NounKind::Any` — and what is under test is unchanged: a verb that *needs*
    // an argument asks for one instead of failing.
    let resolution = resolve("purge", &tower(), Mode::Calm);
    let Resolution::Ambiguous { candidates } = resolution else {
        panic!("expected a prompt, got {resolution:?}");
    };

    let offered: Vec<String> = candidates.iter().map(|c| c.intent.echo()).collect();
    assert!(offered.contains(&"purge archive".to_owned()), "{offered:?}");
}

#[test]
fn disambiguation_never_blocks_during_a_siege() {
    // §6: a modal wait would make ambiguous phrasing cost siege time, which is
    // exactly the typing pressure §14 forbids.
    let resolution = resolve("purge", &tower(), Mode::Siege);
    let Resolution::Resolved { intent, confidence } = resolution else {
        panic!("a siege must never block: {resolution:?}");
    };

    assert_eq!(
        confidence,
        Confidence::Forced,
        "the echo must offer correction"
    );
    assert_eq!(intent.verb, Verb::Purge);
}

#[test]
fn a_clear_winner_is_not_second_guessed() {
    let resolution = resolve("wield laboratory", &tower(), Mode::Calm);
    let Resolution::Resolved { confidence, .. } = resolution else {
        panic!("expected a clean resolution: {resolution:?}");
    };
    assert_eq!(confidence, Confidence::Clear);
}

// ---------------------------------------------------------------------------
// Failure is never bare (§6)
// ---------------------------------------------------------------------------

#[test]
fn nonsense_yields_suggestions_rather_than_an_error() {
    let resolution = resolve("xyzzy plugh", &tower(), Mode::Calm);
    let Resolution::Unresolved { suggestions } = resolution else {
        panic!("expected suggestions, got {resolution:?}");
    };
    assert!(!suggestions.is_empty(), "never a bare error");
}

#[test]
fn empty_input_is_harmless() {
    assert!(matches!(
        resolve("   ", &tower(), Mode::Calm),
        Resolution::Unresolved { .. }
    ));
}

#[test]
fn a_command_naming_something_that_does_not_exist_does_not_invent_it() {
    // "haste" is not a researched essence in this scene. Resolving it anyway
    // would promise a brew the sim cannot perform.
    let resolution = resolve("decoct haste", &tower(), Mode::Calm);
    if let Some(intent) = resolution.intent() {
        assert_ne!(intent.echo(), "decoct haste", "invented an essence");
    }
}

// ---------------------------------------------------------------------------
// Requirements (§6)
// ---------------------------------------------------------------------------

#[test]
fn resolution_is_deterministic() {
    // Replay, offline/online parity, and the balance harness all reduce to this.
    let inputs = [
        "make a potion of clarity",
        "brew",
        "xyzzy",
        "grep march feed.log",
        "go to the laboratory",
    ];
    for input in inputs {
        let first = resolve(input, &tower(), Mode::Calm);
        for _ in 0..8 {
            assert_eq!(resolve(input, &tower(), Mode::Calm), first, "{input:?}");
        }
    }
}

#[test]
fn resolution_is_comfortably_sub_millisecond() {
    // §6: "Never blocks the frame. Sub-millisecond."
    let scene = tower();
    let inputs = [
        "make a potion of clarity",
        "grep march feed.log",
        "go to the sanctum",
        "xyzzy plugh",
    ];

    let start = std::time::Instant::now();
    let rounds = 200;
    for _ in 0..rounds {
        for input in inputs {
            let _ = resolve(input, &scene, Mode::Calm);
        }
    }
    let each = start.elapsed() / (rounds * u32::try_from(inputs.len()).expect("small"));

    assert!(
        each < std::time::Duration::from_millis(1),
        "a parse took {each:?}, which would block the frame"
    );
}
