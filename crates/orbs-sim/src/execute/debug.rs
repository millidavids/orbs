//! A door for a tester, in the builds a tester runs.
//!
//! # Why this is not a verb
//!
//! `debug_spawn` puts reagents on the shelf so a state that takes forty ticks of
//! grinding can be reached in one line. It is deliberately **not** a
//! [`Verb`](crate::parser::Verb), and every reason is about the vocabulary it
//! would otherwise join:
//!
//! - §6.1's tutorial lists the verbs that *work*, and its most important metric
//!   is the dead-end rate. A word that only exists in some builds would be
//!   counted, offered, and then absent from the one the player has.
//! - §6's naming pass exists so no two verbs collide, and `Verb::ALL` is walked
//!   by the completion table, `recall`, and a test that drives every verb through
//!   a real `Sim`. A variant that comes and goes with a `cfg` makes all of those
//!   differ between builds.
//! - It is not fuzzy-matched, because a tester typing `debug_spaw` should be told
//!   so rather than have the tower quietly change under them.
//!
//! So it is matched **exactly**, before the parser sees the line, in the same
//! shape [`SpellWord`](crate::parser::SpellWord) uses and for the same stated
//! reason: *"checked before the fuzzy matcher ever sees the line"*. The parser's
//! vocabulary does not know it exists.
//!
//! # What a release build has
//!
//! No code. The module is `cfg(debug_assertions)`, so the word, the parse, the
//! queue variant and the effect are all absent — a player who types it gets the
//! ordinary *"nothing here answers to that"*, because as far as that build is
//! concerned nothing does.
//!
//! **Four lines of prose do ship**, and that is not an oversight to fix.
//! `prose.toml` is `include_str!`'d whole, so `debug_spawn_done` and its three
//! neighbours are in the binary as text no code can reach. Moving them out to
//! string literals to save four lines would cost rule 6's uniformity — the width
//! lint, the shouting lint and the CP437 lint all read that file — for something
//! nobody can invoke. `the_word_does_nothing_in_a_release_build` is what makes
//! the distinction hold: the strings are dead weight, and the door is shut.
//!
//! **One consequence worth stating**: a session that used it does not replay in a
//! release build. `Submissions` records the typed line like any other, and a
//! release build reading it back resolves nothing. Debug sessions replay in debug
//! builds, which is where they were recorded.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, Recipes};
use crate::session::Scrollback;
use crate::tower::{self, Store};

/// The word, matched exactly and never advertised.
///
/// Underscored on purpose: no verb in §6's vocabulary has an underscore, so this
/// cannot be reached by a typo of a real word, and it reads as a tool rather than
/// as part of the game.
pub const SPAWN: &str = "debug_spawn";

/// `debug_learn [name]` — hand the player a recipe the lens would have found.
///
/// **A state worth testing costs six broken wards to reach**, which is about
/// four hundred ticks of pressing, and the roll is a roll — so a See-it line for
/// discovery would otherwise be *"scry until it happens"*. This is the same
/// argument `debug_spawn` was built on, one domain over.
///
/// Bare, it learns the next unfound secret in the file's own order, which is the
/// order the lens itself reveals them in. Named, it learns that one — and refuses
/// a name that is not a secret, because learning something already known is a
/// state the game cannot reach and therefore not one worth testing from.
pub const LEARN: &str = "debug_learn";

/// What a `debug_learn` line asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lesson {
    /// The secret to learn, or `None` for the next one.
    pub name: Option<String>,
}

/// `debug_ward` — make the open ward's answer whatever the aperture holds.
///
/// The next `probe` breaks the seal. Everything downstream runs as it would
/// have: the yield scales with presses spent, the spill is drawn from the same
/// stream, the discovery is the same roll. What is skipped is the deduction,
/// which is not what a See-it line for the *spill* is looking at.
pub const WARD: &str = "debug_ward";

/// Whether this line is a `debug_ward`.
#[must_use]
pub fn giveaway(line: &str) -> bool {
    line.trim() == WARD
}

/// `debug_course` — leave the standing course one haul from finished.
///
/// A course is `2^n - 1` hauls and the tallest is 127 of them, so a See-it line
/// for the *completion* — the walls going back up, the experience, the record —
/// would otherwise open with a hundred commands. Everything downstream runs as
/// it would have: the last haul is a real haul through the real verb, and what
/// it triggers is the real `finish`.
///
/// It skips the puzzle, which is not what a See-it line for the completion is
/// looking at. `debug_ward` is the same word one room over.
pub const COURSE: &str = "debug_course";

