//! Drawing the spell editor.
//!
//! Here rather than in `orbs-render` because that crate holds no screen painter
//! at all: `Painter` is a primitive with a deliberately empty `[dependencies]`,
//! and every real surface is drawn frontend-side. Rule 2 still holds —
//! `ScreenLayout` hands this function a rectangle and it draws inside it.
//!
//! §14 forbids information carried only visually, which is why the caret's
//! position is *stated* on the status row rather than only brightened.
//!
//! Reaching the stream is not the same as reading well. Every line was two
//! [`Painter::span`]s — the gutter number, then the code — and a span is one
//! utterance, so a reader heard `1` … `attend laboratory` … `2` …. A row is
//! [`Painter::announce`]d once, whole, and its runs drawn with the silent
//! [`Painter::glyphs`]: §14 will not have colour decide where a sentence ends
//! (§19).

use orbs_render::{Frame, Intensity, Painter, Pos, Rect, Role, Span, Style, UtteranceKind};
use orbs_sim::Prose;

use super::Editor;
use super::editor::{Complaint, Mode};

/// How wide the line-number gutter is: a marker column, three digits, a space.
///
/// Five cells, split differently from the four digits and a space it was — that
/// put the marker to the *right* of the number. Three digits spells every line
/// anyone will write, and a sixth column would cost the 80-cell floor.
const GUTTER: u16 = 5;

/// The marker column, and what goes in it.
///
/// Left of the number, so the number does not move: on the right the digits
/// shifted a column the instant an invocation reached the line.
///
/// `»` is CP437 0xAF — see [`title`] for the em-dash this file already lost.
const MARKER: char = '\u{bb}';

/// The smallest pane an editor can honestly be drawn in.
///
/// Two border rows, one status row, and at least one line of the spell — plus a
/// gutter and room for a word. Below this the answer is to say so rather than
/// draw something misleading; the buffer is untouched either way.
const MIN_ROWS: u16 = 4;
const MIN_COLS: u16 = GUTTER + 8;

