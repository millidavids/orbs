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

use super::node::{Cwd, Fixture, Name, Nameable, NodeIds, Protected};
use crate::parser::{NounKind, Verb};

/// The tower root. Protected: §7 guards catastrophic targets in character.
const ROOT: &str = "tower";

/// Branches of the starting tower, and what each holds.
///
/// Order is load-bearing. §6 resolves a tie to whichever noun was registered
/// first, so the spawn order here is part of the world's determinism — see
/// [`node`](super::node).
const BRANCHES: &[Branch] = &[
    Branch {
        name: "laboratory",
        holds: &[
            // No essences here any more. `clarity`, `warding` and `haste` were
            // nodes on the laboratory floor when one command brewed one; §10.1
            // makes them what the **alembic yields**, so they start in nobody's
            // hands. They are still nameable everywhere as recipe `Topic`s
            // (`scene::rebuild`), which is what `grimoire clarity` reads.
            //
            // `crucible` is gone and `balneum_mariae` took the processing stage
            // (§10.1, §19), which leaves `retort` free to stay what it always
            // was — a vessel. That is why `NounKind::Vessel` still has a noun.
            Holding::new(NounKind::Vessel, &["retort"]),
            Holding::new(NounKind::File, &["laboratory.log"]),
        ],
        places: INSTRUMENTS,
        role: None,
        operation: None,
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
        places: &[],
        role: None,
        operation: None,
    },
];

/// §10.1's five instruments, plus the dispensary that feeds them.
///
/// Each is a **place**, so `survey alembic` inspects one from across the
/// laboratory while §19's *"you can only name what is where you are"* still
/// governs what is *inside* it. That rule is why the pipeline names the
/// instrument and never its contents.
///
/// Order is the parse (see [`BRANCHES`]): pipeline order first, the shared heat
/// source, then the store.
const INSTRUMENTS: &[Branch] = &[
    Branch {
        name: "mortar_and_pestle",
        holds: &[],
        places: &[],
        role: None,
        operation: Some(Verb::Grind),
    },
    Branch {
        name: "balneum_mariae",
        holds: &[],
        places: &[],
        role: None,
        operation: Some(Verb::Digest),
    },
    Branch {
        name: "flask_and_rod",
        holds: &[],
        places: &[],
        role: None,
        operation: Some(Verb::Mix),
    },
    Branch {
        name: "alembic",
        holds: &[],
        places: &[],
        role: None,
        operation: Some(Verb::Distil),
    },
    Branch {
        // `kindle` charges and lights in one, exactly as the other four charge
        // and start — the difference is only that lighting is not a *run*, which
        // `start` settles at its `HeatSource` branch rather than here.
        name: "athanor",
        holds: &[],
        places: &[],
        role: Some(Role::Heat),
        operation: Some(Verb::Kindle),
    },
    Branch {
        name: "dispensary",
        // What the player starts with. Charcoal is fuel rather than an
        // ingredient, but it is carried and moved like everything else, which is
        // the whole reason `Reagent` is one kind and not four.
        holds: &[Holding::new(
            NounKind::Reagent,
            &["sage", "rock-salt", "charcoal"],
        )],
        places: &[],
        role: Some(Role::Store),
        operation: None,
    },
];

struct Branch {
    name: &'static str,
    holds: &'static [Holding],
    /// Places *inside* this one. The laboratory's instruments (§10.1).
    places: &'static [Branch],
    /// What makes this fixture behave unlike the rest, if anything.
    role: Option<Role>,
    /// The verb that charges it and starts it — see [`Operation`].
    operation: Option<Verb>,
}

/// A fixture that is not an ordinary instrument.
///
/// **A component, not a name comparison.** Six sites branched on `name ==
/// ATHANOR` or `name != DISPENSARY` — `wield`, `stop`, the panel's state reader,
/// the panel's own filter, `heat::find` and `reachable` — with nothing binding
/// them together. §10 puts five more domains in Phase 3a, and the day a second
/// room gets a forge, `wield forge` would have started a `Working` run with no
/// recipe instead of lighting it, `stop` would have refused to bank its fuel, and
/// the panel would have drawn a filling meter where a draining one belongs. Six
/// edits, none of which the compiler would have asked for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    /// Burns fuel for the instruments that need heat (§10.1's athanor).
    Heat,
    /// A shelf of stock: the fallback a `move` falls back to.
    Store,
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
        // Branches are places the player lives in; losing one would end the
        // slice, so §7's guard covers them as it covers the root.
        raise_branch(world, root, branch, true);
    }

    world.insert_resource(Cwd(root));
}

