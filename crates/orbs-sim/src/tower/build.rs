//! The tower the player starts in.
//!
//! DESIGN.md §15 fixes the slice at **brewing and archive** — the two starting
//! domains — so only those two branches exist. The other five arrive with §10's
//! breadth item in Phase 3a.
//!
//! # Names are not prose
//!
//! Rule 6 and §12 put authored text in hot-reloadable content files, and §19
//! already set where the line falls: *"zero authored prose crosses into Rust"* —
//! the parser's own tables emit **facts**, never sentences, and `Verb::canonical`
//! and `NounKind::label` are const tables nobody calls a violation.
//!
//! So this file names things and nothing else. There is not a sentence in it.
//! The moment a fragment needs deciphered *text*, that text belongs in Phase 1's
//! content file rather than here — and needing it is the signal that Phase 1 has
//! been imported early.

use bevy_ecs::prelude::*;

use super::node::{Cwd, Name, Nameable, NodeIds, Protected};
use crate::parser::NounKind;

/// The tower root. Protected: §7 guards catastrophic targets in character.
const ROOT: &str = "tower";

/// Branches of the starting tower, and what each holds.
///
/// Order is load-bearing. §6 resolves a tie to whichever noun was registered
/// first, so the spawn order here is part of the world's determinism — see
/// [`node`](super::node).
const BRANCHES: &[Branch] = &[
    Branch {
        name: "alembic",
        holds: &[
            Holding::new(NounKind::Essence, &["clarity", "warding", "haste"]),
            Holding::new(NounKind::Vessel, &["retort", "crucible"]),
            Holding::new(NounKind::File, &["alembic.log"]),
        ],
    },
    Branch {
        name: "archive",
        holds: &[
            Holding::new(
                NounKind::Fragment,
                &["sigil-iv", "sigil-ix", "the-quiet-page"],
            ),
            Holding::new(NounKind::File, &["archive.log"]),
        ],
    },
];

struct Branch {
    name: &'static str,
    holds: &'static [Holding],
}

struct Holding {
    kind: NounKind,
    names: &'static [&'static str],
}

impl Holding {
    const fn new(kind: NounKind, names: &'static [&'static str]) -> Self {
        Self { kind, names }
    }
}

/// Raise the starting tower and stand the player at its root.
///
/// Called from `Sim::new`, never by a frontend: if the Bevy build, `orbs-tui`
/// and `orbs-balance` each built their own world they could diverge, which is
/// the failure §13 exists to prevent — *"if the live game and the CLI harness
/// diverged, we would not find out until Phase 3."*
pub fn raise(world: &mut World) {
    let root = spawn(world, None, ROOT, NounKind::Place);
    world.entity_mut(root).insert(Protected);

    for branch in BRANCHES {
        let at = spawn(world, Some(root), branch.name, NounKind::Place);
        // Branches are places the player lives in; losing one would end the
        // slice, so §7's guard covers them too.
        world.entity_mut(at).insert(Protected);

        for holding in branch.holds {
            for name in holding.names {
                let node = spawn(world, Some(at), name, holding.kind);
                if name.ends_with(".log") {
                    world.entity_mut(node).insert(super::sabotage::Log);
                }
            }
        }
    }

    world.insert_resource(Cwd(root));
}

/// Spawn one node under `parent`, in order.
fn spawn(world: &mut World, parent: Option<Entity>, name: &str, kind: NounKind) -> Entity {
    let id = world.resource_mut::<NodeIds>().issue();
    let node = world
        .spawn((id, Name(name.to_owned()), Nameable(kind)))
        .id();
    if let Some(parent) = parent {
        world.entity_mut(node).insert(ChildOf(parent));
    }
    node
}

#[cfg(test)]
mod tests {
    use super::super::node::{children_of, path_of};
    use super::*;
    use crate::Sim;

    #[test]
    fn every_noun_kind_the_slice_uses_has_something_to_resolve_against() {
        // The point of the whole item for §15's gate. Until this existed, the
        // Essence, Vessel, Fragment and Place slots were unfillable, so half the
        // sixteen-verb vocabulary could not be exercised by a tester at all.
        //
        // Walked rather than checked from one spot: a domain's belongings are
        // nameable only from inside it, which is §7's whole point — see
        // `tower::scene`.
        let mut sim = Sim::new(1);
        let mut seen = Vec::new();
        for domain in ["tower", "alembic", "archive"] {
            sim.submit(&format!("attend {domain}"));
            sim.step();
            seen.extend(sim.scene().nouns().iter().map(|noun| noun.kind));
        }

        for wanted in [
            NounKind::Place,
            NounKind::File,
            NounKind::Essence,
            NounKind::Vessel,
            NounKind::Fragment,
        ] {
            assert!(seen.contains(&wanted), "nothing anywhere is a {wanted:?}");
        }
    }

    #[test]
    fn the_tower_is_a_tree_with_the_two_slice_domains() {
        let sim = Sim::new(1);
        let world = sim.world();
        let root = world.resource::<Cwd>().0;

        assert_eq!(path_of(world, root), "/tower");
        let branches: Vec<String> = children_of(world, root)
            .into_iter()
            .filter_map(|child| world.get::<Name>(child).map(|name| name.0.clone()))
            .collect();
        assert_eq!(branches, ["alembic", "archive"]);
    }

    #[test]
    fn a_path_is_built_from_the_place_it_names() {
        let sim = Sim::new(1);
        let world = sim.world();
        let root = world.resource::<Cwd>().0;
        let alembic = children_of(world, root)[0];

        assert_eq!(path_of(world, alembic), "/tower/alembic");
        let first = children_of(world, alembic)[0];
        assert_eq!(path_of(world, first), "/tower/alembic/clarity");
    }

    #[test]
    fn the_same_seed_raises_the_same_tower() {
        // Node identities are what a bound script and a save both point at, so
        // two runs must agree on them.
        let a = Sim::new(7);
        let b = Sim::new(7);
        let ids = |sim: &Sim| -> Vec<u64> {
            let world = sim.world();
            let mut out = Vec::new();
            let mut stack = vec![world.resource::<Cwd>().0];
            while let Some(node) = stack.pop() {
                if let Some(id) = world.get::<super::super::node::NodeId>(node) {
                    out.push(id.get());
                }
                stack.extend(children_of(world, node));
            }
            out
        };
        assert_eq!(ids(&a), ids(&b));
    }

    #[test]
    fn the_root_and_its_branches_cannot_be_destroyed() {
        // §7: destruction is a tool, not a trap. Catastrophic targets refuse.
        let sim = Sim::new(1);
        let world = sim.world();
        let root = world.resource::<Cwd>().0;

        assert!(world.get::<Protected>(root).is_some(), "the tower root");
        for branch in children_of(world, root) {
            assert!(world.get::<Protected>(branch).is_some(), "a live domain");
        }
    }
}
