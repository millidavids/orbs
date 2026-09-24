//! Two bands across a wall — what a siege looks like (§5.1).
//!
//! Drawn beside the transcript whenever a siege runs; the picture is not gated
//! on a word. Rule 2: every figure is one `survey` would give, and the odds sit
//! beside each bar (§5.1).
//!
//! Strength is a bar's *length*, never its hue — §14 forbids meaning that lives
//! only in colour.

use crate::style::{Tint, Wash};

/// The glyph a band's strength is drawn from. CP437 0xDB.
pub const FULL: char = '\u{2588}';

/// What a band has lost. CP437 0xB0.
pub const LOST: char = '\u{2591}';

/// The rule between the two bands. CP437 0xC4.
pub const GROUND: char = '\u{2500}';

/// Yours. Blue only so the two sides differ; the labels carry the identity.
pub const OURS: Tint = Tint::Blue;

/// Theirs.
pub const THEIRS: Tint = Tint::Red;

/// One side of a siege, as the board draws it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Side {
    /// What it is called — the word `survey` takes.
    ///
    /// Handed in by the sim: an unlabelled row is one a player has to guess at
    /// before they can type.
    pub name: &'static str,
    /// How many are standing.
    pub troops: u32,
    /// How much fight they have between them.
    pub vigour: u32,
    /// What they started with, so the bar has a denominator.
    pub full: u32,
    /// The chance one of them tells, as a percentage.
    pub chance: u32,
}

/// A siege in progress.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Rampart {
    /// Yours, drawn below the rule.
    pub garrison: Side,
    /// Theirs, drawn above it.
    pub enemy: Side,
    /// How many rounds have resolved.
    ///
    /// A real round identity: `orbs-balance` keyed that on `tally.len()` and
    /// two rounds collided.
    pub turns: u32,
    /// What the enemy will do next, as a word.
    ///
    /// A `String` rather than an enum: an intent is content, and `orbs-render`
    /// may not depend on `orbs-sim`.
    pub intent: String,
    /// The four places a die can be pledged, in the order they are drawn.
    ///
    /// Three dice against four rows, so one is always empty and the picture's
    /// job is to make which one obvious.
    pub areas: Vec<Allocation>,
    /// The dice still in the coffer, each with what it costs to pledge.
    ///
    /// The cost travels with the die because the price is a sim fact and
    /// `orbs-render` may not depend on `orbs-sim`.
    pub coffer: Vec<(String, u32)>,
    /// What is left to spend on dice this siege.
    pub quintessence: u32,
    /// The line under the rule, already written — prose built from literals in
    /// a painter cannot be hot-reloaded.
    pub tally: String,
}

/// One area of the wall, and what is behind it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Allocation {
    /// What it is called — the word `pledge` takes.
    pub name: &'static str,
    /// The dice pledged here, by name, in the order they were pledged.
    pub dice: Vec<String>,
    /// The least and most it could come to once rolled.
    ///
    /// §5.1's odds-before-the-commitment, carried from a roll to an allocation.
    pub range: (u32, u32),
    /// Whether the coming intent will throw this away.
    pub moot: bool,
}

impl Rampart {
    /// How wide a strength bar is drawn.
    const BAR: u16 = 20;

    /// The widest label, plus its gap — `garrison` is eight.
    ///
    /// `pub` because two other crates need the bar's start column; both had a
    /// hardcoded `+ 10`, which painted over the label when this widened.
    pub const LABEL: u16 = 10;

    /// Room for ` 18/18  55%`.
    const FIGURES: u16 = 13;

    /// The whole board's width.
    pub const COLS: u16 = Self::LABEL + Self::BAR + Self::FIGURES;

    /// A header naming what is coming, a row per side, the rule, and the tally.
    ///
    /// Fixed rather than sized to what is standing: a board that shrank as a
    /// band broke would move the rule a player reads position against.
    #[must_use]
    pub const fn rows() -> u16 {
        // intent, blank, four areas, blank, enemy, rule, garrison, blank,
        // coffer, tally
        13
    }