/// Spawn one branch, its holdings, and any places inside it.
///
/// `protect` marks the targets §7 guards *in character* — the root and the live
/// domains. Instruments deliberately do **not** get it: `purge alembic` should
/// empty the alembic, not refuse. What stops it deleting one is that `purge`
/// clears any place rather than despawning it, so an instrument is safe by being
/// a place. See [`super::work::purge`] for the two tiers.
fn raise_branch(world: &mut World, parent: Entity, branch: &Branch, protect: bool) {
    let at = spawn(world, Some(parent), branch.name, NounKind::Place);
    if protect {
        world.entity_mut(at).insert(Protected);
    } else {
        // Not a domain: furniture in the room above it, so what it holds is
        // nameable from there. See [`Fixture`].
        world.entity_mut(at).insert(Fixture);
    }

    if let Some(operation) = branch.operation {
        world.entity_mut(at).insert(super::Operation(operation));
    }

    match branch.role {
        Some(Role::Heat) => {
            world.entity_mut(at).insert(super::HeatSource);
        }
        Some(Role::Store) => {
            // **Protected.** The dispensary was an ordinary fixture, so
            // `purge dispensary` scoured it — despawning `sage`, `rock-salt` and
            // `charcoal` at once. No recipe produces sage or charcoal, so the
            // athanor could never be lit again and no potion could ever be
            // brewed: an unwinnable tower from one command, against §7's
            // *"destruction is a tool, not a trap"* and §11.5's *"not automating
            // is never ruinous, only slower"*. An instrument is safe by being
            // emptied rather than deleted; a shelf of stock is safe by refusing.
            world.entity_mut(at).insert((super::Store, Protected));
        }
        None => {}
    }

    for holding in branch.holds {
        for name in holding.names {
            let node = spawn(world, Some(at), name, holding.kind);
            if is_log(name) {
                world.entity_mut(node).insert(super::sabotage::Log);
            }
        }
    }

    for inner in branch.places {
        raise_branch(world, at, inner, false);
    }
}

/// Whether a name is a log surface, and so a target §8.1 can poison.
///
/// Case-insensitive: `ends_with(".log")` is a byte comparison, so a `FEED.LOG`
/// added to [`BRANCHES`] would spawn without the [`Log`](super::sabotage::Log)
/// marker and be quietly immune to sabotage — a content typo with no symptom
/// until a siege fails to land a tell.
fn is_log(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("log"))
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
        for domain in ["tower", "laboratory", "archive"] {
            sim.submit(&format!("attend {domain}"));
            sim.step();
            seen.extend(sim.scene().nouns().iter().map(|noun| noun.kind));
        }

        // Derived from the signatures rather than listed, so retiring a verb or
        // adding one cannot leave this asserting about a slot nothing fills.
        // Free-text kinds resolve against the player's typing, not the world.
        for wanted in crate::parser::Verb::ALL
            .into_iter()
            .flat_map(crate::parser::Verb::signature)
            .map(|slot| slot.kind)
            .filter(|kind| {
                !matches!(
                    kind,
                    NounKind::Pattern | NounKind::Count | NounKind::Any | NounKind::Topic
                )
            })
        {
            // Two kinds have nothing in the tower yet, both on purpose:
            //
            // - `Essence` — §10.1 makes a potion something the alembic *yields*,
            //   so nothing is an essence until the player brews one.
            // - `Script` — there are no spells until Phase 1's script engine.
            //   `scribe`/`bind`/`invoke` are dark for exactly this reason, which
            //   `is_live` already records.
            if matches!(wanted, NounKind::Essence | NounKind::Script) {
                continue;
            }
            assert!(seen.contains(&wanted), "nothing anywhere is a {wanted:?}");
        }
    }

    #[test]
    fn every_place_leaf_is_unique() {
        // **What makes leaf echoes safe.** `Intent::echo` draws a place as its
        // last segment, because the full path clipped the destination off a
        // three-argument `move` at the 80×22 floor. That is only unambiguous
        // while no two places share a leaf.
        //
        // `score_against` matches a phrase against the full name or the leaf and
        // nothing between, so a collision cannot be echoed around — there is no
        // `laboratory/alembic` form the parser would accept. The answer is to
        // forbid the collision, in the shape of the naming pass's own tests, so
        // the day §10's seventh domain wants a second `dispensary` this fails
        // rather than the echo quietly starting to lie.
        let sim = Sim::new(1);
        let world = sim.world();
        let root = world.resource::<Cwd>().0;

        let mut leaves: Vec<String> = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            for child in children_of(world, node) {
                if world.get::<Nameable>(child).map(|kind| kind.0) == Some(NounKind::Place) {
                    if let Some(name) = world.get::<Name>(child) {
                        leaves.push(name.0.clone());
                    }
                    stack.push(child);
                }
            }
        }

        let mut seen = leaves.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            leaves.len(),
            "two places share a leaf, so an echo cannot say which: {leaves:?}"
        );
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
        assert_eq!(branches, ["laboratory", "archive"]);
    }

    #[test]
    fn a_path_is_built_from_the_place_it_names() {
        let sim = Sim::new(1);
        let world = sim.world();
        let root = world.resource::<Cwd>().0;
        let laboratory = children_of(world, root)[0];

        assert_eq!(path_of(world, laboratory), "/tower/laboratory");
        let first = children_of(world, laboratory)[0];
        assert_eq!(path_of(world, first), "/tower/laboratory/retort");
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
