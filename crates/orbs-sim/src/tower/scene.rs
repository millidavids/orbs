//! What the player can currently name.
//!
//! DESIGN.md §6, step 2: fuzzy match against known vocabulary **and entities that
//! currently exist**. That second half is what separates this from a command
//! parser — `decoct clarity` resolves because clarity is an essence, and stops
//! resolving the moment it is not.
//!
//! # Why this walks the tree instead of querying
//!
//! §6 resolves a scoring tie *"to whichever noun was registered first, so the
//! result never depends on iteration luck"*. Registration order is therefore
//! part of the parse, and a global `Query` would supply archetype order — which
//! is **not** insertion order, and which changes when a component is added or
//! removed. Starting a brew adds a component; that would move its entity to a
//! new table, reorder the noun list, flip a tie, and change what a phrase
//! resolves to. Two runs from one seed would diverge and no test would see it.
//!
//! Walking `Children` from the root is insertion-ordered by construction, so the
//! scene is the same on every run and after every mutation.
//!
//! # Why per tick rather than on change
//!
//! Rebuilding when something changes is a cache-invalidation bug waiting for the
//! first system that mutates without setting a marker, and it would make replay
//! depend on which mutations remembered to. Per tick is trivially replay-safe,
//! and at tens of nouns and 1 Hz the cost is not measurable.

use bevy_ecs::prelude::*;

use super::node::{Cwd, Name, Nameable, children_of, path_of};
use crate::execute::LOG;
use crate::parser::{NounKind, Scene};

/// Rebuild the nameable surface of the world.
///
/// # You can only name what is where you are
///
/// §7 makes the tree the tower and navigation diegetic — *"paths are places"* —
/// so a domain's contents are nameable **only while the player is standing in
/// that domain**. `decoct clarity` works in `/tower/laboratory` and nowhere else.
///
/// That is the base state, not a limitation: §19 settles **pane addressing** —
/// *"named by domain, routed within the focused set"* — as a **Phase 2** item,
/// which is precisely the unlock that later lets a player act on a domain
/// without walking to it. Acting at a distance has to *become* possible, and it
/// cannot if it was free from the start.
///
/// **Places are exempt.** Every place stays nameable from everywhere, because
/// navigation is how you reach the thing you cannot yet name — gating movement
/// on being somewhere would be a lock whose key is behind it.
pub fn rebuild(world: &mut World) {
    let Some(cwd) = world.get_resource::<Cwd>().copied() else {
        return;
    };

    // §3's log is nameable from the moment the game starts and is not a node in
    // the tree, so it is registered first — dropping it here would silently kill
    // `peruse orb.log` and `sift <pattern> orb.log`.
    let mut scene = Scene::new().with(NounKind::File, LOG);

    // Every place, wherever the player is. Depth-first from the root, children
    // in spawn order.
    for node in walk(world, root_of(world, cwd.0)) {
        if world.get::<Nameable>(node).map(|n| n.0) == Some(NounKind::Place) {
            // A place answers to its full path; §6's matcher also accepts the
            // last segment, which is what makes `attend laboratory` reach
            // `/tower/laboratory` (§7: players say the place, not the path).
            scene = scene.with(NounKind::Place, &path_of(world, node));
        }
    }

    // Everything else: only what is here.
    for node in children_of(world, cwd.0) {
        let (Some(name), Some(kind)) = (world.get::<Name>(node), world.get::<Nameable>(node))
        else {
            continue;
        };
        if kind.0 == NounKind::Place {
            continue;
        }
        scene = scene.with(kind.0, &name.0.clone());
    }

    world.insert_resource(scene);
}

/// Every node under `from`, depth-first, children in spawn order.
fn walk(world: &World, from: Entity) -> Vec<Entity> {
    let mut stack = vec![from];
    let mut seen = Vec::new();
    while let Some(node) = stack.pop() {
        seen.push(node);
        let mut kids = children_of(world, node);
        // Reversed onto the stack so they pop back in spawn order.
        kids.reverse();
        stack.extend(kids);
    }
    seen
}

