//! The tower the player starts in.
//!
//! Names are not prose: a table may emit facts, never sentences (rule 6, §19),
//! so this file names things and nothing else.

use bevy_ecs::prelude::*;

use super::charm::{self, Kind};
use super::circle::{GLYPHS, Glyph, HUMOURS, Humour};
use super::maze::Way;
use super::node::{Cwd, Fixture, Name, Nameable, NodeIds, Protected};
use super::siege::{self, Area};
use super::stock::Stock;
use super::{lattice, pylon, ward};
use crate::parser::{NounKind, Verb};

/// The tower. Protected: §7 guards catastrophic targets in character.
const ROOT: &str = "tower";

/// The player's spellbook — §8's `/grimoire`, a sibling of `/tower`.
///
/// Not a §9 activity domain: you go there to write, and what you wrote runs
/// elsewhere. A sibling, because a spell is a book you carry rather than
/// something kept in a room — which is also why spells are nameable from
/// anywhere. Public because sealing needs it — see `opened::is_room`.
pub const GRIMOIRE: &str = "grimoire";

/// Branches of the starting tower, and what each holds.
///
/// Order is load-bearing: §6 resolves a noun tie to whichever registered first,
/// so spawn order is part of the world's determinism (see
/// [`node`](super::node)). A new domain goes on the end, or every existing
/// reading after it re-resolves.
const BRANCHES: &[Branch] = &[
    domain(
        "laboratory",
        &[
            // No essences: §10.1 makes potions what the alembic yields. The
            // retort stays a vessel, so `NounKind::Vessel` has a noun.
            Holding::new(NounKind::Vessel, &["retort"]),
            Holding::new(NounKind::File, &["laboratory.log"]),
        ],
        INSTRUMENTS,
    ),
    // What the archive yields is `fragment`, kept on the lectern that made it.
    domain(
        "archive",
        &[Holding::new(NounKind::File, &["archive.log"])],
        ARCHIVE,
    ),
    // Scrying (§10): a far wizard's orb sealed with a ward — see `tower::ward`.
    // No materials: a broken seal yields experience, a log and sometimes a recipe.
    domain("lens", &[Holding::new(NounKind::File, &["lens.log"])], LENS),
    // Defense (§10): wards drawn from the wellspring into the barrier — see
    // `tower::pylon` and `tower::erosion`. `sanctum`, not §10's `battlements/`
    // (§19). No endless base reagent, ever: `sabotage::substitution` picks with
    // `% piles.len()`, and a third pile moves every rate `orbs-balance` pins.
    domain(
        "sanctum",
        &[Holding::new(NounKind::File, &["sanctum.log"])],
        SANCTUM,
    ),
    // Reachable from every room, starting empty: what the player has finished.
    // See [`Role::Keep`] and `tower::keep`.
    Branch {
        name: super::ARSENAL,
        holds: &[Holding::new(NounKind::File, &["arsenal.log"])],
        places: &[],
        role: Some(Role::Keep),
        operation: None,
        group: None,
    },
    // After the arsenal: "the end" means the end of this list. A hold refills
    // the standing garrison (§19, *Supply*), so no materials of its own.
    domain(
        "menagerie",
        &[Holding::new(NounKind::File, &["menagerie.log"])],
        MENAGERIE,
    ),
    // Where a siege is fought (§5.1). What it spends comes from the arsenal.
    domain(
        siege::BAILEY,
        &[Holding::new(NounKind::File, &["bailey.log"])],
        BAILEY,
    ),
    // §10's Enchanting, at §7's `/tower/forge`. A charm costs quintessence, a
    // resource, so a log and nothing else.
    domain(
        charm::FORGE,
        &[Holding::new(NounKind::File, &["forge.log"])],
        FORGE,
    ),
];

