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

/// The tower. Protected: §7 guards catastrophic targets in character.
const ROOT: &str = "tower";

/// The player's spellbook — §8's `/grimoire`, a **sibling** of `/tower`.
///
/// # A root domain, and deliberately not a §9 activity domain
///
/// §15 fixes the slice at two starting domains and §9 gives one pane per domain,
/// *"two exist at the start"*; the roadmap's next domain is scrying, and its See
/// it line is *"discover it in play rather than starting with it."* A third
/// pane-bearing domain here would collide with all three.
///
/// It does not, because **§9's panes are per *activity* — the seven you
/// multiplex between — and writing is not one of them.** You do not run the
/// grimoire concurrently with brewing; you go and write, and what you wrote runs
/// somewhere else. It is addressable, it holds files, and the editor takes the
/// main pane while it is open. It adds nothing to the pane count.
///
/// # Why a sibling rather than `/tower/grimoire`
///
/// A spell is not kept in a room. §8 has the player editing `.spell` files in
/// their own editor, and the fiction that survives that is a book you carry, not
/// a shelf you walk to — which is also why `tower::scene` registers spells as
/// nameable from anywhere rather than only from inside here.
const GRIMOIRE: &str = "grimoire";

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
            // (`scene::rebuild`), which is what `recall clarity` reads.
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
        // **`sigil-iv`, `sigil-ix` and `the-quiet-page` are gone**, and they were
        // the last of the `divine` that took a fragment. Once `research` opened a
        // the stacks instead of consuming a sigil (§19), nothing produced them,
        // nothing consumed them and no prose said what one *was* — which is
        // verbatim the complaint §19 records against `shard-of-dawn` and its
        // three siblings: *"four invented names standing in for a decision nobody
        // made"*. Three more of the same, left on the floor of the room that
        // stopped needing them.
        //
        // What the archive yields now is `fragment`, and it accumulates on the
        // lectern that made it rather than lying about.
        holds: &[Holding::new(NounKind::File, &["archive.log"])],
        places: ARCHIVE,
        role: None,
        operation: None,
    },
    // **The one room you can reach from any other**, and it starts empty: what
    // is in it is what the player has finished. See [`Role::Keep`] and
    // `tower::keep` for why the exemption is narrow and why this is not a second
    // dispensary.
    //
    // **Last, and the order is load-bearing.** §6 resolves a tie to whichever
    // noun was registered first, so raising the arsenal before the archive would
    // silently reorder every existing reading — `spawn order is part of the
    // world's determinism` (see [`node`](super::node)). A new domain goes on the
    // end.
    Branch {
        name: super::ARSENAL,
        holds: &[Holding::new(NounKind::File, &["arsenal.log"])],
        places: &[],
        role: Some(Role::Keep),
        operation: None,
    },
];