/// Whether this line is a `debug_course`.
#[must_use]
pub fn shortcut(line: &str) -> bool {
    line.trim() == COURSE
}

/// `debug_swap` — substitute a reagent where the player is standing.
///
/// §8.1's world surface arrives on a 1200-tick roll, which is twenty minutes of
/// waiting for a See-it line. Same argument as every other word in this file.
pub const SWAP: &str = "debug_swap";

/// Whether this line is a `debug_swap`.
#[must_use]
pub fn swapping(line: &str) -> bool {
    line.trim() == SWAP
}

/// `debug_take <id>` — hold a mastery node without earning it.
///
/// **Three distillations is 24 experience**, which is about two hundred ticks of
/// setup before a See-it line for the *gated* thing can begin — and §8's channel
/// puts three words behind two of those nodes, so every line about `queue`,
/// `pull` or `alongside` would have opened with the laboratory. Same argument as
/// `debug_spawn` and `debug_learn`: the state is worth testing and the road to
/// it is not what the line is looking at.
///
/// **It skips the earning and nothing else.** The grant is the real grant
/// through `Taken::hold`, so `spell::budget`, `is_gated` and
/// `compile::check_learned` all see exactly what a played tower would — which is
/// what makes it a shortcut rather than a second implementation.
///
/// What it does *not* skip is whether the node is real: an id nothing grants is
/// refused, because a tower holding a marker is a state the game cannot reach
/// and therefore not one worth testing from. That is `debug_learn`'s rule about
/// secrets, one screen over.
pub const TAKE: &str = "debug_take";

/// Read a `debug_take` line, if that is what this is.
///
/// `None` for anything else; `Some(None)` for a bare `debug_take`, which lists
/// what there is to take — `debug_spawn`'s shape, and for its reason: a tester
/// who has to read `progression.toml` to find an id is a tester the tool is
/// failing.
#[must_use]
pub fn taking(line: &str) -> Option<Option<String>> {
    let rest = line.trim().strip_prefix(TAKE)?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let id = rest.trim();
    Some((!id.is_empty()).then(|| id.to_lowercase()))
}

/// Read a `debug_learn` line, if that is what this is.
#[must_use]
pub fn lesson(line: &str) -> Option<Lesson> {
    let rest = line.trim().strip_prefix(LEARN)?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let name = rest.trim();
    Some(Lesson {
        name: (!name.is_empty()).then(|| name.to_owned()),
    })
}

/// What a tester asked for, if this line is a spawn order at all.
///
/// `debug_spawn ground-sage` is one; `debug_spawn ground-sage 5` is five. A count
/// that is not a number is **refused rather than defaulted** — `debug_spawn sage
/// lots` meaning one sage is the kind of quiet reinterpretation this whole
/// subsystem spent a week removing.
///
/// # And optionally where
///
/// `debug_spawn fragment 4 lectern` puts them in the lectern. The destination
/// was added when the archive gained an instrument that *consumes* stock: there
/// is exactly one `Store` in the tower and it is in the laboratory, so §7 made
/// every archive state unreachable from this tool — four fragments on a lectern
/// could be reached by walking the stacks four times and by nothing else, which is the
/// forty ticks of grinding this exists to skip, several hundred times over.
///
/// **Any fixture, from anywhere.** The same argument the shelf lookup makes:
/// requiring the tester to be standing in the right room first puts back the
/// walking the tool is for.
#[must_use]
pub fn order(line: &str) -> Option<Order> {
    let rest = line.trim().strip_prefix(SPAWN)?;
    // `debug_spawnage` is not this word. Anything after it must be whitespace.
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let mut words = rest.split_whitespace();
    let Some(name) = words.next() else {
        // Bare, which is how you ask what there is.
        return Some(Order::List);
    };
    let count = match words.next() {
        None => 1,
        // **Zero is not a count, it is a request for nothing.** `stock::give`
        // would spawn a node holding `Counted(0)`, and `stock::take` documents
        // the invariant that breaks: *"a pile that reaches zero is despawned, so
        // *is there any* stays the same question it always was — a node
        // existing."* A nought-node answers `has ash` yes for ever while nothing
        // can ever be taken from it, which is precisely a world state the game
        // cannot otherwise reach — the thing this tool refuses to create.
        Some(count) => count.parse().ok().filter(|count| *count > 0)?,
    };
    // **A destination is not a number**, and refusing one here rather than at
    // the shelf is what keeps the old guarantee. `debug_spawn sage 2 3` was
    // refused outright before there was a third slot; read as a *place* called
    // `3` it would find nothing and say so, which is a worse answer to what is
    // almost certainly a mistyped count.
    let into = match words.next() {
        None => None,
        Some(word) if word.parse::<u32>().is_ok() => return None,
        Some(word) => Some(word.to_owned()),
    };
    // **Every word read, or none of them** — the rule the question grammar next
    // door is built on, and it belongs here for the same reason: `debug_spawn
    // sage 2 lectern spare` naming one destination is a line half-obeyed.
    if words.next().is_some() {
        return None;
    }
    Some(Order::Spawn {
        name: name.to_owned(),
        count,
        into,
    })
}

