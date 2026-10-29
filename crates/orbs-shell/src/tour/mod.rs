//! The tour: every surface of the game, posed and played one scene at a time
//! (DESIGN.md §19, *the tour*).
//!
//! The catalogue and its reading are here so both the window and the tests
//! read one file; playing a scene is the window's.

mod catalogue;
mod expect;
mod scene;
mod stage;
mod steps;
#[cfg(test)]
mod tests;
mod timeline;

pub use catalogue::{Catalogue, FILE};
pub use expect::{Expect, Surface, moving};
pub use scene::{Scene, Start, TOUR, requested};
pub use stage::{NEEDS_DEBUG, Staged, for_dump, stage};
pub use steps::{FIRST_PAUSE, Named, PAUSE, Step, Stroke, When, strokes};
pub use timeline::{Advance, KEYING, Seen, TYPING, Timeline};
