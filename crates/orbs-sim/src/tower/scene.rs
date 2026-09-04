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

use super::node::{Cwd, Fixture, Name, Nameable, children_of, path_of};
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
/// *"named by domain, routed within the focused set"* — as a **Phase 10** item,
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
    let scene = scene_at(world, cwd.0);
    world.insert_resource(scene);
}

/// What could be named **if the player were standing at `at`**.
///
/// # Why this is separate from [`rebuild`]
///
/// A spell runs in **its own domain**, wherever the player happens to be
/// standing — so every one of its lines is read against a scene that is not the
/// live one. `grind sage` resolves in the laboratory and nowhere else (§7,
/// `Scene::offers`), and a spell cast from the archive would lose every
/// location-scoped verb in the file if it were read against the room the player
/// is in.
///
/// So `spell::compile` asks this once, for the spell's domain, when the spell is
/// cast; the runner asks it again per line, having swapped `Cwd` for exactly the
/// length of one instruction. It takes `&World` and **returns** rather than
/// inserting, because the alternative — swap `Cwd`, call `rebuild`, swap back —
/// would clobber the live `Scene` from inside an input call, and input touches
/// only session state.
#[must_use]
pub fn scene_at(world: &World, at: Entity) -> Scene {
    let cwd = Cwd(at);

    // §3's log is nameable from the moment the game starts and is not a node in
    // the tree, so it is registered first — dropping it here would silently kill
    // `peruse orb.log` and `sift <pattern> orb.log`.
    let mut scene = Scene::new().with(NounKind::File, LOG);

    // Every recipe is a `Topic`, nameable from anywhere, because a manual is not
    // a thing in a room — §6.1 registers recipes beside their essence so
    // `make a potion of clarity` answers with how, which is the tutorial entry
    // point now that `decoct` is retired (§19).
    //
    // ...and every subject the manual can answer on, from [`Topics`], which is
    // snapshotted **once** at construction. Reading `Prose::topics()` live here
    // made hot-reloading prose change what the parser can resolve: renaming
    // `recall_brewing` mid-session dropped `Topic:brewing` and added another,
    // so `recall brewing` started resolving somewhere else and `(seed,
    // submissions)` no longer replayed to the same world — which is exactly what
    // `Sim::set_prose` documents as safe, on the grounds that prose "reaches no
    // decision". A noun *is* a decision.
    //
    // No intermediate `Vec<String>`: these ran per tick, cloning every name into
    // an owned string only to hand `Scene::with` a `&str` it copies again.
    // **A secret is subtracted here, and it has to be here.** `Topics` is
    // snapshotted once at construction (`sim.rs`) so a prose hot-reload cannot
    // change what a phrase resolves to — and a secret potion's `recall_` page is
    // a prose key, so it is in that snapshot from tick 0. Gating at snapshot
    // time would be gating a thing that is built once and never rebuilt; gating
    // here is a set subtraction against the world as it stands, which is what
    // `scene_at` is for.
    //
    // Get this wrong and `recall <secret>` answers before the player has found
    // it — with every test green, because the recipe still refuses to fire.
    let hidden: Vec<String> = {
        let recipes = world.resource::<crate::content::Recipes>();
        let known = super::known(world);
        let opened = world.resource::<super::Opened>();
        // Found *or* earned: a gated product the room's line has not reached is
        // as unsayable as a secret the lens has not found. **And a shut room's
        // name**, so its manual page is not a subject yet — the rail's dark box
        // says *there is more* without saying what, and `recall archive`
        // answering would spend that.
        recipes
            .secrets()
            .into_iter()
            .chain(recipes.gated())
            .filter(|made| !known.knows(recipes, made))
            // **`is_room`, not `DOMAINS`** — the fourth site of the "which names
            // can be shut" rule, and the one that did not follow when the
            // grimoire left `DOMAINS`. `is_room` exists precisely because that
            // rule had copies that disagreed; a shut grimoire's name stopped
            // being withheld here the moment the list shrank, which is inert
            // only until somebody authors a `recall_grimoire` page and then
            // fails silently.
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

    // Every substance the laboratory has a **word** for, which is not the same
    // list as what is here — that is the point. It stops one real name being
    // fuzzed into another when the first is out of stock; see `Scene::knowing`
    // for the `digest ground-sage` defect that named it.
    // **Also subtracted from `knowing`.** A secret left in the known-substance
    // list is a *word the parser recognises*, so typing it answers *"there is
    // none here"* rather than *"no such thing"* — a different answer from a
    // nonsense word, which is exactly the discriminator `Scene::knowing` exists
    // to draw, and enough to tell a player a name they have not earned is real.
    let substances: Vec<String> = crate::content::Recipes::substances(world)
        .into_iter()
        .filter(|name| !is_hidden(name))
        .collect();
    scene = scene.knowing(substances);

    // **And every word a verb answers to, for the same rule and the same
    // reason.** `Scene::knowing`'s rule is *a phrase that is itself a word the
    // game knows only ever matches exactly*, and it was installed over
    // substances alone — so a **verb's** own word could still fuzz into a noun.
    // Three did, all at or above the score that rejected a name elsewhere:
    //
    //   `purge walk` -> `wall`    750, and the two are words about one screen
    //   `purge edit` -> `exit`    750
    //   `purge make` -> `marks`   600
    //
    // Each is §19's `gained` leak: a reading is a `NounKind::Sense`, which
    // `NounKind::Any` reaches, so a destructive verb answered *"there is no wall
    // within reach"* to a player who typed a word the game taught them. Renaming
    // the readings would have closed three instances of an open class; this
    // closes the class, and the next reading anybody authors is safe by default.
    //
    // It reaches the manual too, and that was the worse half: `recall edit`
    // answered with the *maze's way out* rather than with the spell editor.
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

    // **Every verb, as something the manual can be asked about.** `recall grind`
    // has to resolve, and `NounKind::Command` is what makes that possible without
    // widening `NounKind::Any` — see the kind's own doc for the three places 27
    // canonicals leak through if they are registered as `Topic` instead.
    //
    // **Nameable from everywhere, unlike the overview's listing.** A bare
    // `recall` shows what works in this room (§7); a *page* answers from
    // anywhere, because a manual you can only read in the right room has a lock
    // on it. Places and spells already have this exemption for the same reason.
    // **Every *word*, not every canonical.** `recall walk` and `recall edit` are
    // questions a player asks with the word they typed, and a table holding only
    // the arcane form answered neither — both fell through to the noun
    // vocabulary and came back with a maze reading. `single_words` includes each
    // canonical, because a canonical is a one-word arcane entry by rule
    // (`canonical_names_are_one_short_word`), so this is a widening rather than a
    // second list to keep in step.
    for (word, _) in crate::parser::single_words() {
        scene = scene.with(NounKind::Command, word);
    }

    // **Every spell, wherever the player is standing.**
    //
    // The same exemption places have, for the same reason. Without it a `.spell`
    // is nameable only from inside `/grimoire`, because everything that is not a
    // place is registered from `cwd` — and `/grimoire` is a protected domain
    // rather than a `Fixture`, so its contents do not reach out the way the
    // dispensary's do. That would make `invoke first_light` work only where
    // there is no laboratory to run it in, which is precisely nowhere useful.
    //
    // It does not loosen §19's *"you can only name what is where you are"*. That
    // rule stops you acting on another **domain** at a distance — the archive's
    // fragments from the laboratory — and a spell is not a domain. It is the
    // book you carry: §8 has the player keeping their spellbook in vim, and the
    // fiction that survives that is something on your person, not a shelf you
    // walk to.
    //
    // `execute::dispatch` records the cost of getting this wrong: a `Script`
    // slot with nothing in scope is *worse than a dead end*, because
    // `bind night_watch` then falls through to `sift` and reports success.
    for node in walk(world, super::filesystem_root(world, cwd.0)) {
        if world.get::<Nameable>(node).map(|n| n.0) == Some(NounKind::Script)
            && let Some(name) = world.get::<Name>(node)
        {
            scene = scene.with(NounKind::Script, &name.0);
        }
    }

    // **Everything the arsenal holds, wherever the player is standing.**
    //
    // The third exemption, and the same one spells have for the same reason:
    // finished work is carried, not shelved. A potion is brewed in the laboratory
    // to be used elsewhere and a scroll is assembled in the archive to be spent
    // elsewhere, so an arsenal nameable only from inside itself would be a room
    // you walk to in order to look at things you cannot then use.
    //
    // **Registered after the spells and before the readings**, which is a
    // decision rather than a detail: `scene_at`'s order *is* the tie-break (§6
    // resolves a tie to whatever was registered first), and putting this ahead of
    // the readings keeps the words a solver names in the same relative order they
    // have always been in.
    //
    // It does not loosen §7. What is still forbidden is acting on another
    // **domain** at a distance — the archive's fragments from the laboratory —
    // which `tower::keep` says at length, and which the arsenal refusing stock at
    // the door is what keeps honest.
    //
    // **And naming is only half.** Three lookups had to learn this too, or a word
    // resolves at full confidence and then reports "no such thing" — see
    // `tower::keep`.
    for node in super::keeping(world) {
        let (Some(name), Some(kind)) = (world.get::<Name>(node), world.get::<Nameable>(node))
        else {
            continue;
        };
        scene = scene.with(kind.0, &name.0);
    }

    // **The maze's readings, always, whether or not a maze is open.**
    //
    // These are the words a solver's `if` names — `if north has 1 or fewer
    // marks` — and they have to resolve at the moment the spell is
    // **cast**, which is exactly when none of them is true of anything.
    // `spell::compile` resolves a condition's names against the room as it is,
    // and nulls the whole condition for a name it cannot place; that guard is
    // right (§19: `has ground-slat` answered "no" for ever) and it cannot tell a
    // not-yet-existing state from a typo. So the scene offers the vocabulary
    // rather than the instances.
    //
    // **Stable is the point.** `bind::stand` recasts a held spell every lap, so
    // a solver must compile identically every time — a vocabulary that came and
    // went with the maze would make a bound spell work on some laps and not
    // others, with nothing on screen saying why.
    // `back` rides with the readings: it is a word a solver names in an `if`,
    // so it has to resolve at *cast* exactly as they do.
    //
    // So do `spoil` and every errand word. An errand is set by a scroll spent
    // *after* the spell was written, so at cast there is never one on — which is
    // exactly the case the chain exists for: a vocabulary that came and went
    // with the world would make `if the stacks has gleaning` compile to a dead
    // branch, and a solver that could not ask which maze it was in is two
    // solvers the player has to choose between by hand.
    for reading in super::maze::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // **And the lens's, for exactly the same reason.** A spell is compiled at
    // cast, when no ward is open — so `closer` and `further` resolve against
    // nothing unless they are here unconditionally. A solver whose every rung
    // compiled to a dead branch is the defect this chain exists to prevent, and
    // it would be silent: the spell casts, runs, and does nothing for ever.
    //
    // **Six words, and they are all deltas.** `aligned`, `astray` and `spent`
    // are published as *counts on the record* — the player reads them, the sheet
    // draws them — and are deliberately absent from this list, so a spell cannot
    // ask for them. That is the whole of §19's correction: a codemaker answers
    // *which way did it move*, and everything else was the orb doing the
    // player's bookkeeping.
    for reading in super::ward::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // **And the sanctum's, for the third time and the same reason.** A course is
    // drawn and finished inside one solve, so at cast there is never one
    // standing — `potency` and `odd` would resolve against nothing and
    // `spell::compile` would null the whole condition, which is the silent
    // failure this chain exists to prevent.
    //
    // **`integrity` is here too, though it is always published.** It reads like
    // the exception and is not: a spell is compiled in `Sim::bare`'s world as
    // readily as in a played one, and a vocabulary that depended on a system
    // having run would make a spell compile differently on the first tick than
    // on the second.
    //
    // **Last, and after the other two**, which is the registration-order rule:
    // §6 resolves a tie to whichever noun came first, so appending leaves every
    // word a maze or ward solver names in exactly the order it has always had.
    for reading in super::pylon::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // **And the menagerie's, for the fourth time and the same reason.** A chant
    // is drawn and finished inside one solve, so at cast there is never one
    // running — `next` and `remaining` would resolve against nothing and
    // `spell::compile` would null the whole condition.
    //
    // **Appended after the sanctum's**, which is the registration-order rule
    // again: §6 resolves a tie to whichever noun came first, so every word an
    // existing solver names keeps exactly the order it has always had.
    for reading in super::chant::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // **And the bailey's, for the fifth time and the same reason.** A siege is
    // begun and finished inside one solve, so at cast there is never one
    // running — `few`, `hurt` and `outnumbered` would resolve against nothing
    // and `spell::compile` would null the whole condition, which is the defect
    // this chain exists to prevent and it is silent: the spell casts, runs, and
    // does nothing for ever.
    //
    // It is not hypothetical here. `besieging` was written before this loop
    // existed and every one of its four questions came back *"that question
    // means nothing"* — the ladder dead, the `repeat until` stopping on its
    // first evaluation, and the spell doing nothing but `defend`.
    //
    // **The three intents are in the list too**, because a decision tree that
    // wants to answer a volley differently from an onslaught asks `if the
    // rampart has volley`, and that is the whole of what telegraphing buys a
    // spell.
    //
    // **Appended after the menagerie's**, which is the registration-order rule
    // for the fifth time: §6 resolves a tie to whichever noun came first, so
    // every word an existing solver names keeps the order it has always had.
    for reading in super::siege::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // ...and the forge's, appended after the bailey's for the sixth time.
    //
    // **Unconditional, like every other domain's.** A spell written before the
    // player has ever opened a lattice must still compile, or the words would
    // only mean something in a room the author was standing in — which is the
    // defect §19 records the bailey shipping with, where `besieging`'s four
    // questions all came back *"that question means nothing"* and the spell ran
    // and did nothing, for ever.
    for reading in super::charm::readings() {
        scene = scene.with(NounKind::Sense, reading);
    }

    // Every place, wherever the player is. Depth-first from the root, children
    // in spawn order.
    for node in walk(world, super::filesystem_root(world, cwd.0)) {
        if world.get::<Nameable>(node).map(|n| n.0) == Some(NounKind::Place) {
            // **A shut room stays a place the parser knows**, deliberately.
            // Dropping it from the scene made `attend archive` fuzz into a
            // numbered prompt offering four *other* rooms — §15's dead end,
            // reached by the one word a new player is most likely to try. Kept
            // nameable, the word reaches `attend`, whose gate answers *"the
            // archive is not yours yet"* — which is what the weave's details
            // panel already says the first station opens, so the name was never
            // the secret. What stays hidden is the room's manual page, above.
            // **A satchel from another room is not offered at all**, and it is
            // the one place this walk is not exhaustive. Every domain has one
            // and they are all called `satchel`, so registering the lot would
            // put six paths in the scene sharing a last segment — and §6's
            // matcher accepts the last segment, which is what makes `attend
            // laboratory` reach `/tower/laboratory`. The word would then resolve
            // to whichever was registered first, from every room in the tower:
            // `queue` put three names in the menagerie's and `survey satchel`
            // read the *laboratory's* and reported it empty, one line apart, on
            // the first line anybody ran.
            //
            // Scoping it here rather than renaming the node keeps the meaning
            // the mechanic wants — *the satchel is the one where you stand* — and
            // it is what makes `build`'s exemption from
            // `every_place_leaf_is_unique` true rather than merely argued: the
            // leaves collide in the tree and never in the scene.
            //
            // It costs naming another room's satchel by its full path, which §7
            // forbids acting on anyway.
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
            // bench rather than somewhere you travel to, and the sage in the
            // mortar is plainly within reach of someone standing in the
            // laboratory. Without this, §10.1's own loop cannot be typed:
            // `move husks from alembic to dispensary` could not name `husks`.
            //
            // This does **not** loosen "you can only name what is where you
            // are". That rule is about acting on another *domain* at a distance
            // — the archive's fragments from the laboratory — and a domain is
            // not a `Fixture`. Phase 10's pane addressing is still what relaxes
            // it in general.
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
/// **Snapshotted, not read live from [`Prose`](crate::content::Prose).** Every
/// `recall_` key is a `NounKind::Topic` in the scene, so reading them each tick
/// meant a prose hot-reload could change what a phrase resolves to — a *decision*
/// — while `Sim::set_prose` promises the opposite. The cost is that adding a new
/// manual subject needs a relaunch rather than a save; the lines themselves still
/// reload, which is what a writer is actually iterating on.
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

    /// The names the scene offers **as things**, rather than as manual subjects.
    ///
    /// Every material has a `recall` page now, and a page is readable from
    /// anywhere — *"a manual you can only read in the right room has a lock on
    /// it"*, which is the rule verb pages already follow. So a reagent's name is
    /// in the scene everywhere as a `Topic`, and §7's scoping is a claim about
    /// the **kind**: in the laboratory `sage` is something you can grind, and in
    /// the archive it is only something you can read about.
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
        // also what gives §19's Phase 10 pane addressing something to be an unlock
        // *from* — acting at a distance has to become possible.
        //
        // Tested with `retort` rather than `clarity`: recipe names are `Topic`s
        // nameable everywhere (§6.1), because a manual is not a thing in a room.
        //
        // The archive's side is its **log**, and it used to be `sigil-iv`. The
        // three sigils were the last of the `divine` that consumed a fragment,
        // and they went when nothing produced them, consumed them or said what
        // one was (§19). A domain log is the honest replacement: every domain
        // has one, it is `NounKind::File` rather than a place, and it is
        // registered from `cwd` like everything else that is not a place — so it
        // makes the same claim about the same rule.
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
        // `husks`. The domain rule above is untouched — a domain is not a
        // fixture.
        //
        // **Asked of `things`, not `names`.** Every material has a manual page,
        // and a page is readable anywhere — so `sage` is in the scene from the
        // archive too, as a `Topic`. What §7 scopes is the *thing*: in the
        // laboratory it is something you can grind, and in the archive it is
        // something you can read about and nothing else.
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
