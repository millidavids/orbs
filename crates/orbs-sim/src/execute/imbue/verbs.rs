//! The forge's three words.
//!
//! `imbue` opens a lattice, `snap` turns a column, `anneal` lets it fall. Only
//! `anneal` advances the world or holds the tower's one production slot —
//! §10's scarcity here, and why enchanting competes with brewing.

use bevy_ecs::prelude::*;

use orbs_render::Role;

use crate::parser::{Intent, Verb};
use crate::tower::charm::Kind;
use crate::tower::lattice::{Binding, COLUMNS, Lattice};
use crate::tower::{self, charm};

use super::publish::publish;
use super::shared::{besieged, fixture, say};

/// `imbue <tool> <charm>` — open a lattice.
///
/// Instant and free; `anneal` is what costs. §11.5 is *"cost is the resource,
/// never progress"*, and charging for looking is what §5.1 forbids.
pub(in crate::execute) fn imbue(world: &mut World, intent: &Intent) {
    let Some(lattice) = fixture(world) else {
        say(world, Verb::Imbue, "forge_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Binding>(lattice).is_some() {
        say(world, Verb::Imbue, "forge_busy", &[], Role::Cost);
        return;
    }

    let mut named = intent.arguments.iter().map(|argument| &argument.value);
    let (Some(tool), Some(charm_word)) = (named.next(), named.next()) else {
        say(world, Verb::Imbue, "forge_unopened", &[], Role::Cost);
        return;
    };
    // The leaf, for every sentence below: a resolved place arrives as a full
    // path, and prose echoing it shows a player the tower's internals. The
    // bailey records the same trap.
    let charm_word = crate::parser::leaf(charm_word).to_owned();
    let Some(kind) = Kind::from_word(&charm_word) else {
        say(
            world,
            Verb::Imbue,
            "forge_no_charm",
            &[("name", &charm_word)],
            Role::Cost,
        );
        return;
    };
    // A charm the tower has not earned refuses in voice (§11.5). The forge
    // opens knowing `hurried`; the rest come with the mastery lines.
    if !world
        .resource::<tower::Opened>()
        .has(&tower::charm_key(kind.word()))
    {
        say(
            world,
            Verb::Imbue,
            "forge_charm_unopened",
            &[("name", &charm_word)],
            Role::Cost,
        );
        return;
    }

    // A tool anywhere in the tower — this domain's one widening of §7, and a
    // small one: an instrument is a `NounKind::Place`, which `tower::scene`
    // already registers from everywhere, so it arrives resolved.
    let tool = tool.to_owned();
    let Some(target) = tower::find_by_path(world, &tool) else {
        say(
            world,
            Verb::Imbue,
            "forge_no_tool",
            &[("name", crate::parser::leaf(&tool))],
            Role::Cost,
        );
        return;
    };
    // A charm holds to a tool, never to a reading or a pile — otherwise imbuing
    // the hem yields a charm no read site asks about, the silent nothing §6
    // forbids.
    if !holds_a_charm(world, target) {
        say(
            world,
            Verb::Imbue,
            "forge_no_tool",
            &[("name", crate::parser::leaf(&tool))],
            Role::Cost,
        );
        return;
    }

    // One draw here rather than in a system, which is `defend`'s rule:
    // `RngStream::Forge` advances only when a player asks, so a later forge
    // system can join the schedule without shifting a replay.
    let drawn = {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        let bits: u64 = rand::Rng::random(rngs.stream(crate::rng::RngStream::Forge));
        Lattice::from_bits(bits)
    };
    let under_siege = besieged(world);
    let cost = priced(world, kind, under_siege);

    world.entity_mut(lattice).insert(Binding {
        kind: charm_word.clone(),
        tool: tool.clone(),
        lattice: drawn,
        spent: 0,
    });
    publish(world, lattice);
    say(
        world,
        Verb::Imbue,
        "forge_opened",
        &[("name", &charm_word), ("detail", &cost.to_string())],
        Role::Success,
    );
}

/// What a charm costs this tower: the authored price, less the Ley Line's
/// `thrift`, never below one.
///
/// One function for both sites, so the forge cannot quote one number and
/// charge another.
fn priced(world: &World, kind: Kind, besieged: bool) -> u32 {
    let asked = world
        .resource::<crate::content::Charms>()
        .cost(kind, besieged);
    asked.saturating_sub(tower::grant::thrift(world)).max(1)
}

/// Whether a charm laid on `node` would reach anything.
///
/// A fixture that is not a store and not a reading — an instrument, the stacks,
/// the rampart — or a spell, which is what `shielded` protects.
fn holds_a_charm(world: &World, node: Entity) -> bool {
    let is_spell = world.get::<tower::Name>(node).is_some_and(|name| {
        std::path::Path::new(&name.0)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("spell"))
    });
    is_spell
        || (world.get::<tower::Fixture>(node).is_some()
            && world.get::<tower::Store>(node).is_none()
            && world.get::<tower::Reading>(node).is_none())
}

/// `snap <column>` — turn one glyph and its neighbours.
///
/// Instant and free, like `pledge`'s choosing half — charging to arrange the
/// board would make a player pay to change their mind.
pub(in crate::execute) fn snap(world: &mut World, intent: &Intent) {
    let Some(lattice) = fixture(world) else {
        say(world, Verb::Snap, "forge_nowhere", &[], Role::Cost);
        return;
    };
    if world.get::<Binding>(lattice).is_none() {
        say(world, Verb::Snap, "forge_unopened", &[], Role::Cost);
        return;
    }
    let Some(named) = intent.arguments.first() else {
        say(world, Verb::Snap, "forge_unopened", &[], Role::Cost);
        return;
    };
    let word = crate::parser::leaf(&named.value).to_owned();
    let Some(column) = COLUMNS.iter().position(|name| *name == word) else {
        say(
            world,
            Verb::Snap,
            "forge_no_column",
            &[("name", &word)],
            Role::Cost,
        );
        return;
    };

    if let Some(mut binding) = world.get_mut::<Binding>(lattice) {
        binding.lattice.snap(column);
    }
    publish(world, lattice);
    say(
        world,
        Verb::Snap,
        "forge_snapped",
        &[("name", &word)],
        Role::Success,
    );
}

