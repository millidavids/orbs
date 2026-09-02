//! Every reading the forge publishes.
//!
//! **Published only while true**, which is the tower's rule and sharper here
//! than anywhere: `spell::watch` answers `is empty` by asking whether a node has
//! children, and `many_at` answers an *absent* reading with **nought**. So a
//! column that always carried `lit` could never be dark, and a tool that always
//! carried `graced` could never be free of a charm — and the first rung of every
//! solver written for this room would be dead.
//!
//! That is also why there is no `dark`. A column publishes `lit` when its
//! residue glyph is alight and nothing when it is not, so `if the apex has no
//! lit` is the question — which is the tower's absent-is-nought idiom rather
//! than a word it would have had to invent. (`dark` was unavailable in any case:
//! it scores 750 against the maze's `marks`.)

use bevy_ecs::prelude::*;

use crate::execute::readings::{clear, reading, room_of};
use crate::tower::charm::Kind;
use crate::tower::lattice::{Binding, COLUMNS};
use crate::tower::{self, charm};

/// Republish everything about the lattice.
///
/// **Takes the lattice rather than reading `Cwd`**, which is the rule the lens
/// paid for: a bound solver working while the player stands in the laboratory
/// would otherwise find no lattice, publish nothing, and leave every reading
/// frozen at whatever it last said.
pub(crate) fn publish(world: &mut World, lattice: Entity) {
    let Some(room) = room_of(world, lattice) else {
        return;
    };
    let binding = world.get::<Binding>(lattice).cloned();

    clear(world, lattice);
    if let Some(binding) = &binding {
        // Which charm is open, so a spell can tell one lattice from another.
        //
        // **And nothing else.** There was a `if binding.lattice.set()` arm
        // raising `graced` here, and it could never fire: a fall that lights
        // removes the `Binding` *before* publishing, so `publish` never sees one
        // whose `set()` is true, and on a failure it is false. Dead code reading
        // as a live rule about when the lattice is graced — which is a worse
        // thing to leave than a missing rule, because the next reader trusts it.
        tower::raise_reading(world, lattice, &binding.kind);
    }

    // The residue, one column at a time. **This is the whole signal**: the eight
    // openings leave eight distinct residues, so what these three say identifies
    // the answer completely — and the table that reads them lives in the
    // player's spell, which is §8.1's *"a rule, not a memory"*.
    for (index, name) in COLUMNS.iter().enumerate() {
        let Some(node) = reading(world, room, name) else {
            continue;
        };
        clear(world, node);
        if let Some(binding) = &binding
            && binding.lattice.residue()[index]
        {
            tower::raise_reading(world, node, "lit");
        }
    }

    charms(world, room);
}

