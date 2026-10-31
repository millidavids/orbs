//! What a question is about.
//!
//! A place in the spell's room, or an arsenal store from anywhere; and for
//! `is`, a spell, from any spell, since a spell is nameable from anywhere
//! (§19, *The tower's spell*).

use bevy_ecs::prelude::{Entity, World};

use crate::parser::{NounKind, SpellState};
use crate::tower::{self, Cwd};

/// The place `named`, or `None` with the name noted once as one the tower lacks.
pub(super) fn find(world: &World, named: &str, missing: &mut Vec<String>) -> Option<Entity> {
    noted(place(world, named), named, missing)
}

/// What an `is` asks about: a place `named`, then the spell, then anything
/// else of that name — `compile`'s order, so a spell called `charcoal` is not
/// answered by the charcoal on a shelf.
///
/// A name ending `.spell` is `compile`'s answer already, and needs no walk.
pub(super) fn find_is(world: &World, named: &str, missing: &mut Vec<String>) -> Option<Entity> {
    if crate::content::without_extension(named) != named {
        return noted(spell(world, named), named, missing);
    }
    let found = place(world, named);
    let a_place = found.filter(|node| {
        world
            .get::<tower::Nameable>(*node)
            .is_some_and(|kind| kind.0 == NounKind::Place)
    });
    noted(
        a_place.or_else(|| spell(world, named)).or(found),
        named,
        missing,
    )
}

/// Whether `at` is in `state`.
///
/// A spell works while it runs or is held: a held spell has no `Running` for
/// the tick between laps, and would read idle there and be started twice.
pub(super) fn is(world: &World, at: Entity, state: SpellState) -> bool {
    if world
        .get::<tower::Nameable>(at)
        .is_some_and(|kind| kind.0 == NounKind::Script)
    {
        let working =
            world.get::<super::Running>(at).is_some() || world.get::<super::Bound>(at).is_some();
        return match state {
            SpellState::Working => working,
            SpellState::Idle => !working,
            SpellState::Empty => world
                .get::<tower::Held>(at)
                .is_none_or(|held| held.0.iter().all(|line| line.trim().is_empty())),
        };
    }
    // A room, which only the tower's spell can name: by the instrument the
    // rail reports it as. A run in hand is work; a lit hearth or a held spell
    // is not (`wait::room`), or a room would read busy for good.
    if tower::domain_of(world, at) == Some(at) {
        let instruments = tower::instruments_in(world, at);
        let reported = tower::reported(&instruments);
        let working = reported.is_some_and(|instrument| {
            matches!(
                instrument.state,
                tower::State::Working | tower::State::Scouring
            )
        });
        return match state {
            SpellState::Working => working,
            SpellState::Idle => !working,
            SpellState::Empty => {
                reported.is_none_or(|instrument| instrument.state == tower::State::Empty)
            }
        };
    }
    // The panel's word, not `busy()`: an athanor's fire is `Burning`, which
    // `busy()` cannot see — see [`State::is_busy`](tower::State::is_busy).
    // `Empty` is about contents, since the athanor never reports
    // `State::Empty`.
    match state {
        SpellState::Idle => !tower::state_at(world, at).is_busy(),
        SpellState::Working => tower::state_at(world, at).is_busy(),
        // A satchel answers from its queue: `Stock` collapses duplicates, and a
        // full satchel read empty (§19).
        SpellState::Empty => world.get::<tower::Satchel>(at).map_or_else(
            || tower::children_of(world, at).is_empty(),
            tower::Satchel::is_empty,
        ),
    }
}

/// The room's child `named`, then an arsenal *store* only: `for each store`
/// binds a cursor to one from anywhere (§19), and a wider fallback would answer
/// a laboratory's missing clarity from the arsenal's shelf, false for ever.
///
/// The tower's spell stands at `/tower`, so its room's children are the rooms,
/// and anything in them is found by name too: it watches the whole tower.
fn place(world: &World, named: &str) -> Option<Entity> {
    let cwd = world.resource::<Cwd>().0;
    let leaf = crate::parser::leaf(named).to_owned();
    let here = |at: Entity| {
        tower::children_of(world, at).into_iter().find(|node| {
            world
                .get::<tower::Name>(*node)
                .is_some_and(|name| name.0 == leaf)
        })
    };
    // Not `here` at the tower: its children are the rooms, sealed ones too.
    let found = if super::whole::at_the_tower(world, cwd) {
        anywhere(world, cwd, &leaf)
    } else {
        here(cwd)
    };
    found.or_else(|| {
        tower::keep(world).and_then(here).filter(|node| {
            world
                .get::<tower::Grouped>(*node)
                .is_some_and(|group| group.0 == tower::STORE)
        })
    })
}

/// Anything in an open room called `leaf`, depth first. Not a satchel, one per
/// room, so the name is nobody's; and neither a sealed room nor anything in it,
/// or the tower's spell would find what discovery has not. Rooms are the
/// tower's children, so the seal is asked once a room, and a room itself, the
/// commonest question, is found before any walk.
fn anywhere(world: &World, tower: Entity, leaf: &str) -> Option<Entity> {
    let called = |node: Entity| {
        world
            .get::<tower::Name>(node)
            .is_some_and(|name| name.0 == leaf)
    };
    let mut stack: Vec<Entity> = tower::children_of(world, tower)
        .into_iter()
        .filter(|room| tower::sealed_room_of(world, *room).is_none())
        .collect();
    if let Some(room) = stack.iter().copied().find(|room| called(*room)) {
        return Some(room);
    }
    stack.reverse();
    while let Some(node) = stack.pop() {
        if called(node) && world.get::<tower::Satchel>(node).is_none() {
            return Some(node);
        }
        let mut below = tower::children_of(world, node);
        below.reverse();
        stack.extend(below);
    }
    None
}

/// The spell called `named`, wherever it is kept, with or without `.spell`.
fn spell(world: &World, named: &str) -> Option<Entity> {
    tower::reach::look(world)
        .scope(tower::reach::Scope::Tower)
        .kind(NounKind::Script)
        .naming(tower::reach::Naming::Script)
        .find(&crate::content::with_extension(named))
}

fn noted(found: Option<Entity>, named: &str, missing: &mut Vec<String>) -> Option<Entity> {
    if found.is_none() && !missing.iter().any(|already| *already == named) {
        missing.push(named.to_owned());
    }
    found
}
