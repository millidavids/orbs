//! O.R.B.S. — the shell every frontend shares.
//!
//! `orbs-render` decides how a *cell* is addressed and what a `Style` means;
//! this crate decides what a **screen** is. Where the instrument panel goes,
//! what the tower rail says about a room nobody is standing in, what the spell
//! editor does with Backspace, how far back the transcript has scrolled — all of
//! it takes a [`Sim`](orbs_sim::Sim) and a [`Screen`] and hands back a
//! [`Frame`](orbs_render::Frame).
//!
//! Drawing that Frame is a frontend's, and there are two: `orbs` puts it on a
//! GPU behind a CRT, `orbs-tui` writes it to a terminal.
//!
//! # Why this crate exists
//!
//! It needs both `orbs-sim` and `orbs-render`, and neither may depend on the
//! other in that direction — `orbs-render` has an empty `[dependencies]` on
//! purpose and may never learn what a recipe is. So the painters had nowhere to
//! live but a frontend, and for four phases that was fine, because there was
//! only one. DESIGN.md §19 rests part of its editor-ownership argument on
//! exactly that: *"`orbs-tui` is ten lines with nothing to diverge from."*
//!
//! The moment a second frontend is real, that stops being true, and the choice
//! is one shell or two that disagree. This is the one shell.
//!
//! # What is *not* here
//!
//! Anything that needs an engine, a window or a GPU: the camera and the 4:3
//! letterbox, the CRT, the glyph atlas, the phosphor palette, `winit`'s
//! press/release model and the held-key bookkeeping it forces. Those are
//! `orbs`'s. A terminal has none of them and loses nothing informational, which
//! is architectural rule 2 restated as a crate boundary.

mod bench;
mod board;
mod chant;
mod dump;
mod editor;
mod environment;
mod focus;
mod glance;
mod guide;
mod keys;
mod lexing;
mod line;
mod linear;
mod loom;
mod offering;
pub mod panel;
mod post;
mod prompt;
mod prose;
mod pylon;
mod rail;
mod rampart;
mod reveal;
mod save;
mod screen;
mod scrollback;
mod sheet;
mod shortcuts;
mod stacks;
mod stage;
pub mod tabbing;
mod tapestry;
mod transition;

pub use bench::Bench;
pub use dump::{requested as dump_requested, run as dump, run_script as dump_script};
pub use editor::{Editor, Mode as EditorMode, Outcome as EditorOutcome};
pub use environment::{RUSTC, SEED, seed, wizard};
pub use focus::{Focus, Open};
pub use glance::Panel;
pub use guide::{Entry, Guide, guide};
pub use keys::{Key, apply, apply_to_chant, apply_to_editor, apply_to_maze, apply_to_weave};
pub use line::Line;
pub use linear::{Linear, toggle as toggle_linear};
pub use offering::{Ghost, Offered};
pub use prompt::{View, paint, paint_booting, paint_too_small};
pub use prose::{CONTENT_DIR, PROSE, load, read};
pub use reveal::Reveal;
pub use save::{
    OFF as SAVE_OFF, Opened, SAVE_PATH, SAVE_VAR, away_for, path as save_path, read as read_save,
    write as write_save,
};
pub use screen::Screen;
pub use scrollback::{Scroll, page_step};
pub use shortcuts::{TRACE_PATH, cycle_register, export_trace, toggle_patient};
pub use stage::{Boot, Stage};
pub use tapestry::{Outcome as WeaveOutcome, Tapestry};
pub use transition::PaneTransition;