/// What each charm is doing across the whole tower, on the charm's own node.
///
/// **Neither word carries a count, and that is what makes this cheap enough to
/// run on a tick.** `graced` was a count of ticks remaining, which changes every
/// single tick — so keeping it true meant despawning and re-raising a node once
/// a second, for ever. Insertion order is the parse in this tower and `NodeId`s
/// travel in the save, so per-tick churn is not a performance worry but a
/// correctness one.
///
/// Presence and a threshold change **rarely** — when a charm is laid, when it
/// crosses into [`EBBING_AT`](charm::EBBING_AT), and when it lapses — so this
/// can compare and do nothing, which is what it does on almost every tick.
///
/// What is lost is *how long is left* as a number a spell can read. That is no
/// loss: §8.1 wants a spell to hold **a rule, not a memory**, and *"is it nearly
/// out"* is a rule where *"it has 47 ticks"* is a memory. The exact figure is on
/// the panel and in `survey`, where a person reads it.
pub(crate) fn charms(world: &mut World, room: Entity) {
    // **One query for all five, hoisted out of the loop.** This called
    // `world.query::<&Charmed>()` *inside* the `Kind::ALL` loop — a fresh
    // `QueryState` matched against every archetype, five times a tick — and the
    // compare-and-do-nothing above cannot help, because the query runs before
    // the comparison. `meditate 3600` collapses those ticks into one `step`, so
    // it was ~18,000 query states inside a single command.
    let longest = longest_of_each(world);
    for kind in Kind::ALL {
        let Some(node) = reading(world, room, kind.word()) else {
            continue;
        };
        let left = longest[kind as usize];
        let wanted: &[&str] = match left {
            0 => &[],
            n if n <= charm::EBBING_AT => &[charm::GRACED, charm::EBBING],
            _ => &[charm::GRACED],
        };

        // **Compare before clearing**, or this is the churn the header refuses.
        let held: Vec<String> = tower::children_of(world, node)
            .into_iter()
            .filter_map(|child| world.get::<tower::Name>(child).map(|name| name.0.clone()))
            .collect();
        if held.len() == wanted.len()
            && wanted.iter().all(|word| held.iter().any(|had| had == word))
        {
            continue;
        }

        // **A charm ending says so**, which every other expiring thing in the
        // tower does — the fire gutters, a chant lapses — and this did not.
        // ROADMAP's exit for the phase is *"the buff decays"*, and a row that
        // simply stopped draining with no line was the whole of how a player
        // found out. `forge_charm_gone` had been authored for this and was
        // reached by nothing.
        //
        // Only on the *edge*, and only from graced to gone: the comparison above
        // is what makes that once rather than every tick thereafter.
        let lapsed = left == 0 && held.iter().any(|had| had == charm::GRACED);

        clear(world, node);
        for word in wanted {
            tower::raise_reading(world, node, word);
        }

        if lapsed {
            let message = world.resource::<crate::content::Prose>().line(
                "forge_charm_gone",
                &[("name", kind.word()), ("detail", charm::FORGE)],
            );
            world
                .resource_mut::<crate::session::Scrollback>()
                .records_mut()
                .push(orbs_render::RecordKind::Completion)
                .text(
                    orbs_render::FieldName::Name,
                    crate::parser::Verb::Imbue.canonical(),
                )
                .text(orbs_render::FieldName::Message, &message)
                .text(orbs_render::FieldName::Source, charm::LATTICE)
                .role(orbs_render::Role::Cost)
                .finish();
        }
    }
}

/// Keep the charm words true as time passes.
///
/// **Appended to the schedule and drawing nothing**, which is the licence
/// `settling`, `erode`, `lapse_chant` and `regenerate` already hold: a system
/// that only reads the clock cannot perturb any `RngStream`.
///
/// Without it the words are only true when a forge verb happens to run — so a
/// charm could lapse, and `ebbing` never appear, until somebody typed something
/// in the forge. The maintenance spell of Phase 9's fourth box is *entirely*
/// built on `ebbing` arriving on its own, so this is not a polish item: without
/// it that box cannot be ticked at all.
///
/// It is `debug_siege`'s republish rule as a standing system rather than a
/// one-off — *"a stale board is what a decision tree would read"*.
pub fn lapse(world: &mut World) {
    let Some(room) = tower::find_by_path(world, "/tower/forge") else {
        return;
    };
    charms(world, room);
}

/// The most ticks any tool in the tower has left, of every kind at once.
///
/// **The longest rather than the nearest**, so `ebbing` fires when the *last*
/// one is running out. A spell that renewed on the first tool to dip would relay
/// a charm that had most of its life left, once a lap, for ever.
///
/// **One pass for all five**, indexed by `Kind`, because this runs on a tick.
fn longest_of_each(world: &mut World) -> [u64; Kind::ALL.len()] {
    let now = *world.resource::<crate::tick::Tick>();
    let mut out = [0u64; Kind::ALL.len()];
    let mut held = world.query::<&charm::Charmed>();
    for charmed in held.iter(world) {
        for kind in Kind::ALL {
            let left = charmed.left(kind, now);
            let slot = &mut out[kind as usize];
            if left > *slot {
                *slot = left;
            }
        }
    }
    out
}

// **There was a `Cwd`-based `refresh` here and it is gone.**
//
// It resolved the forge through `readings::fixture`, which answers *where the
// player is standing* — and at `Sim::with_schedule` time that is the filesystem
// root, so the one thing it was called for silently did nothing. That is the
// trap `publish_dice` records paying for one room over, reintroduced three lines
// under its own warning.
//
// Nothing else wanted it: the verbs call `publish` with the lattice they already
// hold, and the bootstrap calls [`lapse`], which addresses the forge by path.