/// The forge: the lattice a charm is bound on, and its three columns.
///
/// `imbue` is the lattice's operation; `snap` and `settle` anchor to it, since a
/// column is read and snapped, not stood in. `group` is on the columns and never
/// the lattice, so `for each column` walks them.
const FORGE: &[Branch] = &[
    fixture(charm::LATTICE, Verb::Imbue),
    // Three columns, not nine cells: a Lights Out solution is settled by its top
    // row, and nine cell names do not survive the naming sweep anyway.
    reading(lattice::COLUMNS[0], Some("column")),
    reading(lattice::COLUMNS[1], Some("column")),
    reading(lattice::COLUMNS[2], Some("column")),
    // The five charms as places, so `imbue mortar_and_pestle hurried` resolves,
    // and `for each charm` finds the one that is ebbing.
    reading(Kind::Hurried.word(), Some("charm")),
    reading(Kind::Fruitful.word(), Some("charm")),
    reading(Kind::Bountiful.word(), Some("charm")),
    reading(Kind::Whetted.word(), Some("charm")),
    reading(Kind::Shielded.word(), Some("charm")),
];

/// The bailey: the rampart you watch from, and the two sides.
///
/// `defend` is the rampart's operation; `deploy`, `quaff` and `hold` anchor to
/// it, since a band is read, not stood in. A `Role::Reading` place is one a
/// question can name — `if the enemy has outnumbered` — without being one you
/// can `attend`.
const BAILEY: &[Branch] = &[
    fixture(siege::RAMPART, Verb::Defend),
    // The four areas: `pledge d20 buckler`, and `for each area` to find the
    // empty one.
    reading(Area::Line.word(), Some("area")),
    reading(Area::Buckler.word(), Some("area")),
    reading(Area::Succour.word(), Some("area")),
    reading(Area::Sortie.word(), Some("area")),
    // The coffer is `Role::Reading` too, though it contains readings: the
    // publisher finds a node by that role, and without it `survey coffer` said
    // it held nothing. The dice exist whether or not they are free; what the
    // coffer publishes is what changes.
    reading(siege::COFFER, None),
    reading(siege::POOL[0].word(), Some("die")),
    reading(siege::POOL[1].word(), Some("die")),
    reading(siege::POOL[2].word(), Some("die")),
    reading(siege::GARRISON, Some("band")),
    reading(siege::ENEMY, Some("band")),
];

/// The menagerie: the circle, its three glyphs, and the six humours a glyph can
/// be limned with.
///
/// `summon` is the circle's; `limn` anchors to it. Named by the model's own
/// `const fn`s, so the word registered and the word `limn` looks up cannot drift.
const MENAGERIE: &[Branch] = &[
    fixture("circle", Verb::Summon),
    reading(Glyph::Keystone.word(), Some(GLYPHS)),
    reading(Glyph::Sunwise.word(), Some(GLYPHS)),
    reading(Glyph::Widdershins.word(), Some(GLYPHS)),
    reading(Humour::Yoke.word(), Some(HUMOURS)),
    reading(Humour::Spurn.word(), Some(HUMOURS)),
    reading(Humour::Heed.word(), Some(HUMOURS)),
    reading(Humour::Eschew.word(), Some(HUMOURS)),
    reading(Humour::Oppose.word(), Some(HUMOURS)),
    reading(Humour::Mirror.word(), Some(HUMOURS)),
];

/// The menagerie's roster: the standing garrison, read rather than stood in
/// (§19, *Supply*). Raised last of everything; see [`raise`].
const ROSTER: Branch = reading(siege::ROSTER, None);

/// A room: its log and stock, and the places in it.
const fn domain(
    name: &'static str,
    holds: &'static [Holding],
    places: &'static [Branch],
) -> Branch {
    Branch {
        name,
        holds,
        places,
        role: None,
        operation: None,
        group: None,
    }
}

/// A place in a room with no verb of its own: an instrument that is `wield`ed,
/// or a store.
const fn place(name: &'static str, role: Option<Role>) -> Branch {
    Branch {
        name,
        holds: &[],
        places: &[],
        role,
        operation: None,
        group: None,
    }
}

