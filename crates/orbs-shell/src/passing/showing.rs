//! What the pane is showing, and what kind of change two of them make.

use orbs_render::Passage;

use crate::focus::{Focus, Open};
use crate::glance::Panel;

/// What the pane is showing. Two of these differing is a crossing.
///
/// Three fields, each earning its place by a false positive it prevents. A
/// fourth would need the same argument.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Showing {
    /// The surface that has taken the whole pane, if one has.
    ///
    /// [`Focus::takes_the_pane`], not the whole `Focus`: `Focus::Reading` would
    /// fire a crossing on `PgUp`, and a scrolled-back transcript replaces
    /// nothing.
    tool: Option<Focus>,
    /// The room. [`Panel::room`], which is the room and not the leaf.
    room: String,
    /// Whether `F5`'s linear mirror is up.
    ///
    /// It is not a `Focus` variant at all — `prompt::paint` branches on
    /// `linear.showing()` separately — so without this the one surface that
    /// replaces the pane without taking the keyboard could never cross.
    mirrored: bool,
}

impl Showing {
    /// What the frontends' own state adds up to.
    ///
    /// One function so the two builds cannot disagree about what a screen change
    /// is, which is `focus::Focus::of`'s argument and this is the same shape.
    #[must_use]
    pub fn of(open: Open, panel: &Panel, mirrored: bool) -> Self {
        let focus = Focus::of(open);
        let tool = focus.takes_the_pane().then_some(focus);
        Self {
            tool,
            room: panel.room.clone(),
            // Only when a tool is not already holding the pane, the order
            // `prompt::paint_view` branches in. `F5` is not gated on focus, so
            // pressing it inside the editor peeled the editor apart and back
            // together over a screen that had not changed.
            mirrored: mirrored && tool.is_none(),
        }
    }

    /// What kind of change this is, if it is one.
    ///
    /// Ordered, because two can differ at once: a spell that walks and then
    /// opens a maze moves both in one tick. The pane-replacing kinds win —
    /// sparing the transcript while it is being replaced draws a crossing around
    /// a hole.
    pub(super) fn change(&self, next: &Self) -> Option<Change> {
        if self.tool != next.tool {
            Some(Change::Tool)
        } else if self.mirrored != next.mirrored {
            Some(Change::Mirror)
        } else if self.room != next.room {
            Some(Change::Room)
        } else {
            None
        }
    }
}

/// What kind of screen change is being crossed.
///
/// The motion names the kind, which is §19's *"tempo is the organising
/// principle"* applied to screens: what is running should be legible from across
/// the room without reading a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Change {
    /// `attend` somewhere else. The boards change; the transcript does not.
    Room,
    /// A full-pane surface opened or shut — `wander`, `edit`, `weave`.
    Tool,
    /// `F5`'s linear mirror, on or off. The same content, re-read.
    Mirror,
}

impl Change {
    /// The shape this kind of change leaves by.
    pub(super) const fn passage(self) -> Passage {
        match self {
            Self::Room => Passage::Wipe,
            Self::Tool => Passage::Gather,
            Self::Mirror => Passage::Furl,
        }
    }
}
