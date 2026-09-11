//! What a siege does to the tower behind you (DESIGN.md §5.1, §8.1).
//!
//! §5.1 splits aberrations in two, and the split is the design:
//!
//! | | **Nuisance (idle)** | **Adversarial (siege)** |
//! |---|---|---|
//! | Source | Environmental, random | An intelligent enemy |
//! | Surfaces | Environmental only — never scripts, schedules or logs | All four (§8.1) |
//! | Response | Repair occupies a pane | Diagnose and repair under pressure |
//!
//! **Adversarial aberrations are siege-only**, which is what makes Phase A
//! genuinely safe (pillar 4) and what makes the four-surface model the siege's
//! signature rather than the tower's. `sabotage::drift` and
//! `sabotage::substitution` are the calm layer and are untouched by this module;
//! nothing here runs unless a siege is being fought.
//!
//! # This is the premise, and it is the last clause of it
//!
//! CLAUDE.md: *"you progress by writing scripts that teach the orb to do your
//! work. **Sieges then test everything you automated — because the enemy attacks
//! the automation.**"* Six domains produce and, until now, nothing consumed. The
//! two surfaces this module adds are the ones that reach a *script*: its text,
//! and its clock.
//!
//! So the intervention loop is **debugging**, which is what makes the scrying
//! domain load-bearing rather than flavour — and why the §8.1 audit was built
//! first. `verify` is the diagnosis, and *which surface do I inspect first* is
//! the binding constraint §5.1 names by name.
//!
//! # It is telegraphed, like everything else the enemy does
//!
//! A round announces its intent before it acts, and so does this: the round that
//! sabotages *says so on the transcript* without saying **what** it touched.
//! That is the whole shape of the puzzle — you are told there is a lie and you
//! spend a look finding it, which is the decision the cooldown prices.
//!
//! # One draw, unconditionally, before anything can return
//!
//! CLAUDE.md's determinism rule. The roll happens on every resolved round
//! whether or not a siege is running hard enough to sabotage, and the *target*
//! is chosen from the quotient of the roll that already fired — never from a
//! second draw taken only when the first one succeeded, which is §19's `drift`
//! defect.

use bevy_ecs::prelude::*;

use super::node::{Held, Name};
use super::spell::Bound;
use crate::rng::{RngStream, Rngs};

/// How often a round sabotages something, as one chance in this many.
///
/// **Not every round, deliberately.** §5.1 caps aberration arrival *"so repairs
/// cannot spiral"*, and a siege that lied on every round would leave a player
/// doing nothing but auditing — the failure mode where the *fight* becomes the
/// thing you have no time for. One round in three is often enough that the
/// player has to keep an eye on the tower and rare enough that they can still
/// fight the siege in front of them.
pub const ODDS: u32 = 3;

/// A bound spell whose schedule an enemy has retimed.
///
/// **The subtlest of the four surfaces**, and the reason it is worth having: a
/// retimed spell's *text* is perfect, so `peruse` shows exactly what the player
/// wrote and reading it harder never finds anything. Only `verify` does.
#[derive(Component, Debug, Clone)]
pub struct Retimed {
    /// How many extra ticks each of its steps now costs.
    pub drag: u64,
}

/// A spell whose text an enemy has rewritten.
///
/// **The true lines are kept**, which is `Substituted`'s rule and for its
/// reason: a corruption that destroyed the player's own writing would be theft
/// rather than sabotage, and §5.1 keeps *misdirection* as the thing scrying
/// exists to see through. It also makes the repair a repair.
#[derive(Component, Debug, Clone)]
pub struct Rewritten {
    /// What the player actually wrote.
    pub was: Vec<String>,
    /// How the orb read it — what ran before the strike, and what a repair puts
    /// back beside [`was`](Self::was).
    pub read: super::Read,
}

/// Which of the two adversarial surfaces a round reached, if either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reached {
    /// A spell's text was rewritten.
    Script,
    /// A bound spell's clock was dragged.
    Clock,
}

