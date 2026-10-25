//! `debug_spawn`: what it puts where, carried out on the tick after it is typed.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::word::{Order, SPAWN};
use crate::content::{Prose, Recipes};
use crate::session::Scrollback;
use crate::tower::{self, Store};

/// Carry out `order`, on the tick boundary like every other effect.
pub fn run(world: &mut World, order: &Order) {
    match order {
        Order::List => say(
            world,
            "debug_spawn_known",
            &known(world).join(", "),
            Role::Normal,
        ),
        Order::Spawn { name, count, into } => spawn(world, name, *count, into.as_deref()),
    }
}

/// Put `count` of `name` on the shelf, or wherever `into` names.
fn spawn(world: &mut World, name: &str, count: u32, into: Option<&str>) {
    // Known names only: `xyzzy` would put a node in the tower no recipe, no
    // instrument and no `survey` row knows what to do with.
    let Some(known) = known(world)
        .into_iter()
        .find(|word| word.eq_ignore_ascii_case(name))
    else {
        say(world, "debug_spawn_unknown", name, Role::Danger);
        return;
    };

    // Where the game would have left it, by default, so a tester needs no
    // knowledge of the layout; a named destination overrides it.
    let into = match into {
        Some(into) => shelf(world, into),
        None => tower::home(world, &known).or_else(|| dispensary(world)),
    };
    let Some(into) = into else {
        say(world, "debug_spawn_nowhere", &known, Role::Danger);
        return;
    };
    // The kind the laboratory would have given it. Spawning `clarity` as a
    // reagent puts a node in the tower with the right name and the wrong kind —
    // one no `distil` could have made and no slot will take.
    let kind = tower::kind_of(world, &known);

    // The arsenal's door holds for a tester too, asked of the kind as the door
    // asks (§19): a tool that builds impossible worlds files false bugs.
    if world.get::<tower::Keep>(into).is_some()
        && !matches!(
            kind,
            crate::parser::NounKind::Essence | crate::parser::NounKind::Scroll
        )
    {
        say(world, "debug_spawn_unkept", &known, Role::Danger);
        return;
    }
    tower::give(world, into, &known, kind, count);
    // Stamped to a full store, since standing is a rate this word never
    // records and a rate of one is `thin`. A test wanting a thin store ages it
    // with `meditate`.
    for _ in 0..tower::FRESH_AT {
        tower::made(world, &known);
    }
    say(world, "debug_spawn_done", &known, Role::Success);
}

/// The tower's shelf, wherever the player is standing.
///
/// Walked from the root, so no walking is put back. `None` only if the tower
/// has no `Store`, a build bug reported rather than panicked on.
fn dispensary(world: &World) -> Option<Entity> {
    let mut stack = vec![tower::root(world)];
    while let Some(node) = stack.pop() {
        if world.get::<Store>(node).is_some() {
            return Some(node);
        }
        stack.extend(tower::children_of(world, node));
    }
    None
}

/// The shelf called `named`, anywhere in the tower.
///
/// Somewhere a `move` could reach, as `pipeline::reachable` sees it: a fixture,
/// or the arsenal, which is a domain. Naming `laboratory` finds nothing.
fn shelf(world: &World, named: &str) -> Option<Entity> {
    let leaf = crate::parser::leaf(named);
    let mut stack = vec![tower::root(world)];
    while let Some(node) = stack.pop() {
        // A way is a fixture but no shelf: `research::publish` empties it next
        // step, silently.
        let holds_stock = (world.get::<tower::Fixture>(node).is_some()
            || world.get::<tower::Keep>(node).is_some())
            && world.get::<tower::Reading>(node).is_none();
        if holds_stock
            && world
                .get::<tower::Name>(node)
                .is_some_and(|name| name.0 == leaf)
        {
            return Some(node);
        }
        stack.extend(tower::children_of(world, node));
    }
    None
}

/// Every name that can be spawned.
///
/// [`Recipes::substances`], which the parser uses too, so a real substance is
/// never fuzzed into another (§19).
fn known(world: &World) -> Vec<String> {
    Recipes::substances(world)
}

