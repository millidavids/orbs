//! What the player can currently name.
//!
//! §6, step 2: entities that exist now, not only vocabulary. Walks the tree,
//! never a query, because registration order breaks ties and archetype order
//! shifts (see `tower::node`). Rebuilt every tick rather than on change, so
//! replay cannot depend on which mutation remembered to mark it.

use bevy_ecs::prelude::*;

use super::node::{Cwd, Fixture, Name, Nameable, children_of, path_of};
use crate::execute::LOG;
use crate::parser::{NounKind, Scene};

/// The room whose circle binds demons, the one place their names resolve.
const MENAGERIE: &str = "menagerie";

/// Rebuild the nameable surface of the world.
///
/// You can only name what is where you are (§7); acting at a distance is an
/// unlock (§19). Places are exempt, or movement would be a lock whose key is
/// behind it.
pub fn rebuild(world: &mut World) {
    let Some(cwd) = world.get_resource::<Cwd>().copied() else {
        return;
    };
    let scene = scene_at(world, cwd.0);
    world.insert_resource(scene);
}

/// What could be named if the player were standing at `at`.
///
/// For a spell, which runs in its own domain wherever the player stands. It
/// returns rather than inserting, so the live `Scene` is never clobbered from
/// inside an input call.
#[must_use]
pub fn scene_at(world: &World, at: Entity) -> Scene {
    let cwd = Cwd(at);

    // §3's log is nameable from the moment the game starts and is not a node in
    // the tree, so it is registered first — dropping it here would silently kill
    // `peruse orb.log` and `sift <pattern> orb.log`.
    let mut scene = Scene::new().with(NounKind::File, LOG);

    // Every recipe and manual subject is a `Topic`, nameable anywhere, from the
    // [`Topics`] snapshot (see there). A secret is subtracted here, against the
    // world as it stands, or `recall <secret>` answers before it is found.
    let hidden: Vec<String> = {
        let recipes = world.resource::<crate::content::Recipes>();
        let known = super::known(world);
        let opened = world.resource::<super::Opened>();
        // Found or earned; and a shut room's page is not a subject yet, or
        // `recall archive` would spend the rail's dark box.
        recipes
            .secrets()
            .into_iter()
            .chain(recipes.gated())
            .filter(|made| !known.knows(recipes, made))
            // `is_room`, not `DOMAINS`, which lacks the grimoire.
            .chain(
                super::opened::ROOMS
                    .into_iter()
                    .filter(|room| !opened.is_open(room)),
            )
            // A demon the circle cannot bind yet, for a gated product's reason:
            // its `recall_` page is a topic from tick 0 (§19, *Supply*).
            .chain(
                world
                    .resource::<crate::content::Spendables>()
                    .demons()
                    .filter(|demon| !opened.has(&super::opened::demon_key(demon))),
            )
            .map(str::to_owned)
            .collect()
    };
    let is_hidden = |name: &str| hidden.iter().any(|made| made == name);

    {
        let recipes = world.resource::<crate::content::Recipes>();
        for output in recipes.outputs() {
            if is_hidden(output) {
                continue;
            }
            scene = scene.with(NounKind::Topic, output);
        }
    }

    // Every substance with a word, here or not, so an absent one does not fuzz
    // into another (`Scene::knowing`). Secrets subtracted, or "none here" tells
    // a player the name is real.
    let substances: Vec<String> = crate::content::Recipes::substances(world)
        .into_iter()
        .filter(|name| !is_hidden(name))
        .collect();
    scene = scene.knowing(substances);

    // And every word a verb answers to, or `purge walk` fuzzed to `wall` and
    // `recall edit` to `exit` (§19's `gained` leak).
    scene = scene.knowing(crate::parser::single_words().map(|(word, _)| word.to_owned()));
    {
        let topics = world.resource::<Topics>();
        for topic in &topics.0 {
            if is_hidden(topic) {
                continue;
            }
            scene = scene.with(NounKind::Topic, topic);
        }
    }

    // Every demon the circle can bind, as the essence it is in the arsenal, so
    // `summon golem` resolves before the first one exists (§19, *Supply*).
    // At the circle only: registered everywhere, a demon was a fourth answer
    // to `move nonsense` in the laboratory. After the topics, so no earlier tie
    // moves.
    let at_the_circle = super::domain_of(world, cwd.0)
        .and_then(|domain| world.get::<Name>(domain))
        .is_some_and(|domain| domain.0 == MENAGERIE);
    if at_the_circle {
        for demon in world.resource::<crate::content::Spendables>().demons() {
            if is_hidden(demon) {
                continue;
            }
            scene = scene.with(NounKind::Essence, demon);
        }
    }

    // Every verb word as a `Command`, so `recall grind` resolves anywhere
    // without widening `NounKind::Any` — a manual readable only in the right
    // room has a lock on it.
    for (word, _) in crate::parser::single_words() {
        scene = scene.with(NounKind::Command, word);
    }
    // What `status` narrows to, anywhere. After the commands, so no earlier
    // tie moves.
    for word in crate::parser::REPORTS {
        scene = scene.with(NounKind::Report, word);
    }

    // Every spell, wherever the player stands: the book you carry, not a
    // domain, so §19's rule still holds — and a `Script` slot with nothing in
    // scope once sent `bind night_watch` to `sift`.
    for node in walk(world, super::filesystem_root(world, cwd.0)) {
        if world.get::<Nameable>(node).map(|n| n.0) == Some(NounKind::Script)
            && let Some(name) = world.get::<Name>(node)
        {
            scene = scene.with(NounKind::Script, &name.0);
        }
    }

    // The arsenal's contents, wherever the player stands — carried, not shelved
    // (see `tower::keep`). After the spells and before the readings, for the
    // tie-break.
    let mut kept: Vec<&str> = Vec::new();
    for node in super::keeping(world) {
        let (Some(name), Some(kind)) = (world.get::<Name>(node), world.get::<Nameable>(node))
        else {
            continue;
        };
        scene = scene.with(kind.0, &name.0);
        kept.push(&name.0);
    }

    // And every open supply, wherever its stock went: one `move` carries the
    // whole batch out, and `stop warding` then fuzzed into a spell. After the
    // arsenal, so no earlier tie moves.
    if let Some(supplies) = world.get_resource::<super::supply::Supplies>() {
        for (named, _) in supplies.iter() {
            if !kept.contains(&named) {
                scene = scene.with(super::kind_of(world, named), named);
            }
        }
    }

    // Every domain's readings, always: a solver's `if` resolves at cast, when
    // none is true of anything, and a bound spell recasts every lap, so a
    // vocabulary that came and went with a puzzle would work on some laps only.
    // In this order, for the tie-break.
    let vocabularies: [fn() -> Vec<&'static str>; 8] = [
        super::maze::readings,
        super::ward::readings,
        super::pylon::readings,
        super::circle::readings,
        super::siege::readings,
        super::charm::readings,
        super::stores::readings,
        super::supply::readings,
    ];
    for reading in vocabularies.into_iter().flat_map(|readings| readings()) {
        scene = scene.with(NounKind::Sense, reading);
    }

    // Every place, wherever the player is. Depth-first from the root, children
    // in spawn order.
    for node in walk(world, super::filesystem_root(world, cwd.0)) {
        if world.get::<Nameable>(node).map(|n| n.0) == Some(NounKind::Place) {
            // A shut room stays a place the parser knows, so `attend archive`
            // is refused in voice rather than fuzzed into four other rooms
            // (§15). Another room's satchel is not offered, or the word
            // resolves to the first registered of six.
            if world.get::<super::Satchel>(node).is_some()
                && !children_of(world, cwd.0).contains(&node)
            {
                continue;
            }
            // A place answers to its full path; §6's matcher also accepts the
            // last segment, which is what makes `attend laboratory` reach
            // `/tower/laboratory` (§7: players say the place, not the path).
            scene = scene.with(NounKind::Place, &path_of(world, node));
        }
    }

    // The verbs the instruments *here* answer to (§7, `Scene::offers`). Derived
    // from the fixtures rather than listed, so a domain that raises a tool with
    // its own verb gets that verb in scope without touching the parser.
    for node in children_of(world, cwd.0) {
        if let Some(operation) = world.get::<super::Operation>(node) {
            scene = scene.offering(operation.0);
        }
    }

    // Everything else: only what is here.
    for node in children_of(world, cwd.0) {
        let (Some(name), Some(kind)) = (world.get::<Name>(node), world.get::<Nameable>(node))
        else {
            continue;
        };
        if kind.0 == NounKind::Place {
            // ...and what is in this room's fixtures — see [`Fixture`].
            if world.get::<Fixture>(node).is_some() {
                for held in children_of(world, node) {
                    let (Some(name), Some(kind)) =
                        (world.get::<Name>(held), world.get::<Nameable>(held))
                    else {
                        continue;
                    };
                    if kind.0 != NounKind::Place {
                        scene = scene.with(kind.0, &name.0);
                    }
                }
            }
            continue;
        }
        scene = scene.with(kind.0, &name.0);
    }

    scene
}

