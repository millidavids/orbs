//! The parser, exercised the way a player would.
//!
//! DESIGN.md §6 is the spec and its worked examples are the fixtures. The Phase 0
//! exit gate measures this system and nothing else, so these tests are written as
//! claims about player experience rather than about functions.

use orbs_sim::parser::{Confidence, Mode, NounKind, Register, Resolution, Scene, Verb, resolve};

/// The slice's world: two starting domains, thin (DESIGN.md §15).
fn tower() -> Scene {
    Scene::new()
        .with(NounKind::Place, "/tower/alembic")
        .with(NounKind::Place, "/tower/archive")
        .with(NounKind::Place, "/tower/battlements")
        .with(NounKind::File, "feed.log")
        .with(NounKind::File, "purge_cycle.log")
        .with(NounKind::Essence, "clarity")
        .with(NounKind::Essence, "warding")
        .with(NounKind::Vessel, "alembic")
        .with(NounKind::Script, "night_watch")
        .with(NounKind::Fragment, "sigil-iv")
        .with(NounKind::Topic, "brewing")
        .with(NounKind::Any, "sludge")
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
    // Verbatim from §6.
    assert_eq!(echo("make a potion of clarity"), "decoct clarity");
    assert_eq!(echo("grep march feed.log"), "sift march feed.log");
}

#[test]
fn all_three_registers_reach_the_same_canonical_command() {
    for input in ["survey", "ls", "look"] {
        assert_eq!(echo(input), "survey", "{input:?}");
    }
    for input in ["attend alembic", "cd alembic", "go to the alembic"] {
        assert_eq!(echo(input), "attend /tower/alembic", "{input:?}");
    }
    for input in ["peruse feed.log", "cat feed.log", "read the feed.log"] {
        assert_eq!(echo(input), "peruse feed.log", "{input:?}");
    }
}

#[test]
fn the_echo_is_always_arcane() {
    // §6: whichever register is canonical is the one players absorb, so a shell
    // native typing `rm` must still be shown `purge`.
    assert_eq!(echo("rm sludge"), "purge sludge");
    assert_eq!(echo("man brewing"), "grimoire brewing");
    assert_eq!(echo("run night_watch"), "invoke night_watch");
    assert_eq!(echo("cron night_watch"), "bind night_watch");
}

#[test]
fn every_verb_is_reachable_from_plain_english() {
    // Half the Phase 0 gate's testers self-report no shell experience.
    let plain = [
        ("go to alembic", Verb::Attend),
        ("look", Verb::Survey),
        ("read feed.log", Verb::Peruse),
        ("search march feed.log", Verb::Sift),
        ("how are things", Verb::Status),
        ("explain brewing", Verb::Grimoire),
        ("inspect night_watch", Verb::Verify),
        ("take it back", Verb::Undo),
        ("rest 30", Verb::Meditate),
        ("brew clarity", Verb::Decoct),
        ("collect alembic", Verb::Decant),
        ("get rid of sludge", Verb::Purge),
        ("study sigil-iv", Verb::Decipher),
        ("write night_watch", Verb::Inscribe),
        ("schedule night_watch", Verb::Bind),
        ("cast night_watch", Verb::Invoke),
    ];

    for (input, expected) in plain {
        let resolution = resolve(input, &tower(), Mode::Calm);
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
    assert_eq!(echo("brew clarty"), "decoct clarity");
    assert_eq!(echo("survy"), "survey");
    assert_eq!(echo("invok night_watch"), "invoke night_watch");
}

#[test]
fn abbreviations_resolve() {
    assert_eq!(echo("sur"), "survey");
    assert_eq!(echo("grim brewing"), "grimoire brewing");
}

#[test]
fn filler_is_ignored() {
    assert_eq!(echo("please go to the alembic"), "attend /tower/alembic");
    assert_eq!(echo("brew me a potion of warding"), "decoct warding");
}

#[test]
fn a_place_can_be_named_by_its_leaf_or_its_path() {
    assert_eq!(echo("attend alembic"), "attend /tower/alembic");
    assert_eq!(echo("attend /tower/alembic"), "attend /tower/alembic");
}

// ---------------------------------------------------------------------------
// Disambiguation (§6)
// ---------------------------------------------------------------------------

#[test]
fn a_missing_argument_becomes_a_numbered_prompt() {
    // §6's worked example: "start potion" -> "brew --recipe=?" -> a numbered
    // list of what could fill it.
    let resolution = resolve("brew", &tower(), Mode::Calm);
    let Resolution::Ambiguous { candidates } = resolution else {
        panic!("expected a prompt, got {resolution:?}");
    };

    let offered: Vec<String> = candidates.iter().map(|c| c.intent.echo()).collect();
    assert!(
        offered.contains(&"decoct clarity".to_owned()),
        "{offered:?}"
    );
    assert!(
        offered.contains(&"decoct warding".to_owned()),
        "{offered:?}"
    );
}

#[test]
fn disambiguation_never_blocks_during_a_siege() {
    // §6: a modal wait would make ambiguous phrasing cost siege time, which is
    // exactly the typing pressure §14 forbids.
    let resolution = resolve("brew", &tower(), Mode::Siege);
    let Resolution::Resolved { intent, confidence } = resolution else {
        panic!("a siege must never block: {resolution:?}");
    };

    assert_eq!(
        confidence,
        Confidence::Forced,
        "the echo must offer correction"
    );
    assert_eq!(intent.verb, Verb::Decoct);
}

#[test]
fn a_clear_winner_is_not_second_guessed() {
    let resolution = resolve("decoct clarity", &tower(), Mode::Calm);
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
        "go to the alembic",
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
        "go to the battlements",
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