/// Say what happened. §3 forbids unlogged output, and a debug tool is not
/// exempt — a tester who cannot see that the spawn failed is a tester chasing
/// the wrong bug.
fn say(world: &mut World, key: &str, detail: &str, role: Role) {
    let message = world
        .resource::<Prose>()
        .line(key, &[("name", detail), ("detail", detail)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, SPAWN)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the place called `place` holds after `line`, by name and count.
    ///
    /// One place, since the tower always has the endless `sage`.
    fn held_by(line: &str, place: &str) -> Vec<(String, u32)> {
        let mut sim = crate::Sim::new(1);
        sim.submit(line);
        sim.step();
        let world = sim.world();

        let mut stack = vec![tower::root(world)];
        while let Some(node) = stack.pop() {
            if world
                .get::<tower::Name>(node)
                .is_some_and(|name| name.0 == place)
            {
                return tower::holdings(world, node);
            }
            stack.extend(tower::children_of(world, node));
        }
        panic!("the tower has no `{place}`");
    }

    #[test]
    fn every_name_the_tool_offers_lands_in_the_room_it_belongs_to() {
        // The list, the door and the layout agree, against `tower::home`: a word
        // listed and then refused sends a tester hunting in the game.
        let sim = crate::Sim::new(1);
        let names = known(sim.world());
        assert!(!names.is_empty(), "the tool offers nothing at all");

        for name in names {
            let want = {
                let world = sim.world();
                let node = tower::home(world, &name)
                    .unwrap_or_else(|| panic!("`{name}` is offered and has no home"));
                world
                    .get::<tower::Name>(node)
                    .map_or_else(String::new, |place| place.0.clone())
            };
            let held = held_by(&format!("debug_spawn {name}"), &want);
            assert!(
                held.iter().any(|(held, count)| *held == name && *count > 0),
                "`{name}` is offered by the list and did not land in `{want}`: {held:?}",
            );
        }
    }

    #[test]
    fn every_material_the_game_has_is_one_the_tool_can_make() {
        // The other direction: a material no recipe names would be unspawnable,
        // and both files would still parse.
        let sim = crate::Sim::new(1);
        let names = known(sim.world());
        let materials = crate::content::Materials::builtin();
        for material in materials.names() {
            assert!(
                names.iter().any(|known| known == material),
                "`{material}` is authored in materials.toml and cannot be spawned",
            );
        }
    }

    #[test]
    fn the_arsenal_is_a_destination_and_keeps_its_door() {
        // The arsenal is a **domain**, not a `Fixture`, so the shelf lookup
        // refused it — which put every arsenal state back out of a tester's
        // reach, the exact gap a named destination was added to close.
        let kept = held_by("debug_spawn clarity 1 arsenal", tower::ARSENAL);
        assert!(
            kept.iter().any(|(name, _)| name == "clarity"),
            "a potion could not be put in the arsenal: {kept:?}",
        );

        // And the door holds for a tester too: stock in the arsenal is a state no
        // `move` could produce.
        let kept = held_by("debug_spawn sage 1 arsenal", tower::ARSENAL);
        assert!(
            !kept.iter().any(|(name, _)| name == "sage"),
            "the arsenal took a reagent from the tool: {kept:?}",
        );
    }

    #[test]
    fn a_way_is_not_a_shelf() {
        // A way is a `Fixture` that `research::publish` empties next step.
        for way in ["north", "east", "south", "west"] {
            let held = held_by(&format!("debug_spawn sage 1 {way}"), way);
            assert!(
                !held.iter().any(|(name, _)| name == "sage"),
                "`{way}` took a reagent it will silently destroy: {held:?}",
            );
        }

        // ...and an ordinary domain is still refused, which is the rule the ways
        // are an exception *to* rather than a change in it.
        let held = held_by("debug_spawn sage 1 laboratory", "laboratory");
        assert!(
            !held.iter().any(|(name, _)| name == "sage"),
            "stock was put on a domain node, where `move` cannot reach it",
        );
    }
}
