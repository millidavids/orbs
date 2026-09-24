//! The manual in the Bevy shell: opening it, feeding it keys.
//!
//! [`ManualReader`] is the surface and knows nothing about Bevy. This is the
//! wiring, and it is a separate file for the reason `menuing.rs` is.
//!
//! It opens *over* the menu — the one pair that coexists, since every other
//! surface is exclusive. The menu stays up behind it, so `Focus` orders the
//! manual first, and closing it hands the keyboard back to the menu.
//!
//! [`type_into_manual`] runs whenever a key arrives, declines inside, and is
//! ordered *before* [`open_requested`]: a gated `MessageReader` keeps its
//! cursor and would read the word that opened the surface straight back into
//! it. `menuing.rs` carries the argument at length.

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use orbs_shell::ManualReader;

/// The manual, if it is open.
///
/// A resource rather than a component, for `Standing`'s reason: there is one
/// screen, it is modal, and an entity would invite a second.
#[derive(Resource, Debug, Default)]
pub(crate) struct Reading {
    reader: Option<ManualReader>,
    /// Whether the manual took this frame's keys, even if it then closed.
    ///
    /// `type_into_menu` declines while `is_open()` but runs after this, with
    /// its own `MessageReader` cursor — so the key that closed the manual was
    /// already gone from `is_open()` and Escape closed the menu underneath it
    /// too. Set for any key this module consumed, taken by `type_into_menu` the
    /// same frame. `orbs-tui` needs none of it: one key, one `Owner`.
    took: bool,
}

impl Reading {
    /// The open manual, to draw.
    pub(crate) const fn get_mut(&mut self) -> Option<&mut ManualReader> {
        self.reader.as_mut()
    }

    /// Whether the manual has the keyboard.
    #[must_use]
    pub(crate) const fn is_open(&self) -> bool {
        self.reader.is_some()
    }

    /// Whether the manual has already answered this frame's keys.
    ///
    /// Read-and-clear: one system asks once a frame, and leaving it set would
    /// make the *next* frame's keys vanish. See [`took`](Self::took).
    pub(crate) const fn took_the_keys(&mut self) -> bool {
        std::mem::replace(&mut self.took, false)
    }

    /// Say the manual consumed a key this frame.
    pub(crate) const fn take_the_keys(&mut self) {
        self.took = true;
    }

    /// Put it up on this book.
    pub(crate) fn open(&mut self, book: Vec<orbs_sim::content::Chapter>) {
        self.reader = Some(ManualReader::of(book));
    }

    /// Swap the book under an open reader, keeping its place.
    pub(crate) fn restock(&mut self, book: Vec<orbs_sim::content::Chapter>) {
        if let Some(reader) = self.reader.as_mut() {
            reader.restock(book);
        }
    }

    /// Take it down.
    pub(crate) fn close(&mut self) {
        self.reader = None;
    }
}

/// The menu asked for the manual, waiting for the book.
///
/// A message rather than the key handler doing it: the book is assembled from
/// the `Sim`, and `type_into_menu` holds none — *"nothing the menu does reaches
/// the world"* is what lets the menu survive a swap.
#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct OpenManualMessage;

/// Open the manual when the menu has asked for it.
pub(crate) fn open_requested(
    mut asked: MessageReader<OpenManualMessage>,
    mut reading: ResMut<Reading>,
    tower: Res<crate::sim::Tower>,
) {
    if asked.read().next().is_none() {
        return;
    }
    reading.open(orbs_shell::manual_book(tower.sim()));
}

/// Re-assemble an open manual when the authored chapters have changed.
///
/// Rule 6's last mile here: without it an open reader kept painting the book it
/// was opened on, so a writer editing `manual.toml` saw the log say *reloaded*
/// and the screen say otherwise.
///
/// Gated on the message rather than on `Tower`, which changes every tick — and
/// on the reader being open, because `manual_book` walks every prose key.
pub(crate) fn restock_requested(
    mut changed: MessageReader<crate::sim::ManualChangedMessage>,
    mut reading: ResMut<Reading>,
    tower: Res<crate::sim::Tower>,
) {
    if changed.read().next().is_none() {
        return;
    }
    reading.restock(orbs_shell::manual_book(tower.sim()));
}

/// Run condition: the manual is open and might need re-assembling.
pub(crate) fn reading_the_manual(reading: Res<Reading>) -> bool {
    reading.is_open()
}

/// Run condition: nothing is holding the keyboard *above* the transcript.
///
/// `scroll_back`/`scroll_forward` are gated on `booted` alone, so with a
/// chapter open one `PgUp` paged the chapter and the transcript underneath it
/// at once.
///
/// The manual only, not every surface: the transcript scrolling from behind the
/// editor and the weave is a feature
/// (`the_transcript_scrolls_without_asking_for_permission`). What differs here
/// is that the manual answers the same keys.
pub(crate) fn not_reading_the_manual(reading: Res<Reading>) -> bool {
    !reading.is_open()
}

/// Feed keys to the open manual.
///
/// Ungated and clearing, ordered before [`open_requested`] — see the module doc,
/// and `menuing.rs` for the shipped defect that shape exists to prevent.
pub(crate) fn type_into_manual(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    quiet: Res<super::input::Quiet>,
    mut reading: ResMut<Reading>,
) {
    let stale_chord = super::input::chord_is_stale(quiet.gap());
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    let mut closed = false;
    let mut took = false;
    {
        let Some(reader) = reading.get_mut() else {
            keys.clear();
            return;
        };
        for event in keys.read() {
            if event.state != ButtonState::Pressed {
                continue;
            }
            if chord && !stale_chord {
                continue;
            }
            let Some(key) = super::input::pressed(event) else {
                continue;
            };
            took = true;
            closed |= orbs_shell::apply_to_manual(&key, reader).is_some();
        }
    }
    if took {
        // Before the close, so the menu still hears about it — see
        // `Reading::took`.
        reading.take_the_keys();
    }
    if closed {
        reading.close();
    }
}