/// The one place in a room that `operation` charges and starts.
const fn fixture(name: &'static str, operation: Verb) -> Branch {
    Branch {
        name,
        holds: &[],
        places: &[],
        role: None,
        operation: Some(operation),
        group: None,
    }
}

/// A place a spell reads and an argument names, but nobody stands in; one of
/// the set `for each <group>` walks, if it is in one.
const fn reading(name: &'static str, group: Option<&'static str>) -> Branch {
    Branch {
        name,
        holds: &[],
        places: &[],
        role: Some(Role::Reading),
        operation: None,
        group,
    }
}

/// A reading a verb also acts on, as a socket is `dial`led and a station
/// `haul`ed to.
const fn station(name: &'static str, operation: Verb, group: &'static str) -> Branch {
    Branch {
        name,
        holds: &[],
        places: &[],
        role: Some(Role::Reading),
        operation: Some(operation),
        group: Some(group),
    }
}

/// The archive: the lectern a scroll is assembled on, a store, the stacks where
/// the maze lives, and the four ways its reading can go.
const ARCHIVE: &[Branch] = &[
    // Assembly only: four fragments are moved in and `wield`ed. Walking the
    // stacks was split off so each has its own row, `stop` and state (§19).
    place("lectern", None),
    // A room with stock needs somewhere to set it down, or `empty` never works.
    // `cabinet` scores 572 against `combine`, under the 600 floor; `shelf`,
    // `chest`, `press` and `carrel` do not.
    place("cabinet", Some(Role::Store)),
    fixture("stacks", Verb::Research),
    // `group`, because `Role::Reading` also covers the lens's sockets and sigils
    // and a `for each` over the role would hand a lens spell ten things.
    reading(Way::North.word(), Some("way")),
    reading(Way::East.word(), Some("way")),
    reading(Way::South.word(), Some("way")),
    reading(Way::West.word(), Some("way")),
];

/// The lens: one instrument, four sockets and six sigils.
///
/// Both verbs are scoped through `Operation`s: the prism's `probe` and each
/// socket's `dial`. There is no second instrument for `scry`, which failed the
/// naming sweep (`scr` reaches `scribe`).
const LENS: &[Branch] = &[
    // `aligned`, `astray` and `spent` are published as its children, as the maze
    // publishes on its ways.
    fixture("prism", Verb::Probe),
    station(ward::SOCKETS[0], Verb::Dial, "socket"),
    station(ward::SOCKETS[1], Verb::Dial, "socket"),
    station(ward::SOCKETS[2], Verb::Dial, "socket"),
    station(ward::SOCKETS[3], Verb::Dial, "socket"),
    // Each sigil publishes its own `marks`, and nothing about the code.
    reading(ward::SIGILS[0], Some("sigil")),
    reading(ward::SIGILS[1], Some("sigil")),
    reading(ward::SIGILS[2], Some("sigil")),
    reading(ward::SIGILS[3], Some("sigil")),
    reading(ward::SIGILS[4], Some("sigil")),
    reading(ward::SIGILS[5], Some("sigil")),
];

/// The sanctum: one engine and the three stations a ward passes between.
///
/// `muster` is the pylon's and each station carries `haul`, the lens's
/// arrangement. In the course's order, source first, as the board reads left to
/// right; `pylon::Course` addresses them by index.
const SANCTUM: &[Branch] = &[
    fixture("pylon", Verb::Muster),
    station(pylon::STATIONS[0], Verb::Haul, "station"),
    station(pylon::STATIONS[1], Verb::Haul, "station"),
    station(pylon::STATIONS[2], Verb::Haul, "station"),
];

