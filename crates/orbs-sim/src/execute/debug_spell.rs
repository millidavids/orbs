//! A working spell for a tester, in the builds a tester runs.
//!
//! # Why this exists beside `debug_spawn` rather than inside it
//!
//! [`debug`](super::debug) skips the forty ticks of grinding that reaching a
//! state costs. This skips the **eighty lines** that reaching an *automated*
//! state costs: the ladder that solves a maze is twenty-four rungs, and typing
//! it by hand to check that the archive still works is the same class of tedium
//! one directory over.
//!
//! They are separate modules because they are separate concerns — `debug.rs` is
//! already long, and a spawn order and a spell have nothing in common but the
//! `cfg`.
//!
//! # What a release build has
//!
//! No code, and no content. The module is `cfg(debug_assertions)` and
//! [`DEV_SPELLS`] is `include_str!`'d under the same `cfg`, so the word, the
//! parse, the queue variant, the effect *and* the eighty lines of ladder are all
//! absent. `debug_spawn`'s four lines of prose ship because `prose.toml` is one
//! file; this file is its own, so nothing of it does.
//!
//! # The two places it deliberately does not copy `debug_spawn`
//!
//! **It refuses outside the spell's own domain.** `debug_spawn`'s headline
//! property is *"any fixture, from anywhere"*, on the argument that making a
//! tester walk first puts back the walking the tool exists to skip. That
//! argument does not survive here: [`Sim::write_spell`](crate::Sim::write_spell)
//! homes a new spell to whichever domain the player is standing in, so
//! `debug_spell threading` typed in the laboratory would write an *archive*
//! spell homed in the laboratory — whose `follow` and `research` lines cannot
//! resolve there. A broken spell reported as written down is a worse outcome
//! than a refusal that names the room.
//!
//! **It records the write, not the typed line.** `debug_spawn` pushes its line
//! into [`Submissions`](crate::session::Submissions) and re-runs it on replay.
//! Doing that here would record *twice* — once as the typed line and once as the
//! `Wrote` that `write_spell` already pushes — and a replay would re-match the
//! word and push both again. Recording only the write is also the stronger
//! guarantee: a replay reproduces the **lines that actually ran**, even if this
//! file is edited afterwards.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, Spells};
use crate::session::Scrollback;
use crate::tower;

/// The word, matched exactly and never advertised.
///
/// Underscored like `debug_spawn`, and for the same reason: no verb in §6's
/// vocabulary has an underscore, so this cannot be reached by a typo of a real
/// word and it reads as a tool rather than as part of the game.
pub const SPELL: &str = "debug_spell";

/// The ladders, compiled in only where the word exists.
const DEV_SPELLS: &str = include_str!("../../content/dev_spells.toml");

/// What a tester asked for, if this line is a spell order at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    /// Write the named spell into the room it is authored for.
    Write(String),
    /// Say which spells there are.
    List,
}

/// Read `line` as a spell order.
///
/// Bare is how you ask what there is, exactly as `debug_spawn` bare lists what
/// it can make.
#[must_use]
pub fn order(line: &str) -> Option<Order> {
    let rest = line.trim().strip_prefix(SPELL)?;
    // `debug_spelling` is not this word. Anything after it must be whitespace.
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let mut words = rest.split_whitespace();
    let Some(name) = words.next() else {
        return Some(Order::List);
    };
    // One name and nothing else. A second word is a tester meaning something
    // this word cannot do, and guessing which half they meant is the quiet
    // reinterpretation `debug_spawn` refuses for a count.
    if words.next().is_some() {
        return None;
    }
    Some(Order::Write(name.to_owned()))
}

/// The authored dev spells.
///
/// # Panics
///
/// If the file is malformed — a build-time authoring error, covered by
/// `the_dev_file_parses`.
#[must_use]
pub fn spells() -> Spells {
    Spells::parse(DEV_SPELLS).expect("dev_spells.toml is malformed")
}

/// Carry out `order`, on the tick boundary like every other effect.
///
/// Returns the lines to write, if any — the caller owns the write, because
/// `Sim::write_spell` is what records it and this has no `&mut Sim`.
pub fn run(world: &mut World, order: &Order) -> Option<(String, Vec<String>)> {
    let book = spells();
    match order {
        Order::List => {
            let names: Vec<&str> = book.iter().map(|(name, _)| name).collect();
            say(world, "debug_spell_known", &names.join(", "), Role::Normal);
            None
        }
        Order::Write(name) => write(world, &book, name),
    }
}

/// Resolve `name` against the book and the room, or say why not.
fn write(world: &mut World, book: &Spells, name: &str) -> Option<(String, Vec<String>)> {
    let wanted = crate::content::without_extension(name);
    let Some((_, spell)) = book.iter().find(|(known, _)| *known == wanted) else {
        say(world, "debug_spell_unknown", wanted, Role::Danger);
        return None;
    };

    // **The room has to match, and this is the departure from `debug_spawn`.**
    // A spell is written *for* a domain and `scribe::write` homes a new one to
    // where the player stands, so writing an archive spell from the laboratory
    // would produce a file whose every line fails to resolve.
    let cwd = world.resource::<tower::Cwd>().0;
    let here = tower::domain_of(world, cwd)
        .or(Some(cwd))
        .and_then(|node| world.get::<tower::Name>(node))
        .map(|name| name.0.clone())
        .unwrap_or_default();
    if here != spell.domain {
        say(world, "debug_spell_elsewhere", &spell.domain, Role::Danger);
        return None;
    }

    say(world, "debug_spell_done", wanted, Role::Success);
    Some((wanted.to_owned(), spell.lines.clone()))
}

/// Say what happened. §3 forbids unlogged output, and a debug tool is not
/// exempt — a tester who cannot see that the write was refused is a tester
/// chasing the wrong bug.
fn say(world: &mut World, key: &str, detail: &str, role: Role) {
    let message = world
        .resource::<Prose>()
        .line(key, &[("name", detail), ("detail", detail)]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, SPELL)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_word_is_matched_exactly() {
        assert_eq!(order("debug_spell"), Some(Order::List));
        assert_eq!(
            order("debug_spell threading"),
            Some(Order::Write("threading".to_owned())),
        );
        assert_eq!(
            order("  debug_spell  threading  "),
            order("debug_spell threading")
        );
        // Not this word.
        assert_eq!(order("debug_spelling"), None);
        assert_eq!(order("scribe threading"), None);
        // One name only.
        assert_eq!(order("debug_spell threading twice"), None);
    }

    #[test]
    fn the_dev_file_parses() {
        let book = spells();
        assert!(!book.is_empty(), "the dev spellbook is empty");
    }

    #[test]
    fn every_dev_spell_is_written_for_a_domain_it_could_run_in() {
        // A domain named here that the tower does not raise would make the word
        // refuse for ever with a room nobody can reach.
        for (name, spell) in spells().iter() {
            assert!(
                !spell.domain.is_empty(),
                "{name} is written for no domain at all",
            );
            assert!(!spell.lines.is_empty(), "{name} has no lines to write",);
        }
    }
}
