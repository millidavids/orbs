//! The save format, held to the only claim that matters.
//!
//! # What "it works" means here
//!
//! Not *the document round-trips* — a save that carries half the world round-
//! trips its half perfectly. The claim is that **a loaded tower keeps running
//! the same world**, so the test steps the saved world and the loaded one side
//! by side for hundreds of ticks and requires them to stay identical. A
//! component nobody remembered to carry diverges within a few hundred ticks: a
//! fire that is not lit, a brew that never lands, a stream that rolls a
//! different number.
//!
//! # Which test is the instrument, and which one is not
//!
//! `a_loaded_tower_keeps_running_the_same_world` is the instrument. It is the
//! only test here that **restores**, so it is the only one that can notice a
//! component nobody carried: the loaded athanor is cold, the heated stage never
//! lands, and the difference surfaces within a few hundred ticks in fields the
//! document *does* carry.
//!
//! `two_routes_to_one_world_write_the_same_save` is **not**, and the first draft
//! of this file claimed it was. Both of its sides are `capture()` on a world
//! built by ordinary play — neither is restored — so a field `capture` omits is
//! missing from both documents and it passes. Its real value is as a fifth
//! `(seed, submissions)` determinism check, comparing far more world state than
//! the four message-stream ones that already exist. Worth keeping, worth not
//! mistaking.

// `unwrap` on a render is right here and nowhere else: a save that will not
// render *is* the failure this file exists to find, and the panic names the file
// and line as clearly as an `expect` would.
#![allow(clippy::unwrap_used)]

use orbs_sim::{Save, Sim};

/// The `orbs-balance` seed set, for the same reason it uses them: one seed is
/// one sample, and `drift` and `substitution` fire on a schedule that is fixed
/// in tick space per seed. 3 poisons a log early and 0 swaps a reagent; 11 and
/// 42 are quiet through both.
const SEEDS: [u64; 4] = [0, 3, 11, 42];

/// How far the two worlds are run side by side after the load.
///
/// Long enough for the laboratory's slowest stage to land twice, a `repeat` to
/// lap, and — on the noisy seeds — a log to be poisoned under both worlds at
/// once.
///
/// **Not long enough for a reagent swap**, and that is measured rather than
/// assumed: the first ambient one lands at tick 685 on the kindest of these four
/// seeds and 4,387 on the unkindest, so no lie ever settles inside this window.
/// An earlier version of this comment claimed otherwise.
/// `a_renamed_node_keeps_its_place_among_its_siblings` is where that case lives,
/// and it steps until a swap really fires.
const LOCKSTEP: u64 = 600;

/// A world with something happening in every subsystem the save has to carry.
///
/// **One builder, shared by every test in this file**, and that is deliberate
/// rather than tidy. `every_component_the_world_holds_is_one_the_save_carries`
/// can only see components that are *present*, and twelve of them are
/// conditional — a `Ward` exists only while a reading is open, a `Quickened`
/// only inside a window. The lint is exactly as strong as the world it is
/// pointed at, so the world it is pointed at is the one the lockstep test uses.
fn a_busy_tower(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    for line in commands() {
        sim.submit(line);
        sim.step();
    }
    sim
}

/// The session the fixture plays, and what the two profiles can each reach.
///
/// **`debug_spawn` and the dev-spell shelf are `cfg(debug_assertions)`**, so a
/// release build cannot reach a quickening window, a bound spell or 16
/// experience without brewing for thousands of ticks. Six sibling test files
/// answer that by gating the whole file on `debug_assertions`; this one does
/// not, because the save format **ships in release** and a format with no
/// release test is the half that matters going untested.
///
/// So the debug-only half is conditional and
/// [`the_test_world_actually_holds_everything_it_is_meant_to`] asserts only what
/// the profile in hand can actually reach. The round-trip, the lockstep and the
/// completeness lint run in both.
fn commands() -> Vec<&'static str> {
    let mut lines = vec![
        // The laboratory: a fire lit, and a byproduct on a shelf.
        "attend laboratory",
        "kindle charcoal",
    ];

    if cfg!(debug_assertions) {
        lines.extend([
            // Two distillations, which is 16 experience and the first
            // concentration slot. There is no way to grant it — concentration is
            // derived from work completed and `debug_spawn` deliberately earns
            // nothing — so reaching a `Bound` spell at all means running these.
            "debug_spawn clarified-draught 2",
            "distil clarified-draught",
            "meditate 60",
            "empty alembic",
            "distil clarified-draught",
            "meditate 60",
            // A quickening window.
            "debug_spawn quickening-scroll",
            "wield quickening-scroll",
        ]);
    }

    lines.extend([
        // The lens: a ward open, pressed and part-dialled.
        "attend lens",
        "probe",
        "dial second borax",
        "probe",
        // The archive: a maze open and part-walked.
        "attend archive",
        "research",
    ]);

    // **The bearings a maze actually has.** `follow east` was in this list for
    // one version and refused on three of the four seeds — *"there is no way
    // east"* — so the reading never moved and `came` stayed `None`. A maze is
    // generated, so the walk has to ask rather than assume: all four bearings
    // are tried, and the ones that are walls cost a refused command and nothing
    // else.
    lines.extend(["follow north", "follow east", "follow south", "follow west"]);

    if cfg!(debug_assertions) {
        lines.extend([
            "debug_spawn fragment 3",
            // A spell both held and part-way through a lap.
            "bind threading",
            "meditate 3",
        ]);
    }

    lines.extend([
        // **Work in flight goes last, and that ordering is the point.** A grind
        // is eight ticks; started at the top of this list it would have landed
        // twenty ticks before the snapshot, and every assertion in this file
        // would have been comparing two worlds with nothing happening in them.
        // `the_test_world_actually_holds_everything_it_is_meant_to` caught
        // exactly that.
        "attend laboratory",
        "grind sage",
        // **Two, not four**, in a debug build: the quickening window above is
        // still open, so the grind is halved to four ticks and `meditate 4`
        // landed it exactly.
        if cfg!(debug_assertions) {
            "meditate 2"
        } else {
            "meditate 4"
        },
    ]);

    lines
}

