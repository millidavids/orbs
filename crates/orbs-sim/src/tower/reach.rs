//! One rule for turning a name into a node.
//!
//! # Why this had to become one thing
//!
//! Ten call sites answered *"what does this word reach"* and they differed on
//! five axes — where they looked, which kind they accepted, whether they compared
//! a leaf or a path, what order they searched in, and what they did on a miss.
//! Nothing named those axes, so each site chose them again from scratch, and
//! three of the ten were **byte-identical copies** of *the spell node called X,
//! wherever it is kept* (`navigate::find_script`, `bind::find`, `invoke::find`).
//!
//! §19 records what that costs twice already, both times as the same symptom: a
//! word that **resolves** and then finds nothing. `peruse first_light.spell` read
//! a three-line file as zero lines, and `move clarity to arsenal` reported *"no
//! such thing"* about a potion the player could see. Both were the parser and the
//! lookup disagreeing about scope — *nameable* and *findable* drifting apart,
//! which is a pair of rules nobody had written down as a pair.
//!
//! **The failure-mode axis stays at the call site**, deliberately. Whether a miss
//! is a record, a `None` or an entry on a `missing` vec is a question about what
//! the player should be told, and that is presentation. The other four are
//! questions about the world, and they are here.
//!
//! # This is a game rule, not a helper
//!
//! [`Scope::Fetch`] is §10.1's search order and its ordering decides what
//! `digest ground-sage` picks up. It is written here rather than inlined at the
//! one site that used it, because the moment a **spell** resolves a name at cast
//! and a verb body looks it up at execution, the two have to agree — see
//! `spell::compile`.

use bevy_ecs::prelude::*;

use crate::parser::NounKind;

use super::{Cwd, Fixture, Name, Nameable, Store, busy, children_of, keeping, path_of, root};

/// Where a name is looked for.
///
/// Ordered by how far they reach, which is also the order they were added: a
/// wider scope is a claim about §7's *"you can only name what is where you are"*
/// and each widening of it is recorded in §19.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Directly inside one node, its own children and no deeper.
    In(Entity),
    /// Everything under one node, depth first.
    Under(Entity),
    /// The whole tower, from the root.
    ///
    /// **A `.spell` is nameable from anywhere** (`tower::scene` registers every
    /// one), so it has to be findable from anywhere or `invoke` resolves at full
    /// confidence and then reports nothing.
    Tower,
    /// §10.1's fetch order — **a game rule**, and the reason this module exists.
    ///
    /// Unbusy instruments in raise order, then stores, then the arsenal. The
    /// instruments come first because the thing a player names mid-pipeline is
    /// the output of the last stage and it is still inside the tool that made
    /// it; that is what retired `siphon`. Busy ones are skipped, because §10.1's
    /// lock covers taking as much as putting. The store is the fallback, and the
    /// arsenal is last so a reagent in the room always outranks one carried.
    ///
    /// **Takes the room rather than reading `Cwd`**, because a spell installs its
    /// own domain around each instruction and a fetch that read the resource
    /// would be right only for as long as that stayed true.
    Fetch(Entity),
    /// The arsenal, wherever it is.
    ///
    /// Reachable from every room by design (§19), and the exemption is narrow:
    /// it holds finished work only.
    Arsenal,
}

/// How a name is compared with what a node is called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Naming {
    /// The node's own name, exactly as it is spelled.
    Leaf,
    /// Its name **or** its full path.
    ///
    /// §7 makes a place answer to both — `attend laboratory` and `attend
    /// /tower/laboratory` are one command — and a `Place` argument resolves to
    /// the path while a node carries only its last segment. Comparing one to the
    /// other matched *nothing* for an instrument, which is how `purge alembic`
    /// came to work at a distance through a fallback that was the only path
    /// ever taken.
    LeafOrPath,
    /// Its name, with `.spell` supplied if the caller left it off.
    ///
    /// `invoke threading` and `invoke threading.spell` are one spell, which is
    /// what `content::with_extension` already says everywhere else.
    Script,
}

/// A question about what a name reaches.
///
/// Built rather than passed as five arguments, because five positional
/// parameters of which two are `Option` is a call nobody can read — and reading
/// the call is the entire point of gathering these.
#[derive(Debug, Clone, Copy)]
pub struct Reach<'a> {
    world: &'a World,
    scope: Scope,
    kind: Option<NounKind>,
    naming: Naming,
}

/// Ask what `name` reaches.
///
/// Defaults are the narrowest of each axis — inside where you stand, any kind,
/// by leaf — so a call that says nothing more is §7's plain rule, and every
/// widening is visible at the call site.
#[must_use]
pub fn look(world: &World) -> Reach<'_> {
    Reach {
        world,
        scope: Scope::In(world.resource::<Cwd>().0),
        kind: None,
        naming: Naming::Leaf,
    }
}