/// Roll for sabotage, and do it. Called once per resolved round.
///
/// **The draw is unconditional and happens first**, before any check that could
/// return — CLAUDE.md's rule, and the one that keeps a replay valid whatever the
/// tower happens to contain. The target is then chosen from the *quotient* of
/// that same roll, so no second draw is ever taken.
pub fn strike(world: &mut World) -> Option<Reached> {
    let roll: u32 = {
        let mut rngs = world.resource_mut::<Rngs>();
        // A wide draw, so the quotient below has room to choose a target from.
        rand::Rng::random_range(rngs.stream(RngStream::Siege), 0..u32::from(u16::MAX))
    };

    if !roll.is_multiple_of(ODDS) {
        return None;
    }
    // **The quotient chooses, never a second draw.** A draw taken only when the
    // first one fired would make the stream depend on its own results, which is
    // the defect §19 records in `drift` and the reason `substitution` picks its
    // pile this way too.
    let choice = roll / ODDS;

    // **Sorted by name, never by entity.** An ECS query has no order worth
    // relying on, and a siege whose sabotage moved between two runs of one seed
    // would fail every replay test in the file.
    let mut scripts: Vec<(Entity, String)> = world
        .query::<(Entity, &Name, &Held)>()
        .iter(world)
        .map(|(entity, name, _)| (entity, name.0.clone()))
        .collect();
    scripts.sort_by(|a, b| a.1.cmp(&b.1));
    scripts.retain(|(entity, _)| world.get::<Rewritten>(*entity).is_none());

    let mut bound: Vec<(Entity, String)> = world
        .query::<(Entity, &Name, &Bound)>()
        .iter(world)
        .map(|(entity, name, _)| (entity, name.0.clone()))
        .collect();
    bound.sort_by(|a, b| a.1.cmp(&b.1));
    bound.retain(|(entity, _)| world.get::<Retimed>(*entity).is_none());

    // **The clock first when there is anything bound**, because that is the
    // surface that punishes automation — pillar 3, *"automation is progression,
    // and automation is attack surface"*. A tower with nothing bound has nothing
    // to retime, and falls back to the text.
    if !bound.is_empty() {
        let (node, _) = bound[choice as usize % bound.len()];
        // **A `shielded` spell is struck and holds.** Skipped *after* the
        // choice, never filtered out of `bound` — filtering would change which
        // spell the same roll reaches and move every seed's world; this spends
        // the enemy's turn instead and leaves the arithmetic exactly as it was.
        if held(world, node) {
            return None;
        }
        // The drag is derived from the same roll for the third time, so this
        // whole function takes exactly one draw.
        let drag = u64::from(choice % 3) + 1;
        world.entity_mut(node).insert(Retimed { drag });
        super::poison(world, node);
        return Some(Reached::Clock);
    }

    if !scripts.is_empty() {
        let (node, _) = scripts[choice as usize % scripts.len()];
        if held(world, node) {
            return None;
        }
        rewrite(world, node);
        return Some(Reached::Script);
    }

    None
}

/// Whether a live `shielded` charm is holding this node.
///
/// **The clock, never `With<Charmed>`.** A charm is an interval with no expiry
/// system, so a lapsed one is still a present component — a component test would
/// shield a spell for ever after its first charm, which is the one shape this
/// whole feature must not have.
fn held(world: &World, node: Entity) -> bool {
    super::charmed(world, node, super::charm::Kind::Shielded)
}

