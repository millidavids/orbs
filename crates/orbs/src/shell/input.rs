//! The line the player is typing.
//!
//! # Read `text`, not `logical_key`
//!
//! The spacebar's logical key is [`Key::Space`], **not** `Key::Character(" ")`,
//! on every platform winit supports. A buffer built by matching
//! `Key::Character` therefore silently loses the space bar, and `look around`,
//! `sift march feed.log` — every multi-word command in DESIGN.md §6.1 — becomes
//! untypeable. Nothing in a test suite catches that; it dies on the first
//! keystroke a person types.
//!
//! [`KeyboardInput::text`] carries the correct locale-mapped character for every
//! layout, handles the Windows dead-key case where one press yields two
//! characters, and is `Some(" ")` for space. `logical_key` is consulted for
//! exactly two things: Enter and Backspace.
//!
//! # `text` is not safe to append
//!
//! It contains control characters — winit documents Enter as `Some("\r")`, and
//! Tab and Escape arrive the same way. A control character in the buffer would
//! reach [`Painter::put_str`](orbs_render::Painter), occupy a cell, and draw
//! **nothing**, because the renderer skips glyphs outside the CP437 repertoire.
//! The caret would drift away from the text with no visible cause.
//!
//! So every character is filtered through [`orbs_render::is_renderable`], which
//! is the repertoire itself rather than an approximation of it: `is_ascii_graphic`
//! would also throw away the accented range the font can draw.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;

/// The parser's own limit (§6). Matching it here means the buffer never holds
/// something the parser would truncate — a shorter cap would silently change
/// what a command means, and a longer one would let the player type into a void.
const MAX_INPUT: usize = 512;

/// The command line as it currently stands.
#[derive(Resource, Debug, Default)]
pub(crate) struct Line {
    text: String,
}

impl Line {
    /// The tail of the line that fits `width` cells, and the caret's column
    /// within it.
    ///
    /// Without this the line has no viewport: at the 80×22 floor the prompt
    /// leaves 72 cells, past which `put_str` clips silently *and*
    /// [`Frame::set_cursor`](orbs_render::Frame::set_cursor) refuses an off-grid
    /// position — so the player types into a dead line with no caret and no
    /// explanation. `sift "march north" /tower/laboratory/feed.log` is 44
    /// characters, so 72 is not a theoretical limit.
    ///
    /// One cell is always reserved for the caret, which is what makes the line
    /// scroll rather than stall.
    pub(crate) fn viewport(&self, width: u16) -> (&str, u16) {
        if width == 0 {
            return ("", 0);
        }
        let width = usize::from(width);
        let count = self.text.chars().count();
        if count < width {
            return (&self.text, to_col(count));
        }
        let skip = count + 1 - width;
        let start = self
            .text
            .char_indices()
            .nth(skip)
            .map_or(self.text.len(), |(index, _)| index);
        (&self.text[start..], to_col(width - 1))
    }
}

fn to_col(count: usize) -> u16 {
    u16::try_from(count).unwrap_or(u16::MAX)
}

/// A line the player finished.
#[derive(Message, Debug, Clone)]
pub(crate) struct SubmittedMessage {
    /// What they typed, verbatim.
    pub(crate) line: String,
}

/// Feed keystrokes into the line.
///
/// Runs in `Update`, never `FixedUpdate`: ticks are 1 Hz (§5.0) and typing
/// sampled at 1 Hz would be unusable. Nothing is lost by the split — Bevy runs
/// the fixed loop *before* `Update` in a frame, so a keystroke can never be
/// observed between two ticks of the same frame.
pub(crate) fn type_into_line(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    mut line: ResMut<Line>,
    mut submitted: MessageWriter<SubmittedMessage>,
) {
    // Chords are commands, not text. Alt is deliberately **not** in this list:
    // AltGr is how European layouts type `@`, `#` and `\`, and guarding on it
    // would make those characters untypeable for the players who need them.
    let chord = held.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    for event in keys.read() {
        // `repeat` is deliberately not filtered: held Backspace should delete
        // more than one character, which is what every text field does.
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Enter => {
                let finished = std::mem::take(&mut line.text);
                submitted.write(SubmittedMessage { line: finished });
            }
            Key::Backspace => {
                line.text.pop();
            }
            _ => {
                if chord {
                    continue;
                }
                let Some(text) = &event.text else {
                    continue;
                };
                for glyph in text.chars().filter(|c| orbs_render::is_renderable(*c)) {
                    if line.text.chars().count() >= MAX_INPUT {
                        break;
                    }
                    line.text.push(glyph);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_of(text: &str) -> Line {
        Line {
            text: text.to_owned(),
        }
    }

    #[test]
    fn a_short_line_is_shown_whole_with_the_caret_after_it() {
        let line = line_of("survey");
        assert_eq!(line.viewport(40), ("survey", 6));
    }

    #[test]
    fn a_long_line_scrolls_and_keeps_its_caret_on_screen() {
        // The failure this exists to prevent: past the pane width the text stops
        // appearing and `set_cursor` refuses an off-grid position, so the player
        // types into a dead line.
        let line = line_of("abcdefghij");
        let (visible, caret) = line.viewport(5);
        assert_eq!(visible, "ghij", "the tail must stay visible");
        assert_eq!(caret, 4, "the caret must stay inside the viewport");
        assert!(usize::from(caret) < 5);
    }

    #[test]
    fn the_caret_never_leaves_the_grid_at_any_length() {
        for length in 0..200usize {
            let line = line_of(&"x".repeat(length));
            for width in [1u16, 8, 72] {
                let (visible, caret) = line.viewport(width);
                assert!(caret < width, "len {length} width {width} caret {caret}");
                assert!(visible.chars().count() < usize::from(width) + 1);
            }
        }
    }

    #[test]
    fn a_zero_width_viewport_is_empty_rather_than_a_panic() {
        assert_eq!(line_of("survey").viewport(0), ("", 0));
    }

    #[test]
    fn a_multibyte_glyph_is_not_split() {
        // `░` is three bytes and one cell. Slicing the tail by byte offset would
        // panic mid-character; the viewport has to seek by `char_indices`.
        let line = line_of("░░░░░");
        let (visible, caret) = line.viewport(3);
        assert_eq!(visible, "░░", "one cell is reserved for the caret");
        assert_eq!(caret, 2);
    }
}