/// Walk up from `node` to the tower root.
fn root_of(world: &World, node: Entity) -> Entity {
    let mut at = node;
    while let Some(parent) = world.get::<ChildOf>(at).map(ChildOf::parent) {
        at = parent;
    }
    at
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    fn names(sim: &Sim) -> Vec<String> {
        sim.scene()
            .nouns()
            .iter()
            .map(|noun| noun.name.clone())
            .collect()
    }

    #[test]
    fn the_log_survives_a_rebuild() {
        // It is not a node in the tree, so a rebuild that only walks the tree
        // would drop it — and `peruse orb.log` would stop resolving with no
        // other symptom.
        let mut sim = Sim::new(1);
        sim.step_n(3);
        assert!(names(&sim).iter().any(|name| name == LOG));
    }

    #[test]
    fn the_order_is_the_order_things_were_spawned() {
        // §6 breaks scoring ties by registration order, so this *is* the parse.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        let found = names(&sim);

        let place = |want: &str| found.iter().position(|n| n == want);
        assert!(
            place("/tower/laboratory") < place("/tower/archive"),
            "branches keep their declared order",
        );
        assert!(
            place("clarity") < place("warding") && place("warding") < place("haste"),
            "essences keep theirs: {found:?}",
        );
        assert!(
            place("/tower") < place("clarity"),
            "places are registered before belongings",
        );
    }

    #[test]
    fn the_order_survives_a_component_being_added() {
        // The hazard this module exists for. Adding a component moves an entity
        // to a different archetype, so a query would reorder the scene here and
        // silently change which noun a tied phrase resolves to.
        let mut sim = Sim::new(1);
        sim.step();
        let before = names(&sim);

        let root = sim.world().resource::<Cwd>().0;
        let victim = super::children_of(sim.world(), root)[0];
        sim.world_mut().entity_mut(victim).insert(Marker);
        sim.step();

        assert_eq!(before, names(&sim), "the scene reordered");
    }

    #[derive(Component)]
    struct Marker;

    #[test]
    fn a_domains_belongings_are_nameable_only_from_inside_it() {
        // §7: the tree is the tower and navigation is diegetic. Brewing happens
        // in the laboratory because that is where the essences are, which is also
        // what gives §19's Phase 2 pane addressing something to be an unlock
        // *from* — acting at a distance has to become possible.
        let mut sim = Sim::new(1);
        sim.step();
        assert!(!names(&sim).iter().any(|name| name == "clarity"));

        sim.submit("attend laboratory");
        sim.step();
        assert!(names(&sim).iter().any(|name| name == "clarity"));
        assert!(!names(&sim).iter().any(|name| name == "sigil-iv"));

        sim.submit("attend archive");
        sim.step();
        assert!(names(&sim).iter().any(|name| name == "sigil-iv"));
        assert!(!names(&sim).iter().any(|name| name == "clarity"));
    }

    #[test]
    fn every_place_stays_reachable_from_everywhere() {
        // Gating movement on being somewhere would be a lock whose key is
        // behind it.
        let mut sim = Sim::new(1);
        for step in ["attend laboratory", "attend archive", "attend tower"] {
            sim.submit(step);
            sim.step();
            let found = names(&sim);
            for place in ["/tower", "/tower/laboratory", "/tower/archive"] {
                assert!(found.iter().any(|name| name == place), "{step}: {place}");
            }
        }
    }

    #[test]
    fn the_log_is_readable_from_anywhere() {
        // §3's stream is not a node in the tree and belongs to no domain.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        assert!(names(&sim).iter().any(|name| name == LOG));
    }

    #[test]
    fn the_scene_is_rebuilt_after_the_caller_s_systems_not_beside_them() {
        // The hazard: `rebuild` sharing a schedule with whatever a frontend adds
        // through `with_schedule` is an ambiguity, not an ordering — and Bevy's
        // topsort was running the caller's systems first despite `rebuild` being
        // inserted first. A separate pass makes the order a fact.
        //
        // A system that renames a node must therefore be visible to the scene in
        // the *same* tick, never the next.
        let mut sim = Sim::with_schedule(1, |schedule| {
            schedule.add_systems(rename_once);
        });
        sim.step();

        // A place registers by its full path, not its last segment.
        assert!(
            names(&sim).iter().any(|name| name == "/tower/renamed"),
            "the scene went a tick stale: {:?}",
            names(&sim),
        );
    }

    /// Rename the first branch, once.
    fn rename_once(mut done: Local<bool>, cwd: Res<Cwd>, mut names: Query<&mut Name>) {
        if *done {
            return;
        }
        *done = true;
        let _ = cwd;
        if let Some(mut name) = names.iter_mut().find(|name| name.0 == "laboratory") {
            name.0 = "renamed".to_owned();
        }
    }

    #[test]
    fn two_runs_name_the_world_identically() {
        let mut a = Sim::new(0xC0FFEE);
        let mut b = Sim::new(0xC0FFEE);
        a.step_n(5);
        b.step_n(5);
        assert_eq!(names(&a), names(&b));
    }
}
