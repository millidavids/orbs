//! The puzzles a fixture can hold open — work that takes no production slot and
//! inserts no `Working`, and is still work.

use bevy_ecs::prelude::*;

/// A puzzle open on a fixture.
///
/// **One list, matched exhaustively everywhere the question is asked.** A maze
/// in the stacks, a ward at the prism, a charm on the lattice, a course at the
/// pylon and a beast at the circle all hold a fixture busy without a `Working`
/// component — a spell's `summon` or `haul` must never wait on the puzzle it is
/// solving — so every place that asks *"is anything happening here"* has to
/// know about each of them. `panel::read` answered that with one `if let` a
/// puzzle and `execute::pipeline::stop` with another, as two lists; §19 records
/// the pylon's arm being forgotten in `stop` once, and the circle had to be
/// added to both. **A variant added here is a compile error in both** until each
/// has decided what it does with it.
///
/// A fixture holds at most one: every puzzle has its own instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Open {
    /// The stacks' maze, at the lectern.
    Maze,
    /// A far orb's ward, at the prism.
    Ward,
    /// A charm being bound, at the lattice.
    Binding,
    /// A course of wards, at the pylon.
    Course,
    /// A beast waiting at the circle.
    Beast,
}

impl Open {
    /// Every puzzle, for a pass that has to visit each whether or not one is open.
    pub const ALL: [Self; 5] = [
        Self::Maze,
        Self::Ward,
        Self::Binding,
        Self::Course,
        Self::Beast,
    ];

    /// The verb the puzzle's fixture carries as its `Operation` — how a pass
    /// finds the fixture of a puzzle that is *not* open, which `on` cannot.
    #[must_use]
    pub const fn operation(self) -> crate::parser::Verb {
        use crate::parser::Verb;
        match self {
            Self::Maze => Verb::Research,
            Self::Ward => Verb::Probe,
            Self::Binding => Verb::Imbue,
            Self::Course => Verb::Muster,
            Self::Beast => Verb::Summon,
        }
    }

    /// The puzzle open on `node`, if one is.
    #[must_use]
    pub fn on(world: &World, node: Entity) -> Option<Self> {
        let holds = |present: bool, open: Self| present.then_some(open);
        holds(world.get::<super::Maze>(node).is_some(), Self::Maze)
            .or_else(|| holds(world.get::<super::Ward>(node).is_some(), Self::Ward))
            .or_else(|| {
                holds(
                    world.get::<super::lattice::Binding>(node).is_some(),
                    Self::Binding,
                )
            })
            .or_else(|| holds(world.get::<super::Course>(node).is_some(), Self::Course))
            .or_else(|| {
                holds(
                    world.get::<super::circle::Beast>(node).is_some(),
                    Self::Beast,
                )
            })
    }
}
