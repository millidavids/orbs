//! O.R.B.S. — the shell every frontend shares.
//!
//! `orbs-render` decides how a cell is addressed and what a `Style` means; this
//! crate decides what a screen is. Where the instrument panel goes, what the
//! tower rail says about a room nobody is standing in, what the spell editor
//! does with Backspace, how far back the transcript has scrolled — all of it
//! takes a [`Sim`](orbs_sim::Sim) and a [`Screen`] and hands back a
//! [`Frame`](orbs_render::Frame). Drawing that Frame is a frontend's, and there
//! are two: `orbs` puts it on a GPU behind a CRT, `orbs-tui` writes it out.
//!
//! It exists because it needs both `orbs-sim` and `orbs-render` and neither may
//! depend on the other in that direction, so the painters had nowhere to live
//! but a frontend. For four phases that was fine, because there was one — §19
//! rests part of its editor-ownership argument on *"`orbs-tui` is ten lines with
//! nothing to diverge from."* With a second frontend real, the choice is one
//! shell or two that disagree.
//!
//! Not here: anything needing an engine, a window or a GPU — the camera and the
//! 4:3 letterbox, the CRT, the glyph atlas, the phosphor palette, `winit`'s
//! press/release model. Those are `orbs`'s. A terminal has none of them and
//! loses nothing informational, which is rule 2 as a crate boundary.

mod bench;
mod beside;
mod board;
mod circle;
mod dump;
mod editor;
mod environment;
mod focus;
mod gauges;
mod glance;
mod guide;
mod keys;
mod lattice;
mod lexing;
mod line;
mod linear;
mod loom;
mod manual;
mod menu;
mod offering;
pub mod panel;
mod passing;
mod post;
mod prompt;
mod prose;
mod pylon;
mod rail;
mod rampart;
mod reveal;
mod road;
mod save;
mod screen;
mod scrollback;
mod seed;
pub mod settings;
mod sheet;
mod shortcuts;
mod stacks;
mod stage;
mod stations;
pub mod tabbing;
mod tapestry;
mod threshold;
mod transition;

pub use bench::Bench;
pub use dump::{
    PASSAGE as PASSAGE_VAR, requested as dump_requested, run as dump, run_script as dump_script,
};
pub use editor::{Editor, Mode as EditorMode, Outcome as EditorOutcome};
pub use environment::{LENGTH, RUSTC, SEALED, augury, fresh, scrivener, wizard};
pub use focus::{Focus, Open};
pub use glance::Panel;
pub use guide::{Entry, Guide, guide};
pub use keys::{
    Key, apply, apply_to_editor, apply_to_manual, apply_to_maze, apply_to_menu, apply_to_weave,
};
pub use line::Line;
pub use linear::{LINEAR as LINEAR_SETTING, Linear, toggle as toggle_linear};
pub use manual::{
    Outcome as ManualOutcome, Reader as ManualReader, Showing as ManualShowing,
    book as manual_book, paint as paint_manual,
};
pub use menu::{Driver, Menu, Outcome as MenuOutcome, SETTINGS_ROWS, Stance, WORDS as MENU_WORDS};
pub use offering::{Ghost, Offered};
pub use passing::{Passing, Showing};
pub use prompt::{View, paint, paint_booting, paint_too_small};
pub use prose::{
    CONTENT_DIR, DIRECTORY as CONTENT_DIRECTORY, MANUAL, PROSE, load, read, read_manual,
};
pub use reveal::Reveal;
pub use save::{
    Held, OFF as SAVE_OFF, Opened, SAVE_PATH, SAVE_VAR, SLOTS as SAVE_SLOTS, Slot, abandon,
    app_data, away_for, free_slot, migrate as migrate_saves, path as save_path, read as read_save,
    read_from as read_save_from, saves, slot_path, write as write_save, write_to as write_save_to,
};
pub use screen::Screen;
pub use scrollback::{Scroll, page_step};
pub use seed::{SEED, SEEDS, fresh_seed, new_game_seed, seed};
pub use shortcuts::{TRACE_PATH, cycle_register, export_trace};
pub use stage::{Boot, Stage};
pub use tapestry::{Outcome as WeaveOutcome, Tapestry};
pub use threshold::{THRESHOLD, Threshold};
pub use transition::PaneTransition;
