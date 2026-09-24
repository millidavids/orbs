//! A door for a tester, in the builds a tester runs.
//!
//! `debug_spawn` puts reagents on the shelf so a state costing forty ticks of
//! grinding is one line away. It is deliberately not a
//! [`Verb`](crate::parser::Verb): §6.1's tutorial counts the verbs that *work*
//! and would offer one absent from the player's build, `Verb::ALL` is walked by
//! the completion table, `recall` and a test that drives every verb, and a
//! tester typing `debug_spaw` should be told so rather than have the tower
//! change under them. So it is matched exactly, before the parser sees the line,
//! in [`SpellWord`](crate::parser::SpellWord)'s shape.
//!
//! A release build has no code for it — the module is `cfg(debug_assertions)`,
//! so a player who types it gets the ordinary *"nothing here answers to that"*.
//! Four lines of prose do ship, because `prose.toml` is `include_str!`'d whole
//! and moving them to string literals would cost rule 6's uniformity (the width,
//! shouting and CP437 lints all read that file) for something nobody can invoke.
//!
//! One consequence: a session that used it does not replay in a release build.
//! Debug sessions replay in debug builds, which is where they were recorded.

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
/// The state costs six broken wards — four hundred ticks of pressing — and the
/// roll is a roll, so a See-it line for discovery would otherwise be *"scry
/// until it happens"*. `debug_spawn`'s argument, one domain over.
///
/// Bare, it learns the next unfound secret in the file's order, which is the
/// order the lens reveals them in. Named, it learns that one, and refuses a name
/// that is not a secret — the game cannot reach that state either.
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
/// A course is `2^n - 1` hauls and the tallest is 127, so a See-it line for the
/// *completion* would open with a hundred commands. Everything downstream runs
/// as it would have — the last haul is real and triggers the real `finish`. It
/// skips the puzzle, which is not what the line is looking at.
pub const COURSE: &str = "debug_course";

/// Whether this line is a `debug_course`.
#[must_use]
pub fn shortcut(line: &str) -> bool {
    line.trim() == COURSE
}

/// `debug_circle` — limn the circle so the waiting beast's next call holds it.
///
/// The temper is not touched, unlike `debug_ward`'s rewritten answer: a beast's
/// temper is a puzzle the table drew, and rewriting it could leave one no circle
/// answers. This limns the glyphs to a solution instead, and republishes because
/// their readings are what a spell reads next. The next `summon` is a real call
/// through the real verb; only the reasoning is skipped.
pub const CIRCLE: &str = "debug_circle";

/// Whether this line is a `debug_circle`.
#[must_use]
pub fn beckoned(line: &str) -> bool {
    line.trim() == CIRCLE
}

/// `debug_siege` — leave the standing siege one round from won.
///
/// A siege is a dozen rounds and a `hold` is a whole turn of decisions, so a
/// See-it line for the *ending* would open with a dozen commands and depend on
/// the dice. The last round is a real `hold` with real rolls, triggering the
/// real `settle`. `debug_course` one room over, and it republishes for the same
/// reason: a stale board is what a decision tree would read.
pub const SIEGE: &str = "debug_siege";

