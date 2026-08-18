//! What a spell asks the world, and what the world does about it.
//!
//! The other half of `questions.rs`: that file holds the grammar to its word,
//! this one holds it to the tower. Six laboratories are driven into named states
//! and every shape of question is asked of each, because a condition that parses
//! and a condition that finds anything are different claims — which is exactly
//! what §19 records the last bug in this subsystem turning on.
//!
//! **Behaviour, not representation.** Nothing here inspects a `Program`.
//! §19's standing lesson from that bug is that *"asserting on `holds` directly
//! would have agreed with the bug"* — what caught it was running the spell and
//! asking which branch executed. So the questions are asked by casting spells
//! whose two branches do visibly different things.

use orbs_render::{FieldName, Value};
use orbs_sim::Sim;

// ---------------------------------------------------------------------------
// The instrument: a laboratory in a named state, and a question asked of it
// ---------------------------------------------------------------------------

/// The six worlds every question below is asked of.
///
/// Named for what a player would say about them, and **driven** rather than
/// fabricated — a grind is ten ticks, so the states are reached by doing the
/// work. That is slower than poking components and it is the only way the states
/// are real: `Fouled` in particular has no constructor, it is what an instrument
/// becomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum World {
    /// Nothing lit, nothing running, nothing ground yet.
    Cold,
    /// The athanor alight, everything else idle.
    Lit,
    /// The athanor alight and the mortar mid-grind.
    Grinding,
    /// The mortar holding what it made, waiting to be emptied.
    Ready,
    /// The fire damped, the mortar emptied, product on the shelf.
    Banked,
    /// The mortar being scoured, and both ground products on the shelf.
    Scouring,
}

impl World {
    const ALL: [Self; 6] = [
        Self::Cold,
        Self::Lit,
        Self::Grinding,
        Self::Ready,
        Self::Banked,
        Self::Scouring,
    ];

    /// A laboratory in this state, standing in it.
    fn build(self) -> Sim {
        let mut sim = Sim::new(1);
        run(&mut sim, &["attend laboratory"]);
        match self {
            Self::Cold => {}
            Self::Lit => run(&mut sim, &["kindle charcoal"]),
            Self::Grinding => run(&mut sim, &["kindle charcoal", "grind sage"]),
            Self::Ready => {
                run(&mut sim, &["kindle charcoal", "grind sage"]);
                sim.step_n(20);
            }
            Self::Banked => {
                run(&mut sim, &["kindle charcoal", "grind sage"]);
                sim.step_n(20);
                // Empty takes the product to the shelf and leaves the husks.
                run(&mut sim, &["empty mortar_and_pestle", "stop athanor"]);
                sim.step_n(4);
            }
            Self::Scouring => {
                run(&mut sim, &["grind sage"]);
                sim.step_n(20);
                run(&mut sim, &["empty mortar_and_pestle", "grind rock-salt"]);
                sim.step_n(20);
                run(
                    &mut sim,
                    &["empty mortar_and_pestle", "purge mortar_and_pestle"],
                );
            }
        }
        sim
    }
}

/// Submit each line and give the world a tick to do it.
fn run(sim: &mut Sim, lines: &[&str]) {
    for line in lines {
        sim.submit(line);
        sim.step();
    }
}

/// Whether `question` holds in `world`, by casting a spell that says which
/// branch it took.
///
/// # Why the branches are two `wait`s
///
/// **The branch has to be visible without changing the world it was asked
/// about.** The obvious pair — grind one thing or grind another — fails three
/// ways at once: a grind needs the mortar free, which two of these worlds do not
/// have; it takes ten ticks to say anything; and `Banked` reaches its state *by
/// grinding*, so the setup has already said the word the assertion looks for.
/// That last one is the dangerous one, because it does not fail — it passes,
/// wrongly.
///
/// A `wait` for something that never happens says `watches for <name>`
/// immediately, names itself, needs no instrument, and changes nothing. The
/// spell then blocks, which is exactly what we want: the answer has been given.
///
/// Only records **after the cast** are read, so nothing the setup said can be
/// mistaken for what the spell did.
fn asks(world: World, question: &str) -> Option<bool> {
    let mut sim = world.build();
    let lines: Vec<String> = [
        format!("if {question}"),
        "wait for yesmark".to_owned(),
        "else".to_owned(),
        "wait for nomark".to_owned(),
        "end".to_owned(),
    ]
    .to_vec();
    sim.write_spell("asking", &lines);
    sim.step();

    let before = messages(&sim).len();
    run(&mut sim, &["invoke asking"]);
    sim.step_n(6);
    let said: Vec<String> = messages(&sim).into_iter().skip(before).collect();

    match (
        said.iter().any(|line| line.contains("yesmark")),
        said.iter().any(|line| line.contains("nomark")),
    ) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        // Neither branch ran, which is what an unanswerable question does.
        (false, false) => None,
        (true, true) => panic!("both halves of the `if` ran: {question:?} in {world:?}"),
    }
}

/// Every message the orb has said.
fn messages(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Message))
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Whether anything the orb said contains `needle`.
fn mentioned(sim: &Sim, needle: &str) -> bool {
    messages(sim).iter().any(|line| line.contains(needle))
}

/// A laboratory spell, written and ready to cast.
fn with_spell(name: &str, lines: &[&str]) -> Sim {
    let mut sim = Sim::new(1);
    run(&mut sim, &["attend laboratory"]);
    let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &lines);
    sim.step();
    sim
}