/// The archive's one instrument, and the four ways its reading can go.
///
/// The lectern is where a maze is opened and where fragments are assembled, and
/// giving the archive a fixture at all is what retires three defects at once:
/// `divine`'s completion had no sentence, a running `divine` could not be
/// stopped (`stop` finds its target through `Fixture`), and the domain drew no
/// panel. None of the three was worth patching separately — they were one
/// absence.
const ARCHIVE: &[Branch] = &[
    // **Assembly, and nothing else now.** The lectern used to be both halves of
    // the archive at once — the stacks open on it *and* a scroll coming
    // together in it — which §19 records as the first instrument in the game
    // that could be doing two things at once, and treated as a curiosity rather
    // than as the design problem it was. `stop lectern` had to decide which of
    // the two it meant; the panel had one row for both; and `follow`'s scope had
    // nowhere to go because a fixture carries exactly one `Operation` and this
    // one had spent it on `research`.
    //
    // It has **no operation** now: four fragments are moved in and `wield`ed,
    // which is the ordinary way an instrument runs. Its picture comes from
    // having recipes rather than from a verb — see `panel::craft_of`.
    Branch {
        name: "lectern",
        holds: &[],
        places: &[],
        role: None,
        operation: None,
    },
    // **Every domain that holds stock needs somewhere to put it**, and the
    // archive had none — so `empty lectern` answered *"there is nowhere here to
    // put what the lectern holds"*, and `tower::home` had no archive home to send
    // a `fragment` to, which put the archive's only stock out of a tester's reach
    // in the room it is used in.
    //
    // **It was added for a stronger reason that has since gone**: the lectern
    // shed `dust`, which was trapped in the instrument that made it and
    // clearable only by `purge`. The byproduct went instead — §10.1 keeps that
    // mechanic in the laboratory — and this stayed, because the rule it serves is
    // about *stock*, not about waste. A room with an instrument and nowhere to
    // set anything down is a room where `empty` is a word that can never work.
    //
    // **`cabinet`, measured rather than chosen** — 572 against `combine`, its
    // nearest word, where the resolver's floor is 600. `shelf` and `chest` both
    // land *on* the floor (600, against `help` and `check`), and `press`,
    // `stacks` and `carrel` are over it. `almery` — a monastic book cupboard — is
    // safest at 429 and was passed over for being a word nobody can type on a
    // first guess, which is the same objection §19 records against the four
    // invented shard names.
    Branch {
        name: "cabinet",
        holds: &[],
        places: &[],
        role: Some(Role::Store),
        operation: None,
    },
    // **The stacks: an endless library, and the maze lives here.**
    //
    // §10 calls the archive *decipherment*, and the stacks are what that became
    // — but walking is not assembly, and the two were sharing a fixture
    // for no better reason than that the archive had only one. Splitting them
    // gives each its own row on the panel, its own `stop`, and its own state, so
    // *"is a reading open"* and *"is a scroll coming together"* stop being one
    // question with two answers.
    //
    // **Last, and the order is load-bearing.** §6 resolves a tie to whichever
    // noun was registered first, so a fixture inserted ahead of the existing ones
    // would silently change what an existing phrase resolves to. A new place goes
    // on the end — the same rule the arsenal follows.
    Branch {
        name: "stacks",
        holds: &[],
        places: &[],
        role: None,
        operation: Some(Verb::Research),
    },
    Branch {
        name: "north",
        holds: &[],
        places: &[],
        role: Some(Role::Reading),
        operation: None,
    },
    Branch {
        name: "east",
        holds: &[],
        places: &[],
        role: Some(Role::Reading),
        operation: None,
    },
    Branch {
        name: "south",
        holds: &[],
        places: &[],
        role: Some(Role::Reading),
        operation: None,
    },
    Branch {
        name: "west",
        holds: &[],
        places: &[],
        role: Some(Role::Reading),
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
        // What the player starts with, and never runs out of. Charcoal is fuel
        // rather than an ingredient, but it is carried and moved like everything
        // else, which is the whole reason `Reagent` is one kind and not four —
        // and it is endless for the same reason the other two are: a cold
        // athanor with nothing to burn is a laboratory with nothing to do.
        holds: &[Holding::endless(
            NounKind::Reagent,
            &["sage", "rock-salt", "charcoal"],
        )],
        places: &[],
        role: Some(Role::Store),
        operation: None,
    },
];

/// Every fixture the tower raises that has a verb of its own.
///
/// **What `progression.toml` may price.** Its `[earns]` keys used to be checked
/// against `recipes.toml` alone, which was right while every instrument that
/// *ran* also transformed something — and stopped being right twice over. The
/// athanor has a verb and no recipe, so it could never have been priced; the
/// `stacks` has a verb and no recipe and earns for every walk finished, so it
/// had to be.
///
/// A fixture with neither — the dispensary, the cabinet — is a shelf, and pricing
/// one would be authoring a number nothing can ever pay.
#[must_use]
pub fn operated() -> Vec<&'static str> {
    fn walk(branches: &'static [Branch], into: &mut Vec<&'static str>) {
        for branch in branches {
            if branch.operation.is_some() {
                into.push(branch.name);
            }
            walk(branch.places, into);
        }
    }
    let mut names = Vec::new();
    walk(BRANCHES, &mut names);
    names
}

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
    /// Where finished work is kept, reachable from every room (§7's one
    /// exemption). See [`super::Keep`].
    Keep,
    /// One of the four ways the archive's reading can go.
    ///
    /// A **place**, because `spell::compile` resolves the place half of a
    /// question against `NounKind::Place` and nothing else — `if north has
    /// passage` cannot be written unless `north` is one. That is also why it is
    /// a `Role` rather than a bare fixture: a place in the scene is a place you
    /// can `attend`, and walking into a compass bearing is not a thing the
    /// wizard does. The role is what `attend` refuses on, and what keeps these
    /// four off the instrument panel.
    Reading,
}

