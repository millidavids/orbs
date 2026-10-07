//! `status` — how the tower stands, in one glance.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind};

use crate::content::Prose;
use crate::rng::Rngs;
use crate::session::{Pending, Scrollback};
use crate::tick::Tick;
use crate::tower;

/// Report what the world currently is.
///
/// Every value here belongs to a subsystem that otherwise has no way of being
/// seen from the game — the tick and seed are the determinism spine.
pub(super) fn status(world: &mut World) {
    let tick = world.resource::<Tick>().get();
    let seed = world.resource::<Rngs>().master_seed();
    let logged = world.resource::<Scrollback>().records().len();
    let queued = world.resource::<Pending>().len();
    // §11.5's two progression numbers, and the only place both are readable.
    // `concentration` is derived from `experience`: a player looking at 12
    // wants to know what 16 buys.
    let earned = world.resource::<tower::Experience>().get();
    let held = tower::concentration(world);
    // §11.5's second number, and the only one that can fall. One buys
    // capability and stays bought; the other is standing and can be lost.
    let known = world.resource::<tower::Renown>().get();
    // §11.5's mana, and its ceiling — which the Ley Line's `pool` and `floor`
    // raise. Nothing else on screen said the pool's size outside a siege board.
    let pool = u64::from(world.resource::<tower::Quintessence>().get());
    let ceiling = u64::from(tower::ceiling(world));

    // Read before the scrollback is borrowed, and one walk of `Running` rather
    // than a second — `tower::running_spells` is what the rail folds.
    let casting = tower::running_spells(world);
    let ledger = ledger(world);

    let mut scrollback = world.resource_mut::<Scrollback>();
    let rows = scrollback.records_mut();
    for (name, value) in [
        ("tick", tick),
        ("seed", seed),
        ("experience", earned),
        ("renown", known),
        ("concentration", quantity(held)),
        ("quintessence", pool),
        ("ceiling", ceiling),
        ("logged", quantity(logged)),
        ("queued", quantity(queued)),
    ] {
        rows.push(RecordKind::Status)
            .text(FieldName::Name, name)
            .count(FieldName::Quantity, value)
            .finish();
    }

    // What the tower has done in all (§19, number go up). Readings again, under
    // a heading of their own, so they take a column of their own. Before
    // `casting`, which returns early when nothing runs.
    if !ledger.is_empty() {
        rows.push(RecordKind::Section)
            .text(FieldName::Kind, LEDGER)
            .finish();
        for (name, value) in &ledger {
            rows.push(RecordKind::Status)
                .text(FieldName::Name, name)
                .count(FieldName::Quantity, u64::from(*value))
                .finish();
        }
    }

    // The full answer behind the rail's `+n`, which until now pointed at
    // nothing. The rail is the glance and this is the answer.
    //
    // A section, not more `Status` rows — the reading column above is guarded
    // by a *shape* test (two or more records, each a name and a numeric
    // quantity), so a row carrying a spell's room would break its alignment.
    // Absent when nothing runs, because a section that is usually a bare rule
    // teaches the eye to skip it.
    if casting.is_empty() {
        return;
    }
    rows.push(RecordKind::Section)
        .text(FieldName::Kind, CASTING)
        .finish();
    for one in casting {
        // `Detail` is what makes this a described listing rather than a tiled
        // one (`record/view.rs` decides on the field's presence): tiled, a run
        // of names reads `tending  threading` with nothing saying where either
        // is.
        let where_it_is = if one.cursors > 1 {
            format!("{}, on {} cursors", one.domain, one.cursors)
        } else {
            one.domain.clone()
        };
        rows.push(RecordKind::Entry)
            .text(FieldName::Name, &one.spell)
            .text(FieldName::Detail, &where_it_is)
            .finish();
    }
}

/// The ledger's rows as `status` lists them: every count that is not nought,
/// each followed by a spell's share of it when a spell had one, and last what
/// the tower's spells have done in all.
///
/// A share is named in full, never *of the row above*: a record stands alone
/// for `sift` and a pipe (rule 4).
fn ledger(world: &World) -> Vec<(String, u32)> {
    let prose = world.resource::<Prose>();
    let mut lines = Vec::new();
    for row in tower::ledger(world) {
        if row.count == 0 {
            continue;
        }
        let label = prose.line(&format!("ledger_{}", row.id), &[]);
        if row.by_spell > 0 {
            let share = prose.line("ledger_by_spell", &[("name", &label)]);
            lines.push((label, row.count));
            lines.push((share, row.by_spell));
        } else {
            lines.push((label, row.count));
        }
    }
    let spells = world.resource::<tower::Tally>().count(tower::SPELL);
    if spells > 0 {
        lines.push((prose.line("ledger_spells", &[]), spells));
    }
    lines
}

/// The heading over the ledger. A table entry, for [`CASTING`]'s reason.
const LEDGER: &str = "ledger";

/// The heading `status` puts over what is running.
///
/// A table entry beside `Verb::canonical`, not authored prose — it is the same
/// class of thing as a noun-kind label, which is what every other section
/// heading in a listing is.
const CASTING: &str = "casting";

/// A count as a record value, saturating rather than wrapping.
fn quantity(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}
