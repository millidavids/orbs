//! A figure of syllables travelling toward the circle — what a chant looks like.
//!
//! Drawn beside the transcript whenever a chant is running, which is the rule
//! the archive's map, the lens's sheet and the sanctum's board all follow: **the
//! picture is not gated on a word**, because watching a bound spell sing one and
//! singing it yourself are different activities and only the second involves
//! typing.
//!
//! # It carries nothing the readings lack, and one thing they cannot arrange
//!
//! Rule 2's line. Every syllable here is one a spell could read off `next`, and
//! the count under the rule is what `survey circle` says. What the picture adds
//! is that the syllables to come are legible **at once and in order**, which is
//! the whole of what makes a figure singable — the same thing the sanctum's
//! board adds by holding three stations side by side.
//!
//! **Without it the domain is unplayable by hand**, and that is not a figure of
//! speech: the aperture moves every tick and `survey` costs one, so a player
//! with no board cannot ever learn what is coming in time to answer it.
//!
//! # A syllable's identity is its glyph, never its colour
//!
//! §14 forbids meaning that lives only in hue. The four arrows are four shapes,
//! so a figure reads in greyscale, in a dump, and under every deficiency —
//! `Syllable::glyph` is where they are chosen and the sim holds a test that they
//! are all in the code page. The tint here is one colour for the lane that is
//! about to land and is pure enrichment: take it away and the aperture row is
//! still the row above the rule.

use crate::style::{Style, Tint};

/// The lane rule — where a syllable lands, at the top of the board. CP437 0xC4.
pub const APERTURE: char = '\u{2500}';

/// What a struck syllable leaves behind.
///
/// **The lens's pegs, deliberately.** A filled mark against a hollow one is
/// already the game's way of saying *this one landed and that one did not*, and
/// a second vocabulary for the same idea would be one more thing to learn. The
/// first draft used `ú`/`ø`, and `every_glyph_the_figure_draws_is_in_the_code_page`
/// refused them on its first run — the `▪`/`►` defect §19 records twice, caught
/// this time before it drew a `?`.
pub const STRUCK: char = '•';

/// What a missed one leaves.
pub const MISSED: char = '○';

/// The lane about to land, and the only tint on the board.
pub const TINT: Tint = Tint::Violet;

/// How many syllables of the figure are on screen at once.
///
/// **Fixed, and deliberately not sized to what is left.** A board that shrank as
/// the figure ran out would move the aperture rule every tick, and the aperture
/// rule is the one thing a player reads position against — the sanctum's ground
/// line, exactly.
///
/// Eight is far enough ahead to plan a phrase and near enough that the row
/// nearest the rule is unmistakably next.
pub const AHEAD: usize = 8;

/// A chant, as the board needs it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Figure {
    /// The syllables still to come, nearest first, as lane indices `0..4`.
    ///
    /// Nearest first because that is the order the player answers them in; the
    /// painter reverses it to draw travel *toward* the rule.
    pub coming: Vec<usize>,
    /// What each lane is called, left to right.
    ///
    /// **Handed in by the sim, not held here.** `sing skyward` names a lane by
    /// word, so a board whose lanes are unlabelled is one a player has to count
    /// along before they can type — §19 records the lens's sheet paying for
    /// exactly that omission. They are content, which rule 6 keeps out of a
    /// painter.
    pub lanes: Vec<(char, &'static str)>,
    /// How the figure has gone so far, oldest first: `true` struck.
    pub sung: Vec<bool>,
    /// How many ticks the nearest syllable still has before it lands.
    ///
    /// **Nought means it is at the rule and strikeable**, which is the one fact
    /// a player must be able to read and could not. The board was drawn from
    /// `coming` alone, so it was byte-identical for a syllable's whole approach
    /// and then simply lost one — the two ticks where a press *counts* looked
    /// exactly like the two where it answers "too soon", and the timing mechanic
    /// was invisible on the surface built to show it.
    ///
    /// It is **information, not decoration**: the sim publishes the same number
    /// as the `until` reading a spell bides on, so a hand player and a solver
    /// are reading one fact. That is why it rides the view rather than being an
    /// animation a frontend interpolates.
    pub until: u32,
    /// How many syllables are still to come, this one included.
    ///
    /// **Not `coming.len()`**, which is capped at [`AHEAD`] because that is all
    /// the board has rows for. The two were being used interchangeably and a
    /// screen reader was told *"8 to come"* while the board drew *"12 to come"*
    /// — for the first five syllables of every chant, on §14's route.
    pub remaining: usize,
    /// The line under the rule, already written.
    ///
    /// **Handed in rather than composed here**, and it is rule 6 rather than
    /// tidiness: `7 to come, 2 missed` is a sentence, and a sentence built from
    /// literals in a painter is prose that cannot be hot-reloaded and would be
    /// the only authored English in this crate.
    pub tally: String,
}