/// Draw the editor into `pane`.
///
/// Returns where the caret belongs, so the caller can place it on the frame. Its
/// position is information (rule 2); only its blink is the frontend's own.
pub fn paint(
    frame: &mut Frame,
    editor: &mut Editor,
    pane: Rect,
    prose: &Prose,
) -> Option<(u16, u16)> {
    if pane.is_empty() || pane.rows < MIN_ROWS || pane.cols < MIN_COLS {
        return None;
    }
    // The guide takes columns off the right or none at all — the tower rail's
    // rule. At the 80×22 floor the buffer needs every column, so the guide is
    // absent there by design.
    let (pane, aside) = split_for_guide(pane, editor.guiding());
    if let Some(aside) = aside {
        crate::guide::paint(frame, aside, editor.guide(), prose);
    }

    let mut painter = frame.painter(pane);
    let area = painter.area();
    painter.border(area, Some(&title(editor, prose)), Style::DIM);

    // Inside the border: one row of chrome at the bottom for the status line.
    let text_rows = area.rows.saturating_sub(3);
    editor.scroll_to(text_rows as usize);

    let text_col = area.col.saturating_add(1).saturating_add(GUTTER);
    let width = area.cols.saturating_sub(2 + GUTTER);
    let (caret_row, caret_column) = editor.caret();
    // Which buffer row the orb is executing, zero-based to match `index`. A
    // spell's lines are numbered from one everywhere a player sees them.
    let running = editor
        .running_line()
        .and_then(|line| usize::try_from(line).ok())
        .and_then(|line| line.checked_sub(1));
    let mut caret = None;

    for row in 0..text_rows {
        let index = editor.top() + row as usize;
        if index >= editor.len() {
            break;
        }
        let y = area.row.saturating_add(1).saturating_add(row);
        let here = running == Some(index);

        // Its own column, left of the number, and drawn separately because it
        // carries `Role::Success` — merging it into the number would tint the
        // digits or lose the colour.
        //
        // Silent: the border already says *the orb is on line 8*.
        if here {
            painter.glyphs(
                Pos::new(area.col.saturating_add(1), y),
                &MARKER.to_string(),
                Style::default()
                    .with_role(Role::Success)
                    .with_intensity(Intensity::Bright),
            );
        }
        // The number is a fact about which line you are on, so it is spoken with
        // the line rather than beside it — `1 attend laboratory`, not a bare
        // command with no position. Same column with or without the marker.
        //
        // Drawn structurally here and announced once below; as two `span`s it
        // was two utterances (§19, `0.3.23`).
        painter.glyphs(
            Pos::new(area.col.saturating_add(2), y),
            &format!("{:>3} ", index + 1),
            if here { Style::default() } else { Style::DIM },
        );
        // `interpret` draws the orb's reading in place of the line, at the same
        // numbers, so the two views line up. A line it cannot read shows as the
        // player wrote it, and the mark below says so.
        let reading = editor.reading(index);
        let text = match (editor.mode(), reading) {
            // Clipped through `arriving` like the buffer's own lines, which cuts
            // on a character boundary — a byte slice can split `é` and panic.
            // `≈` marks a line the orb read for you, the glyph the prompt draws
            // on a divined command. Without it a line read correctly and one
            // read wrongly look identical.
            //
            // A glyph rather than a colour, because §14 will not have colour
            // carry information.
            (Mode::Reading, Some(reading)) => {
                let heard = if reading.was.is_some() {
                    format!("≈ {}", reading.heard)
                } else {
                    reading.heard.clone()
                };
                orbs_render::arriving(&heard, u32::from(width)).to_owned()
            }
            _ => editor.visible(index, width).to_owned(),
        };
        // The fault is on the text, not in the gutter: a sixth gutter cell would
        // cost the 80-column floor, and the marker cell already holds the
        // running-line `»`.
        let unread = reading.is_some_and(|reading| reading.fault.is_some());
        let style = match (here, unread) {
            (_, true) => Style::default().with_role(Role::Danger),
            (true, false) => Style::default().with_intensity(Intensity::Bright),
            (false, false) => Style::default(),
        };
        // One utterance for the whole row, number included, in the order the eye
        // reads it. `announce` rather than a wider `span` because the two runs
        // carry different styles and a span carries one.
        painter.announce(
            UtteranceKind::Text,
            style.role,
            &format!("{} {text}", index + 1),
        );
        // A run at a time, so each part of the line carries its own hue — and
        // silently, so the row stays the one utterance announced above:
        // highlighting multiplies the runs per line by five or six.
        //
        // §14 is satisfied by the text, not the colour. See `parser::lexeme`.
        paint_line(&mut painter, Pos::new(text_col, y), &text, style);

        if index == caret_row && editor.mode() == Mode::Editing {
            // Clamped: a caret past the right edge would land on the border.
            // `width - 1` because `width` is a count and this is an index —
            // clamping to `width` parked a full-width line's caret on it.
            let column = u16::try_from(caret_column)
                .unwrap_or(u16::MAX)
                .min(width.saturating_sub(1));
            caret = Some((text_col.saturating_add(column), y));
        }
    }

    status(&mut painter, editor, area, prose);

    // In command state the caret belongs on the command row, because that is
    // where typing goes — its position is information (rule 2).
    if editor.mode() == Mode::Command {
        let typed = u16::try_from(editor.command().chars().count()).unwrap_or(u16::MAX);
        caret = Some((
            area.col
                .saturating_add(TYPED)
                .saturating_add(typed)
                .min(area.col.saturating_add(area.cols).saturating_sub(2)),
            area.row.saturating_add(area.rows).saturating_sub(2),
        ));
    }
    caret
}

/// How wide the guide is when it is there.
///
/// Wide enough for a definition to wrap into two or three rows, and narrow
/// enough to leave the buffer what a spell needs — `threading`'s longest line
/// is 62 cells.
const GUIDE: u16 = 30;

/// The code columns the buffer keeps before a guide is allowed to exist at all.
///
/// Measured against the content: `threading`'s longest line is 62 cells, so a
/// narrower buffer starts wrapping ordinary spells.
const BUFFER_FLOOR: u16 = 60;

/// The narrowest pane that can hold a buffer *and* a guide.
///
/// Derived rather than picked: the buffer's floor, its gutter, its two borders
/// and the guide — 97, so the game's own editor pane (104 columns) hosts one and
/// the 80×22 authoring floor does not.
///
/// Below it the guide yields whole, the tower rail's rule: a reference squeezed
/// into the columns a spell needs has stopped being one.
const HOSTS_GUIDE: u16 = BUFFER_FLOOR + GUTTER + 2 + GUIDE;

/// Split `pane` into the buffer's part and the guide's, if it fits.
const fn split_for_guide(pane: Rect, guiding: bool) -> (Rect, Option<Rect>) {
    if !guiding || pane.cols < HOSTS_GUIDE || pane.rows < MIN_ROWS {
        return (pane, None);
    }
    let kept = pane.cols.saturating_sub(GUIDE);
    (
        Rect::new(pane.col, pane.row, kept, pane.rows),
        Some(Rect::new(
            pane.col.saturating_add(kept),
            pane.row,
            GUIDE,
            pane.rows,
        )),
    )
}

