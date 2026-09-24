//! What the player can currently name.
//!
//! DESIGN.md §6, step 2: fuzzy match against known vocabulary and entities that
//! currently exist. That second half is what separates this from a command
//! parser — `decoct clarity` resolves because clarity is an essence, and stops
//! resolving the moment it is not.
//!
//! Walks the tree rather than querying, because §6 breaks a scoring tie by
//! registration order, so that order *is* part of the parse. A `Query` supplies
//! archetype order, which is not insertion order and shifts when a component is
//! added — starting a brew would reorder the noun list, flip a tie, and change
//! what a phrase resolves to, with no test seeing it. `Children` from the root
//! is insertion-ordered by construction.
//!
//! Rebuilt per tick rather than on change: a marker somebody forgets to set is a
//! cache-invalidation bug that makes replay depend on which mutations
//! remembered. At tens of nouns and 1 Hz the cost is not measurable.

use bevy_ecs::prelude::*;

use super::node::{Cwd, Fixture, Name, Nameable, children_of, path_of};
use crate::execute::LOG;
use crate::parser::{NounKind, Scene};

/// Rebuild the nameable surface of the world.
///
/// You can only name what is where you are. §7 makes the tree the tower and
/// navigation diegetic — *"paths are places"* — so a domain's contents are
/// nameable only while the player stands in it. That is the base state, not a
/// limitation: §19's pane addressing is the Phase 10 unlock that lets a player
/// act on a domain without walking to it, and acting at a distance has to
/// *become* possible.
///
/// Places are exempt: navigation is how you reach what you cannot yet name, so
/// gating movement on being somewhere would be a lock whose key is behind it.
pub fn rebuild(world: &mut World) {
    let Some(cwd) = world.get_resource::<Cwd>().copied() else {
        return;
    };
    let scene = scene_at(world, cwd.0);
    world.insert_resource(scene);
}