/// Corrupt one line of a spell, keeping what it said.
///
/// **One line, not the file.** A spell rewritten wholesale is a spell the player
/// throws away and writes again, which is not a puzzle; one line changed is a
/// thing they have to *find*, which is what `peruse` and `interpret` are for.
fn rewrite(world: &mut World, node: Entity) {
    let Some(held) = world.get::<Held>(node).cloned() else {
        return;
    };
    if held.0.is_empty() {
        return;
    }
    // **Only lines with something to misdirect**, walked from the tick's
    // position. No draw here: the tick is already part of the replayed state,
    // and a second draw would be the defect this module's header warns about.
    let candidates: Vec<usize> = (0..held.0.len())
        .filter(|at| corruptible(&held.0[*at]))
        .collect();
    if candidates.is_empty() {
        return;
    }
    let from = usize::try_from(world.resource::<crate::tick::Tick>().get()).unwrap_or(0);
    let at = candidates[from % candidates.len()];

    let mut lines = held.0.clone();
    // **The last *word*, not the line**, and this is the whole of what makes it
    // sabotage rather than vandalism. `claimed` appends a sigil so the resolver
    // will *not* fold the name back — applied to a whole line it produced
    // `repeat 3-` and `end-`, which the parser cannot read at all: the line
    // faults at `Role::Danger`, latches `‼` on the rail, and an unclosed block
    // stops the whole spell compiling. This module's own header says *"a line
    // the orb cannot read at all would fault loudly and give the game away"*,
    // and the first version did exactly that.
    //
    // A corrupted **argument** is the shape wanted: `grind sage` becomes `grind
    // sage-`, which parses, runs, and quietly does nothing — the misdirection
    // §5.1 keeps scrying for.
    lines[at] = corrupt(&lines[at]);

    // **Both, and the corrupted line copied verbatim into the reading.**
    // `spell::compile` reads `Read`, so corrupting only `Held` would leave the
    // sabotage visible to `peruse` and invisible to the runner — the player
    // would see a broken line while the orb ran the clean one, and `purge` would
    // repair nothing. That is CLAUDE.md's premise clause switched off: *sieges
    // test everything you automated because the enemy attacks the automation.*
    //
    // Copied rather than re-read, because there is no reader here and could not
    // be: this runs inside `step`, where rule 1 bars the model and rule 3 bars
    // anything that would make a replay depend on it. A verbatim copy is exactly
    // what the game did before readings existed, when `Held` was what compiled.
    //
    // **That one line of the reading, and no other.** The first version wrote
    // the whole corrupted text into `Read`, which threw away the orb's reading
    // of every line the enemy never touched: a spell the reader had made
    // runnable went back to its player's loose words, most of which the orb
    // cannot run. The reading from before the strike is what `Rewritten` keeps,
    // so a repair puts back what ran rather than only what was typed.
    //
    // The reader's name stays on it. The struck line is keyed by its corrupted
    // text, so a later save by that reader keeps the lie exactly as a save kept
    // it before readings existed — and a reader that re-reads it still meets
    // the sigil, which the resolver was built never to fold back.
    let reading = world.get::<super::Read>(node).map_or_else(
        || super::Read::verbatim(&held.0),
        |read| read.aligned(&held.0),
    );
    let mut read = reading.lines.clone();
    read[at] = lines[at].clone();
    let corrupted = super::Read::new(&lines, read, reading.by);
    world.entity_mut(node).insert(Rewritten {
        was: held.0,
        read: reading,
    });
    world.entity_mut(node).insert(Held(lines));
    world.entity_mut(node).insert(corrupted);
    super::poison(world, node);
}

/// Append the substitution sigil to a line's last word.
///
/// **Lines with no argument are left alone**, which is why the caller walks to
/// one rather than taking whatever the tick names: a bare `probe`, an `end` or
/// an `else` has nothing to misdirect, and suffixing the keyword breaks the
/// parse instead of the meaning.
/// **The line is kept and only its last word is replaced.** It was rebuilt with
/// `words.join(" ")`, which reflows it — `    grind    sage` came back as
/// `    grind sage-`, so the enemy changed the player's *formatting* as well as
/// one word. §19 is careful that the orb never rewrites a spell's text, and a
/// whitespace change `verify` cannot explain is noise on top of the one-word lie
/// this surface is supposed to be.
///
/// Splitting at the last word's own offset keeps everything before it byte for
/// byte, indentation and interior spacing alike.
fn corrupt(line: &str) -> String {
    let mut words = line.split_whitespace();
    let Some(last) = words.next_back() else {
        return line.to_owned();
    };
    if words.next().is_none() {
        return line.to_owned();
    }
    // **After the early returns**, because `claimed` allocates and the two
    // returns above discard it — the lie was being built for lines that were
    // never going to be corrupted.
    let lie = super::sabotage::claimed(last);
    let Some(at) = line.rfind(last) else {
        return line.to_owned();
    };
    format!("{}{lie}", &line[..at])
}