/// Report the first line where two documents part, rather than both in full.
///
/// A save of a busy tower is a few hundred lines. Printed twice, side by side,
/// the one line that matters is unfindable — and the whole reason this test
/// exists is to *name* the field nobody carried.
fn same(left_sim: &Sim, right_sim: &Sim, note: &str) {
    let (left, right) = (
        left_sim.snapshot().to_toml().unwrap(),
        right_sim.snapshot().to_toml().unwrap(),
    );
    if left == right {
        return;
    }
    let mut context = String::new();
    for (n, (a, b)) in left.lines().zip(right.lines()).enumerate() {
        if a != b {
            // Neutral labels: this is called with (loaded, lived) in most
            // places and (replayed, played) in one, and naming one pair in the
            // helper made the other read backwards.
            context = format!("line {}:\n  left:  {a}\n  right: {b}", n + 1);
            break;
        }
    }
    if context.is_empty() {
        context = format!(
            "the documents are the same for {} lines and then one ends ({} vs {})",
            left.lines().count().min(right.lines().count()),
            left.lines().count(),
            right.lines().count(),
        );
    }
    panic!("{note}\n{context}");
}

/// One save as text, with the `[rng]` table struck out.
///
/// For the one comparison that must not be allowed to pass on the strength of
/// the numbers it is testing: see `the_stream_positions_are_not_interchangeable`.
fn without_rolls(sim: &Sim) -> String {
    let text = sim.snapshot().to_toml().unwrap();
    let mut out = String::new();
    let mut skipping = false;
    for line in text.lines() {
        if line.starts_with('[') {
            skipping = line.starts_with("[rng]");
        }
        if !skipping {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Save it, load it, and run both on. The primary property.
#[test]
fn a_loaded_tower_keeps_running_the_same_world() {
    for seed in SEEDS {
        let mut lived = a_busy_tower(seed);
        let save = lived.snapshot();

        // **Through the text, which is the path a player takes.** Loading from
        // the in-memory `Save` skips `to_toml` and `from_toml` entirely — so a
        // field lost on parse and re-added on print would compare equal, the
        // version gate would guard a route no test took, and `MazeSave.walls`
        // (the one multi-line string, where TOML trims a newline after `"""`)
        // would never be read back at all.
        let text = save.to_toml().expect("a save renders");
        let read = Save::from_toml(&text).expect("a save reads back");
        let mut loaded = Sim::restored(&read);

        same(
            &loaded,
            &lived,
            &format!("seed {seed}: the loaded world does not describe itself the same way"),
        );

        for tick in 0..LOCKSTEP {
            lived.step();
            loaded.step();
            // Compared every fifty ticks rather than every one: the assertion
            // renders two whole documents, and the divergences this is looking
            // for persist rather than flickering.
            if tick % 50 == 0 {
                same(
                    &loaded,
                    &lived,
                    &format!(
                        "seed {seed}: the two worlds parted {} ticks after the load",
                        tick + 1
                    ),
                );
            }
        }
    }
}

/// The clock, the rolls, and the work in flight all resume rather than restart.
///
/// The lockstep test above would catch each of these, but it would report them
/// as *"the two worlds parted"* — which says a save is broken without saying
/// which half. These say which half.
#[test]
fn what_a_save_carries_is_still_true_after_it_is_loaded() {
    let lived = a_busy_tower(3);
    let save = lived.snapshot();
    let loaded = Sim::restored(&save);

    assert_eq!(loaded.tick(), lived.tick(), "the clock restarted");
    assert_ne!(loaded.tick().get(), 0, "the test world never ran");
    assert_eq!(
        loaded.experience(),
        lived.experience(),
        "the work already done was forgotten",
    );

    // The eight stream positions, checked by their effect rather than by
    // reading them back: both worlds take the next hundred ticks' worth of
    // rolls and must reach the same world. A restore that rebuilt from the seed
    // alone would replay the session's whole schedule of drifts and swaps.
    let mut lived = lived;
    let mut loaded = loaded;
    lived.step_n(100);
    loaded.step_n(100);
    same(
        &loaded,
        &lived,
        "the random streams did not resume where they stood",
    );
}

/// Two routes to one world, and the same bytes out of both.
///
/// **A determinism check, not a completeness one** — see the file header. Both
/// sides run the same `capture` over a world that was never restored, so this
/// cannot see a field the save omits. What it *does* see is the world itself
/// diverging: `(seed, submissions)` is `session.rs`'s stated replay contract and
/// had no consumer at all until now.
#[test]
fn two_routes_to_one_world_write_the_same_save() {
    for seed in SEEDS {
        let played = a_busy_tower(seed);

        // The journal, applied to a fresh world exactly as `Sim::replay`
        // documents: stand on the recorded tick, then hand the submission over.
        let mut replayed = Sim::new(seed);
        for (tick, submission) in played.submissions().all().to_vec() {
            while replayed.tick() < tick {
                replayed.step();
            }
            replayed.replay(submission);
        }
        while replayed.tick() < played.tick() {
            replayed.step();
        }

        same(
            &replayed,
            &played,
            &format!(
                "seed {seed}: replaying the session reached a world the save describes differently"
            ),
        );
    }
}

/// A save is TOML, and it is TOML a person can read.
#[test]
fn the_document_is_text_and_it_reads_back() {
    let sim = a_busy_tower(3);
    let text = sim.snapshot().to_toml().expect("a save renders");

    // §15 makes this a commercial decision rather than a nicety: *"saves are
    // readable and editable… hand-editing a TOML file only affects the person
    // doing it."* So the things a person would look for are spelled out.
    assert!(text.contains("[world]"), "no world table:\n{text}");
    assert!(
        text.contains("/tower/laboratory"),
        "nodes are not addressed by path:\n{text}",
    );
    assert!(
        !text.contains("Entity"),
        "an entity id reached the file — it means nothing across a save",
    );

    let read = Save::from_toml(&text).expect("a save reads back");
    assert_eq!(read.to_toml().unwrap(), text, "the document is not stable");
}

/// A save from a later build is refused, not half-read.
#[test]
fn a_save_from_a_future_format_is_refused_rather_than_misread() {
    let sim = Sim::new(0);
    let written = format!("format = {}", orbs_sim::save::FORMAT);
    let text = sim
        .snapshot()
        .to_toml()
        .unwrap()
        .replace(&written, "format = 99");

    let error = Save::from_toml(&text).expect_err("a future format must not load");
    let said = error.to_string();
    assert!(
        said.contains("later version"),
        "the refusal does not say why: {said}",
    );
}

/// ...and so is one from an earlier build.
///
/// **The direction this did not check**, and the lens rework is why it matters:
/// `WardSave` lost seven fields and `shift` changed vocabulary, and serde drops
/// what it no longer knows without a word. A format-1 save therefore opened
/// straight into the redesigned ward and resumed a reading whose answers were
/// scored by a codemaker that no longer exists — a tower that loads, looks
/// right, and is quietly wrong, which is exactly what the `Ahead` arm above
/// refuses in the other direction.
#[test]
fn a_save_from_an_earlier_format_is_refused_rather_than_misread() {
    let sim = Sim::new(0);
    let written = format!("format = {}", orbs_sim::save::FORMAT);
    let text = sim
        .snapshot()
        .to_toml()
        .unwrap()
        .replace(&written, "format = 1");

    let error = Save::from_toml(&text).expect_err("an older format must not load");
    let said = error.to_string();
    assert!(
        said.contains("earlier version"),
        "the refusal does not say why: {said}",
    );
}

/// Nonsense is an error, never a panic.
#[test]
fn a_corrupt_save_is_an_error_rather_than_a_crash() {
    for rubbish in [
        "",
        "garbage",
        "[world]\nformat = \"not a number\"\n",
        "\0\0\0",
    ] {
        assert!(
            Save::from_toml(rubbish).is_err(),
            "{rubbish:?} was accepted as a save",
        );
    }
}

/// The builder reaches every state it claims to.
///
/// **Without this the whole file is theatre.** Every assertion above compares
/// two documents, and two documents describing a world where nothing is
/// happening agree perfectly. `bind` refusing for want of experience, a `probe`
/// resolving to something else, a dev spell that is not on the shelf — each
/// would leave the round-trip green and the coverage nil, which is exactly the
/// failure §19 records `tests/agrees.rs` being rewritten to avoid: *"they would
/// both pass with this crate's entire harness deleted."*
#[test]
fn the_test_world_actually_holds_everything_it_is_meant_to() {
    // **Every seed, not one.** The maze is generated, so a bearing that is a
    // wall on seed 3 is a corridor on seed 11 — checking one seed leaves the
    // other three's coverage to luck.
    for seed in SEEDS {
        let save = a_busy_tower(seed).snapshot();
        let has = |what: &str, found: bool| {
            assert!(found, "seed {seed}: the test world has no {what}");
        };

        has(
            "lit athanor",
            save.nodes.iter().any(|n| n.burning.is_some()),
        );
        has("open maze", save.nodes.iter().any(|n| n.maze.is_some()));
        has("open ward", save.nodes.iter().any(|n| n.ward.is_some()));
        has(
            "work in flight",
            save.nodes.iter().any(|n| n.working.is_some()),
        );

        // **A byproduct, not merely stock.** `any(stock.is_some())` was true of
        // a tower nobody had touched — the dispensary ships three endless piles
        // — so it asserted that the game exists. What a grind actually leaves is
        // a *counted* pile, and that is the thing a save has to carry.
        has(
            "counted stock",
            save.nodes
                .iter()
                .any(|n| n.stock.as_deref().is_some_and(|s| s != "endless")),
        );

        // ...and the same correction for the maze. `Maze::new` marks the
        // starting square, so `!marks.is_empty()` was true the instant a maze
        // opened. Two marked squares means the reading actually moved.
        let maze = save
            .nodes
            .iter()
            .find_map(|n| n.maze.as_ref())
            .expect("a maze");
        has("walked maze", maze.marks.len() > 1);
        has("maze the reading has entered", maze.came.is_some());

        // A ward that has been pressed rather than merely opened — the answer,
        // the aperture and the press history are what a save has to carry, and
        // an unpressed ward carries none of them.
        let ward = save
            .nodes
            .iter()
            .find_map(|n| n.ward.as_ref())
            .expect("a ward");
        has("pressed ward", ward.pressed && ward.spent > 0);
        has("press history", ward.history.len() > 1);

        // A socket dialled off the opening figure. `socket_marks` said this
        // directly and went with the ratchet that needed it, so what a save
        // carries now is where the aperture ended up — and `Ward::new` opens it
        // on the first four sigils, fixed, precisely so this comparison means
        // something.
        has("dialled socket", ward.aperture != [0, 1, 2, 3]);

        // **The record tail, checked for content rather than length.** `!is_empty`
        // was true of `Sim::new` alone, because `tower::report` writes the boot
        // card. What a save has to carry is what the *player* did.
        has(
            "the player's own commands in the tail",
            save.records.iter().any(|r| r.kind == "input"),
        );

        // The debug doors are `cfg(debug_assertions)`, so these are what a
        // release build cannot reach. Asserted rather than skipped silently.
        if cfg!(debug_assertions) {
            has("earned experience", save.progress.experience > 0);
            has(
                "running spell",
                save.nodes.iter().any(|n| n.running.is_some()),
            );
            has("held spell", save.nodes.iter().any(|n| n.bound.is_some()));
            has(
                "quickening window",
                save.nodes.iter().any(|n| n.quickened.is_some()),
            );
        }
    }
}

/// Nothing in the world is a component or resource the save has never heard of.
///
/// # Why this is a lint and not a comment
///
/// A snapshot save has exactly one fatal failure mode, and it is silent: a
/// component added in a later phase that nobody remembers to serialise. Every
/// test above still passes — the round-trip round-trips what it carries, and the
/// two worlds agree because neither of them has the field. The player finds out
/// when their fire goes out on load.
///
/// So this walks what the world is actually *made of* and fails **by name**
/// against a table. Adding a component and forgetting the save breaks the build
/// with the type's own name in the message, which is the same instrument
/// `every_material_has_a_home_a_move_can_reach` and
/// `every_glyph_the_rail_draws_is_in_the_code_page` already are.
///
/// # What it cannot see
///
/// Only components that are **present at that instant**. Twelve of them are
/// conditional — a `Ward` exists while a reading is open, a `Quickened` inside a
/// window — so the lint is exactly as strong as the world it is pointed at.
/// That is why it is pointed at [`a_busy_tower`], the same builder the lockstep
/// test uses and the one
/// `the_test_world_actually_holds_everything_it_is_meant_to` guards. It narrows
/// the hole; it does not close it.
///
/// # Why `type_id` and not `name`
///
/// `ComponentInfo::name` returns a `DebugName`, which without `bevy_utils`'s
/// `debug` feature is the literal string `"<Enable the debug feature to see the
/// name>"` — and `crates/orbs/Cargo.toml` builds Bevy with `default-features =
/// false` and no `debug`. So the *matching* is on `type_id`, which is always
/// there; `name` is used only for the failure message, and `orbs-sim` takes that
/// feature as a **dev-dependency** so the message carries a real path under
/// `cargo test` without reaching the shipped binary.
#[test]
fn every_component_the_world_holds_is_one_the_save_knows_about() {
    use std::any::TypeId;

    // Carried in the document. Adding a row here is the second half of adding a
    // field to `save::node::NodeSave`.
    let carried: Vec<(TypeId, &str)> = vec![
        (TypeId::of::<orbs_sim::tower::NodeId>(), "NodeId"),
        (TypeId::of::<orbs_sim::tower::Name>(), "Name"),
        (TypeId::of::<orbs_sim::tower::Nameable>(), "Nameable"),
        (TypeId::of::<orbs_sim::tower::Protected>(), "Protected"),
        (TypeId::of::<orbs_sim::tower::Fixture>(), "Fixture"),
        (TypeId::of::<orbs_sim::tower::HeatSource>(), "HeatSource"),
        (TypeId::of::<orbs_sim::tower::Operation>(), "Operation"),
        (TypeId::of::<orbs_sim::tower::Store>(), "Store"),
        (TypeId::of::<orbs_sim::tower::Keep>(), "Keep"),
        (TypeId::of::<orbs_sim::tower::Reading>(), "Reading"),
        (TypeId::of::<orbs_sim::tower::Grouped>(), "Grouped"),
        (TypeId::of::<orbs_sim::tower::Held>(), "Held"),
        (TypeId::of::<orbs_sim::tower::Domain>(), "Domain"),
        (TypeId::of::<orbs_sim::tower::Stock>(), "Stock"),
        (TypeId::of::<orbs_sim::tower::Working>(), "Working"),
        (TypeId::of::<orbs_sim::tower::Triaging>(), "Triaging"),
        (TypeId::of::<orbs_sim::tower::Bidden>(), "Bidden"),
        (TypeId::of::<orbs_sim::tower::Product>(), "Product"),
        (TypeId::of::<orbs_sim::tower::Quickened>(), "Quickened"),
        (TypeId::of::<orbs_sim::tower::Burning>(), "Burning"),
        (TypeId::of::<orbs_sim::tower::Banked>(), "Banked"),
        (TypeId::of::<orbs_sim::tower::Ash>(), "Ash"),
        (TypeId::of::<orbs_sim::tower::Poisoned>(), "Poisoned"),
        (TypeId::of::<orbs_sim::tower::Substituted>(), "Substituted"),
        (TypeId::of::<orbs_sim::tower::Log>(), "Log"),
        (TypeId::of::<orbs_sim::tower::Maze>(), "Maze"),
        (TypeId::of::<orbs_sim::tower::Ward>(), "Ward"),
        (TypeId::of::<orbs_sim::tower::spell::Running>(), "Running"),
        (TypeId::of::<orbs_sim::tower::spell::Bound>(), "Bound"),
    ];

    // Not carried, each for a reason. A row here is a decision, not a shrug.
    let excused: Vec<(TypeId, &str)> = vec![
        // The tree's shape, rebuilt from every node's path.
        (TypeId::of::<bevy_ecs::hierarchy::ChildOf>(), "ChildOf"),
        (TypeId::of::<bevy_ecs::hierarchy::Children>(), "Children"),
    ];

    let known: std::collections::HashSet<TypeId> =
        carried.iter().chain(&excused).map(|(id, _)| *id).collect();

    // **Both worlds, because the reach of this lint is the reach of the world it
    // is pointed at.** Some components are mutually exclusive — an athanor
    // cannot be lit and banked at once — so one fixture cannot hold everything,
    // and the caveat below is narrowed by looking at two rather than restated.
    let busy = a_busy_tower(3);
    let odd = a_tower_in_the_odd_states(3);

    // **Only archetypes that are nodes.** Bevy 0.19 stores every resource on an
    // entity of its own, so an unfiltered archetype walk meets all thirty-odd of
    // them as "components" — and a resource is not a thing a node can be made
    // of. `NodeId` is on every node and on nothing else, which makes it the
    // filter, and it makes the question the lint asks precise: *what can a node
    // be made of that the save has never heard of?*
    let mut strangers: Vec<String> = Vec::new();
    for sim in [&busy, &odd] {
        let world = sim.world();
        let node_id = world
            .components()
            .get_id(TypeId::of::<orbs_sim::tower::NodeId>())
            .expect("NodeId is registered — the tower is raised");

        for archetype in world
            .archetypes()
            .iter()
            .filter(|archetype| archetype.contains(node_id))
        {
            for id in archetype.components() {
                let Some(info) = world.components().get_info(*id) else {
                    continue;
                };
                if info
                    .type_id()
                    .is_none_or(|type_id| known.contains(&type_id))
                {
                    continue;
                }
                strangers.push(info.name().to_string());
            }
        }
    }
    strangers.sort_unstable();
    strangers.dedup();

    assert!(
        strangers.is_empty(),
        "the world holds components the save does not know about: {strangers:?}\n\
         Add each to `save::node::NodeSave` and to the table in this test, or to \
         the excused list with the reason it is derived.",
    );
}
/// Print a real save, so a person can read one.
///
/// **This is the See-it line for §15's promise**, and it is `#[ignore]`d rather
/// than asserted because the claim is *"saves are readable and editable"* — a
/// judgement only eyes make. `the_document_is_text_and_it_reads_back` above
/// checks the things a test can check; this is how you check the rest.
///
/// ```bash
/// cargo test -p orbs-sim --test persistence -- --ignored show_a_save --nocapture
/// ```
///
/// What to look for: the maze drawn as a maze, a node's path where an id would
/// have been, and an instrument mid-run saying which tick it lands on.
#[test]
#[ignore = "prints a save so a person can read it — the See-it line for §15"]
fn show_a_save() {
    println!("{}", a_busy_tower(3).snapshot().to_toml().unwrap());
}

/// Nothing in the world is a *resource* the save has never heard of.
///
/// # Why this is separate from the component lint
///
/// Because it was missing, and the gap cost a real defect. `Choices` — the
/// numbered disambiguation prompt — is a resource, survives across ticks until
/// it is answered, and was not in the document: save with a prompt open, reload,
/// and the question was still on the transcript while the answer no longer
/// resolved. §15's dead end, arriving through the affordance built to remove
/// one. The component lint could never have seen it, and it is carried now —
/// see `an_open_question_survives_a_reload`.
///
/// # Why it matches on names rather than `TypeId`
///
/// Most of these types are private to their module and cannot be named from an
/// integration test at all, so there is no `TypeId::of::<T>()` to compare
/// against. The `bevy_ecs` `debug` feature — a dev-dependency, for exactly this
/// — makes `ComponentInfo::name` a real path, and a rename then forces a
/// conscious update here, which is what a lint is for.
#[test]
fn every_resource_the_world_holds_is_one_the_save_knows_about() {
    // In the document.
    const CARRIED: [&str; 9] = [
        "orbs_sim::tick::Tick",
        "orbs_sim::rng::Rngs",
        "orbs_sim::tower::node::NodeIds",
        "orbs_sim::tower::node::Cwd",
        "orbs_sim::session::Wizard",
        "orbs_sim::tower::experience::Experience",
        "orbs_sim::tower::mastery::Taken",
        "orbs_sim::tower::learned::Learned",
        "orbs_sim::session::Choices",
    ];

    // Not in the document. Each line is a decision; none of them is a shrug.
    const EXCUSED: [(&str, &str); 16] = [
        (
            "orbs_sim::session::Scrollback",
            "a bounded tail travels; the arena does not",
        ),
        ("orbs_sim::tower::brief::Marks", "carried inside [progress]"),
        (
            "orbs_sim::parser::scene::Scene",
            "rebuilt every tick by tower::rebuild",
        ),
        (
            "orbs_sim::tower::scene::Topics",
            "derived from Prose at construction",
        ),
        (
            "orbs_sim::parser::trace::ParseLog",
            "telemetry; no world state",
        ),
        (
            "orbs_sim::session::Submissions",
            "a journal describes a session, and a resumed session is a new one",
        ),
        (
            "orbs_sim::session::Pending",
            "drained at the top of every tick, and a save is taken at a boundary",
        ),
        (
            "orbs_sim::session::Skip",
            "spun out inside Sim::step's own loop; always nought between ticks",
        ),
        ("orbs_sim::content::prose::Prose", "content, compiled in"),
        ("orbs_sim::content::recipe::Recipes", "content, compiled in"),
        ("orbs_sim::content::fuel::Fuels", "content, compiled in"),
        (
            "orbs_sim::content::material::Materials",
            "content, compiled in",
        ),
        ("orbs_sim::content::spell::Spells", "content, compiled in"),
        (
            "orbs_sim::content::progression::Progression",
            "content, compiled in",
        ),
        (
            "orbs_sim::tower::spell::run::Caller",
            "set around one instruction and cleared; never crosses a tick",
        ),
        (
            "orbs_sim::execute",
            "the five take-once frontend handshakes; a request, not state",
        ),
    ];

    let sim = a_busy_tower(3);
    let mut strangers: Vec<String> = Vec::new();
    for (info, _) in sim.world().iter_resources() {
        let name = info.name().to_string();
        // Bevy's own bookkeeping is not ours to carry.
        if !name.starts_with("orbs_sim::") {
            continue;
        }
        let known = CARRIED.contains(&name.as_str())
            || EXCUSED.iter().any(|(excused, _)| name.starts_with(excused));
        if !known {
            strangers.push(name);
        }
    }
    strangers.sort_unstable();
    strangers.dedup();

    assert!(
        strangers.is_empty(),
        "the world holds resources the save does not know about: {strangers:#?}\n\
         Add each to `save::document::ProgressSave` and to CARRIED, or to EXCUSED \
         with the reason it need not travel.",
    );
}

/// A renamed node keeps its place among its siblings.
///
/// # The defect this pins
///
/// `sabotage::substitute` renames a reagent pile in place — `sage` becomes
/// `sage-` — so on the next load its saved path matches nothing the raised tower
/// has. Before the fix it was *spawned* and appended to the end of the
/// dispensary while the raised `sage` was swept, and the pile changed slot.
///
/// That is not cosmetic. `tower::node` opens by saying anything a player can see
/// must come from walking `Children` in insertion order, because §6 resolves
/// noun ties to whichever was registered first — so a reload silently changed
/// which noun an ambiguous phrase resolved to, and the two worlds never
/// re-converged.
///
/// `debug_swap` is the tester's shortcut for the ambient system, which fires at
/// one swap an hour and would otherwise need ~900 ticks of stepping to reach.
#[test]
fn a_renamed_node_keeps_its_place_among_its_siblings() {
    for seed in SEEDS {
        let mut lived = a_busy_tower(seed);

        // **Stepped to a real ambient swap, not `debug_swap`.** That word takes
        // the alphabetically-first endless pile, which is `charcoal` — and
        // charcoal is **last** in the dispensary's `Children` order, so
        // re-appending it lands it in the slot it already had and the defect is
        // invisible. It is also the pile the ambient system exempts, because a
        // swap that takes the fire stops every heated stage in the tower.
        //
        // The first version of this test used it and passed with the fix
        // removed, which is the whole failure mode this file exists to refuse.
        let swapped = (0..6000).find_map(|_| {
            lived.step();
            lived
                .snapshot()
                .nodes
                .into_iter()
                .find(|node| node.substituted.is_some())
                .map(|node| node.path)
        });
        let Some(swapped) = swapped else {
            panic!("seed {seed}: no ambient swap in 6000 ticks, so this proves nothing");
        };

        let save = lived.snapshot();
        let text = save.to_toml().expect("a save renders");
        let mut loaded = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));

        same(
            &loaded,
            &lived,
            &format!("seed {seed}: {swapped} moved slot on the way back in"),
        );

        // ...and the two worlds stay together, including through the tick the
        // lie settles back to the truth (`WEARS_OFF` is 300).
        for tick in 0..400 {
            lived.step();
            loaded.step();
            if tick % 50 == 0 {
                same(
                    &loaded,
                    &lived,
                    &format!("seed {seed}: parted {} ticks after a swap", tick + 1),
                );
            }
        }
    }
}

/// An open numbered question can still be answered after a reload.
///
/// # The dead end this pins
///
/// §6 has the orb number its readings when several score alike, and `session`'s
/// own doc says why that needs somewhere to live: *"without somewhere to hold the
/// list the question is rhetorical — the prompt appears, the digit resolves
/// against the verb vocabulary as a miss, and the player is in a **dead end**.
/// §15's gate calls the dead-end metric more important than the raw resolution
/// rate."*
///
/// A save that dropped `Choices` recreated that dead end exactly: the question
/// was still on the transcript, and the answer no longer resolved. The save
/// carries the **line**, not the readings, and asks again on the way in.
///
/// `purge` is the ambiguity fixture the codebase already keeps for this — §19
/// records `brew` losing that role when `recall`'s slot changed, and three tests
/// silently asserting nothing as a result.
#[test]
fn an_open_question_survives_a_reload() {
    let mut lived = Sim::new(11);
    lived.submit("attend laboratory");
    lived.step();
    lived.submit("purge s");
    lived.step();

    let offered = lived.choices().len();
    assert!(
        offered > 1,
        "`purge s` did not raise a numbered prompt, so this proves nothing",
    );

    let save = lived.snapshot();
    assert!(
        save.progress.asked.is_some(),
        "the save does not carry the open question",
    );

    let text = save.to_toml().expect("a save renders");
    let mut loaded = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));
    assert_eq!(
        loaded.choices().len(),
        offered,
        "the numbered list came back a different length",
    );

    // ...and the digit still means what the prompt said it meant. Both worlds
    // answer, and both must reach the same place.
    lived.submit("1");
    lived.step();
    loaded.submit("1");
    loaded.step();
    same(
        &loaded,
        &lived,
        "answering after a reload reached a different world",
    );
}

