//! The parser, exercised the way a player would.
//!
//! DESIGN.md §6 is the spec and its worked examples are the fixtures. The Phase 0
//! exit gate measures this system and nothing else, so these tests are written as
//! claims about player experience rather than about functions.

use orbs_sim::parser::{
    Confidence, Mode, NounKind, Register, Resolution, Scene, Verb, analyse, resolve,
};

/// The slice's world: two starting domains, thin (DESIGN.md §15).
///
/// It stands nowhere in particular and offers no fixture's verb, which is what
/// `the_retired_brewing_words_all_still_land_somewhere_deliberate` needs to
/// reach `Resolution::Elsewhere`. Use [`anywhere`] for a test about phrasing.
fn tower() -> Scene {
    Scene::new()
        .with(NounKind::Place, "/tower/laboratory")
        .with(NounKind::Place, "/tower/archive")
        .with(NounKind::Place, "/tower/sanctum")
        .with(NounKind::File, "feed.log")
        .with(NounKind::File, "purge_cycle.log")
        .with(NounKind::Essence, "clarity")
        .with(NounKind::Essence, "warding")
        // Recipes are `Topic` nouns beside their `Essence` (§6.1), so `make a
        // potion of clarity` answers with the recipe.
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
/// A bare [`tower`] scopes out `research`, `follow`, `wander` and the laboratory
/// operations, so a test about phrasing would also be measuring which room a verb
/// lives in — `every_verb_is_reachable_from_plain_english` had `study` coming back
/// `Elsewhere { Research }`.
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
    // Verbatim from §6. `make a potion of clarity` answers with the recipe
    // because §15 needs the phrase to resolve somewhere useful, not because a
    // verb exists to satisfy it — `decoct` is retired (§19).
    assert_eq!(echo("make a potion of clarity"), "recall clarity");
    assert_eq!(echo("grep march feed.log"), "sift march feed.log");
}

#[test]
fn the_retired_brewing_words_all_still_land_somewhere_deliberate() {
    // The Phase 0 naming pass's rule: a released word does not stop resolving,
    // it resolves to whatever it is nearest. These three stay claimed.
    //
    // `mix` and `distil` left the set — §10.1 gave them to the flask and the
    // alembic, so out of the laboratory they reach `Elsewhere` rather than the
    // manual. A better answer than a recipe nobody asked for.
    for input in ["decoct clarity", "brew clarity", "make clarity"] {
        assert_eq!(echo(input), "recall clarity", "{input:?}");
    }

    // ...and this scene stands nowhere in particular, so the two that left are
    // out of scope — still recognised, which is `Resolution::Elsewhere`.
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
    // it did, `best_match` filtered to `File` and the spell you had just
    // written could not be read back.
    assert_eq!(echo("peruse night_watch"), "peruse night_watch");
    assert_eq!(echo("cat night_watch"), "peruse night_watch");
}

#[test]
fn peruse_refuses_things_that_are_not_text() {
    // Why `Readable` rather than `Any`: widening `peruse` makes a spell
    // readable and passes the whole suite, but `peruse sage` then resolves at
    // full confidence and reports a zero-line read of a reagent. `peruse` owns
    // `read`, `cat`, `open` and `show`, so it is where a shell-naive tester
    // lands earliest, and §15 weighs dead ends above resolution rate.
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
    // The matcher, the numbered prompt and Tab completion each answered "does
    // this noun fit this slot?" in their own words; `NounKind::accepts` is the
    // single answer now, and this is where a disagreement would show. Under
    // `Any` a bare `peruse` offered places — `1. peruse /tower/laboratory`.
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
        // `collect` was `siphon`'s and moved to `empty` when `siphon` retired
        // (§19); the other two nearest are `purge` and `stop`, the pair this
        // room can least afford to confuse.
        ("collect laboratory", Verb::Empty),
        ("get rid of sludge", Verb::Purge),
        // `research` takes no argument now (§19), so what this pins is the
        // word: `study` has to reach `research` and nothing else.
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
    // Was `grim brewing`, when the manual was called `grimoire`; that
    // abbreviation reaches `grind` now, which is the clash the rename removed.
    assert_eq!(echo("reca brewing"), "recall brewing");
}