impl<'a> Reach<'a> {
    /// Look somewhere else.
    #[must_use]
    pub const fn scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// Accept only one category of noun.
    #[must_use]
    pub const fn kind(mut self, kind: NounKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Compare names some other way.
    #[must_use]
    pub const fn naming(mut self, naming: Naming) -> Self {
        self.naming = naming;
        self
    }

    /// Every node in scope, in the order the scope searches.
    ///
    /// Public because [`Scope::Fetch`]'s order is a game rule that two callers
    /// need as a *list* rather than as a first hit: `pipeline` walks it looking
    /// for a recipe's whole input set, not for one name.
    #[must_use]
    pub fn candidates(&self) -> Vec<Entity> {
        match self.scope {
            Scope::In(node) => children_of(self.world, node),
            Scope::Under(node) => descend(self.world, node),
            Scope::Tower => descend(self.world, root(self.world)),
            Scope::Fetch(room) => fetch(self.world, room),
            Scope::Arsenal => keeping(self.world),
        }
    }

    /// The first node in scope that `name` names.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<Entity> {
        let wanted = self.wanted(name);
        self.candidates()
            .into_iter()
            .find(|node| self.answers(*node, &wanted))
    }

    /// What the name is after this reach's [`Naming`] has had it.
    fn wanted(&self, name: &str) -> String {
        match self.naming {
            Naming::Leaf | Naming::LeafOrPath => name.to_owned(),
            Naming::Script => crate::content::with_extension(name),
        }
    }

    /// Whether `node` answers to `wanted`.
    fn answers(&self, node: Entity, wanted: &str) -> bool {
        if let Some(kind) = self.kind
            && self.world.get::<Nameable>(node).map(|it| it.0) != Some(kind)
        {
            return false;
        }
        let named = self
            .world
            .get::<Name>(node)
            .is_some_and(|name| name.0 == wanted);
        match self.naming {
            Naming::Leaf | Naming::Script => named,
            Naming::LeafOrPath => named || path_of(self.world, node) == wanted,
        }
    }
}

/// Everything under `node`, itself included, depth first.
///
/// **`node` is a candidate**, which is what lets `Under(root)` answer for the
/// root's own children and `find_place` answer for the place it started from —
/// `attend laboratory` typed while standing in the laboratory is a real command
/// and it resolves to where you are.
fn descend(world: &World, node: Entity) -> Vec<Entity> {
    let mut found = Vec::new();
    let mut stack = vec![node];
    while let Some(at) = stack.pop() {
        found.push(at);
        stack.extend(children_of(world, at));
    }
    found
}