/// A hostile but well-formed save is refused a panic, not just a parse error.
///
/// `a_corrupt_save_is_an_error_rather_than_a_crash` only feeds rubbish to
/// `from_toml`, which is the easy half — the parser rejects it and nothing else
/// runs. The half that matters is a document that **parses** and then goes
/// through `Sim::restored`, because that is where an index or an `unwrap` would
/// be, and §15 says in as many words that a player may edit this file.
///
/// The ward case is not hypothetical: `Ward::seat` range-checks a sigil on the
/// way in, so live play cannot seat one past the end — but `press` indexes
/// `sigil_marks` directly, so a hand-edited aperture panicked on the next
/// `probe` rather than on the way in.
#[test]
fn a_hostile_but_well_formed_save_does_not_panic() {
    let text = a_busy_tower(3)
        .snapshot()
        .to_toml()
        .expect("a save renders");

    let hostile = [
        // A sigil index past the end of the sigil table.
        ("aperture = [", "aperture = [99, "),
        ("held = [", "held = [99, "),
        ("code = [", "code = [255, "),
        // A reading standing outside its own maze, and a spoil nobody can reach.
        ("at = 97", "at = 999999"),
        ("exit = ", "exit = 888888 # "),
        // A path that names nothing, in each place a reference can appear.
        (
            "subject = \"/tower",
            "subject = \"/nowhere/at/all\" # \"/tower",
        ),
        ("cwd = \"/tower", "cwd = \"\" # \"/tower"),
        // A count where a word belongs, and a word where a count belongs.
        ("kind = \"reagent\"", "kind = \"not-a-kind\""),
        ("stock = \"endless\"", "stock = \"not-a-number\""),
        ("operation = \"grind\"", "operation = \"not-a-verb\""),
    ];

    for (needle, replacement) in hostile {
        let edited = text.replacen(needle, replacement, 1);
        assert_ne!(edited, text, "the hand-edit {needle:?} did not apply");

        // Parsing may refuse it — that is a fine answer. What may not happen is
        // a panic, here or on any of the next hundred ticks.
        if let Ok(save) = Save::from_toml(&edited) {
            let mut loaded = Sim::restored(&save);
            loaded.step_n(100);
            loaded.submit("survey");
            loaded.step();
        }
    }
}