/// What could be named if the player were standing at `at`.
///
/// Separate from [`rebuild`] because a spell runs in its own domain wherever the
/// player stands: `grind sage` resolves in the laboratory and nowhere else (§7,
/// `Scene::offers`), so a spell cast from the archive would lose every
/// location-scoped verb if read against the room the player is in.
///
/// `spell::compile` asks once for the spell's domain at cast; the runner asks
/// again per line, having swapped `Cwd` for one instruction. It takes `&World`
/// and returns rather than inserting — swap `Cwd`, `rebuild`, swap back would
/// clobber the live `Scene` from inside an input call, and input touches only
/// session state.
#[must_use]
pub fn scene_at(world: &World, at: Entity) -> Scene {
    let cwd = Cwd(at);

    // §3's log is nameable from the moment the game starts and is not a node in
    // the tree, so it is registered first — dropping it here would silently kill
    // `peruse orb.log` and `sift <pattern> orb.log`.
    let mut scene = Scene::new().with(NounKind::File, LOG);

    // Every recipe is a `Topic`, nameable from anywhere, because a manual is not
    // a thing in a room — §6.1 registers recipes beside their essence so
    // `make a potion of clarity` answers with how, the tutorial entry point now
    // that `decoct` is retired (§19). Plus every subject the manual answers on,
    // from [`Topics`], snapshotted once at construction: read live, renaming
    // `recall_brewing` mid-session moved `Topic:brewing` and `(seed,
    // submissions)` stopped replaying to the same world. `Sim::set_prose` calls
    // prose safe because it "reaches no decision" — a noun *is* a decision. No
    // intermediate `Vec<String>`: these run per tick, and cloning each name only
    // hands `Scene::with` a `&str` it copies again.
    //
    // A secret is subtracted here rather than at snapshot time: its `recall_`
    // page is a prose key, so it is in the snapshot from tick 0, and gating here
    // is a set subtraction against the world as it stands. Get it wrong and
    // `recall <secret>` answers before the player has found it, with every test
    // green because the recipe still refuses to fire.
    let hidden: Vec<String> = {
        let recipes = world.resource::<crate::content::Recipes>();
        let known = super::known(world);
        let opened = world.resource::<super::Opened>();
        // Found *or* earned: a gated product the room's line has not reached is
        // as unsayable as a secret the lens has not found. And a shut room's
        // name, so its manual page is not a subject yet — the rail's dark box
        // says *there is more* without saying what, and `recall archive` would
        // spend that.
        recipes
            .secrets()
            .into_iter()
            .chain(recipes.gated())
            .filter(|made| !known.knows(recipes, made))
            // `is_room`, not `DOMAINS`: the fourth site of the "which names can
            // be shut" rule, and the one that did not follow when the grimoire
            // left `DOMAINS`. A shut grimoire's name stopped being withheld here
            // when the list shrank — inert until somebody authors a
            // `recall_grimoire` page, and then silent.
            .chain(
                super::opened::ROOMS
                    .into_iter()
                    .filter(|room| !opened.is_open(room)),
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

    // Every substance the laboratory has a *word* for, which is deliberately not
    // the same list as what is here: it stops one real name fuzzing into another
    // when the first is out of stock (`Scene::knowing`, and the
    // `digest ground-sage` defect). Secrets are subtracted too — a recognised
    // word answers "there is none here" rather than "no such thing", which tells
    // a player a name they have not earned is real.
    let substances: Vec<String> = crate::content::Recipes::substances(world)
        .into_iter()
        .filter(|name| !is_hidden(name))
        .collect();
    scene = scene.knowing(substances);

    // And every word a verb answers to, for the same rule: `Scene::knowing` was
    // installed over substances alone, so a verb's own word could still fuzz
    // into a noun. Three did, at or above the score that rejected a name
    // elsewhere:
    //
    //   `purge walk` -> `wall`    750, and the two are words about one screen
    //   `purge edit` -> `exit`    750
    //   `purge make` -> `marks`   600
    //
    // A reading is a `NounKind::Sense` and `NounKind::Any` reaches it, so a
    // destructive verb answered "there is no wall within reach" to a player who
    // typed a word the game taught them (§19's `gained` leak). Closing the class
    // beats renaming three readings, and it reaches the manual too — `recall
    // edit` answered with the maze's way out rather than the spell editor.
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

    // Every verb, as something the manual can be asked about. `recall grind` has
    // to resolve, and `NounKind::Command` does it without widening
    // `NounKind::Any` — the kind's own doc has the three places 27 canonicals
    // leak through if registered as `Topic`.
    //
    // Nameable from everywhere, unlike the overview's listing: a bare `recall`
    // shows what works in this room (§7), but a manual you can only read in the
    // right room has a lock on it. Every *word*, not every canonical — `recall
    // walk` and `recall edit` fell through to the noun vocabulary and came back
    // with a maze reading. `single_words` includes each canonical by rule
    // (`canonical_names_are_one_short_word`), so this widens rather than adding a
    // second list to keep in step.
    for (word, _) in crate::parser::single_words() {
        scene = scene.with(NounKind::Command, word);
    }

    // Every spell, wherever the player is standing — the exemption places have.
    // Without it a `.spell` is nameable only from inside `/grimoire`, since
    // everything that is not a place is registered from `cwd` and `/grimoire` is
    // a protected domain rather than a `Fixture`. `invoke first_light` would work
    // only where there is no laboratory to run it in.
    //
    // It does not loosen §19's "you can only name what is where you are", which
    // stops you acting on another *domain* at a distance. A spell is not a domain
    // but the book you carry — §8 has the player keeping their spellbook in vim.
    // `execute::dispatch` has the cost of getting it wrong: a `Script` slot with
    // nothing in scope makes `bind night_watch` fall through to `sift` and report
    // success.
    for node in walk(world, super::filesystem_root(world, cwd.0)) {
        if world.get::<Nameable>(node).map(|n| n.0) == Some(NounKind::Script)
            && let Some(name) = world.get::<Name>(node)
        {
            scene = scene.with(NounKind::Script, &name.0);
        }
    }

    // Everything the arsenal holds, wherever the player is standing — the third
    // exemption, for the same reason: finished work is carried, not shelved. A
    // potion is brewed in the laboratory to be used elsewhere, so an arsenal
    // nameable only from inside itself would be a room you walk to in order to
    // look at things you cannot then use.
    //
    // After the spells and before the readings, because `scene_at`'s order *is*
    // the tie-break (§6) and this keeps the words a solver names in the relative
    // order they have always had.
    //
    // §7 still forbids acting on another *domain* at a distance, which
    // `tower::keep` covers. Naming is only half of it: three lookups had to learn
    // this too, or a word resolves at full confidence and then reports "no such
    // thing".
    for node in super::keeping(world) {
        let (Some(name), Some(kind)) = (world.get::<Name>(node), world.get::<Nameable>(node))
        else {
            continue;
        };
        scene = scene.with(kind.0, &name.0);
    }

    // The maze's readings, always, whether or not a maze is open. These are the
    // words a solver's `if` names — `if north has 1 or fewer marks` — and they
    // must resolve at *cast*, which is exactly when none of them is true of
    // anything. `spell::compile` nulls a condition whose names it cannot place;
    // that guard is right (§19: `has ground-slat` answered "no" for ever) and it
    // cannot tell a not-yet-existing state from a typo, so the scene offers the
    // vocabulary rather than the instances.
    //
    // Stability is the point: `bind::stand` recasts a held spell every lap, so a
    // vocabulary that came and went with the maze would make a bound spell work
    // on some laps and not others, with nothing on screen saying why.
    //
    // `back`, `spoil` and every errand word ride along for the same reason. An
    // errand is set by a scroll spent after the spell was written, so at cast
    // there is never one on — `if the stacks has gleaning` would compile to a
    // dead branch, and a solver that could not ask which maze it was in is two
    // solvers the player chooses between by hand.
    for reading in super::maze::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // And the lens's, for the same reason. Six words, all deltas: `aligned`,
    // `astray` and `spent` are published as counts on the record and
    // deliberately absent here, so a spell cannot ask for them — §19's
    // correction is that a codemaker answers *which way did it move*, and the
    // rest was the orb doing the player's bookkeeping.
    for reading in super::ward::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // And the sanctum's, third and the same reason. `integrity` is here too
    // though it is always published: a spell compiles in `Sim::bare`'s world as
    // readily as in a played one, and a vocabulary that depended on a system
    // having run would compile differently on the first tick than on the second.
    // Appended after the other two, for §6's tie-break.
    for reading in super::pylon::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // And the menagerie's, fourth and the same reason: at cast no beast is
    // waiting, so `fervour` and the humour a glyph carries resolve against
    // nothing and `spell::compile` nulls the condition. In the chant's slot,
    // after the sanctum's, for §6's registration-order tie-break.
    for reading in super::circle::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // And the bailey's, fifth and the same reason. Not hypothetical —
    // `besieging` predates this loop and all four of its questions came back
    // "that question means nothing", the `repeat until` stopping on its first
    // evaluation and the spell doing nothing but `defend`. The three intents are
    // here too, because a tree that answers a volley differently from an
    // onslaught asks `if the rampart has volley`. Appended after the
    // menagerie's.
    for reading in super::siege::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // ...and the forge's, appended after the bailey's for the sixth time.
    // Unconditional like every other domain's: a spell written before the player
    // has ever opened a lattice must still compile, or the words would only mean
    // something in the room the author stood in — the defect §19 records the
    // bailey shipping with.
    for reading in super::charm::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // ...and the arsenal's three, appended after the forge's for the seventh
    // time. Unconditional for the same reason: a spell that keeps its own stores
    // up is written once and cast anywhere, and words that only resolved while
    // standing in the bailey would be the defect above.
    for reading in super::stores::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // Every place, wherever the player is. Depth-first from the root, children
    // in spawn order.
    for node in walk(world, super::filesystem_root(world, cwd.0)) {
        if world.get::<Nameable>(node).map(|n| n.0) == Some(NounKind::Place) {
            // A shut room stays a place the parser knows. Dropped from the
            // scene, `attend archive` fuzzed into a numbered prompt offering
            // four *other* rooms — §15's dead end, reached by the word a new
            // player is most likely to try. Kept nameable, `attend`'s gate
            // answers "the archive is not yours yet", and the weave's details
            // panel already names the first station. The manual page stays
            // hidden, above.
            //
            // A satchel from another room is not offered at all — the one place
            // this walk is not exhaustive. Every domain has one, all called
            // `satchel`, and §6's matcher accepts a last segment, so six such
            // paths make the word resolve to whichever was registered first:
            // `survey satchel` in the menagerie read the *laboratory's* and
            // reported it empty.
            //
            // Scoping here rather than renaming the node keeps the meaning the
            // mechanic wants, and makes `build`'s exemption from
            // `every_place_leaf_is_unique` true rather than argued: the leaves
            // collide in the tree, never in the scene. It costs naming another
            // room's satchel by full path, which §7 forbids acting on anyway.
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
            // ...and what is *in* the fixtures of this room. An instrument is a
            // place so it can be surveyed and attended, but it is furniture on a
            // bench, and the sage in the mortar is within reach of someone in
            // the laboratory. Without this §10.1's own loop cannot be typed:
            // `move husks from alembic to dispensary` could not name `husks`.
            //
            // It does not loosen "you can only name what is where you are",
            // which is about acting on another *domain* at a distance; a domain
            // is not a `Fixture`, and Phase 10's pane addressing still relaxes
            // the rule in general.
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
/// Snapshotted, not read live from [`Prose`](crate::content::Prose): every
/// `recall_` key is a `NounKind::Topic`, so reading them per tick let a prose
/// hot-reload change what a phrase resolves to — a *decision*, which
/// `Sim::set_prose` promises it never touches. The cost is that a new manual
/// subject needs a relaunch; the lines themselves still reload.
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
        // §7: the tree is the tower and navigation is diegetic. Brewing happens
        // in the laboratory because that is where the instruments are, which is
        // what gives §19's Phase 10 pane addressing something to be an unlock
        // *from*.
        //
        // Tested with `retort` rather than `clarity`: recipe names are `Topic`s
        // nameable everywhere (§6.1).
        //
        // The archive's side is its log, which replaced `sigil-iv` when the
        // three sigils went with the `divine` that consumed a fragment (§19). A
        // domain log makes the same claim about the same rule: `NounKind::File`
        // rather than a place, registered from `cwd`.
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
        // The `Fixture` rule. An instrument is furniture on a bench, not
        // somewhere you travel to, and §10.1's own loop cannot be typed
        // otherwise: `move husks from alembic to dispensary` has to name
        // `husks`. A domain is not a fixture, so the domain rule above holds.
        //
        // Asked of `things`, not `names`: `sage` has a manual page and so is in
        // the scene from the archive too, as a `Topic`. §7 scopes the *thing*.
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
        // `rebuild` sharing a schedule with whatever a frontend adds through
        // `with_schedule` is an ambiguity, not an ordering — Bevy's topsort ran
        // the caller's systems first despite `rebuild` being inserted first. A
        // separate pass makes the order a fact: a renamed node is visible to the
        // scene in the *same* tick.
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