impl Figure {
    /// One lane's column, in cells.
    ///
    /// Ten, because `earthward` is nine and a column with no gap beside it runs
    /// into its neighbour. The name is what sets this, exactly as the sanctum's
    /// `wellspring` sets its column.
    const COLUMN: u16 = 10;

    /// The whole board's width.
    pub const COLS: u16 = Self::COLUMN * 4;

    /// The header naming the lanes.
    const HEAD: u16 = 1;

    /// The aperture rule, and the tally beneath it.
    const FOOT: u16 = 2;

    /// The rows a board wants, whatever is on it.
    #[must_use]
    pub const fn rows() -> u16 {
        #[allow(clippy::cast_possible_truncation)]
        let ahead = AHEAD as u16;
        Self::HEAD + ahead + Self::FOOT
    }

    /// The board's size in character cells.
    #[must_use]
    pub const fn size() -> (u16, u16) {
        (Self::COLS, Self::rows())
    }

    /// One row of the board, as glyphs and their styles.
    ///
    /// `None` past the end. Row 0 names the lanes, row 1 is the **rule**, and
    /// rows below it are the travelling syllables **nearest first** — so a
    /// syllable enters at the foot of the board and rises to the rule.
    ///
    /// # Upward, and it is the one thing about this board that is not the
    /// sanctum's
    ///
    /// Every other picture in the game is static: a Hanoi stack sits, a maze
    /// waits, a ward sheet accumulates downward like a transcript. This one
    /// *moves*, and which way it moves is a readability decision rather than a
    /// taste one. Notes rising to a fixed line at the top is what every rhythm
    /// game does, and the reason is that the line stays put while the eye tracks
    /// approach — put the line at the bottom and the player reads the newest
    /// arrival and the thing they must answer at the same end of the board.
    ///
    /// It was drawn downward first and looked wrong immediately.
    #[must_use]
    pub fn row(&self, index: usize) -> Option<Vec<(char, Style, Option<Tint>)>> {
        let index = u16::try_from(index).ok()?;
        if index >= Self::rows() {
            return None;
        }
        if index < Self::HEAD {
            return Some(self.header_row());
        }
        let body = index - Self::HEAD;
        if body == 0 {
            return Some(Self::rule_row());
        }
        let distance = usize::from(body - 1);
        if distance < AHEAD {
            return Some(self.travel_row(distance));
        }
        Some(self.tally_row())
    }

    /// The board as plain text, for a dump and for tests.
    #[must_use]
    pub fn line(&self, index: usize) -> Option<String> {
        Some(self.row(index)?.into_iter().map(|(ch, ..)| ch).collect())
    }

    /// The lane names, each centred over its column.
    fn header_row(&self) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = Vec::with_capacity(Self::COLS as usize);
        for (_, name) in &self.lanes {
            push(&mut row, &centred(name, Self::COLUMN), Style::NORMAL, None);
        }
        row
    }

    /// One rank of travelling syllables.
    ///
    /// `distance` counts from the rule: 0 is the syllable about to land.
    fn travel_row(&self, distance: usize) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = Vec::with_capacity(Self::COLS as usize);
        // **The whole figure slides by `until`.** A syllable `until` ticks away
        // sits `until` rows further from the rule, so the picture moves on every
        // tick rather than only when one is consumed — and the row *on* the rule
        // is occupied exactly when a press would strike.
        let travelled = usize::try_from(self.until).unwrap_or(usize::MAX);
        let arriving = distance
            .checked_sub(travelled)
            .and_then(|index| self.coming.get(index))
            .copied();
        for (lane, (glyph, _)) in self.lanes.iter().enumerate() {
            // **Bright and tinted only at the rule.** Everything further off is
            // dim, so the eye is drawn to the one that has to be answered now —
            // and the *position* still says which it is with the colour gone,
            // which is what §14 asks of it.
            let (style, tint) = if distance == 0 {
                (Style::BRIGHT, Some(TINT))
            } else {
                (Style::DIM, None)
            };
            let cell = if arriving == Some(lane) {
                centred(&glyph.to_string(), Self::COLUMN)
            } else {
                " ".repeat(Self::COLUMN as usize)
            };
            push(&mut row, &cell, style, tint);
        }
        row
    }

    /// The rule the syllables land on.
    fn rule_row() -> Vec<(char, Style, Option<Tint>)> {
        let mut row = Vec::with_capacity(Self::COLS as usize);
        push(
            &mut row,
            &APERTURE.to_string().repeat(Self::COLS as usize),
            Style::NORMAL,
            None,
        );
        row
    }

    /// How the figure has gone, then what the sim called it.
    ///
    /// The pegs are the lens's idea: a run of marks is a shape you can take in
    /// without counting, where a number has to be read.
    fn tally_row(&self) -> Vec<(char, Style, Option<Tint>)> {
        let mut row = Vec::with_capacity(Self::COLS as usize);
        let pegs: String = self
            .sung
            .iter()
            .map(|struck| if *struck { STRUCK } else { MISSED })
            .collect();
        let line = format!("{pegs}  {}", self.tally);
        push(&mut row, &centred(&line, Self::COLS), Style::NORMAL, None);
        row
    }
}