/// A spell whose text moved while it was not running lets go, and says so.
///
/// The one branch in `adopt` that a happy-path round-trip can never reach: the
/// fingerprint is recomputed on load and, when it disagrees, the run ends rather
/// than resuming into a program its position does not belong to. §8's failure
/// taxonomy is *"scripts always log and never halt"*, so it has to be said out
/// loud — halting quietly is the one thing that rule forbids.
#[test]
fn a_spell_whose_text_moved_under_it_lets_go() {
    let save = a_busy_tower(3).snapshot();

    let Some(spell) = save
        .nodes
        .iter()
        .find_map(|node| node.running.as_ref().map(|run| run.spell.clone()))
    else {
        // A release build cannot reach a running spell — the dev-spell shelf is
        // `cfg(debug_assertions)` — so there is nothing here to move under.
        // Named rather than passed over in silence.
        #[cfg(debug_assertions)]
        panic!("a debug build should have a running spell to test with");
        #[cfg(not(debug_assertions))]
        return;
    };

    // Rewrite the spell's own text in the document, exactly as a player editing
    // the file would, leaving the fingerprint describing what it used to say.
    let text = save.to_toml().expect("a save renders");
    let mut edited = Save::from_toml(&text).expect("a save reads back");
    for node in &mut edited.nodes {
        if node.path == spell {
            node.held = Some(vec!["survey".to_owned()]);
        }
    }

    let loaded = Sim::restored(&edited);
    assert!(
        loaded.snapshot().nodes.iter().all(|n| n.running.is_none()),
        "the spell resumed into a program its position does not belong to",
    );
    assert!(
        loaded
            .scrollback()
            .records()
            .iter()
            .any(|record| record.to_line().contains("reads differently")),
        "the spell stopped without saying why, which §8's taxonomy forbids",
    );
}