    /// What is still free to pledge, what each costs, and what is left to pay.
    ///
    /// The cost is the price of the decision — §5.1's odds before the
    /// commitment, not only `survey d20`.
    ///
    /// `coffer    d6 1  d8 2  d20 5          7`. A die held but unaffordable
    /// still draws, dimmed by the arithmetic rather than by a style.
    #[must_use]
    pub fn coffer_row(&self) -> String {
        let dice = if self.coffer.is_empty() {
            "--".to_owned()
        } else {
            self.coffer
                .iter()
                .map(|(die, cost)| format!("{die} {cost}"))
                .collect::<Vec<_>>()
                .join("  ")
        };
        // Right-aligned into the column the bands end in, so the pool reads
        // down the board against the two strengths.
        let left = Self::figure(self.quintessence);
        format!(
            "{:<label$}{dice:<width$}{left:>4}",
            "coffer",
            label = Self::LABEL as usize,
            width = (Self::COLS - Self::LABEL - 4) as usize,
        )
    }

    /// How many rows the areas take, and where they start.
    pub const AREAS_AT: u16 = 2;

    /// One area's row, as text.
    ///
    /// `line     d20 d6      2 to 26   `. An empty area draws its name and a
    /// dash: one row is always dark, and which one is the board's main fact.
    #[must_use]
    pub fn area_row(&self, area: &Allocation) -> String {
        let dice = if area.dice.is_empty() {
            "--".to_owned()
        } else {
            area.dice.join(" ")
        };
        // `moot` shows on an empty row too: the warning exists to stop a pledge
        // before it is made (§5.1).
        let worth = if area.moot {
            "moot".to_owned()
        } else if area.dice.is_empty() {
            String::new()
        } else {
            format!("{} to {}", area.range.0, area.range.1)
        };
        format!(
            "{:<label$}{dice:<11}{worth:>10}  ",
            area.name,
            label = Self::LABEL as usize,
        )
    }

    /// How many cells of `FULL` a side's bar gets.
    ///
    /// Ceiling division above nought: a bar that rounded to nothing would say
    /// *routed* about a side still standing.
    #[must_use]
    pub fn filled(side: &Side) -> u16 {
        if side.vigour == 0 || side.full == 0 {
            return 0;
        }
        let scaled = (u64::from(side.vigour) * u64::from(Self::BAR)).div_ceil(u64::from(side.full));
        u16::try_from(scaled)
            .unwrap_or(Self::BAR)
            .clamp(1, Self::BAR)
    }

    /// One side's row, as text.
    ///
    /// The figures are clamped into their width: `{:>3}` is a floor, so a
    /// garrison past 999 vigour pushed the row past [`Self::COLS`] and out
    /// through the border. The bar is unaffected; [`Self::filled`] is a ratio.
    #[must_use]
    pub fn row(&self, side: &Side) -> String {
        let filled = Self::filled(side) as usize;
        let empty = Self::BAR as usize - filled;
        format!(
            "{:<label$}{}{} {:>3}/{:<3} {:>3}%",
            side.name,
            FULL.to_string().repeat(filled),
            LOST.to_string().repeat(empty),
            Self::figure(side.vigour),
            Self::figure(side.full),
            Self::figure(side.chance),
            label = Self::LABEL as usize,
        )
    }

    /// A number that fits the three columns the row reserves for it.
    ///
    /// `1k+` rather than a truncation or a clamp: `1200` cut to `120` is a
    /// different number stated as fact, and `999+` is four characters.
    fn figure(value: u32) -> String {
        if value > 999 {
            "1k+".to_owned()
        } else {
            value.to_string()
        }
    }

    /// Which tint a side's bar takes.
    ///
    /// By name, not by pointer: const eval will not compare `&'static str`
    /// addresses, and two equal names from different places would differ.
    #[must_use]
    pub fn tint(&self, side: &Side) -> Tint {
        if side.name == self.garrison.name {
            OURS
        } else {
            THEIRS
        }
    }