struct Holding {
    kind: NounKind,
    names: &'static [&'static str],
    /// How much of each, for the kinds that are counted.
    stock: super::Stock,
}

impl Holding {
    const fn new(kind: NounKind, names: &'static [&'static str]) -> Self {
        Self {
            kind,
            names,
            stock: super::Stock::Counted(1),
        }
    }

    /// The tower never runs out of these.
    ///
    /// §11.5's ongoing alchemy rests on there always being something to do, and
    /// a laboratory whose sage is spent after one grind has nothing. **What is
    /// made from them is not endless** — that is where the game is: the base
    /// reagents are the floor, and everything derived from them is scarce
    /// because it costs the tower's time to make.
    const fn endless(kind: NounKind, names: &'static [&'static str]) -> Self {
        Self {
            kind,
            names,
            stock: super::Stock::Endless,
        }
    }
}

/// Raise the starting tower and stand the player at its root.
///
/// Called from `Sim::new`, never by a frontend: if the Bevy build, `orbs-tui`
/// and `orbs-balance` each built their own world they could diverge, which is
/// the failure §13 exists to prevent — *"if the live game and the CLI harness
/// diverged, we would not find out until Phase 3."*
pub fn raise(world: &mut World) {
    // **The filesystem root is nameless**, and that is what makes the whole
    // restructure free: `path_of` collects a segment only where a `Name` is
    // present, so a root without one contributes nothing and the paths stay
    // `/tower/laboratory` and `/grimoire/first_light.spell`. It carries a
    // `NodeId` like every other node — a save has to be able to name it — and no
    // `Nameable`, so `find_place` cannot reach it and there is no `attend /`.
    let filesystem = {
        let id = world.resource_mut::<NodeIds>().issue();
        world.spawn((id, Protected)).id()
    };

    let tower = spawn(world, Some(filesystem), ROOT, NounKind::Place);
    world.entity_mut(tower).insert(Protected);
    for branch in BRANCHES {
        // Branches are places the player lives in; losing one would end the
        // slice, so §7's guard covers them as it covers the tower.
        raise_branch(world, tower, branch, true);
    }

    raise_grimoire(world, filesystem);

    // The player starts in the tower, not at the root. `Cwd` is where you
    // stand; `tower::root` is where the tree begins, and the two stopped being
    // the same node here.
    world.insert_resource(Cwd(tower));
}