/// A stream longer than the tail still puts a spell's cursor back where it was.
///
/// # The case no fixture reaches by accident
///
/// The record tail is capped, and every save in every other test here is well
/// under the cap — so `Records::resume` is only ever called with a `dropped` of
/// nought, and the arithmetic that turns a spell's cursor back into an index
/// (`spell::run`'s `seen.saturating_sub(dropped)`) is never exercised against a
/// truncated stream. A real player passes the cap in about twelve minutes.
///
/// What must hold: `sequence` comes back exactly, `dropped` reports the gap, and
/// a spell that was watching the stream carries on rather than going blind.
#[test]
fn a_truncated_record_tail_keeps_the_sequence_it_sat_at_the_end_of() {
    let mut lived = a_busy_tower(3);

    // **Typed at, rather than left to tick.** Waiting for the world to emit
    // 1,200 records on its own does not work: a release build has no bound spell
    // to lap (the dev shelf is `cfg(debug_assertions)`), so the stream sat at 73
    // records after twenty thousand ticks. A command is what makes records — the
    // echo, the answer — so this is both faster and true in either profile.
    while lived.scrollback().records().sequence() < 1_200 {
        lived.submit("status");
        lived.step();
    }

    let save = lived.snapshot();
    assert!(
        (save.records.len() as u64) < save.world.sequence,
        "the stream never overflowed the tail, so this proves nothing \
         ({} records against sequence {})",
        save.records.len(),
        save.world.sequence,
    );

    let text = save.to_toml().expect("a save renders");
    let mut loaded = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));

    let stream = loaded.scrollback().records();
    assert_eq!(
        stream.sequence(),
        save.world.sequence,
        "the sequence did not survive the truncation",
    );
    assert_eq!(
        stream.dropped(),
        save.world.sequence - save.records.len() as u64,
        "`dropped` does not report the gap the tail left",
    );

    // ...and the two worlds still run together, which is what says a spell's
    // cursor still means what it meant.
    for tick in 0..200 {
        lived.step();
        loaded.step();
        if tick % 50 == 0 {
            same(&loaded, &lived, "a truncated tail parted the two worlds");
        }
    }
}