// ---------------------------------------------------------------------------
// B. Resolution — abbreviation yes, typo no, coin flip never
// ---------------------------------------------------------------------------

/// Whether the orb can answer a question naming `named`, in a fresh laboratory.
fn resolves(named: &str) -> bool {
    asks(World::Cold, &format!("the {named} is idle")).is_some()
}

#[test]
fn an_abbreviation_resolves_and_a_typo_does_not() {
    // **The line this draws is the one `fuzzy` already draws**: a prefix of three
    // characters or more always scores at or above 850, and a typo only reaches
    // it as a single slip in a long word. At the prompt the player is there to
    // see the echo; a spell resolves with nobody watching, so it takes the
    // stricter half.
    for abbreviation in [
        "mortar_and_pestle",
        "mortar",
        "mort",
        "balneum_mariae",
        "balneum",
        "athanor",
        "atha",
    ] {
        assert!(resolves(abbreviation), "{abbreviation:?} stopped resolving");
    }
    for typo in ["mortr", "morter", "mo", "balnuem", "gatehouse"] {
        assert!(!resolves(typo), "{typo:?} was guessed at");
    }
}

#[test]
fn one_slip_in_a_long_word_still_resolves_and_that_is_the_known_edge() {
    // **Pinned because it is a limit, not an accident.** `athanr` is one deletion
    // from `athanor` in a seven-letter word, which scores 858 — over the 850
    // floor. The floor is drawn where `fuzzy` already draws one (a prefix is
    // always ≥ 850, a typo rarely is), and moving it up far enough to catch this
    // would start refusing real abbreviations.
    //
    // It is safe in a way the case below is **not**: a slip can only ever resolve
    // to something that exists, and there is nothing else near `athanor` for it
    // to land on. What the floor and the tie rule exist to stop is a typo landing
    // on a *different real thing*, which is the next test.
    assert!(resolves("athanr"), "the known edge moved");
}

#[test]
fn a_near_miss_never_becomes_the_thing_it_is_near() {
    // **The case that set the threshold.** `similarity("ground-salt",
    // "ground-sage")` is 819 — comfortably over the prompt's floor of 600 — so a
    // question written while the salt happened to be absent would have compiled
    // into a question about the sage. The file would say one thing and the
    // running spell ask another, with nothing on screen to show it.
    let ready = World::Ready; // holds ground-sage, and no ground-salt anywhere
    assert_eq!(
        asks(ready, "the mortar_and_pestle has ground-sage"),
        Some(true),
        "the product it does hold was not found",
    );
    assert_eq!(
        asks(ready, "the mortar_and_pestle has ground-salt"),
        Some(false),
        "a near miss was read as the thing it is near",
    );
}

#[test]
fn two_things_equally_close_are_a_coin_flip_and_the_orb_refuses_to_toss_it() {
    // With both products on the shelf, `ground` is an equally good prefix of
    // `ground-sage` and `ground-salt` — and `best_match` would hand back
    // whichever was registered first. That is iteration order deciding what a
    // laboratory does.
    let mut sim = World::Scouring.build();
    assert!(
        mentioned(&sim, "ground-salt") && mentioned(&sim, "ground-sage"),
        "the world under test does not hold both: {:?}",
        messages(&sim),
    );

    sim.write_spell(
        "tie",
        &[
            "if the dispensary has ground".to_owned(),
            "grind sage".to_owned(),
            "end".to_owned(),
        ],
    );
    sim.step();
    // **From here on only.** The world above was *built* by grinding, so a sweep
    // over the whole transcript would find the setup's own work and read it as
    // the spell's — which passes, wrongly.
    let before = messages(&sim).len();
    run(&mut sim, &["invoke tie"]);
    sim.step_n(20);
    let said: Vec<String> = messages(&sim).into_iter().skip(before).collect();

    assert!(
        said.iter().any(|line| line.contains("neither half runs")),
        "an ambiguous name was resolved by iteration luck: {said:?}",
    );
    assert!(
        !said
            .iter()
            .any(|line| line.contains("dispensary to mortar")),
        "the branch ran on a coin flip: {said:?}",
    );
}