/// Centre `text` in `width` cells, clipping if it will not fit.
fn centred(text: &str, width: u16) -> String {
    let width = width as usize;
    let len = text.chars().count();
    if len >= width {
        return text.chars().take(width).collect();
    }
    let left = (width - len) / 2;
    let right = width - len - left;
    format!("{}{text}{}", " ".repeat(left), " ".repeat(right))
}

fn push(row: &mut Vec<(char, Style, Option<Tint>)>, text: &str, style: Style, tint: Option<Tint>) {
    row.extend(text.chars().map(|glyph| (glyph, style, tint)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lanes() -> Vec<(char, &'static str)> {
        vec![
            ('\u{25C4}', "leftward"),
            ('\u{25B2}', "skyward"),
            ('\u{25BC}', "earthward"),
            ('\u{25BA}', "rightward"),
        ]
    }

    /// A figure with its nearest syllable **on** the rule.
    fn figure() -> Figure {
        landing_in(0)
    }

    /// ...and one whose nearest syllable is `until` ticks off.
    fn landing_in(until: u32) -> Figure {
        Figure {
            coming: vec![1, 3, 0],
            lanes: lanes(),
            sung: vec![true, true, false],
            until,
            remaining: 3,
            tally: "3 to come".to_owned(),
        }
    }

    #[test]
    fn every_row_is_exactly_the_boards_width() {
        let figure = figure();
        for index in 0..Figure::rows() as usize {
            let line = figure.line(index).expect("a row");
            assert_eq!(
                line.chars().count(),
                Figure::COLS as usize,
                "row {index}: {line:?}",
            );
        }
        assert!(figure.line(Figure::rows() as usize).is_none());
    }

    /// The header names every lane a player can type, which is the omission
    /// §19 records the lens's sheet shipping with.
    #[test]
    fn the_header_names_every_lane() {
        let head = figure().line(0).expect("a header");
        for (_, name) in lanes() {
            assert!(head.contains(name), "{name} is not on the board: {head:?}");
        }
    }

    /// **They rise.** The rule is at the top and the syllable about to land sits
    /// directly beneath it, so a player reads approach from the foot upward.
    #[test]
    fn the_rule_is_at_the_top_and_the_next_syllable_is_under_it() {
        let figure = figure();
        let rule = usize::from(Figure::HEAD);
        assert!(
            figure.line(rule).expect("the rule").starts_with(APERTURE),
            "the rule is not directly under the lane names",
        );
        let nearest = figure.line(rule + 1).expect("the row under the rule");
        assert!(
            nearest.contains('\u{25B2}'),
            "lane 1 is next and is not under the rule: {nearest:?}",
        );
    }

    /// ...and the one after it is further down, which is what "rising" means.
    #[test]
    fn a_later_syllable_sits_further_from_the_rule() {
        let figure = figure();
        let rule = usize::from(Figure::HEAD);
        let second = figure.line(rule + 2).expect("two below the rule");
        assert!(
            second.contains('\u{25BA}'),
            "lane 3 is second and is not two below the rule: {second:?}",
        );
    }

    /// **The board moves between landings**, which is the whole of the timing
    /// mechanic being visible.
    ///
    /// It did not: `travel_row` read `coming` alone, so a syllable's four-tick
    /// approach drew four identical boards and then the syllable was gone. The
    /// two ticks where a press *strikes* looked exactly like the two where it
    /// answers "too soon".
    #[test]
    fn a_syllable_two_ticks_off_sits_two_rows_from_the_rule() {
        let rule = usize::from(Figure::HEAD);
        let near = landing_in(0);
        let far = landing_in(2);

        assert!(
            near.line(rule + 1).expect("a row").contains('\u{25B2}'),
            "with `until` nought, lane 1 is not on the rule",
        );
        assert!(
            far.line(rule + 1).expect("a row").trim().is_empty(),
            "with `until` two, the rule row is occupied",
        );
        assert!(
            far.line(rule + 3).expect("a row").contains('\u{25B2}'),
            "lane 1 is not two rows further off: {:?}",
            far.line(rule + 3),
        );
    }

    /// An empty board still draws, and draws nothing travelling.
    #[test]
    fn a_figure_with_nothing_coming_is_blank_above_the_rule() {
        let bare = Figure {
            lanes: lanes(),
            ..Figure::default()
        };
        for index in 2..=AHEAD + 1 {
            let line = bare.line(index).expect("a row");
            assert!(line.trim().is_empty(), "row {index}: {line:?}");
        }
    }

    /// §14 — and the sim holds the companion test that the glyphs are CP437.
    #[test]
    fn every_glyph_the_figure_draws_is_in_the_code_page() {
        for glyph in [APERTURE, STRUCK, MISSED] {
            assert!(
                crate::is_renderable(glyph),
                "{glyph:?} has no cell in the code page",
            );
        }
    }
}
