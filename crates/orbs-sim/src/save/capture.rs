//! Reading a world out into a document.
//!
//! Everything here takes `&World` and nothing takes `&mut`. That is not tidiness
//! — it is the guarantee that **saving cannot perturb the world**. A capture that
//! could take a `&mut` could build a `QueryState`, which registers components,
//! which changes archetypes; and a save that quietly changed the thing it was
//! describing would make the lockstep test in `tests/persistence.rs` measure
//! itself.
//!
//! So the walk uses `world.get::<T>(entity)` and [`children_of`], which is also
//! the *ordering* rule `tower::node` insists on: **`Children` order, never a
//! global query.** Archetype order is not insertion order, and a document whose
//! rows moved when a component was added would fail its own byte-for-byte test
//! for a reason that has nothing to do with the save.

use bevy_ecs::prelude::*;
use orbs_render::{Presentation, Role, Value};

use super::document::{Away, FORMAT, MarkSave, ProgressSave, RecordSave, RngSave, Save, WorldSave};
use super::naming;
use super::node::{NodeSave, RunningSave, SpanSave, SubstitutedSave, WorkingSave};
use crate::rng::Rngs;
use crate::session::{Scrollback, Wizard};
use crate::tick::Tick;
use crate::tower::{self, Maze, Stock, Ward};
use crate::tower::{Cwd, Marks};

/// How many records a save carries.
///
/// **A tail, not the stream.** The stream grows without bound — §5's Phase 9a
/// catch-up alone is ~29k steps — and a save that carried all of it would grow
/// with the session until the file was mostly transcript. Five hundred is a few
/// screens of `peruse` and comfortably more than the eight command blocks a
/// 120×45 pane holds, so a returning player finds their logs where they left
/// them.
///
/// What is *not* bounded is `Records::sequence`, which rides in `[world]`. A
/// running spell's cursor is a position in that sequence, so the count has to
/// survive the truncation even though the records do not.
pub(super) const RECORD_TAIL: usize = 500;

/// Read the whole world out.
pub(crate) fn capture(world: &World) -> Save {
    // `Pending` and `Skip` are deliberately absent from the document, and this
    // is where that claim is checked rather than trusted. Both are drained
    // inside `Sim::step` — `run_pending` opens every tick and `Skip` is spun out
    // in `step`'s own `while` loop — so at a tick boundary, which §8 says is the
    // only moment a save is permitted, both are empty. If that ever stops being
    // true, a queued command is being dropped on save and this is the line that
    // says so.
    debug_assert!(
        world.resource::<crate::session::Pending>().is_empty(),
        "a save was taken with commands still queued — §8 permits saves at tick boundaries only",
    );

    let records = world.resource::<Scrollback>().records();
    Save {
        world: WorldSave {
            format: FORMAT,
            built_by: env!("CARGO_PKG_VERSION").to_owned(),
            seed: world.resource::<Rngs>().master_seed(),
            tick: world.resource::<Tick>().get(),
            sequence: records.sequence(),
        },
        // The sim has no wall clock and must not acquire one; `orbs-shell`
        // stamps this on the way to the file. The tick is ours to know.
        away: Away {
            tick: world.resource::<Tick>().get(),
            unix: 0,
        },
        rng: RngSave {
            positions: world
                .resource::<Rngs>()
                .positions()
                .iter()
                .map(u128::to_string)
                .collect(),
        },
        progress: progress(world),
        nodes: nodes(world),
        records: tail(records),
    }
}

/// What the player has earned, found and chosen.
fn progress(world: &World) -> ProgressSave {
    let learned = world.resource::<tower::Learned>();
    let cwd = world.resource::<Cwd>().0;
    ProgressSave {
        wizard: world.resource::<Wizard>().name().to_owned(),
        experience: world.resource::<tower::Experience>().get(),
        taken: world.resource::<tower::Taken>().ids().to_vec(),
        learned: learned.known().map(str::to_owned).collect(),
        fruitless: learned.since(),
        cwd: Some(tower::path_of(world, cwd)),
        register: Some(
            naming::register_word(world.resource::<Scrollback>().records().register()).to_owned(),
        ),
        asked: world
            .resource::<crate::session::Choices>()
            .asked()
            .map(str::to_owned),
        marks: world
            .resource::<Marks>()
            .iter()
            .map(|(domain, mark)| MarkSave {
                domain: domain.to_owned(),
                mark: naming::mark_word(mark).to_owned(),
            })
            .collect(),
    }
}

/// Every named node, in the order a walk from the root meets it.
///
/// The root itself is skipped: it is nameless, so it has no path to be addressed
/// by, and a restore never needs to create it because `tower::raise` already
/// has. Parents therefore always precede their children here, which is what lets
/// a restore create a missing node without looking ahead.
fn nodes(world: &World) -> Vec<NodeSave> {
    let root = tower::filesystem_root(world, world.resource::<Cwd>().0);
    let mut found = Vec::new();
    let mut stack: Vec<Entity> = tower::children_of(world, root);
    stack.reverse();
    while let Some(entity) = stack.pop() {
        found.push(node(world, entity));
        let mut children = tower::children_of(world, entity);
        children.reverse();
        stack.extend(children);
    }
    found
}