#[test]
fn filler_is_ignored() {
    assert_eq!(echo("please go to the laboratory"), "attend laboratory");
    assert_eq!(echo("brew me a potion of warding"), "recall warding");
}

#[test]
fn a_place_can_be_named_by_its_leaf_or_its_path() {
    // Both forms are accepted and both echo the leaf: the echo teaches one
    // canonical form (§7), and the full path clipped the destination off a
    // three-argument `move` at the 80×22 floor.
    assert_eq!(echo("attend laboratory"), "attend laboratory");
    assert_eq!(echo("attend /tower/laboratory"), "attend laboratory");
}

// ---------------------------------------------------------------------------
// Disambiguation (§6)
// ---------------------------------------------------------------------------

#[test]
fn a_missing_argument_becomes_a_numbered_prompt() {
    // §6's worked example: a bare verb yields a numbered list of what could
    // fill it, rather than an error. The fixture was a bare `brew` until
    // `recall`'s slot became optional (§19, `TOPIC_OPTIONAL`); `purge` is a
    // required `NounKind::Any` and tests the same thing.
    let resolution = resolve("purge", &tower(), Mode::Calm);
    let Resolution::Ambiguous { candidates } = resolution else {
        panic!("expected a prompt, got {resolution:?}");
    };

    let offered: Vec<String> = candidates.iter().map(|c| c.intent.echo()).collect();
    assert!(offered.contains(&"purge archive".to_owned()), "{offered:?}");
}

