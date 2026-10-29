use orbs_render::{FieldName, Value};
use orbs_sim::{Save, Sim, tower};

/// Type a line and let the tick that runs it pass.
pub fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// `run`, for each line in turn.
pub fn run_all(sim: &mut Sim, lines: &[&str]) {
    for line in lines {
        run(sim, line);
    }
}

/// A fresh tower whose wizard has walked into `room`.
pub fn attended(seed: u64, room: &str) -> Sim {
    let mut sim = Sim::new(seed);
    run(&mut sim, &format!("attend {room}"));
    sim
}

/// A fresh tower standing at the bailey's wall.
pub fn at_the_wall(seed: u64) -> Sim {
    attended(seed, "bailey")
}

/// A fresh tower standing in the menagerie.
pub fn at_the_circle(seed: u64) -> Sim {
    attended(seed, "menagerie")
}

/// Every message the orb has said.
pub fn said(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Message) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Whether anything the orb said contains `needle`.
pub fn ever_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

/// Every bare word `survey` has printed — a reading with no number on it.
pub fn words(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Name) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Every reading a domain `declares` is among the words `seen`.
///
/// A spell compiles against the declaration whether or not anything publishes
/// it, and `many_at` answers an absent reading with nought — so a dead one
/// gets a confident wrong answer for ever rather than an error.
pub fn assert_all_published<'a>(
    declares: impl IntoIterator<Item = &'a str>,
    seen: impl IntoIterator<Item = String>,
) {
    let seen: std::collections::BTreeSet<String> = seen.into_iter().collect();
    let missing: Vec<&str> = declares
        .into_iter()
        .filter(|word| !seen.contains(*word))
        .collect();
    assert!(
        missing.is_empty(),
        "the domain declares readings no state ever publishes, so a spell asking \
         for one gets nought for ever: {missing:?}",
    );
}

/// Every complaint the orb made about a spell it was given.
///
/// A spell that silently does nothing looks identical to one that ran, so this
/// is the thing to assert on.
pub fn complaints(sim: &Sim) -> Vec<String> {
    said(sim)
        .into_iter()
        .filter(|line| {
            line.contains("means nothing")
                || line.contains("cannot read")
                || line.contains("is not a")
                || line.contains("no part of this")
        })
        .collect()
}

/// Write a spell and give the world a tick to read it.
pub fn write(sim: &mut Sim, name: &str, lines: &[&str]) {
    let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &lines);
    sim.step();
}

/// Write a spell and cast it, through the words a player would use.
pub fn cast(sim: &mut Sim, name: &str, lines: &[&str]) {
    write(sim, name, lines);
    run(sim, &format!("invoke {name}"));
}

/// How far through the one run in flight the tower is, as `(done, total)`.
pub fn meter(sim: &Sim) -> Option<(u64, u64)> {
    sim.instruments()
        .into_iter()
        .find_map(|instrument| instrument.meter)
        .map(|meter| (meter.done, meter.total))
}

/// The siege on the rampart, as it stands.
pub fn the_siege(sim: &mut Sim) -> tower::Siege {
    let world = sim.world_mut();
    world
        .query::<&tower::Siege>()
        .iter(world)
        .next()
        .cloned()
        .expect("a siege on the rampart")
}

/// `fresh`, fed `played`'s journal on the ticks it was typed, and stepped on
/// to where `played` stands.
pub fn replayed(played: &Sim, mut fresh: Sim) -> Sim {
    for (tick, submission) in played.submissions().all().to_vec() {
        while fresh.tick() < tick {
            fresh.step();
        }
        fresh.replay(submission);
    }
    while fresh.tick() < played.tick() {
        fresh.step();
    }
    fresh
}

/// Walk into the laboratory, light the athanor, and distil `count` clarities
/// from spawned draught, emptying the alembic after each — the cheap way to a
/// brew. Debug builds only, since it spawns.
pub fn distil_spawned(sim: &mut Sim, count: usize) {
    run_all(
        sim,
        &[
            "attend laboratory",
            "kindle charcoal",
            &format!("debug_spawn clarified-draught {count}"),
        ],
    );
    for _ in 0..count {
        run(sim, "distil clarified-draught");
        sim.step_n(60);
        run(sim, "empty alembic");
    }
}

/// §10.1's five stages, once, from endless stock, ending in a `clarity` left
/// in the alembic. Assumes the laboratory with the athanor lit; runs in release.
///
/// Scoured before use, never after fouling — `orbs-balance`'s rule: `mix` and
/// `distil` leave their instruments charged, and a charged one refuses a second
/// load in silence.
pub fn brew_one(sim: &mut Sim) {
    run_all(
        sim,
        &[
            "empty mortar_and_pestle",
            "empty balneum_mariae",
            "empty flask_and_rod",
            "empty alembic",
            "grind sage",
            "meditate 9",
            "empty mortar_and_pestle",
            "digest ground-sage",
            "meditate 14",
            "grind rock-salt",
            "meditate 9",
            "empty mortar_and_pestle",
            "mix sage-tincture with ground-salt",
            "meditate 12",
            "distil clarified-draught",
        ],
    );
    sim.step_n(60);
}

/// A save written out and read back, as a player's file is.
pub fn reloaded(save: &Save) -> Sim {
    let text = save.to_toml().expect("a save renders");
    Sim::restored(&Save::from_toml(&text).expect("a save reads back"))
}