/// Whether this line is a `debug_siege`.
#[must_use]
pub fn beleaguered(line: &str) -> bool {
    line.trim() == SIEGE
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
/// Three distillations is 24 experience — two hundred ticks of setup before a
/// See-it line for the *gated* thing can begin, and §8's channel puts three
/// words behind two of those nodes. `debug_spawn`'s argument.
///
/// It skips the earning and nothing else: the grant goes through `Taken::hold`,
/// so `spell::budget`, `is_gated` and `compile::check_learned` see what a played
/// tower would. An id nothing grants is refused — a tower holding a marker is a
/// state the game cannot reach, which is `debug_learn`'s rule about secrets.
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

/// `debug_reach <id>` — reach a mastery station without doing its deed.
///
/// `debug_take`'s argument, one track over: forty potions is an afternoon of
/// the laboratory before a See-it line about what the sixth station opens can
/// begin. It reaches every earlier station on the same line too, because a line
/// is walked in order and a tower with its third station reached and its first
/// not is a state the game cannot reach.
///
/// What it does not skip is what reaching *does*: the grant is the real grant
/// through `mastery::reach`, so what the station opens is opened and said
/// exactly as a played tower would.
pub const REACH: &str = "debug_reach";

/// Read a `debug_reach` line, if that is what this is.
///
/// `None` for anything else; `Some(None)` for a bare `debug_reach`, which lists
/// the stations — `debug_take`'s shape, for its reason.
#[must_use]
pub fn reaching(line: &str) -> Option<Option<String>> {
    let rest = line.trim().strip_prefix(REACH)?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let id = rest.trim();
    Some((!id.is_empty()).then(|| id.to_lowercase()))
}

/// A tester's door to a standing, for the ranks.
///
/// It *sets* the total rather than adding, which lets one word go both ways: a
/// rank is lost by falling back through it, and reaching that state otherwise
/// means losing a siege on purpose. A negative argument is refused with
/// everything else that is not a number — see [`Asking`], which exists because
/// folding that case into "nought" wiped the state a tester was building.
///
/// The ranks run from 25 renown to fifteen thousand, every one behind an hour or
/// a day of play, so without this the titles are a surface nobody can look at.
pub const RENOWN: &str = "debug_renown";

/// What a `debug_renown` line asked for.
///
/// Three states, because two silently destroyed the thing being set up: the
/// argument was `Option<u64>` with an unreadable word folded into `None`, so
/// `debug_renown magisterr` *zeroed* the tower's renown and answered
/// `renown is 0`. `debug_take` and `debug_reach` name the alternatives and
/// mutate nothing, which is this shape.
///
/// Named `Asking` rather than `Standing`, because `tower::Standing` already
/// means what a ley node *is*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asking {
    /// A bare `debug_renown`: say where the tower stands, change nothing.
    Where,
    /// A number: set the total to it.
    Set(u64),
    /// A word that is not a number: refuse, and change nothing.
    Unreadable,
}

/// Read a `debug_renown` line, if that is what this is.
///
/// `None` for anything else — everything that *is* one is an [`Asking`],
/// including the unreadable argument, which is answered here rather than falling
/// through to the parser and fuzzing into something unrelated.
#[must_use]
pub fn standing(line: &str) -> Option<Asking> {
    let rest = line.trim().strip_prefix(RENOWN)?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let total = rest.trim();
    if total.is_empty() {
        return Some(Asking::Where);
    }
    Some(total.parse().map_or(Asking::Unreadable, Asking::Set))
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
/// `debug_spawn ground-sage` is one; `debug_spawn ground-sage 5` is five. A
/// count that is not a number is refused rather than defaulted — `debug_spawn
/// sage lots` meaning one sage is the quiet reinterpretation this subsystem
/// spent a week removing.
///
/// `debug_spawn fragment 4 lectern` says where. The destination was added when
/// the archive gained an instrument that *consumes* stock: the tower's one
/// `Store` is in the laboratory, so §7 made every archive state unreachable from
/// this tool. Any fixture, from anywhere — requiring the tester to stand in the
/// right room first puts back the walking the tool is for.
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
        // Zero is not a count, it is a request for nothing. `stock::give` would
        // spawn a `Counted(0)` node, breaking `stock::take`'s invariant that a
        // pile reaching zero is despawned — so it would answer `has ash` yes for
        // ever while nothing could be taken from it.
        Some(count) => count.parse().ok().filter(|count| *count > 0)?,
    };
    // A destination is not a number, and refusing one here rather than at the
    // shelf keeps the old guarantee: `debug_spawn sage 2 3` read as a *place*
    // called `3` finds nothing and says so, which is a worse answer to what is
    // almost certainly a mistyped count.
    let into = match words.next() {
        None => None,
        Some(word) if word.parse::<u32>().is_ok() => return None,
        Some(word) => Some(word.to_owned()),
    };
    // Every word read, or none of them — the question grammar's rule, and for
    // its reason: `debug_spawn sage 2 lectern spare` naming one destination is a
    // line half-obeyed.
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
    // Known names only: `xyzzy` would put a node in the tower no recipe, no
    // instrument and no `survey` row knows what to do with.
    let Some(known) = known(world)
        .into_iter()
        .find(|word| word.eq_ignore_ascii_case(name))
    else {
        say(world, "debug_spawn_unknown", name, Role::Danger);
        return;
    };

    // Where the thing belongs, by default: sage on the dispensary, a fragment in
    // the archive's cabinet, a potion in the arsenal, because that is where the
    // game would have left each of them. A tester types `debug_spawn <thing>`
    // and it is in the room it is used in, with no knowledge of the layout. A
    // named destination overrides it where the point *is* the layout.
    let into = match into {
        Some(into) => shelf(world, into),
        None => tower::home(world, &known).or_else(|| dispensary(world)),
    };
    let Some(into) = into else {
        say(world, "debug_spawn_nowhere", &known, Role::Danger);
        return;
    };
    // The kind the laboratory would have given it. Spawning `clarity` as a
    // reagent puts a node in the tower with the right name and the wrong kind —
    // one no `distil` could have made and no slot will take.
    let kind = world.resource::<Recipes>().kind_of(&known);

    // The arsenal's door holds for a tester too: it takes finished work only, so
    // a reagent in it is a state no `move` could produce. A tool that can build
    // impossible worlds is one whose bug reports get checked against the tool.
    // Asked of the *kind*, like the door itself, so there is one rule (§19).
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
    // Stamped to a full store, not to one making. A store's standing is a rate
    // recorded at `tally::done`, which this word never goes through, so without
    // a stamp every spawned thing arrives `spent`. One stamp is not enough
    // either: a rate of one is `thin`, so a spawned troop brought half the
    // bodies it should and a garrison-grant test failed on freshness instead.
    // A test that wants a thin store ages it with `meditate`.
    for _ in 0..tower::FRESH_AT {
        tower::made(world, &known);
    }
    say(world, "debug_spawn_done", &known, Role::Success);
}