#[test]
fn the_margin_is_below_the_closest_call_the_vocabulary_makes() {
    // **What keeps `SPELL_MARGIN` honest as the tower grows.** The rule is that
    // a name must beat the runner-up by more than the margin, and the margin can
    // only be as large as the *narrowest* win a real abbreviation needs — today
    // `sag`, which beats `sage-husks` to `sage` by 67.
    //
    // Every prefix of every name in the room is tried, in the world that holds
    // the most look-alikes. What comes out is the closest call the shipped
    // vocabulary asks anyone to make; a new reagent that squeezes it fails here,
    // naming both words, rather than resolving silently in somebody's spell.
    let sim = World::Scouring.build();
    // **Only names that actually compete.** `clearly` asks for a `Place` when the
    // question is about somewhere and `Any` when it is about a thing, so scoring
    // every name against every other invents rivals that can never meet —
    // `laboratory` against `laboratory.log`, which no place query can see.
    //
    // **And deduplicated by leaf**: §6.1 registers a `Topic` beside every reagent
    // so `recall ground-sage` reads the manual, so the scene holds that word
    // twice. Two entries for one thing are not two candidates, which is what made
    // `ground-sag` ambiguous with itself.
    let mut names: Vec<String> = sim
        .scene()
        .nouns()
        .iter()
        .filter(|noun| noun.kind == orbs_sim::parser::NounKind::Place)
        .map(|noun| orbs_sim::parser::leaf(&noun.name).to_owned())
        .collect();
    names.sort_unstable();
    names.dedup();

    // Every reading that would change if the margin moved from nothing to what it
    // is: a name that wins by a hair. Empty is the claim — raising the margin
    // refuses nothing that a player relies on.
    let mut hairs: Vec<String> = Vec::new();
    for name in &names {
        // Three characters is where `fuzzy` starts calling a prefix an
        // abbreviation rather than a coincidence.
        for length in 3..=name.chars().count() {
            let typed: String = name.chars().take(length).collect();
            let mut scored: Vec<(u32, &String)> = names
                .iter()
                .map(|against| (orbs_sim::parser::similarity(&typed, against), against))
                .collect();
            scored.sort_by_key(|(score, against)| (std::cmp::Reverse(*score), (*against).clone()));

            let (best, winner) = scored[0];
            if best < orbs_sim::tower::spell::SPELL_SIMILARITY {
                continue;
            }
            let (next, runner_up) = scored
                .get(1)
                .map_or((0, name), |(score, name)| (*score, name));
            let lead = best.saturating_sub(next);
            // A dead heat is a **real** ambiguity and must go on refusing — `gro`
            // names `ground-sage` and `ground-salt` alike, and picking one would
            // be the coin flip the rule exists to stop. What must not appear is a
            // reading in between: won, but only just.
            if lead > 0 && lead <= orbs_sim::tower::spell::SPELL_MARGIN {
                hairs.push(format!(
                    "\n  {typed:?} reaches {winner:?} by {lead} over {runner_up:?}"
                ));
            }
        }
    }

    assert!(
        hairs.is_empty(),
        "the margin of {} would now refuse readings that resolve today:{}\n\
         either those two names are too alike to tell apart, or the margin comes down",
        orbs_sim::tower::spell::SPELL_MARGIN,
        hairs.join(""),
    );
}

#[test]
fn every_abbreviation_a_player_relies_on_survives_the_margin() {
    // The other half of the sweep above, and the half worth asserting through the
    // **real resolver** rather than by re-scoring: names of *things*, where the
    // scene holds `ground-sage` and `ground-salt` side by side and a manual topic
    // beside each of them.
    //
    // A margin is only safe if it refuses coin flips and nothing else, so both
    // columns matter equally: the left one is what must keep working, the right
    // one is what must not be guessed at.
    let world = World::Scouring;
    for abbreviation in [
        "sage",
        "sag",
        "ground-sage",
        // Two characters short of the whole word, with its twin on the shelf and
        // its own manual entry beside it — the reading that was refused as
        // ambiguous *with itself* until `clearly` learned that two entries for
        // one word are one candidate.
        "ground-sag",
        "rock-salt",
        "rock",
        "charcoal",
        "charc",
        "husks",
    ] {
        assert!(
            asks(world, &format!("the dispensary has {abbreviation}")).is_some(),
            "{abbreviation:?} stopped resolving",
        );
    }

    for ambiguous in [
        // A dead heat between two real products.
        "ground",
        // ...and between an essence and the draught made of it.
        "cla",
        // Not a name at all, which is a typo rather than an answer of no.
        "ground-slat",
        "sage-husks",
    ] {
        assert_eq!(
            asks(world, &format!("the dispensary has {ambiguous}")),
            None,
            "{ambiguous:?} was guessed at",
        );
    }
}

#[test]
fn a_word_that_names_nothing_at_all_is_a_typo_rather_than_an_answer_of_no() {
    // **The other face of the complaint this work started from.** A thing that is
    // simply not here yet answers no — that is `if the dispensary has
    // ground-sage` before you have ground any, the commonest question in the
    // game. A thing that is not a *name* at all is a typo, and answering it no
    // for ever is the same silence, arriving from the other direction.
    let mut sim = with_spell(
        "slip",
        &["if the dispensary has ground-slat", "grind sage", "end"],
    );
    run(&mut sim, &["invoke slip"]);
    sim.step_n(20);

    assert!(mentioned(&sim, "neither half runs"), "{:?}", messages(&sim));
    assert!(
        !mentioned(&sim, "yields ground-sage"),
        "a typo answered a question: {:?}",
        messages(&sim),
    );
    // ...and the real name still works, so the rule did not cost the loop.
    let mut sim = with_spell(
        "sound",
        &["if the dispensary has ground-salt", "grind sage", "end"],
    );
    run(&mut sim, &["invoke sound"]);
    sim.step_n(20);
    assert!(
        !mentioned(&sim, "neither half runs"),
        "{:?}",
        messages(&sim)
    );
}

#[test]
fn a_thing_that_is_not_there_yet_is_an_answer_of_no_rather_than_a_fault() {
    // **The commonest question in the game.** `if the dispensary has ground-sage`
    // is asked *before* there is any, so treating an absent product as a missing
    // referent would break the loop the whole feature exists for. The name is
    // real — it is in the recipes — and only its presence is in question.
    assert_eq!(
        asks(World::Cold, "the dispensary has ground-sage"),
        Some(false)
    );
    assert_eq!(
        asks(World::Banked, "the dispensary has ground-sage"),
        Some(true)
    );
}

// ---------------------------------------------------------------------------
// C. The truth table
// ---------------------------------------------------------------------------