/// Draw one line of a spell, each run in its own hue.
///
/// `style` arrives carrying what the *row* means — `Role::Danger` for a line
/// `interpret` could not read, `Intensity::Bright` for the line the orb is on —
/// and a run's `Lexeme` layers under it, never over: `Style::lexed` yields
/// nothing on an accented cell, so an unreadable line stays wholly red.
///
/// The plain line goes down first, so spacing and anything the lexer declined to
/// name is already there; the runs then redraw in their own hue.
///
/// Offsets are taken against the clipped text `Editor::visible` returns, so they
/// always land, and converted from bytes to columns — `é` is two bytes.
fn paint_line(painter: &mut Painter<'_>, at: Pos, text: &str, style: Style) {
    painter.glyphs(at, text, style);
    for run in orbs_sim::parser::lex(text) {
        let Some(word) = text.get(run.start..run.end) else {
            continue;
        };
        let Ok(offset) = u16::try_from(text[..run.start].chars().count()) else {
            continue;
        };
        let col = at.col.saturating_add(offset);
        let lit = crate::lexing::lit(style, run.kind);
        let drawn = painter.glyphs(Pos::new(col, at.row), word, lit);
        // Over the region the glyphs actually took: a run clipped at the pane
        // edge must not claim colour it did not draw, and the buffer sits
        // beside the guide with a seam between them.
        //
        // And only where the row lets a run style itself: `lit` declines on an
        // accent and on the running line, so a hue cannot paint over the signal
        // saying *this line is broken*. Asked here rather than at the frontend
        // so the two builds cannot disagree.
        if crate::lexing::takes_syntax(style) {
            painter.lit(Rect::new(col, at.row, drawn, 1), run.kind);
        }
    }
}

/// The border title: the spell, where it runs, whether it is unsaved, and where
/// a running invocation has reached.
///
/// No em-dash: CP437 has no `—`, so one drawn here comes out as `?`. The
/// separator is a word, which reads better anyway: a spell is *in* a domain.
///
/// The running line is said here as well as marked in the gutter, because a `»`
/// in a column is the visual-only fact §14 forbids.
fn title(editor: &Editor, prose: &Prose) -> String {
    let dirty = if editor.is_dirty() { " *" } else { "" };
    let mut title = format!("{} in {}{dirty}", editor.name(), editor.domain());
    if let Some(line) = editor.running_line() {
        title.push(' ');
        title.push_str(&prose.line("editor_at_line", &[("count", &line.to_string())]));
    }
    title
}

/// Where the command row's `> ` marker begins, inside the border.
const PROMPT: u16 = 1;

/// Where a typed character lands, and therefore where the caret rests.
///
/// Past `> `, which is [`PROMPT`] and the space after it.
const TYPED: u16 = PROMPT + 2;

/// Where the row's standing message begins — the word list, a complaint, the
/// unread count.
///
/// Clear of [`TYPED`] by three cells: the listing used to begin at [`PROMPT`]
/// like the typed line, so with nothing typed the block caret sat on the `i` of
/// `edit` — the first word a player reads.
///
/// Not merged with the typed column: a caret sits *after* what has been typed
/// and *beside* what is merely offered.
const SAYING: u16 = TYPED + 3;