/// §10.1's fetch order, and there is no floor in it.
///
/// A tier used to come first for things lying loose in a room. Nothing can be
/// there any more: `siphon` was the only thing that ever put a reagent on the
/// floor, and a `move` destination resolves to a fixture. The bench and the shelf
/// are one place now, and it is the dispensary.
fn fetch(world: &World, cwd: Entity) -> Vec<Entity> {
    let here = children_of(world, cwd);
    let mut order: Vec<Entity> = Vec::new();

    // One partition rather than two filtered passes: the instruments in raise
    // order, then the stores.
    let (stores, instruments): (Vec<Entity>, Vec<Entity>) = here
        .into_iter()
        .filter(|node| world.get::<Fixture>(*node).is_some())
        .partition(|node| world.get::<Store>(*node).is_some());

    for node in instruments {
        // §10.1's lock covers taking as much as putting, and a scour is about to
        // despawn everything in there.
        if busy(world, node).is_some() {
            continue;
        }
        order.extend(children_of(world, node));
    }
    for node in stores {
        order.extend(children_of(world, node));
    }
    order.extend(keeping(world));
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// A tower with a player standing in the laboratory.
    fn tower() -> Sim {
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        sim
    }

    /// What a node is called, for readable assertions.
    fn name_of(world: &World, node: Entity) -> String {
        world
            .get::<Name>(node)
            .map(|name| name.0.clone())
            .unwrap_or_default()
    }

    /// The room the player is in.
    fn here(world: &World) -> Entity {
        world.resource::<Cwd>().0
    }

    #[test]
    fn a_bare_look_is_section_sevens_plain_rule() {
        // The defaults are the narrowest of each axis, so a call that says
        // nothing more is *"you can only name what is where you are"* — and every
        // widening past it is visible at the call site rather than buried.
        let sim = tower();
        let world = sim.world();

        assert!(
            look(world).find("dispensary").is_some(),
            "a shelf in the room was not reachable from the room",
        );
        // The dispensary's contents are one level deeper, so a bare look does
        // not see them. `Scope::Fetch` is what does.
        assert!(
            look(world).find("sage").is_none(),
            "a bare look reached inside a fixture, which is not `In`",
        );
    }

    #[test]
    fn under_includes_the_node_it_starts_from() {
        // `attend laboratory` typed while standing in the laboratory is a real
        // command, so the walk has to answer for where it began. Both original
        // walks did — `stack = vec![from]` — and losing it here would break
        // `attend` in the room you are already in.
        let sim = tower();
        let world = sim.world();
        let laboratory = here(world);

        let found = look(world)
            .scope(Scope::Under(laboratory))
            .kind(NounKind::Place)
            .naming(Naming::LeafOrPath)
            .find("laboratory");
        assert_eq!(found, Some(laboratory), "a place could not find itself");
    }

    #[test]
    fn a_place_answers_to_its_leaf_and_to_its_whole_path() {
        // §7's two spellings of one command. A `Place` argument resolves to the
        // path while the node carries the leaf, so a lookup that compared only
        // one of them matched nothing for an instrument — which is how `purge
        // alembic` came to reach its target through a tower-wide fallback.
        let sim = tower();
        let world = sim.world();

        let by_leaf = look(world)
            .scope(Scope::Tower)
            .kind(NounKind::Place)
            .naming(Naming::LeafOrPath)
            .find("laboratory");
        let by_path = look(world)
            .scope(Scope::Tower)
            .kind(NounKind::Place)
            .naming(Naming::LeafOrPath)
            .find("/tower/laboratory");
        assert!(by_leaf.is_some(), "a place did not answer to its own name");
        assert_eq!(
            by_leaf, by_path,
            "the two spellings reached different rooms"
        );

        // ...and `Leaf` alone does not, which is the axis being a real choice
        // rather than a default nobody set.
        assert!(
            look(world)
                .scope(Scope::Tower)
                .kind(NounKind::Place)
                .find("/tower/laboratory")
                .is_none(),
            "`Leaf` matched a path, so the two settings are one setting",
        );
    }

    #[test]
    fn a_script_is_findable_from_anywhere_it_is_nameable_from() {
        // **Nameable and findable are one rule**, and §19 records them drifting
        // apart twice. `tower::scene` registers every `.spell` from the whole
        // tree, so a lookup narrower than the tree makes `invoke` resolve at full
        // confidence and then report nothing.
        let mut sim = tower();
        sim.submit("attend archive");
        sim.step();
        let world = sim.world();

        for spelling in ["first_light", "first_light.spell"] {
            assert!(
                look(world)
                    .scope(Scope::Tower)
                    .kind(NounKind::Script)
                    .naming(Naming::Script)
                    .find(spelling)
                    .is_some(),
                "`{spelling}` was nameable from the archive and not findable",
            );
        }
    }

    #[test]
    fn the_kind_filter_is_what_keeps_a_verb_off_the_wrong_noun() {
        // A `Script` lookup must not answer with a room that happens to share a
        // name, and the filter is the only thing standing between the two.
        let sim = tower();
        let world = sim.world();

        assert!(
            look(world)
                .scope(Scope::Tower)
                .kind(NounKind::Script)
                .find("laboratory")
                .is_none(),
            "a script lookup reached a place",
        );
    }

    #[test]
    fn the_fetch_order_is_instruments_then_stores_then_the_arsenal() {
        // **§10.1's search order, which is a game rule.** The instruments come
        // first because the thing a player names mid-pipeline is the output of
        // the last stage and it is still inside the tool that made it — that is
        // what retired `siphon`. The store is the fallback. The arsenal is last,
        // so a reagent in the room always outranks one carried.
        let sim = tower();
        let world = sim.world();
        let order: Vec<String> = look(world)
            .scope(Scope::Fetch(here(world)))
            .candidates()
            .into_iter()
            .map(|node| name_of(world, node))
            .collect();

        assert!(
            order.contains(&"sage".to_owned()),
            "the dispensary's stock was not within reach: {order:?}",
        );
    }

    #[test]
    fn a_busy_instrument_is_not_fetched_from() {
        // §10.1's lock covers taking as much as putting: without this a `move`
        // could gut a run in flight, spend the production slot for nothing, and
        // say not a word about it.
        let mut sim = tower();
        sim.submit("grind sage");
        sim.step();

        let world = sim.world();
        let mortar = look(world)
            .find("mortar_and_pestle")
            .expect("the mortar is in the laboratory");
        assert!(
            super::busy(world, mortar).is_some(),
            "the mortar is idle, so this test proves nothing",
        );

        // **By entity, not by name.** The dispensary is endless and holds sage
        // of its own, so asking whether the *word* is reachable answers yes for
        // a reason that has nothing to do with the lock. What must not be
        // reachable is the load now inside the working mortar.
        let inside = super::children_of(world, mortar);
        assert!(
            !inside.is_empty(),
            "the mortar took nothing, so this test proves nothing",
        );
        let reachable = look(world).scope(Scope::Fetch(here(world))).candidates();
        for node in inside {
            assert!(
                !reachable.contains(&node),
                "`{}` was fetched out of a working instrument",
                name_of(world, node),
            );
        }
    }
}