/// `anneal` — let the lattice fall, and set the charm if every glyph holds.
///
/// The operation: it holds the tower's one production slot for the charm's
/// `takes`, and spends quintessence whether or not the lattice lights — which
/// is what prices guessing.
pub(in crate::execute) fn anneal(world: &mut World) {
    let Some(lattice) = fixture(world) else {
        say(world, Verb::Anneal, "forge_nowhere", &[], Role::Cost);
        return;
    };
    let Some(binding) = world.get::<Binding>(lattice).cloned() else {
        say(world, Verb::Anneal, "forge_unopened", &[], Role::Cost);
        return;
    };
    let Some(kind) = Kind::from_word(&binding.kind) else {
        say(world, Verb::Anneal, "forge_unopened", &[], Role::Cost);
        return;
    };

    // §10.1's lock: one thing at a time, tower-wide.
    if let Some(why) = tower::busy(world, lattice) {
        tower::refuse_busy(world, Verb::Anneal, lattice, why);
        return;
    }

    let besieged = besieged(world);
    let cost = priced(world, kind, besieged);
    let held = world.resource::<tower::Quintessence>().get();
    if cost > held {
        say(
            world,
            Verb::Anneal,
            "forge_short",
            // `quantity`, not `state` — `forge_short` reads *"…and you hold
            // {quantity}"*, and the wrong key printed that literal at a
            // player. A refusal without its two numbers cannot be planned
            // around, and planning around it is the mechanic.
            &[
                ("name", &binding.kind),
                ("kind", &cost.to_string()),
                ("quantity", &held.to_string()),
            ],
            Role::Cost,
        );
        return;
    }
    world.resource_mut::<tower::Quintessence>().spend(cost);

    // The fall takes time, and the tower's one slot with it. This resolved
    // instantly for a whole step, so `Charm::takes` was dead and §10's scarcity
    // for the domain did not exist at all.
    //
    // The outcome is decided when the work lands, not here: a lattice that
    // resolved at once and then held the slot would be a charm you already had,
    // being paid for afterwards.
    let Some(subject) = world.get::<tower::NodeId>(lattice).copied() else {
        return;
    };
    let takes = world
        .resource::<crate::content::Charms>()
        .get(kind)
        .map_or(1, |charm| charm.takes);
    if !tower::begin(world, lattice, Verb::Anneal, subject, takes) {
        return;
    }

    // The spend is recorded on the binding now, so the tally is true while it
    // falls rather than only after it lands.
    if let Some(mut binding) = world.get_mut::<Binding>(lattice) {
        binding.spent = binding.spent.saturating_add(cost);
    }
    publish(world, lattice);
    say(
        world,
        Verb::Anneal,
        "forge_settling",
        &[("quantity", &takes.to_string())],
        Role::Cost,
    );
}

/// What a fall leaves, once it has taken its time.
///
/// Called from `land::finish` when an `anneal` completes, like the press and
/// the audit: work that makes no material, whose sentence depends on what it
/// found.
pub(crate) fn land(world: &mut World, lattice: Entity) {
    let Some(binding) = world.get::<Binding>(lattice).cloned() else {
        return;
    };
    let Some(kind) = Kind::from_word(&binding.kind) else {
        return;
    };

    let mut binding = binding;
    let lit = binding.lattice.settle();

    if lit {
        lay(world, &binding, kind);
        world.entity_mut(lattice).remove::<Binding>();
        // A fall that lights earns; one that springs back does not.
        // `land::finish` credits in the generic branch this one `continue`s
        // past, so without this the forge paid nothing and `orbs-balance` read
        // it as `0.0000/tick`. Only on success: a spell that reads the residue
        // earns every fall, `tending_blindly` one in eight.
        let earned = tower::worth(world, charm::LATTICE);
        tower::done(world, &tower::Work::at(charm::LATTICE), earned);
        publish(world, lattice);
        say(
            world,
            Verb::Anneal,
            "forge_set",
            &[
                ("name", &binding.kind),
                ("detail", crate::parser::leaf(&binding.tool)),
            ],
            Role::Success,
        );
        return;
    }

    let spent = binding.spent.to_string();
    world.entity_mut(lattice).insert(binding);
    publish(world, lattice);
    say(
        world,
        Verb::Anneal,
        "forge_failed",
        // `forge_failed` reads *"…that cost {quantity}"*, and it is the
        // commonest line in the domain — a wrong key put that literal on screen
        // more than any other sentence the forge says.
        &[("quantity", &spent)],
        Role::Cost,
    );
}

/// Put the charm on the tool.
fn lay(world: &mut World, binding: &Binding, kind: Kind) {
    let Some(target) = tower::find_by_path(world, &binding.tool) else {
        return;
    };
    let Some(ticks) = world
        .resource::<crate::content::Charms>()
        .get(kind)
        .map(|charm| charm.lasts)
    else {
        return;
    };
    let now = *world.resource::<crate::tick::Tick>();
    let mut held = world
        .get::<charm::Charmed>(target)
        .cloned()
        .unwrap_or_else(|| charm::Charmed(Vec::new()));
    held.lay(charm::Charm {
        kind,
        from: now,
        ticks,
    });
    world.entity_mut(target).insert(held);
}