/// Whether a line has an argument worth corrupting.
///
/// A control word on its own — `end`, `else`, `repeat` with a count — has
/// nothing to misdirect. Suffixing it breaks the *parse*, which is loud, and the
/// point of this surface is a lie that runs.
///
/// # Three more shapes, and they were breaking spells rather than lying to them
///
/// The first two guards were the *word count* and a *trailing digit*, which let
/// through every line whose last word happens to be grammar. Measured by running
/// it: `part look()` became `part look()-` and invoking printed three
/// complaints — a `part` that wants a name, an `end` with nothing open, and a
/// call naming no part — so one lie produced an unclosed block and a dead call.
/// `if the mortar is idle` became `is idle-` and the whole guarded block was
/// skipped with *"that question means nothing"*.
///
/// Both are the failure this module's own header forbids: a corrupted line must
/// **parse, run, and quietly do nothing**, because *"a line the orb cannot read
/// at all would fault loudly and give the game away"*. A lie the player is told
/// about is not misdirection.
///
/// So the last word is refused when it is:
///
/// - **a state** — `is idle-` is unreadable, and `SpellState::WORDS` is asked
///   rather than a copy of it, so a ninth spelling cannot drift out of this;
/// - **bracketed** — a `part` heading or a call, where the brackets *are* the
///   notation and suffixing one loses the name that matches definition to call;
/// - **a set** — `for each way-` binds a cursor over nothing, so the body never
///   runs at all rather than running against a lie.
fn corruptible(line: &str) -> bool {
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.len() < 2 {
        return false;
    }
    let last = words[words.len() - 1];
    // A trailing number is a count, and `repeat 3-` does not parse.
    if last.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // `for each way` — the set is the word before the last, so this is asked of
    // `particle()` rather than of the literal, which is what stops it drifting
    // if the grammar ever fixes a second follower.
    if crate::parser::SpellWord::For.particle() == Some(words[words.len() - 2]) {
        return false;
    }
    !crate::parser::SpellState::WORDS.contains(&last) && !last.contains(['(', ')'])
}

/// Give a rewritten spell its own words back.
///
/// The mirror of `sabotage::restore`, and it is what makes notice → `verify` →
/// `purge` a repair loop here as well.
pub fn unwrite(world: &mut World, node: Entity) -> bool {
    let Some(Rewritten { was, read }) = world.get::<Rewritten>(node).cloned() else {
        return false;
    };
    // **The reading goes back with the text, and it is the one that ran.**
    // `rewrite` corrupted both, so restoring one would leave the tower running
    // the enemy's line for ever while `peruse` showed the player their own.
    //
    // It went back verbatim, with a note that a repaired spell compiled its
    // player's words *"until the next save re-reads them"* — and no save did,
    // because every line's text matched what the verbatim copy was keyed by.
    // The reading from before the strike needs no reader to put back.
    world.entity_mut(node).insert(Held(was));
    world.entity_mut(node).insert(read);
    world.entity_mut(node).remove::<Rewritten>();
    true
}