/// §10.1's five instruments, plus the dispensary that feeds them.
///
/// Each is a place, so `survey alembic` reads one from across the room, while
/// what is *inside* is nameable only where you are (§19). Pipeline order, then
/// the heat source, then the store.
const INSTRUMENTS: &[Branch] = &[
    fixture("mortar_and_pestle", Verb::Grind),
    fixture("balneum_mariae", Verb::Digest),
    fixture("flask_and_rod", Verb::Mix),
    fixture("alembic", Verb::Distil),
    Branch {
        // `kindle` charges and lights in one; lighting is not a run, which
        // `start` settles at its `HeatSource` branch.
        name: "athanor",
        holds: &[],
        places: &[],
        role: Some(Role::Heat),
        operation: Some(Verb::Kindle),
        group: None,
    },
    Branch {
        name: "dispensary",
        // Endless: a cold athanor with nothing to burn is a laboratory with
        // nothing to do. Charcoal is fuel but moves like everything else, which
        // is why `Reagent` is one kind.
        holds: &[Holding::endless(
            NounKind::Reagent,
            &["sage", "rock-salt", "charcoal"],
        )],
        places: &[],
        role: Some(Role::Store),
        operation: None,
        group: None,
    },
];

/// Every fixture the tower raises that has a verb of its own — what
/// `progression.toml` may price.
///
/// Not "has a recipe": the athanor and the stacks earn without one, and a shelf
/// with neither would be a number nothing can pay.
#[must_use]
pub fn operated() -> Vec<&'static str> {
    branches()
        .into_iter()
        .filter(|branch| branch.operation.is_some())
        .map(|branch| branch.name)
        .collect()
}

/// Every verb a fixture in the tower declares as its own.
///
/// The content half of [`Verb::anchor`], so a scoped verb is scoped by where its
/// fixture stands. `every_self_anchored_verb_is_declared_by_a_fixture` compares
/// the two.
#[must_use]
pub fn declared() -> Vec<Verb> {
    let mut verbs = Vec::new();
    for verb in branches().into_iter().filter_map(|branch| branch.operation) {
        if !verbs.contains(&verb) {
            verbs.push(verb);
        }
    }
    verbs
}

/// The fixture that declares `verb`, so a refusal can name what is missing
/// (§6). By anchor, so `follow` and `wander` both name the `stacks`.
#[must_use]
pub fn fixture_of(verb: Verb) -> Option<&'static str> {
    let wanted = verb.anchor()?;
    branches()
        .into_iter()
        .find(|branch| branch.operation == Some(wanted))
        .map(|branch| branch.name)
}

/// The verb the place called `name` declares, if the tower has one that does.
#[must_use]
pub fn operation_of(name: &str) -> Option<Verb> {
    branches()
        .into_iter()
        .find(|branch| branch.name == name)
        .and_then(|branch| branch.operation)
}

/// Every branch in the tower, depth first in spawn order.
fn branches() -> Vec<&'static Branch> {
    fn walk(branches: &'static [Branch], into: &mut Vec<&'static Branch>) {
        for branch in branches {
            into.push(branch);
            walk(branch.places, into);
        }
    }
    let mut all = Vec::new();
    walk(BRANCHES, &mut all);
    all
}

struct Branch {
    name: &'static str,
    holds: &'static [Holding],
    /// Places *inside* this one. The laboratory's instruments (§10.1).
    places: &'static [Branch],
    /// What makes this fixture behave unlike the rest, if anything.
    role: Option<Role>,
    /// The verb that charges it and starts it — see [`super::Operation`].
    operation: Option<Verb>,
    /// The set a `for each` walks, if this fixture is one of a set.
    ///
    /// Its own field because `Role::Reading` spans several sets in one room.
    /// Singular, because it names the cursor too: `for each way` binds `way`.
    group: Option<&'static str>,
}