/// The fields the fixture leaves empty, filled and round-tripped.
///
/// # Why they need a test of their own
///
/// Every other assertion here is `snapshot(a) == snapshot(b)`, and a field that
/// is empty in both worlds compares equal whether or not the save carries it. So
/// `progress.learned` could be deleted from the document outright and the rest of
/// this file would stay green — it is `[]` in the fixture, because a secret is
/// found by solving wards for hours.
///
/// `progress.taken` is the same shape and cannot be tested yet: §19 records that
/// Mastery v1 is deliberately read-only, so `take` always refuses and nothing
/// can put an id in that list. Named here so it is not mistaken for covered.
#[test]
#[cfg(debug_assertions)]
fn the_fields_the_fixture_leaves_empty_still_travel() {
    let mut lived = a_busy_tower(3);
    lived.submit("debug_learn");
    lived.step();

    let save = lived.snapshot();
    assert!(
        !save.progress.learned.is_empty(),
        "`debug_learn` found nothing, so this proves nothing",
    );

    let text = save.to_toml().expect("a save renders");
    let loaded = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));
    assert_eq!(
        loaded.snapshot().progress.learned,
        save.progress.learned,
        "a found secret did not survive the reload",
    );

    // ...and it is the *world* that knows, not only the document: a recipe the
    // orb has not found refuses to be recalled.
    same(&loaded, &lived, "a found secret parted the two worlds");
}

