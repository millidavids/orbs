//! Every reading the forge publishes.
//!
//! Published only while true: `spell::watch` answers `is empty` by asking
//! whether a node has children, so a column that always carried `lit` could
//! never be dark and the first rung of every solver would be dead.
//!
//! So there is no `dark` — a column publishes `lit` or nothing, and `if the
//! apex has no lit` is the question. (`dark` was unavailable in any case: it
//! scores 750 against the maze's `marks`.)

use bevy_ecs::prelude::*;

use crate::execute::readings::{clear, reading, room_of};
use crate::tower::charm::Kind;
use crate::tower::lattice::{Binding, COLUMNS};
use crate::tower::{self, charm};

/// Republish everything about the lattice.
///
/// Takes the lattice rather than reading `Cwd`, the rule the lens paid for: a
/// bound solver running while the player stands in the laboratory would find no
/// lattice and leave every reading frozen at whatever it last said.
pub(crate) fn publish(world: &mut World, lattice: Entity) {
    let Some(room) = room_of(world, lattice) else {
        return;
    };
    let binding = world.get::<Binding>(lattice).cloned();

    clear(world, lattice);
    if let Some(binding) = &binding {
        // Which charm is open, so a spell can tell one lattice from another,
        // and nothing else: a `binding.lattice.set()` arm raising `graced` here
        // could never fire, because a fall that lights removes the `Binding`
        // before publishing.
        tower::raise_reading(world, lattice, &binding.kind);
    }

    // The residue, one column at a time — the whole signal. Eight openings
    // leave eight distinct residues, and the table that reads them lives in the
    // player's spell (§8.1's *"a rule, not a memory"*).
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
/// Neither word carries a count, which is what makes this cheap enough to run
/// on a tick: `graced` as ticks-remaining meant despawning and re-raising a
/// node once a second, and insertion order is the parse while `NodeId`s travel
/// in the save. Presence and the [`EBBING_AT`](charm::EBBING_AT) threshold
/// change rarely, so this compares and does nothing on almost every tick. The
/// exact figure is on the panel and in `survey`.
pub(crate) fn charms(world: &mut World, room: Entity) {
    // One query for all five, hoisted out of the loop: a fresh `QueryState` per
    // `Kind` is five a tick, and `meditate 3600` collapses those ticks into one
    // `step` — ~18,000 query states inside a single command.
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

        // Compare before clearing, or this is the churn the header refuses.
        let held: Vec<String> = tower::children_of(world, node)
            .into_iter()
            .filter_map(|child| world.get::<tower::Name>(child).map(|name| name.0.clone()))
            .collect();
        if held.len() == wanted.len()
            && wanted.iter().all(|word| held.iter().any(|had| had == word))
        {
            continue;
        }

        // A charm ending says so; a row that simply stopped draining was how a
        // player found out. Only on the edge, graced to gone — the comparison
        // above is what makes that once rather than every tick after.
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
/// Appended to the schedule and drawing nothing, the licence `settling`,
/// `erode` and `regenerate` already hold: a system that only reads the clock
/// cannot perturb any `RngStream`.
///
/// Without it the words are only true when a forge verb happens to run, and the
/// maintenance spell is built entirely on `ebbing` arriving on its own.
pub fn lapse(world: &mut World) {
    let Some(room) = tower::find_by_path(world, "/tower/forge") else {
        return;
    };
    charms(world, room);
}

/// The most ticks any tool in the tower has left, of every kind at once.
///
/// The longest rather than the nearest, so `ebbing` fires when the last one is
/// running out — renewing on the first tool to dip would relay a charm with
/// most of its life left, once a lap, for ever.
///
/// One pass for all five, indexed by `Kind`, because this runs on a tick.
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

// A `Cwd`-based `refresh` used to live here. It resolved the forge through
// `readings::fixture` — where the player is standing — which at
// `Sim::with_schedule` time is the filesystem root, so it silently did nothing.
// The verbs call `publish` with the lattice they already hold, and the
// bootstrap calls [`lapse`], which addresses the forge by path.