#[test]
fn disambiguation_never_blocks_during_a_siege() {
    // §6: a modal wait would make ambiguous phrasing cost siege time, the
    // typing pressure §14 forbids.
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

// ---------------------------------------------------------------------------
// The augury's router — which lines the deterministic pipeline keeps (§6).
//
// `Analysis::reads_outright` decides what a trained model never sees. Written
// as claims about phrasing rather than scores: the question is whether the orb
// understood the sentence, and the scores are only how that is measured.
// ---------------------------------------------------------------------------

#[test]
fn a_command_typed_properly_is_read_outright() {
    // The mastery arc (§6) is players graduating to the canonical form. It
    // cannot depend on a model, so none of these may reach one.
    for input in [
        "survey",
        "attend laboratory",
        "sift march feed.log",
        "peruse feed.log",
        "recall brewing",
        "ls",
        "go to the laboratory",
        "please rm sludge",
    ] {
        assert!(
            analyse(input, &anywhere(), Mode::Calm).reads_outright(),
            "{input:?} would be handed to the augury"
        );
    }
}

#[test]
fn a_typo_in_an_argument_is_still_read_outright() {
    // `clarty` reaches `clarity` at 819, and the matcher beats any model
    // trained on phrasings at that — while §19's *"the player this game is
    // built for"* types exactly this.
    for input in ["recall clarty", "attend labratory", "sift march fed.log"] {
        assert!(
            analyse(input, &anywhere(), Mode::Calm).reads_outright(),
            "{input:?} would be handed to the augury"
        );
    }
}

#[test]
fn a_sentence_is_not_read_outright_even_when_it_opens_on_a_verb() {
    // The defect the router exists to avoid: `put`, `take` and `make` are all
    // claimed, so a rule asking only whether the head named a verb would run a
    // sentence as a one-word command with the rest left over.
    for input in [
        "put the sage in the mortar and grind it",
        "take the husks out and throw them away",
        "make the sage into a powder for me",
    ] {
        assert!(
            !analyse(input, &anywhere(), Mode::Calm).reads_outright(),
            "{input:?} was read outright"
        );
    }
}

#[test]
fn the_phrasings_the_augury_exists_for_reach_it() {
    for input in [
        "turn the sage into powder",
        "smash the sage",
        "i need some powdered sage",
        "what should i be doing",
    ] {
        assert!(
            !analyse(input, &anywhere(), Mode::Calm).reads_outright(),
            "{input:?} was read outright"
        );
    }
}

#[test]
fn a_deliberate_refusal_is_read_outright() {
    // `Elsewhere` and `InSpell` exist because *"I do not know that word"* would
    // lie about a word the game taught (§19). Handing either to a model trades
    // a good refusal for a guess.
    let elsewhere = analyse("mix", &tower(), Mode::Calm);
    assert!(matches!(elsewhere.resolution, Resolution::Elsewhere { .. }));
    assert!(elsewhere.reads_outright());

    let in_spell = analyse("repeat 3", &tower(), Mode::Calm);
    assert!(matches!(in_spell.resolution, Resolution::InSpell { .. }));
    assert!(in_spell.reads_outright());
}

#[test]
fn a_leftover_word_is_what_separates_a_sentence_from_a_command() {
    // The two halves of the rule, pinned apart. Same verb, same exactness; only
    // the unexplained words differ, and only they change the answer.
    let command = analyse("attend laboratory", &anywhere(), Mode::Calm);
    let sentence = analyse(
        "attend laboratory and then start the mortar",
        &anywhere(),
        Mode::Calm,
    );

    let best = |analysis: &orbs_sim::parser::Analysis| {
        analysis.candidates.first().expect("a reading").clone()
    };
    assert_eq!(best(&command).leftover, 0);
    assert!(best(&sentence).leftover > 0);
    assert!(command.reads_outright());
    assert!(!sentence.reads_outright());
}

#[test]
fn a_verb_that_takes_nothing_names_the_words_it_could_not_use() {
    // `Incomplete`'s other half (§19): it names the slot it waits for, and
    // these verbs have none — so `status gibberish` ran with the word thrown
    // away.
    for (input, verb) in [
        ("status gibberish", Verb::Status),
        ("undo gibberish", Verb::Undo),
    ] {
        let analysis = analyse(input, &anywhere(), Mode::Calm);
        match &analysis.resolution {
            Resolution::TakesNothing {
                verb: named, extra, ..
            } => {
                assert_eq!(*named, verb, "{input:?}");
                assert_eq!(extra, "gibberish", "{input:?}");
            }
            other => panic!("{input:?} ran, or refused the wrong way: {other:?}"),
        }
        // A sentence rather than a command, so a reader sees it first.
        assert!(!analysis.reads_outright(), "{input:?} was read outright");
    }
    // ...and the verb alone, or with only filler after it, still runs.
    assert_eq!(echo("status"), "status");
    assert_eq!(echo("status please"), "status");
}

#[test]
fn a_verb_that_takes_nothing_may_still_name_where_it_acts() {
    // A verb with no slot names nothing but where it acts, and there is one of
    // each — so the place is not a word thrown away. `light athanor`'s
    // exemption, without its operation test.
    for input in ["wander archive", "research archive"] {
        assert!(
            matches!(
                analyse(input, &anywhere(), Mode::Calm).resolution,
                Resolution::Resolved { .. }
            ),
            "{input:?} did not run",
        );
    }
}

#[test]
fn a_reading_that_uses_every_word_runs_before_one_that_leaves_some() {
    // `Sim::submit_reading` took the first reading that resolved, so one that
    // left a word unused beat one behind it that used them all — *"stir the
    // alembic"* ran as `distil alembic`.
    let scene = orbs_sim::content::corpus_scene();
    let readings = ["distil alembic".to_owned(), "survey alembic".to_owned()];
    assert_eq!(
        orbs_sim::parser::reading_to_run(&readings, &scene, Mode::Calm).map(String::as_str),
        Some("survey alembic"),
    );
    // ...and with nothing cleaner behind it, the first that resolves still runs.
    assert_eq!(
        orbs_sim::parser::reading_to_run(&readings[..1], &scene, Mode::Calm).map(String::as_str),
        Some("distil alembic"),
    );
}

#[test]
fn a_verb_that_takes_nothing_refuses_though_another_verb_answered_first() {
    // One `Incomplete` was kept for the line, not one per verb: `verify`'s
    // `audit` scores 600 against `quit` and sits above it, so its answer
    // claimed the only slot and the exactly-typed `quit` below ran bare.
    // `decode` reaches `research` past `recall`'s `decoct` the same way.
    for (input, verb) in [
        ("quit gibberish", Verb::Quit),
        ("decode gibberish", Verb::Research),
    ] {
        match analyse(input, &anywhere(), Mode::Calm).resolution {
            Resolution::TakesNothing {
                verb: named, extra, ..
            } => {
                assert_eq!(named, verb, "{input:?}");
                assert_eq!(extra, "gibberish", "{input:?}");
            }
            other => panic!("{input:?} ran, or refused the wrong way: {other:?}"),
        }
    }
}

#[test]
fn only_a_verb_that_acts_somewhere_may_name_a_place() {
    // The exemption above is for a verb a fixture declares — `wander` at the
    // stacks — which has one place to name and no slot to name it in. The whole
    // tower answers to `status`, `quit` and `undo`, so a place after one of
    // those is discarded like any other word. And `best_match` tries each word,
    // so `undo laboratory move` used to pass as a place named in the tail.
    for input in [
        "status laboratory",
        "quit laboratory",
        "undo laboratory move",
    ] {
        assert!(
            matches!(
                analyse(input, &anywhere(), Mode::Calm).resolution,
                Resolution::TakesNothing { .. }
            ),
            "{input:?} ran with its words thrown away",
        );
    }
}

#[test]
fn a_verb_the_whole_tower_answers_to_may_name_the_tower() {
    // `status` reports on every room at once, so the tower is the one place it
    // can name — *"overview of the tower"* was refused as a word thrown away.
    // A room is narrower, and still refused.
    let scene = anywhere().with(NounKind::Place, "/tower");
    for input in ["overview of the tower", "status tower"] {
        assert!(
            matches!(
                analyse(input, &scene, Mode::Calm).resolution,
                Resolution::Resolved { .. }
            ),
            "{input:?} was refused",
        );
    }
    assert!(matches!(
        analyse("status laboratory", &scene, Mode::Calm).resolution,
        Resolution::TakesNothing { .. }
    ));
}

#[test]
fn punctuation_is_not_a_word_a_verb_was_handed() {
    // `fold` sheds a trailing stop but keeps a token that is nothing else
    // whole, because `?` and `./` are synonyms — so the stop in `status .`
    // arrived as a word said and a line that had always run was refused.
    for input in ["status .", "status ?", "undo !"] {
        assert!(
            matches!(
                analyse(input, &anywhere(), Mode::Calm).resolution,
                Resolution::Resolved { .. }
            ),
            "{input:?} was refused over its punctuation",
        );
    }
}

#[test]
fn a_plain_phrase_may_run_past_its_command_and_one_plain_word_may_not() {
    // *"how are things"* is the plain synonym and the word after it is how
    // people talk; refusing that teaches nothing. One plain word is not a
    // sentence, so `decode gibberish` above is still a word thrown away.
    for input in ["how are things going", "how are things today"] {
        assert!(
            matches!(
                analyse(input, &anywhere(), Mode::Calm).resolution,
                Resolution::Resolved { .. }
            ),
            "{input:?} was refused",
        );
    }
}

#[test]
fn a_siege_runs_a_verb_that_takes_nothing_rather_than_refusing_it() {
    // §6 gives the mode its own answer to ambiguity, and a refusal costs the
    // same turn. `muster the troops` is what a player types with the wall
    // coming down; calm, it is still told.
    let scene = anywhere();
    assert!(
        matches!(
            resolve("muster the troops", &scene, Mode::Siege),
            Resolution::Resolved { .. }
        ),
        "a siege spent a turn refusing the words after a verb",
    );
    assert!(matches!(
        resolve("muster the troops", &scene, Mode::Calm),
        Resolution::TakesNothing { .. }
    ));
}

#[test]
fn a_reading_with_no_argument_does_not_win_by_having_nothing_left_over() {
    // A bare verb accounts for every word it was handed by being handed none,
    // so preferring the reading that leaves nothing over handed it the line —
    // `[grind sage now, quit]` ran `quit`.
    let scene = orbs_sim::content::corpus_scene();
    let readings = ["grind sage now".to_owned(), "quit".to_owned()];
    assert_eq!(
        orbs_sim::parser::reading_to_run(&readings, &scene, Mode::Calm).map(String::as_str),
        Some("grind sage now"),
    );
}

#[test]
fn a_first_choice_that_used_every_word_is_not_jumped() {
    // The half a first fix broke: refusing a bare reading the shortcut also
    // stopped it keeping first place, so *"open the loom"* came back `survey
    // loom` over the `weave` the reader ranked first.
    let scene = orbs_sim::content::corpus_scene();
    let readings = ["status".to_owned(), "survey laboratory".to_owned()];
    assert_eq!(
        orbs_sim::parser::reading_to_run(&readings, &scene, Mode::Calm).map(String::as_str),
        Some("status"),
    );
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