/// Each random stream resumes at its own position, not at some other stream's.
///
/// Five of the eight are still at word nought in any short session — only
/// `Threat`, `Archive` and `Lens` have drawn — so a save that wrote the eight
/// positions in the wrong order, or read them back permuted, would round-trip
/// perfectly and pass every other test in this file.
///
/// This gives every stream a distinct position first, then permutes two of them
/// in the document and requires the world to notice.
#[test]
fn the_stream_positions_are_not_interchangeable() {
    let sim = a_busy_tower(3);
    let save = sim.snapshot();
    assert_eq!(
        save.rng.positions.len(),
        8,
        "a world has eight streams and the save carries that many",
    );

    // Two positions swapped is a different world. If the save were order-blind —
    // or if `Rngs::restore` zipped them onto the wrong streams — this would
    // reload into the same one.
    let mut permuted = save.clone();
    permuted.rng.positions.swap(2, 6);
    assert_ne!(
        permuted.rng.positions, save.rng.positions,
        "the two streams chosen are both at nought, so the swap changed nothing",
    );

    let mut straight = Sim::restored(&save);
    let mut crossed = Sim::restored(&permuted);

    // **Compared with `[rng]` struck out, and that is the whole care here.** The
    // permuted positions are *in* the document, so comparing the documents whole
    // would differ on the `[rng]` table alone — the test would pass with the
    // positions never reaching a stream at all, which is precisely the vacuous
    // shape this file exists to refuse. What must differ is the **world**: two
    // streams crossed means a different log poisoned and a different ward rolled.
    straight.step_n(900);
    crossed.step_n(900);
    assert_ne!(
        without_rolls(&crossed),
        without_rolls(&straight),
        "two streams swapped reached the same world — the positions are not \
         reaching the streams they belong to",
    );
}