/// Hang `/grimoire` off the filesystem root and fill it with the shipped spells.
///
/// Spawned **after** the tower, so §6's register-order tie-break is unchanged
/// for every noun that already existed. Adding the spellbook must not silently
/// re-point a phrase that resolved yesterday.
fn raise_grimoire(world: &mut World, filesystem: Entity) {
    let grimoire = spawn(world, Some(filesystem), GRIMOIRE, NounKind::Place);
    // Losing your spellbook is the one loss the game cannot let a command cause:
    // §7 guards catastrophic targets in character, and every bound script in the
    // tower points into here.
    world.entity_mut(grimoire).insert(Protected);

    let spells = world.resource::<crate::content::Spells>().clone();
    for (name, spell) in spells.iter() {
        let node = spawn(
            world,
            Some(grimoire),
            &crate::content::with_extension(name),
            NounKind::Script,
        );
        world.entity_mut(node).insert((
            super::Held(spell.lines.clone()),
            super::Domain(spell.domain.clone()),
        ));
    }
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
        Some(Role::Keep) => {
            // **`Protected` already**, from being a top-level branch, and that
            // is the answer you want: the arsenal holds everything the player
            // has finished, so `purge arsenal` refusing in character is exactly
            // §7's guard doing its job on the highest-value room in the tower.
            world.entity_mut(at).insert(super::Keep);
        }
        Some(Role::Reading) => {
            // **`Protected` too.** `purge north` would otherwise scour a
            // direction — despawning the reading the maze had just written and
            // leaving a solver asking a question about a place that had gone
            // quiet, which reads exactly like a wall. The same argument the
            // dispensary's makes: a thing the loop depends on is safe by
            // refusing, not by being emptied.
            world.entity_mut(at).insert((super::Reading, Protected));
        }
        None => {}
    }

    for holding in branch.holds {
        for name in holding.names {
            let node = spawn(world, Some(at), name, holding.kind);
            if is_log(name) {
                world.entity_mut(node).insert(super::sabotage::Log);
            }
            // Stock is the only thing that is *counted*. A place, a file, a
            // vessel and a spell are each one thing that is either there or not,
            // and giving them a count would put a `x1` beside every row on a
            // `survey` that means nothing.
            if matches!(holding.kind, NounKind::Reagent | NounKind::Essence) {
                world.entity_mut(node).insert(holding.stock);
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

/// Put one reading inside one of the archive's four ways.
///
/// The maze's whole output channel: what a spell's `if north has passage` asks
/// about. A node rather than a component, because `watch::holds` answers `has`
/// by looking for a **named child**, which is the one read the language has.
pub fn raise_reading(world: &mut World, at: Entity, word: &str) -> Entity {
    spawn(world, Some(at), word, NounKind::Sense)
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
        //
        // **Asked through `accepts` rather than `contains`.** A slot kind is not
        // always a noun's own kind: nothing *is* a `Readable` or an `Any`, and
        // asking whether the scene contains one is a question with no true
        // answer. The question worth asking is whether anything in the world
        // could **fill** the slot, which is the same question the matcher, the
        // numbered prompt and Tab completion all ask.
        //
        // That also lets `Any` back in. It was excluded here for the same reason
        // `Readable` would have had to be, and excluding a kind because the test
        // asks the wrong question is how a stale exemption outlives its cause.
        //
        // Free text stays out: `Pattern` is whatever the player is searching
        // for, `Count` is a number, and `Name` is a spell being **coined** — so
        // nothing in the world enumerates any of the three, and a `Name` slot
        // with something to resolve against would mean `scribe` could only make
        // spells that already exist.
        for wanted in crate::parser::Verb::ALL
            .into_iter()
            .flat_map(crate::parser::Verb::signature)
            .map(|slot| slot.kind)
            .filter(|kind| !matches!(kind, NounKind::Pattern | NounKind::Count | NounKind::Name))
        {
            // Two kinds have nothing in the tower yet, both on purpose:
            //
            // - `Essence` — §10.1 makes a potion something the alembic *yields*,
            //   so nothing is an essence until the player brews one.
            // - `Script` — there are no spells until Phase 1's script engine.
            //   `scribe`/`bind`/`invoke` are dark for exactly this reason, which
            //   `is_live` already records.
            //
            // `Readable` is deliberately **not** here. It accepts `File` as well
            // as `Script`, and the logs exist — so the slot is satisfiable today
            // and stays satisfiable when spells arrive. A slot kind that is
            // unfillable for one of its members is not unfillable.
            if matches!(wanted, NounKind::Essence | NounKind::Script) {
                continue;
            }
            assert!(
                seen.iter().any(|kind| wanted.accepts(*kind)),
                "nothing anywhere can fill a {wanted:?} slot",
            );
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
        let root = crate::tower::root(world);

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

    /// Every name directly under `node`, in spawn order.
    fn names_under(world: &World, node: Entity) -> Vec<String> {
        children_of(world, node)
            .into_iter()
            .filter_map(|child| world.get::<Name>(child).map(|name| name.0.clone()))
            .collect()
    }

    #[test]
    fn the_filesystem_root_is_nameless_and_holds_the_tower_and_the_grimoire() {
        // **The root is not the tower any more**, and this is where that is
        // written down. `/grimoire` is a sibling because a spell is a book you
        // carry rather than a room you walk to (§8), and the root above them
        // carries no `Name` — which is what keeps `path_of` yielding
        // `/tower/laboratory` with no special case anywhere.
        let sim = Sim::new(1);
        let world = sim.world();
        let root = crate::tower::root(world);

        assert_eq!(path_of(world, root), "/", "the root should be nameless");
        assert!(
            world.get::<Name>(root).is_none(),
            "a named root would put its own segment in every path",
        );
        assert!(
            world.get::<Nameable>(root).is_none(),
            "a nameable root would give `attend` somewhere meaningless to go",
        );
        assert_eq!(names_under(world, root), ["tower", "grimoire"]);
    }

    #[test]
    fn the_tower_is_a_tree_with_the_two_slice_domains() {
        // §15 fixes the slice at two starting **activity** domains. `/grimoire`
        // is not one — it is where you write, not something you run — so it is
        // deliberately outside this assertion rather than added to it. See
        // `GRIMOIRE`.
        //
        // **`/tower/arsenal` is the same kind of exception**, and it is inside
        // the assertion rather than outside it because it *is* under `/tower`:
        // it is where finished work is kept, not something you run, so it raises
        // no instrument, offers no verb and draws an empty panel. What it is
        // counted for here is the **order** — §6 resolves a tie to whichever
        // noun was registered first, so a domain added anywhere but the end
        // would silently change what an existing phrase resolves to.
        let sim = Sim::new(1);
        let world = sim.world();
        let tower = children_of(world, crate::tower::root(world))[0];

        assert_eq!(path_of(world, tower), "/tower");
        assert_eq!(
            names_under(world, tower),
            ["laboratory", "archive", super::super::ARSENAL],
        );
    }

    #[test]
    fn a_path_is_built_from_the_place_it_names() {
        let sim = Sim::new(1);
        let world = sim.world();
        let tower = children_of(world, crate::tower::root(world))[0];
        let laboratory = children_of(world, tower)[0];

        assert_eq!(path_of(world, laboratory), "/tower/laboratory");
        let first = children_of(world, laboratory)[0];
        assert_eq!(path_of(world, first), "/tower/laboratory/retort");
    }

    #[test]
    fn the_grimoire_holds_the_shipped_spells_and_they_carry_their_lines() {
        let sim = Sim::new(1);
        let world = sim.world();
        let grimoire = children_of(world, crate::tower::root(world))[1];

        assert_eq!(path_of(world, grimoire), "/grimoire");
        assert!(
            world.get::<Protected>(grimoire).is_some(),
            "losing your spellbook is the one loss a command must not cause",
        );

        let spells = children_of(world, grimoire);
        assert!(!spells.is_empty(), "the grimoire is empty");
        for spell in spells {
            let name = world.get::<Name>(spell).expect("a name").0.clone();
            assert!(name.ends_with(crate::content::EXTENSION), "{name}");
            assert_eq!(
                world.get::<Nameable>(spell).map(|kind| kind.0),
                Some(NounKind::Script),
                "{name} is not nameable as a script",
            );
            let held = world.get::<super::super::Held>(spell).expect("held lines");
            assert!(!held.0.is_empty(), "{name} has no lines");
        }
    }

    #[test]
    fn a_spell_is_nameable_from_anywhere_in_the_tower() {
        // **The rule that keeps `invoke` usable.** Everything that is not a
        // place is registered from `cwd`, and `/grimoire` is a protected domain
        // rather than a `Fixture`, so without the exemption in `scene::rebuild`
        // a spell could only be named while standing in the grimoire — which is
        // the one room with no laboratory to run it in.
        let mut sim = Sim::new(1);
        for domain in ["tower", "laboratory", "archive", "grimoire"] {
            sim.submit(&format!("attend {domain}"));
            sim.step();
            assert!(
                sim.scene()
                    .nouns()
                    .iter()
                    .any(|noun| noun.kind == NounKind::Script),
                "no spell is nameable from {domain}",
            );
        }
    }

    #[test]
    fn a_spell_nameable_from_anywhere_is_readable_from_anywhere() {
        // **The other half of the same rule, and it was missing.** The scene
        // registered spells globally while the lookup still searched only `cwd`
        // and then places, so from the laboratory `peruse first_light.spell`
        // resolved at `Clear` confidence, found nothing, fell through to the
        // record-stream reader and reported a **zero-line read of a
        // three-line file**.
        //
        // Nameable and findable are one rule. Asserting only the first is what
        // let them drift apart.
        for domain in ["tower", "laboratory", "archive", "grimoire"] {
            let mut sim = Sim::new(1);
            sim.submit(&format!("attend {domain}"));
            sim.step();
            sim.submit("peruse first_light.spell");
            sim.step();

            let lines = sim
                .scrollback()
                .records()
                .iter()
                .filter(|record| record.kind() == orbs_render::RecordKind::LogLine)
                .count();
            assert!(
                lines >= 2,
                "reading the spell from {domain} yielded {lines} lines, not its contents",
            );
        }
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
            let mut stack = vec![crate::tower::root(world)];
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
        let root = crate::tower::root(world);

        assert!(world.get::<Protected>(root).is_some(), "the tower root");
        for branch in children_of(world, root) {
            assert!(world.get::<Protected>(branch).is_some(), "a live domain");
        }
    }
}