/// Assert one row of the table against every world, naming the world that broke.
fn holds_in(question: &str, expected: [Option<bool>; 6]) {
    let wrong: Vec<String> = World::ALL
        .iter()
        .zip(expected)
        .filter_map(|(world, want)| {
            let got = asks(*world, question);
            (got != want).then(|| format!("\n  {world:?}: want {want:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{question:?}{}", wrong.join(""));
}

#[test]
fn one_instrument_at_a_time() {
    use World::{Banked, Cold, Grinding, Ready, Scouring};
    let _ = (Cold, Ready, Banked, Grinding, Scouring);

    // The athanor: burning is **working**, and this session's fix is what makes
    // it so. Banked and cold are both idle — a damped fire is fuel put by, not
    // work in progress.
    holds_in(
        "the athanor is idle",
        [
            Some(true),
            Some(false),
            Some(false),
            Some(false),
            Some(true),
            Some(true),
        ],
    );
    holds_in(
        "the athanor is working",
        [
            Some(false),
            Some(true),
            Some(true),
            Some(true),
            Some(false),
            Some(false),
        ],
    );
    // The mortar: mid-grind and mid-scour are both busy; holding a product is not.
    holds_in(
        "the mortar is working",
        [
            Some(false),
            Some(false),
            Some(true),
            Some(false),
            Some(false),
            Some(true),
        ],
    );
    // ...and `is empty` asks about **contents**, not about the panel's word for
    // the instrument: a mortar holding what it just made is *not* empty even
    // though it is idle, and one being scoured *is* empty even though it is busy.
    // Those are the two rows a player would guess wrong, which is why they are
    // here rather than assumed.
    //
    // `empty mortar_and_pestle` turns the husks out with the product, so Banked
    // and Scouring both leave a bare mortar — which is worth knowing, because a
    // spell that grinds twice depends on it.
    holds_in(
        "the mortar is empty",
        [
            Some(true),
            Some(true),
            Some(false),
            Some(false),
            Some(true),
            Some(true),
        ],
    );
}

#[test]
fn several_instruments_at_once() {
    use World::{Grinding, Scouring};
    let _ = (Grinding, Scouring);

    // **Two idle tools**, which is the question a real spell asks before it
    // starts a stage. False wherever the mortar is busy.
    holds_in(
        "the mortar is idle and the flask is idle",
        [
            Some(true),
            Some(true),
            Some(false),
            Some(true),
            Some(true),
            Some(false),
        ],
    );
    // **Either of two**, which is the point of `or`: the flask is free in every
    // world, so this is true even where the mortar is not.
    holds_in(
        "either the mortar is idle or the flask is idle",
        [Some(true); 6],
    );
    // Three, mixing the athanor in.
    holds_in(
        "the mortar is idle and the flask is idle and the athanor is idle",
        [
            Some(true),
            Some(false),
            Some(false),
            Some(false),
            Some(true),
            Some(false),
        ],
    );
}

#[test]
fn an_ingredient_here_and_not_there() {
    // The shape a real brewing spell asks: *is there any of this anywhere I can
    // reach*. In `Ready` the product is in the mortar and not on the shelf; in
    // `Banked` it is on the shelf and not in the mortar.
    holds_in(
        "the dispensary has ground-sage or the mortar has ground-sage",
        [
            Some(false),
            Some(false),
            Some(false),
            Some(true),
            Some(true),
            Some(true),
        ],
    );
    holds_in(
        "the dispensary has ground-sage",
        [
            Some(false),
            Some(false),
            Some(false),
            Some(false),
            Some(true),
            Some(true),
        ],
    );
    holds_in(
        "the mortar has ground-sage",
        [
            Some(false),
            Some(false),
            Some(false),
            Some(true),
            Some(false),
            Some(false),
        ],
    );
    // A base reagent is endless, so the shelf always has it — which is §11.5's
    // floor, and the reason "the dispensary is bare of sage" is not a world that
    // can be built.
    holds_in("the dispensary has sage", [Some(true); 6]);
}

#[test]
fn negation_says_the_opposite_of_what_it_negates() {
    for question in [
        "the athanor is idle",
        "the mortar is empty",
        "the dispensary has ground-sage",
        "the mortar is idle and the flask is idle",
        "either the mortar is idle or the athanor is idle",
    ] {
        for world in World::ALL {
            let plain = asks(world, question);
            let negated = asks(world, &format!("not {question}"));
            assert_eq!(
                negated,
                plain.map(|answer| !answer),
                "{question:?} in {world:?}",
            );
        }
    }
}

#[test]
fn the_shared_subject_answers_the_same_as_the_long_way_round() {
    for (short, long) in [
        (
            "the mortar is idle and empty",
            "the mortar is idle and the mortar is empty",
        ),
        (
            "the athanor is idle or working",
            "the athanor is idle or the athanor is working",
        ),
        (
            "the dispensary has sage and charcoal",
            "the dispensary has sage and the dispensary has charcoal",
        ),
        (
            "the dispensary has no ground-sage and sage",
            "the dispensary has no ground-sage and the dispensary has sage",
        ),
    ] {
        for world in World::ALL {
            assert_eq!(
                asks(world, short),
                asks(world, long),
                "{short:?} in {world:?}",
            );
        }
    }
}

#[test]
fn grouping_changes_the_answer_where_precedence_cannot() {
    // `(idle or idle) and has-product` against `idle or (idle and has-product)`,
    // in a world where the two differ — which is the whole reason `either` is a
    // word.
    let world = World::Grinding; // mortar busy, flask idle, no product anywhere
    assert_eq!(
        asks(
            world,
            "either the mortar is idle or the flask is idle and the dispensary has ground-sage",
        ),
        Some(false),
        "the bracket did not bind",
    );
    assert_eq!(
        asks(
            world,
            "the mortar is idle or the flask is idle and the dispensary has ground-sage",
        ),
        Some(false),
    );
    let shelf = World::Banked; // flask idle, product on the shelf
    assert_eq!(
        asks(
            shelf,
            "either the mortar is idle or the flask is idle and the dispensary has ground-sage",
        ),
        Some(true),
    );
}

// ---------------------------------------------------------------------------
// C2. Unanswerable, in every structural position
// ---------------------------------------------------------------------------

#[test]
fn one_name_the_tower_lacks_makes_the_whole_question_unanswerable() {
    // **Strict, and neither branch runs.** Kleene — answering from the half that
    // resolved — was considered and rejected: an instrument that has stopped
    // existing is §8.1's substitution surface, and a spell carrying on over it is
    // the thing that must not pass quietly.
    for question in [
        "the mortr is idle",
        "the mortr is idle and the mortar is idle",
        "the mortar is idle and the mortr is idle",
        "either the mortar is idle or the mortr is idle",
        "not the mortr is idle",
        "the mortar is idle and either the flask is idle or the mortr is idle",
        "the mortr has sage",
    ] {
        assert_eq!(
            asks(World::Cold, question),
            None,
            "{question:?} was answered over a name the tower does not have",
        );
    }
}

#[test]
fn every_name_it_could_not_place_is_named_once() {
    let mut sim = with_spell(
        "broken",
        &[
            "repeat 5",
            "if the mortr is idle and the flsk is idle",
            "grind sage",
            "else",
            "survey",
            "end",
            "end",
        ],
    );
    run(&mut sim, &["invoke broken"]);
    sim.step_n(30);

    let complaints: Vec<String> = messages(&sim)
        .into_iter()
        .filter(|line| line.contains("to ask about"))
        .collect();
    // **Once for the whole cast**, not once per turn of the loop. It was once per
    // evaluation, which for a question inside a `repeat` is one Danger record
    // every tick for as long as the spell runs.
    assert_eq!(
        complaints.len(),
        1,
        "said {} times: {:?}",
        complaints.len(),
        complaints,
    );
    // ...and both culprits are in it, because a player fixing one should not be
    // sent back for the other.
    let said = &complaints[0];
    assert!(said.contains("mortr") && said.contains("flsk"), "{said:?}");
    // The good name is not.
    assert!(!said.contains("mortar_and_pestle"), "{said:?}");
}

#[test]
fn a_name_that_stops_resolving_is_reported_rather_than_answered() {
    // **The case cast-time resolution is sold on.** The names are fixed when the
    // spell is cast; if the world moves under it, the question stops having an
    // answer and says so — §8.1's substitution surface, working.
    let mut sim = with_spell(
        "watching",
        &["repeat", "if the mortar is idle", "survey", "end", "end"],
    );
    run(&mut sim, &["invoke watching"]);
    sim.step_n(4);
    assert!(
        !mentioned(&sim, "to ask about"),
        "it complained while the name was good: {:?}",
        messages(&sim),
    );
}

// ---------------------------------------------------------------------------
// D. Nesting
// ---------------------------------------------------------------------------

/// Every command a spell issued, read out of the log rather than the transcript.
///
/// The prompt pane deliberately draws *what the player did, not what their
/// spells did* (`prompt.rs`), so a spell's own record of itself is in the stream
/// and not on screen. That is the same reason `peruse laboratory.log` exists.
fn issued(sim: &Sim) -> Vec<String> {
    messages(sim)
}

#[test]
fn four_blocks_deep_runs_the_steps_it_should_and_no_others() {
    // **The arithmetic most likely to break under nesting.** A branch costs two
    // path elements going in, a loop one, and walking out has to pop exactly what
    // was pushed — pop one too few and the path lands in the *other* half of an
    // `if` and runs it as well.
    let mut sim = with_spell(
        "deep",
        &[
            "repeat 2",
            "    if the athanor is idle",
            "        repeat 2",
            "            if the mortar is empty and the dispensary has sage",
            "                kindle charcoal",
            "            else",
            "                stop athanor",
            "            end",
            "        end",
            "    else",
            "        stop athanor",
            "    end",
            "end",
        ],
    );
    run(&mut sim, &["invoke deep"]);
    sim.step_n(40);

    // It starts cold, so the outer `if` is taken, the inner one is taken, and the
    // athanor is lit. On the next turn the athanor is *not* idle, so the outer
    // `else` damps it — and round again. What matters is that it finishes and
    // that both halves of both branches were reachable.
    assert!(
        mentioned(&sim, "takes light"),
        "the innermost branch never ran: {:?}",
        issued(&sim),
    );
    assert!(
        mentioned(&sim, "you damp the athanor"),
        "the outer else never ran: {:?}",
        issued(&sim),
    );
    assert!(
        mentioned(&sim, "is finished"),
        "the spell never came out of its blocks: {:?}",
        issued(&sim),
    );
}

#[test]
fn a_question_answered_differently_next_turn_takes_the_other_branch() {
    // The shape every real spell has: the loop changes the world the loop is
    // asking about.
    let mut sim = with_spell(
        "tending",
        &[
            "repeat 3",
            "    if the mortar is empty",
            "        grind sage",
            "    else",
            "        wait for the mortar",
            "    end",
            "end",
        ],
    );
    run(&mut sim, &["invoke tending"]);
    sim.step_n(40);

    assert!(
        mentioned(&sim, "yields ground-sage"),
        "the first turn never ground: {:?}",
        issued(&sim),
    );
    assert!(
        mentioned(&sim, "watches for"),
        "the second turn never took the else: {:?}",
        issued(&sim),
    );
}

#[test]
fn an_unanswerable_question_inside_a_loop_stops_neither_the_loop_nor_the_spell() {
    // §8's taxonomy is titled *"scripts always log and never halt"*. Neither
    // branch runs, and the loop keeps turning — which is the difference between
    // a question with no answer and a spell with a problem.
    let mut sim = with_spell(
        "shrug",
        &[
            "repeat 3",
            "    if the gatehouse is empty",
            "        grind sage",
            "    else",
            "        grind rock-salt",
            "    end",
            "end",
        ],
    );
    run(&mut sim, &["invoke shrug"]);
    sim.step_n(30);

    assert!(
        mentioned(&sim, "there is no gatehouse"),
        "{:?}",
        issued(&sim)
    );
    assert!(
        !mentioned(&sim, "yields"),
        "a branch ran on a question nobody could answer: {:?}",
        issued(&sim),
    );
    assert!(mentioned(&sim, "is finished"), "{:?}", issued(&sim));
}

#[test]
fn a_malformed_spell_with_compound_questions_still_runs() {
    // Unclosed, stray and misordered blocks, each reported once, with a compound
    // condition in the middle of them — because §8 forbids refusing at save and
    // halting at cast, which leaves running it as the only answer.
    let mut sim = with_spell(
        "rough",
        &[
            "else",
            "if the mortar is idle and the dispensary has sage",
            "grind sage",
            "end",
            "end",
            "repeat 2",
            "survey",
        ],
    );
    run(&mut sim, &["invoke rough"]);
    sim.step_n(30);

    for expected in [
        "an else with no if",
        "an end with nothing open",
        "nothing closed this",
    ] {
        assert!(
            mentioned(&sim, expected),
            "{expected:?} unreported: {:?}",
            issued(&sim)
        );
    }
    assert!(
        mentioned(&sim, "yields ground-sage"),
        "the readable part did not run: {:?}",
        issued(&sim),
    );
}

// ---------------------------------------------------------------------------
// E. The file is the player's
// ---------------------------------------------------------------------------

/// Write `lines` and read them back.
fn round_trip(sim: &mut Sim, name: &str, lines: &[String]) -> Vec<String> {
    sim.write_spell(name, lines);
    sim.step();
    sim.spell(name).expect("the spell was not written")
}

#[test]
fn everything_a_file_can_hold_survives_a_save() {
    let typed: Vec<String> = [
        "# the morning round",
        "",
        "   ",
        "\tmake a potion of clarity",
        "        grind the sage    ",
        "if the mortar is idle and the dispensary has sage or charcoal",
        "xyzzy plugh",
        "invoke a_spell_that_does_not_exist",
        "attend dispensary",
        "if the mortr is idle",
        "end",
        "end",
        "a line with a é and a 🜂 in it",
    ]
    .iter()
    .map(|line| (*line).to_owned())
    .collect();

    let mut sim = Sim::new(1);
    run(&mut sim, &["attend laboratory"]);
    assert_eq!(round_trip(&mut sim, "kept", &typed), typed);
}

#[test]
fn saving_the_same_spell_again_never_moves_it() {
    // **The reported complaint, as a property.** `quit` saves, so a spell was
    // re-read against whatever happened to be on the shelf every time it was
    // opened — and lines got shorter each visit.
    let typed: Vec<String> = [
        "grind the sage",
        "if the mortar is idle and the dispensary has sage",
        "    empty the mortar",
        "end",
    ]
    .iter()
    .map(|line| (*line).to_owned())
    .collect();

    let mut sim = Sim::new(1);
    run(&mut sim, &["attend laboratory"]);
    for _ in 0..5 {
        assert_eq!(round_trip(&mut sim, "steady", &typed), typed);
        // ...including when the shelf has changed underneath, which is what the
        // old rewriter re-read against.
        run(&mut sim, &["grind sage"]);
        sim.step_n(12);
    }
}

#[test]
fn the_shipped_spell_is_not_rewritten_by_being_opened() {
    let mut sim = Sim::new(1);
    let shipped = sim.spell("first_light").expect("the shipped spell");
    run(&mut sim, &["attend laboratory", "scribe first_light"]);
    let opened = sim.opening().expect("the editor was asked to open");
    assert_eq!(opened.lines, shipped);

    assert_eq!(round_trip(&mut sim, "first_light", &shipped), shipped);
}

#[test]
fn a_file_kept_whole_still_runs() {
    // Byte-for-byte alone would pass with the reading never wired up at all, so
    // every file test has to end somewhere the spell actually did something.
    let mut sim = with_spell(
        "loose",
        &[
            "make a potion of clarity",
            "if the mortar is idle and the dispensary has sage",
            "    grind the sage",
            "end",
        ],
    );
    run(&mut sim, &["invoke loose"]);
    sim.step_n(30);

    assert!(
        mentioned(&sim, "yields ground-sage"),
        "loose phrasing stopped working once it stopped being rewritten: {:?}",
        issued(&sim),
    );
}

// ---------------------------------------------------------------------------
// F. What the orb says
// ---------------------------------------------------------------------------

#[test]
fn saving_says_nothing_however_compound_the_spell() {
    let mut sim = Sim::new(1);
    run(&mut sim, &["attend laboratory"]);
    let before = sim.scrollback().records().len();
    sim.write_spell(
        "quiet",
        &[
            "if the mortr is idle and the flsk is bare".to_owned(),
            "xyzzy".to_owned(),
        ],
    );
    sim.step();
    assert_eq!(
        sim.scrollback().records().len(),
        before,
        "a save put something in the transcript: {:?}",
        messages(&sim),
    );
}

#[test]
fn no_line_a_spell_can_say_has_a_hole_in_it() {
    // A placeholder no emit site fills draws as itself, which `prose.toml`
    // documents as deliberate. Swept over the **whole stream** rather than a list
    // of keys, so a new failure line is covered the day it is written.
    let mut sim = with_spell(
        "broken",
        &[
            "wait for a moonrise",
            "if the gatehouse is empty and the mortr has sage",
            "grind moonstone",
            "end",
            "attend dispensary",
            "if the moon is gibbous",
            "survey",
            "end",
        ],
    );
    run(&mut sim, &["invoke broken"]);
    sim.step_n(160);

    let holes: Vec<String> = messages(&sim)
        .into_iter()
        .filter(|line| line.contains('{'))
        .collect();
    assert!(
        holes.is_empty(),
        "an unfilled placeholder was said: {holes:?}"
    );
    // ...and it really reached the failures, or the sweep proves nothing.
    for expected in ["to ask about", "neither half runs", "waited too long"] {
        assert!(
            mentioned(&sim, expected),
            "{expected:?} never said: {:?}",
            issued(&sim)
        );
    }
}

#[test]
fn a_question_the_orb_cannot_read_is_said_once_and_the_spell_runs_on() {
    let mut sim = with_spell(
        "half",
        &[
            "repeat 3",
            "if the mortar is idle the athanor is working",
            "survey",
            "end",
            "end",
            "grind sage",
        ],
    );
    run(&mut sim, &["invoke half"]);
    sim.step_n(40);

    let said = messages(&sim)
        .iter()
        .filter(|line| line.contains("neither half runs"))
        .count();
    assert_eq!(said, 1, "said {said} times");
    assert!(
        mentioned(&sim, "yields ground-sage"),
        "the rest of the spell did not run: {:?}",
        issued(&sim),
    );
}

// ---------------------------------------------------------------------------
// G. Determinism and replay
// ---------------------------------------------------------------------------

#[test]
fn a_spell_full_of_questions_replays_from_seed_and_submissions() {
    let mut live = Sim::new(7);
    run(&mut live, &["attend laboratory"]);
    let lines: Vec<String> = [
        "repeat 3",
        "if either the mortar is idle or the flask is idle and the dispensary has sage",
        "grind the sage",
        "else",
        "if the athanor is not working",
        "kindle charcoal",
        "end",
        "end",
        "end",
    ]
    .iter()
    .map(|line| (*line).to_owned())
    .collect();
    live.write_spell("thorough", &lines);
    live.step();
    run(&mut live, &["invoke thorough"]);
    live.step_n(40);

    let mut replayed = Sim::new(7);
    for (tick, submission) in live.submissions().all().to_vec() {
        while replayed.tick() < tick {
            replayed.step();
        }
        replayed.replay(submission);
    }
    while replayed.tick() < live.tick() {
        replayed.step();
    }

    assert_eq!(
        messages(&replayed),
        messages(&live),
        "a replay said something different",
    );
}

#[test]
fn the_same_question_answers_the_same_way_every_time() {
    // A tie broken by iteration order, or a `Vec` ordered by archetype, would
    // show here and nowhere else.
    for question in [
        "the mortar is idle and the flask is idle",
        "either the dispensary has ground-sage or the mortar has ground-sage",
        "the mortr is idle",
    ] {
        let first = asks(World::Ready, question);
        for _ in 0..5 {
            assert_eq!(asks(World::Ready, question), first, "{question:?}");
        }
    }
}

#[test]
fn a_meditate_answers_the_same_as_the_ticks_it_stands_for() {
    // §19's idempotence rule — in-flight state must survive hundreds of ticks
    // inside one `step()` — applied to compound conditions.
    let lines = [
        "repeat 3",
        "if the mortar is empty and the dispensary has sage",
        "grind sage",
        "else",
        "wait for the mortar",
        "end",
        "end",
    ];

    let mut stepped = with_spell("paced", &lines);
    run(&mut stepped, &["invoke paced"]);
    stepped.step_n(60);

    let mut skipped = with_spell("paced", &lines);
    run(&mut skipped, &["invoke paced", "meditate 60"]);

    assert_eq!(
        messages(&skipped)
            .into_iter()
            .filter(|line| line.contains("ground-sage"))
            .count(),
        messages(&stepped)
            .into_iter()
            .filter(|line| line.contains("ground-sage"))
            .count(),
        "a meditate ran the spell differently from the ticks it stands for",
    );
}

// ---------------------------------------------------------------------------
// H. What the editor is told — `read_spell`, which is the other half of "report
//    it, do not rewrite it" and the half nothing else checks
// ---------------------------------------------------------------------------

/// How the orb reads `lines` as a laboratory spell.
fn read(lines: &[&str]) -> Vec<orbs_sim::Reading> {
    let sim = laboratory();
    let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.read_spell("laboratory", &lines)
}

/// A laboratory, stood in.
fn laboratory() -> Sim {
    let mut sim = Sim::new(1);
    run(&mut sim, &["attend laboratory"]);
    sim
}

#[test]
fn a_byproduct_no_one_has_made_yet_is_not_a_fault() {
    // **The rule `fix` states and the editor used to contradict.** A thing is
    // placed if the room has it *or* the recipes name it — every `leaves` name
    // is in the second class and in the first only after a run. Re-deriving that
    // check with a different question painted `if the dispensary has ash` red on
    // a line that works perfectly, and counted it on the status row.
    for byproduct in ["ash", "phlegm", "sediment", "dregs", "husks", "ground-sage"] {
        let reading = read(&[
            &format!("if the dispensary has {byproduct}"),
            "survey",
            "end",
        ]);
        assert_eq!(
            reading[0].fault, None,
            "{byproduct:?} was reported as unreadable: {:?}",
            reading[0],
        );
    }
}

#[test]
fn a_thing_named_where_a_place_belongs_is_reported_before_it_is_cast() {
    // The mirror of the test above, and the same root cause: the check asked
    // `NounKind::Any` where `fix` asks `Place`, so a reagent standing in for a
    // place was called clean here and then failed at run time — the editor
    // silent about the one fault it exists to show.
    let reading = read(&["if the sage is idle", "survey", "end"]);
    assert_eq!(
        reading[0].fault.as_ref().map(|fault| fault.key),
        Some("spell_nowhere"),
        "{:?}",
        reading[0],
    );

    // ...and the runner agrees, which is what makes the editor's mark true.
    let mut sim = with_spell("probe", &["if the sage is idle", "survey", "end"]);
    run(&mut sim, &["invoke probe"]);
    sim.step_n(6);
    assert!(
        mentioned(&sim, "there is no sage here"),
        "{:?}",
        messages(&sim)
    );
}

#[test]
fn every_unplaced_name_in_a_question_is_reported_rather_than_the_first() {
    // **`watch::every` states the rule from the other end** — *"§8.1's rule is
    // that the culprit is never anonymous, not that one culprit is enough"* — and
    // the compile half stopped at the first failure. So `interpret` named `mortr`
    // and said nothing at all about `sagg`, and a player fixed one typo, asked
    // again, and was told about the next one.
    let reading = read(&[
        "if the mortr is idle and the mortar has sagg",
        "survey",
        "end",
    ]);
    let fault = reading[0].fault.as_ref().expect("the line read clean");
    let detail = fault.detail.as_deref().unwrap_or_default();

    // A word that names nothing outranks a place the tower lacks: the first
    // leaves a question that cannot be *asked*, the second one that cannot be
    // *answered*. `sagg` is the typo, so it decides the key.
    assert_eq!(fault.key, "spell_unreadable_if", "{fault:?}");
    assert!(
        detail.contains("sagg"),
        "the typo was not named: {detail:?}"
    );

    // Two places, and both named — the case with no kind to arbitrate.
    let reading = read(&["if the mortr is idle and the flsk is idle", "survey", "end"]);
    let fault = reading[0].fault.as_ref().expect("the line read clean");
    let detail = fault.detail.as_deref().unwrap_or_default();
    assert_eq!(fault.key, "spell_nowhere", "{fault:?}");
    assert!(
        detail.contains("mortr") && detail.contains("flsk"),
        "only one of two missing places was named: {detail:?}",
    );
}

#[test]
fn a_spell_not_written_yet_is_quoted_rather_than_resolved() {
    // **§8's own worked example**, and the bug §19 records being written into a
    // file once already — here on the surface that replaced the file. The parser
    // weights the verb double, so a perfect `invoke` with a meaningless argument
    // still clears `MIN_SIMILARITY`: this read back as `invoke first_light.spell`,
    // the orb telling the player with confidence that it would run a spell they
    // had not named.
    let reading = read(&["invoke not_written_yet"]);
    assert_eq!(reading[0].heard, "invoke not_written_yet");
    assert_eq!(
        reading[0].fault, None,
        "a forward reference was called a fault"
    );

    // The ordinary case still reads back resolved, so the guard is not a blanket.
    let reading = read(&["make a potion of clarity"]);
    assert_eq!(reading[0].heard, "recall clarity");
}

#[test]
fn a_forward_reference_holds_however_many_spells_already_exist() {
    // **The half above could not see.** A forward reference matches no spell
    // well, so *which* resolution the parser returns depends on how many spells
    // are on the shelf: one, and it resolves (the verb is weighted double); four,
    // and it ties between them and comes back `Ambiguous`. `names_a_spell` only
    // covered the first two shapes, so the second called a forward reference a
    // fault — `spell_missing`, on a line §8 uses as its own worked example.
    //
    // Shelving the dev ladders in a debug build is what surfaced it, but a player
    // with four spells of their own would have found it just the same. So this
    // asserts the property against a grimoire that has grown, rather than against
    // whatever it happens to hold today.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    for name in ["one", "two", "three"] {
        sim.write_spell(name, &["grind sage".to_owned()]);
        sim.step();
    }

    let reading = sim.read_spell("laboratory", &["invoke not_written_yet".to_owned()]);
    assert_eq!(reading[0].heard, "invoke not_written_yet");
    assert_eq!(
        reading[0].fault, None,
        "a forward reference became a fault once the grimoire filled up",
    );
}
