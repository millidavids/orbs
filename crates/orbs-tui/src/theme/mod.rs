//! Where a `Style` finally becomes a colour, on this side of rule 2.
//!
//! `orbs-render` decides *what* appears and never emits a hue; everything here
//! decides *how* it is drawn. The counterpart of the Bevy build's
//! `render::palette`, with sixteen ANSI colours instead of a solved palette.
//!
//! Indexed ANSI 0–15, inheriting the user's terminal theme (§19), so there is no
//! palette to tune and no contrast to verify — the numbers belong to whoever
//! configured the terminal. That holds for the syntax hues too, which is why
//! they are indices here and `Srgba` in the Bevy build.
//!
//! What it costs is §14's guarantee: sixteen colours we do not own cannot be
//! held to the Bevy palette's solved contrast. So the glyph carries the identity
//! and the colour is a hint — a meter is read off a `█`/`▓` boundary against
//! `░`, the ward's six sigils are six shapes, and every `Role` reaches the
//! linear stream beside its text.
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