/// Give a retimed spell its own clock back.
pub fn untime(world: &mut World, node: Entity) -> bool {
    if world.get::<Retimed>(node).is_none() {
        return false;
    }
    world.entity_mut(node).remove::<Retimed>();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// **Every line the enemy is allowed to touch must still parse.**
    ///
    /// This is the module's headline claim — a corrupted line *"parses, runs,
    /// and quietly does nothing"* — and nothing asserted it. The guard was a
    /// word count and a trailing digit, which let through `part look()` and
    /// `if the mortar is idle`; both were verified breaking a real spell into
    /// three complaints rather than misdirecting it.
    ///
    /// Asked of every line of every **shipped** solver, so a spell added to
    /// `dev_spells.toml` in a shape the guard has not met fails here rather than
    /// in a player's siege.
    #[test]
    fn a_line_the_enemy_may_corrupt_still_reads_as_a_line() {
        let spells = crate::content::Spells::builtin();
        let mut broken = Vec::new();
        for (name, spell) in spells.iter() {
            for line in &spell.lines {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || !corruptible(line) {
                    continue;
                }
                let lied = corrupt(line);
                // A spell word still opens it, and the shape it needs still
                // follows — which is what `part`, `for each` and `is <state>`
                // each stopped being true of.
                let before = crate::parser::spell_word(line);
                let after = crate::parser::spell_word(&lied);
                if before != after {
                    broken.push(format!("{name}: {line:?} -> {lied:?} changed its word"));
                }
                // ...and the corruption is one suffix on the last word, never a
                // reflow of the line.
                if !lied.starts_with(line.split_whitespace().next().unwrap_or_default()) {
                    broken.push(format!("{name}: {line:?} -> {lied:?} lost its head"));
                }
            }
        }
        assert!(
            broken.is_empty(),
            "the enemy would break these rather than lie to them: {broken:#?}",
        );
    }

    /// **The lie is one word, and the player's own spacing survives it.**
    #[test]
    fn corrupting_a_line_keeps_every_character_before_its_last_word() {
        let line = "    grind    sage";
        let lied = corrupt(line);
        assert!(
            lied.starts_with("    grind    "),
            "the enemy reflowed the line: {lied:?}",
        );
        assert!(lied.ends_with('-'), "nothing was corrupted: {lied:?}");
    }

    /// The three shapes that were getting through, by name.
    #[test]
    fn a_part_a_call_and_a_state_are_left_alone() {
        for line in [
            "part look()",
            "look()",
            "if the mortar_and_pestle is idle",
            "for each way",
            "repeat 3",
            "end",
        ] {
            assert!(
                !corruptible(line),
                "{line:?} would be corrupted, and it does not survive it",
            );
        }
        // ...and the ordinary case still is corruptible, or the guard has eaten
        // the surface rather than narrowed it.
        assert!(corruptible("grind sage"));
        assert!(corruptible("haul wellspring barrier"));
    }

    #[test]
    fn the_calm_layer_is_never_touched() {
        // **Pillar 4, asserted.** §5.1: *"the enemy never touches scripts,
        // schedules, or logs in the calm layer."* Nothing here runs on a tick;
        // it runs on a resolved round, and `hold` is the only caller.
        let mut sim = Sim::new(11);
        sim.step_n(2000);
        let world = sim.world();
        let rewritten = world
            .iter_entities()
            .filter(bevy_ecs::world::EntityRef::contains::<Rewritten>)
            .count();
        let retimed = world
            .iter_entities()
            .filter(bevy_ecs::world::EntityRef::contains::<Retimed>)
            .count();
        assert_eq!(
            (rewritten, retimed),
            (0, 0),
            "two thousand calm ticks produced adversarial sabotage",
        );
    }

    #[test]
    fn a_rewritten_spell_keeps_what_it_said() {
        // Misdirection, not theft — the rule `substitute` records and the reason
        // the repair loop is a loop.
        let mut sim = Sim::new(11);
        sim.submit("attend archive");
        sim.step();
        sim.submit("debug_spell threading");
        sim.step();

        let node = sim
            .world()
            .iter_entities()
            .find(bevy_ecs::world::EntityRef::contains::<Held>)
            .map(|entity| entity.id())
            .expect("a spell was shelved");
        let before = sim.world().get::<Held>(node).cloned().expect("held").0;

        rewrite(sim.world_mut(), node);
        let after = sim.world().get::<Held>(node).cloned().expect("held").0;
        assert_ne!(before, after, "rewriting changed nothing");
        assert_eq!(
            sim.world().get::<Rewritten>(node).map(|r| r.was.clone()),
            Some(before.clone()),
            "the player's own words were not kept",
        );

        // **The sabotage must reach the program, not only the file.**
        // `spell::compile` reads `Read`, so a rewrite that touched `Held` alone
        // would leave the player seeing a broken line while the orb ran the
        // clean one — the premise clause switched off, and `purge` repairing
        // nothing. This is the assertion that keeps the two together.
        assert_eq!(
            crate::tower::spell::source(sim.world(), node),
            after,
            "the enemy corrupted the file and not what runs",
        );

        assert!(unwrite(sim.world_mut(), node));
        assert_eq!(
            sim.world().get::<Held>(node).cloned().expect("held").0,
            before,
            "a repaired spell did not read as it was written",
        );
        assert_eq!(
            crate::tower::spell::source(sim.world(), node),
            before,
            "a repaired spell still runs the enemy's line",
        );
    }

    #[test]
    fn a_strike_corrupts_one_line_of_the_reading_and_a_repair_restores_all_of_it() {
        // **The orb's reading of every other line survives the strike.** The
        // first version wrote the whole corrupted text into `Read`, so a spell
        // the reader had made runnable went back to its player's loose words —
        // and the repair put back the words, not what had run.
        let mut sim = Sim::new(11);
        let typed: Vec<String> = ["work the sage down", "hang on ten ticks", "grind sage"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        sim.write_spell_reading("morning", &typed, &crate::Copyist::worked());
        sim.step();
        let node = sim
            .world()
            .iter_entities()
            .find(|entity| {
                entity
                    .get::<Name>()
                    .is_some_and(|name| name.0 == "morning.spell")
            })
            .map(|entity| entity.id())
            .expect("the spell was written");
        let before = crate::tower::spell::source(sim.world(), node);
        assert_eq!(
            before,
            ["grind sage", "bide 10", "grind sage"],
            "the fixture did not read the spell",
        );

        rewrite(sim.world_mut(), node);
        let held = sim.world().get::<Held>(node).cloned().expect("held").0;
        let after = crate::tower::spell::source(sim.world(), node);
        let struck: Vec<usize> = (0..typed.len())
            .filter(|at| held[*at] != typed[*at])
            .collect();
        assert_eq!(struck.len(), 1, "one line is struck: {held:?}");
        for at in 0..typed.len() {
            let wanted = if struck.contains(&at) {
                &held[at]
            } else {
                &before[at]
            };
            assert_eq!(&after[at], wanted, "line {at} runs the wrong thing");
        }

        assert!(unwrite(sim.world_mut(), node));
        assert_eq!(
            crate::tower::spell::source(sim.world(), node),
            before,
            "the repair put back the text and not what ran",
        );
    }

    #[test]
    fn only_one_line_is_touched() {
        // A spell rewritten wholesale is one the player throws away; one line
        // changed is one they have to find.
        let mut sim = Sim::new(11);
        sim.submit("attend archive");
        sim.step();
        sim.submit("debug_spell threading");
        sim.step();
        let node = sim
            .world()
            .iter_entities()
            .find(bevy_ecs::world::EntityRef::contains::<Held>)
            .map(|entity| entity.id())
            .expect("a spell was shelved");
        let before = sim.world().get::<Held>(node).cloned().expect("held").0;

        rewrite(sim.world_mut(), node);
        let after = sim.world().get::<Held>(node).cloned().expect("held").0;

        let differing = before.iter().zip(&after).filter(|(a, b)| a != b).count();
        assert_eq!(differing, 1, "more than one line was rewritten");
        assert_eq!(before.len(), after.len(), "the spell changed length");
    }
}

#[cfg(test)]
mod dragging {
    use super::*;
    use crate::Sim;

    #[test]
    fn a_retimed_spell_loses_ticks_at_the_shipped_budget() {
        // **The test that hid the defect asked with an allowance of 4.** The
        // shipped `SCRIPT_BUDGET` is 1, where `allowance - drag` floored at one
        // returns one for every drag — so the surface announced itself, cost an
        // audit and a purge, and changed nothing.
        //
        // Asking with **1** is the whole point: a drag has to cost something at
        // the budget the game actually runs at.
        let mut sim = Sim::new(11);
        sim.submit("attend laboratory");
        sim.step();
        sim.submit("invoke first_light");
        sim.step();

        let node = sim
            .world_mut()
            .query::<(Entity, &crate::tower::spell::Running)>()
            .iter(sim.world())
            .map(|(entity, _)| entity)
            .next();
        let Some(node) = node else { return };

        for drag in [1_u64, 2, 3] {
            sim.world_mut().entity_mut(node).insert(Retimed { drag });
            let over: Vec<usize> = (0..24)
                .map(|_| {
                    sim.step();
                    crate::tower::spell::dragged_for_test(sim.world(), node, 1)
                })
                .collect();
            let ran = over.iter().filter(|left| **left > 0).count();
            assert!(
                ran > 0,
                "a drag of {drag} halted the spell outright, which §8 forbids",
            );
            assert!(
                ran < over.len(),
                "a drag of {drag} cost nothing at a budget of one",
            );
        }
    }
}