/// A fixture that is not an ordinary instrument.
///
/// A component rather than six sites comparing names, which the compiler could
/// not keep in step when a second room got a heat source.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    /// Burns fuel for the instruments that need heat (§10.1's athanor).
    Heat,
    /// A shelf of stock: the fallback a `move` falls back to.
    Store,
    /// Where finished work is kept, reachable from every room (§7's one
    /// exemption). See [`super::Keep`].
    Keep,
    /// A place a question names and nobody walks into: `attend` refuses on the
    /// role, and the panel leaves it off.
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

    /// The tower never runs out of these — the floor §11.5's alchemy stands on.
    /// What is made from them costs the tower's time.
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
/// From `Sim::new`, never a frontend: worlds built separately could diverge,
/// the failure §13 exists to prevent.
pub fn raise(world: &mut World) {
    // The filesystem root is nameless, so paths stay `/tower/laboratory`. It has
    // a `NodeId` for the save and no `Nameable`, so there is no `attend /`.
    let filesystem = {
        let id = world.resource_mut::<NodeIds>().issue();
        world.spawn((id, Protected)).id()
    };

    let tower = spawn(world, Some(filesystem), ROOT, NounKind::Place);
    world.entity_mut(tower).insert(Protected);
    for branch in BRANCHES {
        // §7's guard covers the live domains as it covers the tower.
        raise_branch(world, tower, branch, true);
    }

    // After every branch, so the satchels go on the end of the registration
    // order and move no existing noun.
    for branch in BRANCHES {
        // Not the arsenal: no spell is written for it, so its queue would be read
        // by nothing.
        if matches!(branch.role, Some(Role::Keep)) {
            continue;
        }
        raise_satchel(world, tower, branch.name);
    }

    raise_grimoire(world, filesystem);

    // After the grimoire, for the satchels' reason.
    if let Some(menagerie) = room(world, tower, "menagerie") {
        raise_branch(world, menagerie, &ROSTER, false);
    }

    world.insert_resource(Cwd(tower));
}

/// Hang a satchel in one domain — §8's channel between two spells.
///
/// The same name in every domain, because a spell means the one where it stands
/// and a satchel is never read from another room. `Protected`: `purge satchel`
/// would scour a queue a pipeline is reading (§7).
fn raise_satchel(world: &mut World, tower: Entity, named: &str) {
    let Some(at) = room(world, tower, named) else {
        return;
    };
    let node = spawn(world, Some(at), super::SATCHEL, NounKind::Place);
    world
        .entity_mut(node)
        .insert((Fixture, super::Satchel::default(), Protected));
}

/// The room called `named` directly under the tower.
fn room(world: &World, tower: Entity, named: &str) -> Option<Entity> {
    super::children_of(world, tower)
        .into_iter()
        .find(|node| world.get::<Name>(*node).is_some_and(|name| name.0 == named))
}

/// Hang `/grimoire` off the filesystem root and fill it with the shipped spells.
///
/// After the tower, so no existing noun's tie-break moves.
fn raise_grimoire(world: &mut World, filesystem: Entity) {
    let grimoire = spawn(world, Some(filesystem), GRIMOIRE, NounKind::Place);
    // Every bound script points in here; losing it is the one loss a command
    // must not cause.
    world.entity_mut(grimoire).insert(Protected);

    let spells = world.resource::<crate::content::Spells>().clone();
    for (name, spell) in spells.iter() {
        shelve(world, grimoire, name, spell);
    }

    // The dev ladders, from tick 0, in a debug build only — the same `cfg` as
    // their `include_str!` in `execute::debug_spell`. Each carries its own
    // `Domain`, so nobody has to walk there first.
    #[cfg(debug_assertions)]
    for (name, spell) in crate::execute::dev_spells().iter() {
        shelve(world, grimoire, name, spell);
    }
}