/// What the word was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    /// Put `count` of `name` on the shelf, or in the fixture named by `into`.
    Spawn {
        /// What to make.
        name: String,
        /// How many.
        count: u32,
        /// Which fixture to put them in. The shelf if unsaid.
        into: Option<String>,
    },
    /// Say what can be made.
    List,
}

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
    // **Known names only.** Spawning `xyzzy` would put a node in the tower that
    // no recipe, no instrument and no `survey` row knows what to do with — a
    // world state the game cannot otherwise reach, which is the opposite of what
    // a testing tool is for.
    let Some(known) = known(world)
        .into_iter()
        .find(|word| word.eq_ignore_ascii_case(name))
    else {
        say(world, "debug_spawn_unknown", name, Role::Danger);
        return;
    };

    // **Where the thing belongs, by default.** Not the floor, not wherever the
    // player happens to be, and — since `tower::home` — no longer *always* the
    // laboratory's shelf either: sage lands on the dispensary, a fragment in the
    // archive's cabinet, a potion in the arsenal, because that is where the game
    // itself would have left each of them.
    //
    // That is the whole point of the word. A tester types `debug_spawn <thing>`
    // and the thing is in the room it is used in, reachable, ready to be `move`d
    // or ground or wielded — with no third argument and no knowledge of the
    // tower's layout. A named destination overrides it for the cases where the
    // point *is* the layout, and obeys the same rule about what a shelf is (see
    // [`shelf`]).
    let into = match into {
        Some(into) => shelf(world, into),
        None => tower::home(world, &known).or_else(|| dispensary(world)),
    };
    let Some(into) = into else {
        say(world, "debug_spawn_nowhere", &known, Role::Danger);
        return;
    };
    // **The kind the laboratory would have given it.** A potion is an `Essence`
    // and everything else is crafting stock (`work::produce`), so spawning
    // `clarity` as a reagent would put a node in the tower with the right name
    // and the wrong kind — a thing no `distil` could have made and no slot will
    // take. That is precisely the state this refuses to create, arriving through
    // the check meant to prevent it.
    let kind = world.resource::<Recipes>().kind_of(&known);

    // **And the arsenal's door holds for a tester too.** It takes finished work
    // only, so a reagent standing in it is a state no `move` could produce —
    // the same objection as an unknown name, a nought-count and a wrong kind,
    // which this word already refuses three times over. A testing tool that can
    // build impossible worlds is a tool whose bug reports have to be checked
    // against the tool first.
    //
    // Asked of the **kind**, like the door itself: `tower::admits` owns the rule
    // and this asks it rather than restating it, because two expressions of one
    // rule is how they come to disagree (§19).
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
    say(world, "debug_spawn_done", &known, Role::Success);
}