/// The manual subjects the parser can name, fixed at construction.
///
/// Snapshotted, not read live, or a prose hot-reload could change what a phrase
/// resolves to — a decision `Sim::set_prose` promises never to touch. A new
/// subject needs a relaunch.
#[derive(bevy_ecs::resource::Resource, Debug, Default)]
pub struct Topics(pub Vec<String>);

impl Topics {
    /// The subjects a prose file can answer on.
    #[must_use]
    pub fn of(prose: &crate::content::Prose) -> Self {
        Self(prose.topics().into_iter().map(ToOwned::to_owned).collect())
    }
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

// `root_of` lived here and is now `node::filesystem_root` — the same walk was
// written out in four places, and the day `/grimoire` sits beside `/tower`
// exactly one of them should have to learn about it.

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

    /// The names the scene offers as *things*, rather than as manual subjects.
    ///
    /// Every material has a `recall` page and a page is readable anywhere, so a
    /// reagent's name is in the scene everywhere as a `Topic`. §7's scoping is a
    /// claim about the kind: in the laboratory `sage` is something you grind, in
    /// the archive only something you read about.
    fn things(sim: &Sim) -> Vec<String> {
        sim.scene()
            .nouns()
            .iter()
            .filter(|noun| noun.kind != crate::parser::NounKind::Topic)
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
            place("retort") < place("laboratory.log"),
            "belongings keep theirs: {found:?}",
        );
        assert!(
            place("/tower") < place("retort"),
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
        // §7. With `retort`, since recipe names are `Topic`s nameable everywhere;
        // the archive's side is its log.
        let mut sim = Sim::new(1);
        sim.step();
        assert!(!names(&sim).iter().any(|name| name == "retort"));

        sim.submit("attend laboratory");
        sim.step();
        assert!(names(&sim).iter().any(|name| name == "retort"));
        assert!(!names(&sim).iter().any(|name| name == "archive.log"));

        sim.submit("attend archive");
        sim.step();
        assert!(names(&sim).iter().any(|name| name == "archive.log"));
        assert!(!names(&sim).iter().any(|name| name == "retort"));
    }

    #[test]
    fn what_an_instrument_holds_is_nameable_from_the_room_it_stands_in() {
        // The `Fixture` rule, asked of `things`: `sage` is a `Topic` from the
        // archive too, and §7 scopes the thing.
        let mut sim = Sim::new(1);
        sim.submit("attend laboratory");
        sim.step();
        assert!(
            things(&sim).iter().any(|name| name == "sage"),
            "the dispensary's sage is out of reach from the laboratory"
        );

        sim.submit("attend archive");
        sim.step();
        assert!(
            !things(&sim).iter().any(|name| name == "sage"),
            "it stayed nameable from another domain"
        );
        assert!(
            names(&sim).iter().any(|name| name == "sage"),
            "`recall sage` stopped working outside the laboratory"
        );
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
        // A separate pass, because sharing a schedule with a frontend's systems
        // left the order to Bevy's topsort; a rename is seen the same tick.
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