/// One node, and everything true of it.
fn node(world: &World, entity: Entity) -> NodeSave {
    let at = world.entity(entity);
    NodeSave {
        path: tower::path_of(world, entity),
        id: at.get::<tower::NodeId>().map_or(0, |id| id.get()),
        kind: at
            .get::<tower::Nameable>()
            .map_or("any", |kind| naming::kind_word(kind.0))
            .to_owned(),

        protected: at.contains::<tower::Protected>(),
        fixture: at.contains::<tower::Fixture>(),
        heat_source: at.contains::<tower::HeatSource>(),
        store: at.contains::<tower::Store>(),
        keep: at.contains::<tower::Keep>(),
        reading: at.contains::<tower::Reading>(),
        product: at.contains::<tower::Product>(),
        poisoned: at.contains::<tower::Poisoned>(),
        log: at.contains::<tower::Log>(),

        operation: at
            .get::<tower::Operation>()
            .map(|op| naming::verb_word(op.0).to_owned()),
        stock: at.get::<Stock>().map(|stock| match stock {
            Stock::Endless => "endless".to_owned(),
            Stock::Counted(n) => n.to_string(),
        }),
        held: at.get::<tower::Held>().map(|held| held.0.clone()),
        domain: at.get::<tower::Domain>().map(|domain| domain.0.clone()),
        banked: at.get::<tower::Banked>().map(|banked| banked.0),
        ash: at.get::<tower::Ash>().map(|ash| ash.0.clone()),
        bidden: at.get::<tower::Bidden>().map(|bidden| bidden.0.clone()),
        bound: at
            .get::<tower::spell::Bound>()
            .map(|bound| bound.said.clone()),

        working: at.get::<tower::Working>().map(|work| WorkingSave {
            verb: naming::verb_word(work.verb).to_owned(),
            subject: tower::path_of_id(world, work.subject).unwrap_or_default(),
            started: work.started.get(),
            ends: work.ends.get(),
        }),
        triaging: at.get::<tower::Triaging>().map(|triage| SpanSave {
            started: triage.started.get(),
            ends: triage.ends.get(),
        }),
        // A budget in the world, a completion tick in the file — see `SpanSave`.
        burning: at.get::<tower::Burning>().map(|fire| SpanSave {
            started: fire.lit_at.get(),
            ends: fire.lit_at.get().saturating_add(fire.ticks),
        }),
        quickened: at.get::<tower::Quickened>().map(|quick| SpanSave {
            started: quick.from.get(),
            ends: quick.from.get().saturating_add(quick.ticks),
        }),
        substituted: at.get::<tower::Substituted>().map(|lie| SubstitutedSave {
            was: lie.was.clone(),
            since: lie.since.get(),
        }),

        maze: at.get::<Maze>().map(Maze::to_save),
        ward: at.get::<Ward>().map(Ward::to_save),
        running: at
            .get::<tower::spell::Running>()
            .map(|run| running(world, run)),
    }
}

/// A spell part-way through.
fn running(world: &World, run: &tower::spell::Running) -> RunningSave {
    RunningSave {
        spell: tower::path_of_id(world, run.spell).unwrap_or_default(),
        pc: run.pc.clone(),
        loops: run.loops.iter().map(super::adopt::loop_code).collect(),
        seen: run.seen,
        depth: run.depth,
        unattended: run.unattended,
        at: tower::path_of_id(world, run.at).unwrap_or_default(),
        waiting_since: run.waiting_since.map(Tick::get),
        said: run.said.clone(),
        // The text the program was compiled from, so a restore can tell whether
        // re-deriving it would land on the same tree. See `restore::spell`.
        fingerprint: tower::path_of_id(world, run.spell)
            .and_then(|path| tower::find_by_path(world, &path))
            .and_then(|entity| world.get::<tower::Held>(entity))
            // **The empty fingerprint, not nought.** A spell node with no
            // `Held` is what this falls back to, and `adopt` recomputes it as
            // `fingerprint(&[])` — the FNV offset basis, never zero — so a
            // sentinel of `0` guaranteed a mismatch on the one path that
            // produces it, stopping the run and naming the empty string.
            .map_or_else(
                || super::adopt::fingerprint(&[]),
                |held| super::adopt::fingerprint(&held.0),
            ),
    }
}

/// The last [`RECORD_TAIL`] records, oldest first.
fn tail(records: &orbs_render::Records) -> Vec<RecordSave> {
    let skip = records.len().saturating_sub(RECORD_TAIL);
    records
        .iter()
        .skip(skip)
        .map(|record| RecordSave {
            kind: naming::record_kind_word(record.kind()).to_owned(),
            // Written only when it is not the default. Half the stream is an
            // ordinary line spoken plainly, and two rows saying so on every one
            // of five hundred records is four hundred lines of a save file
            // telling a reader nothing.
            role: (record.role() != Role::default())
                .then(|| naming::role_word(record.role()).to_owned()),
            register: (record.presentation() != Presentation::default())
                .then(|| naming::register_word(record.presentation()).to_owned()),
            quiet: record.is_quiet(),
            spoken: record.spoken().map(str::to_owned),
            fields: record
                .fields()
                .map(|(name, value)| {
                    let (shape, text) = match value {
                        Value::Text(text) => ("text", text.to_owned()),
                        Value::Count(n) => ("count", n.to_string()),
                        Value::Tick(n) => ("tick", n.to_string()),
                    };
                    (naming::field_word(name).to_owned(), shape.to_owned(), text)
                })
                .collect(),
        })
        .collect()
}
