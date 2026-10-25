//! Where a `Style` finally becomes a colour, on this side of rule 2.
//!
//! `render::palette`'s counterpart: ANSI 0–15, the user's own theme (§19), so
//! nothing to tune and no contrast held. The glyph carries identity and the
//! colour hints (§14).
//!
//! Four accepted degradations:
//!
//! - `Presentation` is ignored: Eldritch and Tampered pick a face in the glyph
//!   atlas, and the face here is the terminal's. §19 allows this by name —
//!   `verify` is the authoritative sabotage detector on every surface.
//! - A mixed `Wash` takes its first tint; sixteen indices cannot average, and
//!   the flask's one two-colour band is already told apart by position.
//! - No CRT, no phosphor, no blinking caret. The terminal draws its own cursor,
//!   which is the one a screen reader tracks.
//! - Greyscale has three greys where the window keeps every luminance, so steps
//!   of one weight in a ramp can merge; the glyph and the fill length carry the
//!   reading (§14). See `Ink::without_hue`.

#[cfg(test)]
mod highlighting;
mod ink;
mod resolve;
#[cfg(test)]
mod tests;

pub(crate) use ink::{Ink, Weight};
pub(crate) use resolve::resolve;
