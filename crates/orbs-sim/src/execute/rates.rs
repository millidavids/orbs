//! `status rates` — how fast each open room makes what its ledger counts, what
//! it waits on, and what holds the tower's slot (§19, *the tower speaks up*).
//!
//! The bottleneck made visible: ROADMAP's *the player can see a bottleneck*.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind};

use crate::content::Prose;
use crate::parser::RATES;
use crate::session::Scrollback;
use crate::tower::{self, Hourly};

/// The row naming the tower's one production slot. A table word, like the
/// section heading above it.
const SLOT: &str = "slot";

/// List every open room's rate and wait, then the slot.
pub(super) fn rates(world: &mut World) {
    let yields = tower::yields(world);
    let slot = tower::slot(world);
    let prose = world.resource::<Prose>();

    let mut rows: Vec<(String, String)> = yields
        .iter()
        .map(|one| {
            let label = one
                .row
                .as_ref()
                .map(|id| prose.line(&format!("ledger_{id}"), &[]))
                .unwrap_or_default();
            let rate = match one.rate {
                Hourly::Unknown => prose.line("rates_rate_unknown", &[("name", &label)]),
                Hourly::None => prose.line("rates_rate_none", &[("name", &label)]),
                Hourly::PerHour(count) => prose.line(
                    "rates_rate",
                    &[("name", &label), ("count", &count.to_string())],
                ),
            };
            // The rail's detail speaks this same sentence.
            let wait = tower::wait_sentence(prose, &one.wait);
            let row = prose.line("rates_row", &[("detail", &rate), ("state", &wait)]);
            (one.room.to_owned(), row)
        })
        .collect();
    rows.push((
        SLOT.to_owned(),
        slot.map_or_else(
            || prose.line("rates_slot_free", &[]),
            |slot| {
                prose.line(
                    "rates_slot_held",
                    &[
                        ("name", &slot.name),
                        ("state", slot.doing),
                        ("count", &slot.left.to_string()),
                    ],
                )
            },
        ),
    ));

    let mut scrollback = world.resource_mut::<Scrollback>();
    let records = scrollback.records_mut();
    records
        .push(RecordKind::Section)
        .text(FieldName::Kind, RATES)
        .finish();
    // `Detail` makes it a described listing, as `casting` is: one row each,
    // the room and then its sentence.
    for (name, detail) in rows {
        records
            .push(RecordKind::Entry)
            .text(FieldName::Name, &name)
            .text(FieldName::Detail, &detail)
            .finish();
    }
}