    /// The wash a band's bar draws in — one family, unmixed.
    #[must_use]
    pub const fn wash(tint: Tint) -> Wash {
        Wash::plain(tint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn side(name: &'static str, vigour: u32, full: u32) -> Side {
        Side {
            name,
            troops: full / 3,
            vigour,
            full,
            chance: 50,
        }
    }

    fn area(name: &'static str, dice: &[&str], range: (u32, u32), moot: bool) -> Allocation {
        Allocation {
            name,
            dice: dice.iter().map(|die| (*die).to_owned()).collect(),
            range,
            moot,
        }
    }

    fn board() -> Rampart {
        Rampart {
            garrison: side("garrison", 18, 18),
            enemy: side("enemy", 21, 21),
            turns: 1,
            // Three dice against four areas: one row is always dark, which is
            // what every assertion below is about.
            areas: vec![
                area("line", &[], (0, 0), true),
                area("buckler", &["d20"], (1, 20), false),
                area("succour", &["d6"], (1, 6), false),
                area("sortie", &[], (0, 0), false),
            ],
            coffer: vec![("d8".to_owned(), 2)],
            quintessence: 7,
            intent: "volley".to_owned(),
            tally: "round 1".to_owned(),
        }
    }

    #[test]
    fn an_area_row_shows_its_dice_and_what_they_could_come_to() {
        let board = board();
        let buckler = board.area_row(&board.areas[1]);
        assert!(buckler.contains("buckler"), "{buckler}");
        assert!(buckler.contains("d20"), "{buckler}");
        assert!(
            buckler.contains("1 to 20"),
            "the range is not shown: {buckler}"
        );
    }

    #[test]
    fn a_moot_area_says_so_even_with_nothing_on_it() {
        // The warning stops a pledge before it is made; hidden until something
        // is there, it would arrive after the decision (§5.1).
        let board = board();
        let line = board.area_row(&board.areas[0]);
        assert!(board.areas[0].dice.is_empty(), "the fixture changed");
        assert!(
            line.contains("moot"),
            "an empty area gave no warning before it was pledged to: {line}",
        );
    }

    #[test]
    fn an_empty_area_that_is_not_moot_draws_a_dash_and_no_range() {
        let board = board();
        let sortie = board.area_row(&board.areas[3]);
        assert!(sortie.contains("--"), "{sortie}");
        assert!(
            !sortie.contains(" to "),
            "an empty area claimed a range: {sortie}"
        );
    }

    #[test]
    fn the_coffer_row_says_what_is_left_to_pledge() {
        // Showing four destinations and hiding what goes to them is half a
        // picture.
        let board = board();
        assert!(board.coffer_row().contains("d8"));
        let spent = Rampart {
            coffer: Vec::new(),
            ..board
        };
        assert!(
            spent.coffer_row().contains("--"),
            "an empty coffer did not say so: {}",
            spent.coffer_row(),
        );
    }

    /// The cost is the price of the decision (§5.1), so it goes on the board.
    #[test]
    fn the_coffer_row_prices_every_die_and_says_what_is_left() {
        let board = Rampart {
            coffer: vec![("d6".to_owned(), 1), ("d20".to_owned(), 5)],
            quintessence: 3,
            ..board()
        };
        let row = board.coffer_row();
        assert!(row.contains("d6 1"), "the d6's price is missing: {row}");
        assert!(row.contains("d20 5"), "the d20's price is missing: {row}");
        assert!(
            row.trim_end().ends_with('3'),
            "what is left to spend is missing: {row}",
        );
    }

    /// Only the band rows were width-pinned. `orbs_shell::rampart::paint`
    /// truncates rather than wrapping, so an overrun silently loses the pool.
    #[test]
    fn the_coffer_row_fits_the_board_even_at_absurd_numbers() {
        for (coffer, left) in [
            (vec![], 0),
            (vec![("d6".to_owned(), 1)], 7),
            (
                vec![
                    ("d6".to_owned(), 1),
                    ("d8".to_owned(), 2),
                    ("d20".to_owned(), 5),
                ],
                24,
            ),
            // Nothing produces these, which is why they are worth pinning.
            (vec![("d100".to_owned(), 9999)], 99_999),
        ] {
            let board = Rampart {
                coffer,
                quintessence: left,
                ..board()
            };
            let row = board.coffer_row();
            assert!(
                row.chars().count() <= Rampart::COLS as usize,
                "the coffer row ran past {} cells: {row:?}",
                Rampart::COLS,
            );
        }
    }

    #[test]
    fn a_full_band_fills_its_bar_and_an_empty_one_draws_none() {
        assert_eq!(Rampart::filled(&side("enemy", 21, 21)), Rampart::BAR);
        assert_eq!(Rampart::filled(&side("enemy", 0, 21)), 0);
    }

    #[test]
    fn a_band_with_any_fight_left_draws_at_least_one_cell() {
        // At 1 vigour in 100 the honest fraction rounds to nothing, and a bar of
        // nothing says *routed* about a side still standing.
        for vigour in 1..=40 {
            assert!(
                Rampart::filled(&side("enemy", vigour, 400)) >= 1,
                "{vigour} of 400 drew an empty bar",
            );
        }
    }

    #[test]
    fn a_bar_never_overflows_its_width() {
        // A band mended above what it started with must not draw past the box.
        assert_eq!(Rampart::filled(&side("garrison", 99, 18)), Rampart::BAR);
    }

    #[test]
    fn a_row_is_exactly_the_boards_width() {
        // Every row the same length, or the border draws ragged.
        let board = board();
        for side in [&board.garrison, &board.enemy] {
            assert_eq!(
                board.row(side).chars().count(),
                Rampart::COLS as usize,
                "row {:?} is not {} cells: {:?}",
                side.name,
                Rampart::COLS,
                board.row(side),
            );
        }
    }

    /// `mustered` grows with every `deploy` and `{:>3}` is a floor, so a
    /// four-digit garrison drew a row wider than its box.
    #[test]
    fn a_row_keeps_its_width_at_a_garrison_no_siege_has_ever_had() {
        let mut board = board();
        board.garrison = side("garrison", 3_600, 3_600);
        let row = board.row(&board.garrison);
        assert_eq!(
            row.chars().count(),
            Rampart::COLS as usize,
            "a four-digit garrison widened the row: {row:?}",
        );
        // ...and it says so rather than stating a number it had to cut down.
        assert!(
            row.contains("1k+"),
            "the figure was silently clamped: {row}"
        );
    }

    #[test]
    fn a_row_says_the_numbers_a_survey_would() {
        // Rule 2: the picture carries nothing the readings lack.
        let board = board();
        let row = board.row(&board.garrison);
        assert!(row.contains("garrison"), "{row}");
        assert!(row.contains("18/18"), "{row}");
        assert!(row.contains("50%"), "{row}");
    }

    #[test]
    fn strength_survives_greyscale() {
        // §14: the bar's *length* carries the comparison, so two bands at
        // different strengths differ in glyphs and not only in hue.
        let strong = board().row(&side("enemy", 21, 21));
        let weak = board().row(&side("enemy", 7, 21));
        let count = |row: &str| row.chars().filter(|c| *c == FULL).count();
        assert!(
            count(&strong) > count(&weak),
            "a weakened band drew the same bar as a full one",
        );
    }

    #[test]
    fn every_glyph_the_board_draws_is_in_the_code_page() {
        // §19 records `▪` and `►` shipping as `?`. A painter's glyphs are Rust
        // literals, so `is_renderable` never sees them; this holds them instead.
        for glyph in [FULL, LOST, GROUND] {
            assert!(
                crate::cp437::is_renderable(glyph),
                "{glyph:?} is not in CP437 and will draw as `?`",
            );
        }
    }
}
