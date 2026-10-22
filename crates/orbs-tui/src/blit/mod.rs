//! A `Frame` onto a terminal — this frontend's whole half of rule 2.
//!
//! `orbs-render` said what appears and where; `orbs-shell` said what the screen
//! is. All that is left is writing the cells out, and the only decisions here
//! are how a glyph is encoded and what colour it takes.

mod narrow;
#[cfg(test)]
mod redraws;
#[cfg(test)]
mod scan;
mod screen;
#[cfg(test)]
mod tests;

pub(crate) use screen::Screen;