/// The tower's shelf, wherever the player is standing.
///
/// Walked from the root rather than read out of the current room: a `cwd` check
/// would put back the walking this tool exists to skip.
///
/// `None` only if the tower has no `Store`, which `build::raise` always makes.
/// It reports rather than panicking — a missing dispensary is a build bug, and a
/// sentence naming it beats a stack trace from a debug command.
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
/// Somewhere a `move` could reach, not any node: stock lives inside instruments
/// and stores, and a pile on an ordinary domain node is one `move` cannot pick
/// up. Naming `laboratory` finds nothing and says so rather than half-working.
///
/// The arsenal is the exception because the game makes it one — a domain rather
/// than a `Fixture`, so the fixture test alone refused it and put every arsenal
/// state out of reach. What decides it is whether `pipeline::reachable` can see
/// into the node: a fixture where you stand, or the arsenal from anywhere.
fn shelf(world: &World, named: &str) -> Option<Entity> {
    let leaf = crate::parser::leaf(named);
    let mut stack = vec![tower::root(world)];
    while let Some(node) = stack.pop() {
        // A way is a fixture and is not a shelf — the one case the fixture test
        // gets wrong alone. `north` and its siblings carry `Fixture` so the maze
        // can publish readings into them, and `research::refresh` despawns
        // everything in a way on the next step, so a reagent put there vanishes
        // with no line saying so.
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
    /// One place, not the whole tower: sweeping the tree finds the endless
    /// `sage` on the dispensary whatever the spawn did, so *"is there sage"* is
    /// true before the command runs and the test asserts nothing.
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
        // The list, the door and the layout all have to agree: a word printed by
        // a bare `debug_spawn` and then refused is worse than a shorter list,
        // because the tester believes the tool and looks for the bug in the
        // game. Driven against `tower::home` rather than a room written down
        // here, which would be the hand-kept list the rule replaces.
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
        // The other direction, and the one that rots: the list above is derived
        // from the *recipes*, so a material in `materials.toml` that no recipe
        // names has a colour, a manual route and no way for a tester to hold
        // one — and both files parse perfectly, so nothing says so.
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

        // And the door holds for a tester too: stock in the arsenal is a state no
        // `move` could produce.
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
