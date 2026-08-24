//! What the completer has put in front of the player — the Tab listing and the
//! ghost trailing the caret.
//!
//! Both are answers to *what could this line become*, both are derived purely
//! from [`Line`](super::line::Line) and the sim's `Scene`, and both are held
//! rather than recomputed because a frame runs sixty times a second and neither
//! input changes that often.
//!
//! Nothing here reads a keystroke or touches a window. The system that fills
//! them in is the Bevy build's `shell::input`; this is the state it fills.

use bevy_ecs::prelude::Resource;

/// What Tab last offered, and which of them is in the line.
///
/// A resource rather than a record, and cleared on the next keystroke: readline
/// lists on ambiguity, but the log is the sim's and a Tab press is not something
/// a replay could reproduce.
#[derive(Resource, Debug, Default)]
pub struct Offered {
    /// The candidates, in the order the completer offered them.
    pub options: Vec<String>,
    /// Which one repeated Tab has reached, so the list can mark it.
    ///
    /// Without this the list is a wall of equal-looking words while the line
    /// changes underneath it, and the player has no way to see where they are in
    /// the cycle — which is the whole affordance.
    pub current: Option<usize>,
}

impl Offered {
    /// Whether there is anything to draw.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    /// Forget the listing.
    pub fn clear(&mut self) {
        self.options.clear();
        self.current = None;
    }
}

/// The inline suggestion trailing the caret.
///
/// **Recomputed on change, not per frame.** [`Line::ghost`](super::line::Line)
/// walks the history, then runs the whole completer — which filters every
/// synonym, builds an owned `String` per candidate, sorts, dedups, and collects
/// a `Vec<char>` into a `String` for the common prefix. That ran at 60 Hz off
/// inputs that change on a keystroke or a tick, so ~59 frames in 60 rebuilt a
/// string identical to the one already on screen.
///
/// It stays a pure function of `(line, scene, prompt_open)` — this holds the
/// result, and the run condition names exactly what invalidates it, so there is
/// no second copy able to drift from the line it trails.
#[derive(Resource, Debug, Default)]
pub struct Ghost(pub String);