/// The tower's shelf, wherever the player is standing.
///
/// **Walked from the root rather than read out of the current room.** A tester
/// setting up a state should not have to be standing in the right place first —
/// and the point of the tool is to skip the forty ticks of walking and grinding
/// that reaching the state would otherwise cost, which a `cwd` check would put
/// straight back.
///
/// `None` only if the tower has no `Store` at all, which `build::raise` always
/// makes. It reports rather than panicking: a missing dispensary is a build bug,
/// and a sentence naming it is more use to whoever caused it than a stack trace
/// from a debug command.
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
/// **Somewhere a `move` could reach, not any node.** Stock lives inside
/// instruments and stores; a pile standing on an ordinary domain node is one
/// `move` cannot pick up and `survey` reports oddly, which is again a state the
/// game cannot otherwise reach. Naming `laboratory` therefore finds nothing and
/// says so, rather than half-working.
///
/// **The arsenal is the exception, because the game makes it one.** It is a
/// domain rather than a `Fixture`, so the fixture test alone refused it — and
/// that put every arsenal state back out of a tester's reach, which is the exact
/// gap that made this function take a name in the first place. What decides the
/// question is not what shape the node is but whether `pipeline::reachable` can
/// see into it, and it can see into exactly two things: a fixture where you
/// stand, and the arsenal from anywhere (`tower::keep`).
fn shelf(world: &World, named: &str) -> Option<Entity> {
    let leaf = crate::parser::leaf(named);
    let mut stack = vec![tower::root(world)];
    while let Some(node) = stack.pop() {
        // **A way is a fixture and is not a shelf**, which is the one case the
        // fixture test gets wrong on its own. `north` and its three siblings
        // carry `Fixture` so the maze can publish readings into them — and
        // `research::refresh` despawns *everything* in a way on the step after,
        // so a reagent put there is a pile that vanishes with no line saying so.
        // A tester chasing that would be chasing the tool.
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
/// [`Recipes::substances`] — this was the same union written out here, and it
/// stopped being debug-only when the parser needed it too. The parser's use is
/// the load-bearing one: a word that is a real substance must not be fuzzed into
/// a *different* real substance (§19).
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

    #[test]
    fn the_word_is_matched_exactly() {
        assert_eq!(order("debug_spawn"), Some(Order::List));
        assert_eq!(
            order("debug_spawn sage"),
            Some(Order::Spawn {
                name: "sage".to_owned(),
                count: 1,
                into: None,
            }),
        );
        assert_eq!(
            order("  debug_spawn ground-sage 5  "),
            Some(Order::Spawn {
                name: "ground-sage".to_owned(),
                count: 5,
                into: None,
            }),
        );
        // A destination needs a count in front of it, because the count is
        // positional and always has been. `debug_spawn fragment lectern` is
        // therefore refused rather than read as one fragment somewhere — the
        // quiet reinterpretation this whole word refuses.
        assert_eq!(
            order("debug_spawn fragment 4 lectern"),
            Some(Order::Spawn {
                name: "fragment".to_owned(),
                count: 4,
                into: Some("lectern".to_owned()),
            }),
        );
        assert_eq!(order("debug_spawn fragment lectern"), None);

        // Not this word, and not guessed at: a tester who mistypes it should be
        // told by the ordinary parser rather than have the tower change.
        for other in [
            "debug_spawnage",
            "debug spawn sage",
            "spawn sage",
            "debug_spaw sage",
            "grind sage",
        ] {
            assert_eq!(order(other), None, "{other:?} was read as a spawn");
        }

        // A count that is not a count. Defaulting to one would be the quiet
        // reinterpretation this subsystem exists to refuse.
        assert_eq!(order("debug_spawn sage lots"), None);
        assert_eq!(order("debug_spawn sage 2 3"), None);
    }

    /// What the place called `place` holds after `line`, by name and count.
    ///
    /// **One place, not the whole tower.** Sweeping the tree finds the endless
    /// `sage` on the dispensary's shelf whatever the spawn did, so *"is there
    /// sage"* is true before the command runs — a test that asks it is asserting
    /// nothing, which is the shape §19 records three tests having.
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
        // **The list, the door and the layout all have to agree.** A word printed
        // by a bare `debug_spawn` and then refused — or dropped in a room the
        // tester is not in and cannot reach — is worse than a shorter list: the
        // tester believes the tool and looks for the bug in the game.
        //
        // Driven per name and against `tower::home` rather than a room written
        // down here, because a room written down here is the hand-kept list the
        // rule exists to replace. What this pins is that the tool obeys the rule,
        // and `every_material_has_a_home_a_move_can_reach` pins that the rule has
        // an answer for everything.
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
        // **The other direction, and it is the one that rots.** The list above
        // is derived from the *recipes*, so a material authored in
        // `materials.toml` that no recipe names would have a colour, a manual
        // route and no way for a tester to hold one — and nothing would say so,
        // because both files parse perfectly.
        //
        // It is also the shape of the ask this test was written for: *"make sure
        // all new items are in `debug_spawn`"* is a promise that has to keep
        // being true, and a promise kept by hand is one kept until somebody is
        // busy.
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

        // And the door holds for a tester too. Stock standing in the arsenal is
        // a state no `move` could produce, which is the same objection as an
        // unknown name and a wrong kind — both of which this word already
        // refuses. A tool that can build impossible worlds makes every bug report
        // start by checking the tool.
        let kept = held_by("debug_spawn sage 1 arsenal", tower::ARSENAL);
        assert!(
            !kept.iter().any(|(name, _)| name == "sage"),
            "the arsenal took a reagent from the tool: {kept:?}",
        );
    }

    #[test]
    fn a_way_is_not_a_shelf() {
        // `north` and its three siblings carry `Fixture` so the maze can publish
        // readings into them — and `research::refresh` despawns *everything* in a
        // way on the step after, so a reagent put there is a pile that vanishes
        // with no line saying so. The fixture test alone said yes.
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