/// The bottom row: the `:` line if one is open, a complaint if one is pending,
/// and otherwise the way out.
///
/// It always says something: §6 forbids a dead end, and a player who does not
/// know `:q` has nowhere else to find out.
fn status(painter: &mut Painter<'_>, editor: &Editor, area: Rect, prose: &Prose) {
    let y = area.row.saturating_add(area.rows).saturating_sub(2);
    let width = usize::from(area.cols.saturating_sub(2));

    // The row that must never be blank; in command state it is the whole
    // interface (§6 forbids a dead end).
    //
    // A complaint outranks the help, and a half-typed word outranks both. The
    // unread count sits between: a mark in a column is the visual-only fact §14
    // forbids, so it is *said* — but not over a complaint the player just
    // caused.
    let unread = editor.unread();
    let typing = editor.mode() == Mode::Command && !editor.command().is_empty();
    let (text, style) = match (editor.mode(), editor.complaint()) {
        (Mode::Command, _) if typing => (format!("> {}", editor.command()), Style::default()),
        (_, Some(Complaint::Unknown(word))) => (
            prose.line("editor_unknown", &[("detail", word)]),
            Style::default().with_role(Role::Danger),
        ),
        (Mode::Reading, None) => (prose.line("editor_reading", &[]), Style::DIM),
        (_, None) if unread > 0 => (
            prose.line("editor_unread", &[("count", &unread.to_string())]),
            Style::default().with_role(Role::Danger),
        ),
        (Mode::Command, None) => (prose.line("editor_words", &[]), Style::DIM),
        (Mode::Editing, None) => (prose.line("editor_editing", &[]), Style::DIM),
    };

    // A standing message starts clear of the caret; typed text starts under it.
    // Only the message moves, because the caret has to sit *after* what has
    // been typed.
    let from = if typing { PROMPT } else { SAYING };

    let position = editor.position();
    // The position is pinned right and the message takes what is left — at the
    // 80-cell floor ~63 cells, which `editor_words` is written to fit. The
    // indent comes out of the room, or a shifted message would run under the
    // position rather than being clipped short of it.
    let room = u32::try_from(
        width
            .saturating_sub(position.chars().count() + 1)
            .saturating_sub(usize::from(from - PROMPT)),
    )
    .unwrap_or(u32::MAX);
    painter.span(
        Pos::new(area.col.saturating_add(from), y),
        &Span::new(orbs_render::arriving(&text, room)).with_style(style),
    );

    let at = area
        .col
        .saturating_add(area.cols)
        .saturating_sub(1)
        .saturating_sub(u16::try_from(position.chars().count()).unwrap_or(0));
    painter.span(
        Pos::new(at, y),
        &Span::new(&position).with_style(Style::DIM.with_intensity(Intensity::Dim)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::GridSize;

    /// The editor, painted, with a spell in it.
    fn painted(lines: &[&str]) -> Frame {
        let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
        let mut editor = Editor::open("check.spell", "archive", &lines);
        let mut frame = Frame::new(GridSize::new(80, 20));
        let pane = Rect::new(0, 0, 80, 20);
        paint(&mut frame, &mut editor, pane, &Prose::builtin());
        frame
    }

    /// The caret does not sit on the words it is offering.
    ///
    /// It sat on the `i` of `edit`: command state opens with nothing typed, and
    /// the listing began at the same column as the typing.
    #[test]
    fn the_command_caret_rests_on_a_blank_cell() {
        let lines = vec!["repeat 3".to_owned()];
        let mut editor = Editor::open("check.spell", "archive", &lines);
        let mut frame = Frame::new(GridSize::new(80, 20));
        let pane = Rect::new(0, 0, 80, 20);
        let caret = paint(&mut frame, &mut editor, pane, &Prose::builtin())
            .expect("command state puts the caret on the command row");

        let under = frame
            .cell(Pos::new(caret.0, caret.1))
            .expect("the caret is on the grid")
            .glyph;
        assert_eq!(under, ' ', "the caret is sitting on a word it is offering");

        // ...and the words are still there, three cells to its right.
        let row: String = (0..80)
            .filter_map(|col| frame.cell(Pos::new(col, caret.1)))
            .map(|cell| cell.glyph)
            .collect();
        assert!(row.contains("edit"), "the listing went missing: {row:?}");
    }

    /// Typing takes the row back, and the caret follows what was typed.
    ///
    /// The two columns want opposite things: a caret sits *after* what has been
    /// typed and *beside* what is merely offered.
    #[test]
    fn a_typed_command_keeps_the_caret_behind_it() {
        let lines = vec!["repeat 3".to_owned()];
        let mut editor = Editor::open("check.spell", "archive", &lines);
        for glyph in "gui".chars() {
            editor.type_text(&glyph.to_string());
        }
        let mut frame = Frame::new(GridSize::new(80, 20));
        let pane = Rect::new(0, 0, 80, 20);
        let caret = paint(&mut frame, &mut editor, pane, &Prose::builtin())
            .expect("command state puts the caret on the command row");

        let row: String = (0..80)
            .filter_map(|col| frame.cell(Pos::new(col, caret.1)))
            .map(|cell| cell.glyph)
            .collect();
        assert!(row.contains("> gui"), "the typed line moved: {row:?}");
        assert_eq!(
            caret.0,
            PROMPT + 2 + 3,
            "the caret is not behind what was typed",
        );
    }

    /// The style of the cell holding the first character of `word` on `row`.
    fn weight_of(frame: &Frame, row: u16, word: &str, line: &str) -> Intensity {
        let column = line.find(word).expect("the word is on the line");
        let at = Pos::new(
            1 + GUTTER + u16::try_from(column).expect("a short line"),
            row,
        );
        frame.cell(at).expect("a painted cell").style.intensity
    }

    #[test]
    fn structure_is_bright_content_is_normal_and_filler_recedes() {
        // §4 gives ordinary text intensity and nothing else, so three weights is
        // all highlighting has: where a block opens, what it does, and what the
        // orb strips before reading.
        let line = "repeat until the stacks is idle";
        let frame = painted(&[line]);

        assert_eq!(weight_of(&frame, 1, "repeat", line), Intensity::Bright);
        assert_eq!(weight_of(&frame, 1, "until", line), Intensity::Bright);
        assert_eq!(weight_of(&frame, 1, "the", line), Intensity::Dim);
        assert_eq!(weight_of(&frame, 1, "stacks", line), Intensity::Normal);
    }

    // The arbitration these two tested by hand — a fault outranking the
    // highlighting, the running line staying uniformly bright — moved to
    // `lexing` with `lit`. What stays is the painted frame, the gate that
    // catches a painter calling it wrongly.

    #[test]
    fn a_comment_recedes_whole() {
        // One run, not four words — or `laps` would be drawn as a name inside a
        // sentence the orb never reads.
        let line = "# two laps";
        let frame = painted(&[line]);
        for word in ["#", "two", "laps"] {
            assert_eq!(weight_of(&frame, 1, word, line), Intensity::Dim, "{word}");
        }
    }
}

#[cfg(test)]
mod guiding {
    use super::*;
    use orbs_render::GridSize;

    /// The editor, painted at a given width, with the guide as `guiding` says.
    fn painted(cols: u16, guiding: bool) -> Frame {
        let lines = vec!["repeat 3".to_owned()];
        let mut editor = Editor::open("check.spell", "laboratory", &lines);
        editor.refresh(&orbs_sim::Sim::new(1));
        if !guiding {
            // The editor opens in command state, so this is what a player types.
            for glyph in "guide".chars() {
                editor.type_text(&glyph.to_string());
            }
            editor.enter();
        }
        let mut frame = Frame::new(GridSize::new(cols, 20));
        paint(
            &mut frame,
            &mut editor,
            Rect::new(0, 0, cols, 20),
            &Prose::builtin(),
        );
        frame
    }

    /// Every glyph on the frame, row by row.
    fn drawn(frame: &Frame, cols: u16) -> String {
        (0..20)
            .map(|row| {
                (0..cols)
                    .map(|col| {
                        frame
                            .cell(Pos::new(col, row))
                            .map_or(' ', |cell| cell.glyph)
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn the_guide_is_open_to_begin_with() {
        // A reference nobody knows to ask for helps nobody, so it is open and
        // `guide` closes it.
        let frame = painted(104, true);
        assert!(drawn(&frame, 104).contains("the language"));
    }

    #[test]
    fn the_word_closes_it_and_the_buffer_takes_the_room() {
        let frame = painted(104, false);
        let screen = drawn(&frame, 104);
        assert!(!screen.contains("the language"), "the guide stayed open");
        assert!(screen.contains("repeat 3"), "the buffer went missing");
    }

    /// Yields whole below the width that can host it — the tower rail's rule: a
    /// reference squeezed to eight columns has stopped being one. The 80×22
    /// authoring floor is where this bites.
    #[test]
    fn it_yields_whole_at_the_authoring_floor() {
        let frame = painted(80, true);
        let screen = drawn(&frame, 80);
        assert!(
            !screen.contains("the language"),
            "the guide cramped itself in at the floor instead of yielding",
        );
        assert!(screen.contains("repeat 3"), "the buffer went missing");
    }

    #[test]
    fn every_row_of_the_guide_is_one_utterance() {
        // §14: the guide is *content*, so it must reach a reader — and a
        // definition wrapped across three rows is one sentence, not three.
        let frame = painted(104, true);
        let spoken: Vec<&str> = frame
            .speech()
            .utterances()
            .filter(|utterance| utterance.kind == orbs_render::UtteranceKind::Guide)
            .map(|utterance| utterance.text)
            .collect();
        assert!(!spoken.is_empty(), "the guide said nothing at all");
        assert!(
            spoken.iter().any(|line| line.contains("repeat")),
            "the guide drew words it never spoke: {spoken:?}",
        );
    }
}