/// A world in the states [`a_busy_tower`] cannot reach.
///
/// # Why a second world rather than a longer first
///
/// Some of these are mutually exclusive with what the busy tower holds. `damp`
/// requires `Burning` and removes it, so an athanor cannot be lit and banked at
/// once; a spell-driven run and a player-driven one compete for the single
/// production slot. Contorting one fixture to hold everything would have made it
/// hold each thing less convincingly.
///
/// The completeness lint runs over **both**, because its reach is exactly its
/// world's — that is the caveat it carries, and this is the honest way to narrow
/// it rather than restate it.
fn a_tower_in_the_odd_states(seed: u64) -> Sim {
    let mut sim = Sim::new(seed);
    let mut lines = vec![
        "attend laboratory",
        "kindle charcoal",
        // A spell doing the work, so the run in flight carries `Bidden` — the
        // player's own `grind` never does.
        "invoke first_light",
        "meditate 2",
    ];
    if cfg!(debug_assertions) {
        // A reagent claiming a name that is not its own, and the rail's mark.
        lines.push("debug_swap");
    }
    lines.extend([
        // **`stop athanor`, not `damp`.** Damping has no word of its own — it is
        // what stopping the fire *does*, which is `pipeline`'s own note: "what
        // makes `stop athanor` at the end of a script loop worth writing". A
        // bare `damp` is ambiguous and `damp athanor` fuzzy-matches `purge`,
        // which is how the first version of this fixture ended up scouring.
        "stop athanor",
        // A scour, caught in flight: `PURGE_TICKS` is 4 and this costs one.
        // **An instrument, not a log** — purging a log un-poisons it on the spot
        // and takes no triage slot, so there is nothing in flight to carry.
        "purge alembic",
    ]);

    for line in lines {
        sim.submit(line);
        sim.step();
    }
    sim
}

/// The fields the busy tower never reaches, round-tripped.
///
/// Four of them were carried on faith: written symmetrically in `capture` and
/// `adopt`, and checked by nothing, because no test world ever held one. That is
/// the completeness lint's stated blind spot arriving in practice rather than in
/// a doc comment.
#[test]
fn the_odd_states_travel_too() {
    for seed in SEEDS {
        let mut lived = a_tower_in_the_odd_states(seed);
        let save = lived.snapshot();
        let has = |what: &str, found: bool| {
            assert!(found, "seed {seed}: the odd-states world has no {what}");
        };

        has("banked fuel", save.nodes.iter().any(|n| n.banked.is_some()));
        has(
            "scour in flight",
            save.nodes.iter().any(|n| n.triaging.is_some()),
        );
        has(
            "spell-driven run",
            save.nodes.iter().any(|n| n.bidden.is_some()),
        );
        if cfg!(debug_assertions) {
            has(
                "substituted reagent",
                save.nodes.iter().any(|n| n.substituted.is_some()),
            );
        }

        let text = save.to_toml().expect("a save renders");
        let mut loaded = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));
        same(
            &loaded,
            &lived,
            &format!("seed {seed}: an odd state did not travel"),
        );

        for tick in 0..200 {
            lived.step();
            loaded.step();
            if tick % 40 == 0 {
                same(
                    &loaded,
                    &lived,
                    &format!("seed {seed}: parted {} ticks after the load", tick + 1),
                );
            }
        }
    }
}

/// A spell suspended **inside a part** comes back inside it.
///
/// §8 requires in-flight state to be serialisable, and a call stack is the newest
/// thing that is. A save carrying `pc` and `loops` but not the descents beneath
/// them would restore a spell that had forgotten who called it — and the failure
/// is quiet: it would run to the end of the part and stop, which looks exactly
/// like a spell that finished.
#[test]
fn a_spell_inside_a_part_comes_back_inside_it() {
    let mut lived = Sim::new(11);
    lived.submit("attend laboratory");
    lived.step();
    // Long enough that the run is *inside* the part rather than past it: the
    // grind takes eight ticks and the call is reached on the third step.
    lived.write_spell(
        "tending",
        &[
            "part gathering()".to_owned(),
            "grind sage".to_owned(),
            "empty mortar_and_pestle".to_owned(),
            "end".to_owned(),
            "gathering()".to_owned(),
        ],
    );
    lived.step();
    lived.submit("invoke tending");
    lived.step_n(4);

    let save = lived.snapshot();
    let inside = save
        .nodes
        .iter()
        .find_map(|node| node.running.as_ref())
        .expect("a spell is running");
    assert_eq!(
        inside.part.as_deref(),
        Some("gathering"),
        "the save does not say which part the spell is in",
    );
    assert!(
        !inside.stack.is_empty(),
        "the save carries no caller for the part to return to",
    );

    let text = save.to_toml().expect("a save renders");
    let mut loaded = Sim::restored(&Save::from_toml(&text).expect("a save reads back"));

    // Both worlds run on from the same place, and must reach the same one.
    lived.step_n(30);
    loaded.step_n(30);
    same(
        &loaded,
        &lived,
        "a reload inside a part ran on into a different world",
    );
}