/// Put one spell on the grimoire's shelf as a readable, invocable file, homed
/// in its own `Domain` rather than where anybody stands.
fn shelve(
    world: &mut World,
    grimoire: Entity,
    name: &str,
    spell: &crate::content::Spell,
) -> Entity {
    let node = spawn(
        world,
        Some(grimoire),
        &crate::content::with_extension(name),
        NounKind::Script,
    );
    // `Read` beside `Held` from the start, so `spell::source` never falls back
    // and the save's completeness lint can see it. Nobody read it — see
    // `Read::by`.
    world.entity_mut(node).insert((
        super::Held(spell.lines.clone()),
        super::Read::verbatim(&spell.lines),
        super::Domain(spell.domain.clone()),
    ));
    node
}

/// Spawn one branch, its holdings, and any places inside it.
///
/// `protect` is for the root and the live domains. An instrument is not
/// protected, because `purge alembic` should empty it; see
/// [`super::work::purge`].
fn raise_branch(world: &mut World, parent: Entity, branch: &Branch, protect: bool) {
    let at = spawn(world, Some(parent), branch.name, NounKind::Place);
    if protect {
        world.entity_mut(at).insert(Protected);
    } else {
        // Furniture, so what it holds is nameable from the room. See [`Fixture`].
        world.entity_mut(at).insert(Fixture);
    }

    if let Some(operation) = branch.operation {
        world.entity_mut(at).insert(super::Operation(operation));
    }

    if let Some(group) = branch.group {
        world
            .entity_mut(at)
            .insert(super::Grouped(group.to_owned()));
    }

    match branch.role {
        Some(Role::Heat) => {
            world.entity_mut(at).insert(super::HeatSource);
        }
        Some(Role::Store) => {
            // Protected: `purge dispensary` despawned the endless sage and
            // charcoal, an unwinnable tower in one command (§7, §11.5).
            world.entity_mut(at).insert((super::Store, Protected));
        }
        Some(Role::Keep) => {
            // Already protected as a top-level branch.
            world.entity_mut(at).insert(super::Keep);
        }
        Some(Role::Reading) => {
            // Protected: `purge north` would leave a solver asking about a place
            // gone quiet, which reads exactly like a wall.
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
            // Only stock is counted; a count on a file would draw `x1` on every
            // survey.
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
/// Case-insensitive, or a `FEED.LOG` would be silently immune to sabotage.
fn is_log(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("log"))
}

/// Put one reading inside a place — what `if north has passage` asks about.
///
/// A node, because `watch::holds` answers `has` by a named child.
pub fn raise_reading(world: &mut World, at: Entity, word: &str) -> Entity {
    spawn(world, Some(at), word, NounKind::Sense)
}

/// Put one reading inside a place, carrying a number on `Stock`, so `has 2 or
/// more marks` is the arithmetic `has 4 fragment` uses.
///
/// Still a `Sense`, so `holdings` skips it and recipes never see it. Never
/// nought — see `maze::MARKS`.
pub fn raise_count(world: &mut World, at: Entity, word: &str, count: u32) -> Entity {
    let node = raise_reading(world, at, word);
    world.entity_mut(node).insert(Stock::Counted(count));
    node
}

/// Spawn one node under `parent`, in order.
fn spawn(world: &mut World, parent: Option<Entity>, name: &str, kind: NounKind) -> Entity {
    let id = world.resource_mut::<NodeIds>().issue();
    let node = world
        .spawn((id, Name(name.to_owned()), Nameable(kind)))
        .id();
    if let Some(parent) = parent {
        world.entity_mut(node).insert(ChildOf(parent));
        // A node inherits its parent's seal: readings are published after
        // `tower::seal` ran, and the parent alone is enough, inductively.
        if world.get::<super::Sealed>(parent).is_some() {
            world.entity_mut(node).insert(super::Sealed);
        }
    }
    node
}

#[cfg(test)]
mod tests {
    use super::super::node::{children_of, path_of};
    use super::*;
    use crate::Sim;

    #[test]
    fn every_self_anchored_verb_is_declared_by_a_fixture() {
        // An anchor no fixture declares scopes a verb to nowhere.
        let declared = declared();
        for verb in Verb::ALL {
            if verb.anchor() == Some(verb) {
                assert!(
                    declared.contains(&verb),
                    "`{}` claims to be its own anchor, but no fixture in the tower \
                     declares it — so it resolves in no room at all",
                    verb.canonical(),
                );
            }
        }

        // ...and a declared verb the parser does not anchor is offered everywhere.
        for verb in declared {
            assert_eq!(
                verb.anchor(),
                Some(verb),
                "a fixture declares `{}`, but `Verb::anchor` does not scope it, so \
                 it is offered everywhere",
                verb.canonical(),
            );
        }
    }

    #[test]
    fn every_noun_kind_the_slice_uses_has_something_to_resolve_against() {
        // Walked, because a domain's belongings are nameable only from inside it.
        let mut sim = Sim::new(1);
        let mut seen = Vec::new();
        for domain in ["tower", "laboratory", "archive"] {
            sim.submit(&format!("attend {domain}"));
            sim.step();
            seen.extend(sim.scene().nouns().iter().map(|noun| noun.kind));
        }

        // Derived from the signatures, through `accepts`, the question the
        // matcher asks. Free text stays out: nothing enumerates a pattern, a
        // count or a name being coined.
        for wanted in crate::parser::Verb::ALL
            .into_iter()
            .flat_map(crate::parser::Verb::signature)
            .map(|slot| slot.kind)
            .filter(|kind| !matches!(kind, NounKind::Pattern | NounKind::Count | NounKind::Name))
        {
            // Nothing is an essence until one is brewed. `Script` has its own
            // test, `a_spell_is_nameable_from_anywhere_in_the_tower`.
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
        // An echo draws a place as its leaf, unambiguous only while no two places
        // share one. The satchel is exempt: one in every domain, always meaning
        // the one where you stand.
        let sim = Sim::new(1);
        let world = sim.world();
        let root = crate::tower::root(world);

        let mut leaves: Vec<String> = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            for child in children_of(world, node) {
                if world.get::<Nameable>(child).map(|kind| kind.0) == Some(NounKind::Place) {
                    if let Some(name) = world.get::<Name>(child)
                        && world.get::<crate::tower::Satchel>(child).is_none()
                    {
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
        // The order is what is pinned: a domain goes on the end of `BRANCHES`.
        let sim = Sim::new(1);
        let world = sim.world();
        let tower = children_of(world, crate::tower::root(world))[0];

        assert_eq!(path_of(world, tower), "/tower");
        assert_eq!(
            names_under(world, tower),
            [
                "laboratory",
                "archive",
                "lens",
                "sanctum",
                super::super::ARSENAL,
                "menagerie",
                siege::BAILEY,
                charm::FORGE,
            ],
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
        // Or a spell could be named only in the grimoire, the one room with
        // nothing to run it on.
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
        // Nameable and findable are one rule; the lookup once searched only
        // `cwd` and read a spell back as zero log lines.
        for domain in ["tower", "laboratory", "archive", "grimoire"] {
            let mut sim = Sim::new(1);
            sim.submit(&format!("attend {domain}"));
            sim.step();
            sim.submit("peruse first_light.spell");
            sim.step();

            // `ScriptLine`: a spell read back as log output is the same defect.
            let lines = sim
                .scrollback()
                .records()
                .iter()
                .filter(|record| record.kind() == orbs_render::RecordKind::ScriptLine)
                .count();
            assert!(
                lines >= 2,
                "reading the spell from {domain} yielded {lines} lines, not its contents",
            );
        }
    }

    #[test]
    fn the_same_seed_raises_the_same_tower() {
        // A bound script and a save both point at node identities.
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
        // §7: destruction is a tool, not a trap.
        let sim = Sim::new(1);
        let world = sim.world();
        let root = crate::tower::root(world);

        assert!(world.get::<Protected>(root).is_some(), "the tower root");
        for branch in children_of(world, root) {
            assert!(world.get::<Protected>(branch).is_some(), "a live domain");
        }
    }
}
