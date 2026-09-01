//! Two bands across a wall — what a siege looks like (§5.1).
//!
//! Drawn beside the transcript whenever a siege is running, which is the rule
//! the archive's map, the lens's sheet and the sanctum's board all follow: **the
//! picture is not gated on a word**, because watching a bound decision tree
//! fight one and fighting it yourself are different activities and only the
//! second involves typing.
//!
//! # It carries nothing the readings lack
//!
//! Rule 2's line. Every figure here is one the player could read off `survey
//! garrison` and `survey enemy` — troops, vigour, the telegraphed intent, and the
//! odds. What the picture adds is that both bands and what is coming are legible
//! **at once**, which is what makes a position readable rather than arithmetic.
//!
//! # Strength is length, never colour
//!
//! §14 forbids meaning that lives only in hue, and the whole domain is a
//! comparison of two quantities — so a band's strength is a *bar* whose length
//! is its vigour, and it reads in greyscale and in a dump. The two tints are
//! enrichment: they say which side you are looking at, which the labels already
//! say.
//!
//! # The odds are on the board, before the commitment
//!
//! §5.1's fairness rule, drawn: *show the odds before the commitment and the
//! roll after it.* The chance each side has to tell is printed beside its bar,
//! so a player choosing whether to spend a potion is choosing under known risk.
//! A surprise would not be a decision.

use crate::style::{Tint, Wash};

/// The glyph a band's strength is drawn from. CP437 0xDB.
pub const FULL: char = '\u{2588}';

/// What a band has lost. CP437 0xB0.
pub const LOST: char = '\u{2591}';

/// The rule between the two bands. CP437 0xC4.
pub const GROUND: char = '\u{2500}';

/// Yours. Blue, which the palette already reads as *ours* nowhere else — the
/// point is only that the two sides differ, and the labels carry the identity.
pub const OURS: Tint = Tint::Blue;

/// Theirs.
pub const THEIRS: Tint = Tint::Red;

/// One side of a siege, as the board draws it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Side {
    /// What it is called — the word `survey` takes.
    ///
    /// **Handed in by the sim, not held here.** `survey garrison` names a band
    /// by word, so a board whose rows are unlabelled is one a player has to
    /// guess at before they can type. The lens's sheet paid for this omission
    /// once and the sanctum's board records it.
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
    /// **Carried, so a caller has a real round identity.** `orbs-balance`'s
    /// driver keyed "have I already spent this round" on `tally.len()` — the
    /// byte length of a rendered prose line, which only changes when a digit
    /// count does. Two consecutive rounds collided and the policy silently
    /// stopped using its arsenal.
    pub turns: u32,
    /// What the enemy will do next, as a word.
    ///
    /// **The whole of what telegraphing costs**, and the reason it is a `String`
    /// rather than an enum: an intent is content, and `orbs-render` may not
    /// depend on `orbs-sim`.
    pub intent: String,
    /// The four places a die can be pledged, in the order they are drawn.
    ///
    /// **The decision, drawn.** Three dice against four rows, so one is always
    /// empty and the picture's job is to make *which one* obvious at a glance —
    /// alongside what each is worth and which the coming intent will throw away.
    pub areas: Vec<Allocation>,
    /// The dice still in the coffer, each with what it costs to pledge.
    ///
    /// **The cost travels with the die**, because `orbs-render` may never depend
    /// on `orbs-sim` and the price is a sim fact — the same reason a band's name
    /// is handed in rather than known here.
    pub coffer: Vec<(String, u32)>,
    /// What is left to spend on dice this siege.
    pub quintessence: u32,
    /// The line under the rule, already written.
    ///
    /// Handed in rather than composed here, for `Pylon::tally`'s reason: a
    /// sentence built from literals in a painter is prose that cannot be
    /// hot-reloaded and would be the only authored English in this crate.
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
    /// **The range, and it is the whole reason a die is a gamble rather than a
    /// number.** §5.1 shows the odds before the commitment; this is that rule
    /// carried from a roll to an allocation.
    pub range: (u32, u32),
    /// Whether the coming intent will throw this away.
    pub moot: bool,
}

impl Rampart {
    /// How wide a strength bar is drawn.
    const BAR: u16 = 20;

    /// The widest label, plus its gap — `garrison` is eight.
    ///
    /// **`pub`, because two other crates need the bar's start column.**
    /// `orbs-shell`'s painter and the `screens` example both tinted from a
    /// hardcoded `+ 10`; widening this would have moved the text and left both
    /// tints painting over the label, with every test still green —
    /// `a_row_is_exactly_the_boards_width` measures the string, not the rect.
    pub const LABEL: u16 = 10;

    /// Room for ` 18/18  55%`.
    const FIGURES: u16 = 13;

    /// The whole board's width.
    pub const COLS: u16 = Self::LABEL + Self::BAR + Self::FIGURES;

    /// A header naming what is coming, a row per side, the rule, and the tally.
    ///
    /// **Fixed, and deliberately not sized to what is standing.** A board that
    /// shrank as a band broke would move the rule every round, and the rule is
    /// what a player reads position against — the sanctum's constraint exactly.
    #[must_use]
    pub const fn rows() -> u16 {
        // intent, blank, four areas, blank, enemy, rule, garrison, blank,
        // coffer, tally
        13
    }

    /// What is still free to pledge, what each costs, and what is left to pay.
    ///
    /// **Without it the player cannot see what they have left**, and the whole
    /// decision is *which die goes where* — a board that shows the four
    /// destinations and hides the three things going to them is half a picture.
    ///
    /// **The costs are on the board and not only in `survey d20`**, which is this
    /// domain's headline rule rather than a nicety: *show the odds before the
    /// commitment*, asserted three times in this file. Once a pledge costs
    /// something, the cost **is** the price of the decision — a board that showed
    /// the range and hid the price would be showing half the bargain.
    ///
    /// `coffer    d6 1  d8 2  d20 5          7` — each die with what it takes,
    /// then what is left. A die held but unaffordable still draws, dimmed by the
    /// arithmetic rather than by a style: you can see the `d20` and see that 5 is
    /// more than the 3 you hold.
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
        // **Right-aligned into the same column the bands end in**, so the pool
        // reads down the board against the two strengths rather than floating.
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
    /// `line     d20 d6      2 to 26   ` — the dice on it and what they could
    /// come to. An empty area draws its name and a dash, because **an empty row
    /// is the most important thing on this board**: with three dice and four
    /// areas one is always dark, and the picture's job is to make which one
    /// obvious.
    #[must_use]
    pub fn area_row(&self, area: &Allocation) -> String {
        let dice = if area.dice.is_empty() {
            "--".to_owned()
        } else {
            area.dice.join(" ")
        };
        // **`moot` shows on an empty row too, and that is the point.** The
        // warning exists to stop a pledge *before* it is made, so hiding it
        // until something is already there would be advice arriving after the
        // decision — §5.1's odds-before-the-commitment rule, applied to the one
        // choice this board is for.
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
    /// **Ceiling division above nought**, so a band with any fight left draws at
    /// least one cell. A bar that rounded to nothing would say *routed* about a
    /// side that is still standing, which is the one thing this picture must
    /// never do.
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
    /// **The figures are clamped into the width they were given, because `>3` is
    /// a floor and not a ceiling.** `{:>3}` pads a short number and *widens* for
    /// a long one, so a garrison past 999 vigour would push the row past
    /// [`Self::COLS`] and out through the border — and `mustered` grows with
    /// every `deploy`, so nothing in the sim bounds it.
    ///
    /// A four-digit garrison is 334 troops and is not a state the game reaches
    /// today, which is exactly why this is worth pinning: a picture that comes
    /// apart only at a value nobody has produced yet is a defect that ships. The
    /// bar is unaffected — [`Self::filled`] is a ratio and stays true.
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
    /// **`1k+` rather than a truncation or a clamp**, and it is exactly three
    /// characters — the first draft of this was `999+`, which is four and would
    /// have overflowed the field it was written to fit. `1200` cut to `120` is a
    /// number the board states as fact, and clamped to `999` it is a different
    /// one; `1k+` is *true at every magnitude* and says only what it knows.
    ///
    /// `chance` is a percentage and can never reach this. Vigour can.
    fn figure(value: u32) -> String {
        if value > 999 {
            "1k+".to_owned()
        } else {
            value.to_string()
        }
    }

    /// Which tint a side's bar takes.
    ///
    /// **By name, not by pointer.** Comparing `&'static str` addresses is not
    /// something const eval will do, and it would also be true of two equal
    /// names from different places — a fragile answer to an easy question.
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
            // the shape every assertion below is really about.
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
        // **The warning exists to stop a pledge before it is made.** Hidden
        // until something is already there, it would be advice arriving after
        // the decision — which is §5.1's odds-before-the-commitment rule broken
        // on the one choice this board is for.
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
        // A board that shows four destinations and hides the three things going
        // to them is half a picture.
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

    /// **The price is on the board, which is the domain's headline rule.**
    ///
    /// *Show the odds before the commitment* is asserted three times in this
    /// file, and once a pledge costs something the cost **is** the price of the
    /// decision. A board that drew the range and hid the price would be showing
    /// half the bargain — and `survey d20` is not the board.
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

    /// **The row a player reads the whole decision off, pinned to the board.**
    ///
    /// Only the two *band* rows were width-pinned, and this one grew two fields
    /// at once. `orbs_shell::rampart::paint` truncates rather than wrapping, so
    /// an overrun here loses the rightmost thing on the row — which is now the
    /// pool — silently, and only on the rounds where the numbers are widest.
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
        // **The one thing this picture must never do** is say *routed* about a
        // side that is still standing. At 1 vigour in 100 the honest fraction
        // rounds to nothing, so the bar is floored at one.
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

    /// **The width has to hold for numbers nothing produces yet**, because
    /// `mustered` grows with every `deploy` and the sim puts no ceiling on it.
    /// `{:>3}` is a floor, so a four-digit garrison drew a row two cells wider
    /// than the box that contains it — a picture that comes apart at a value
    /// nobody has reached is a defect that ships quietly.
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
        // literals, so `is_renderable` — which is for authored prose — never
        // sees them, and this is what holds them instead.
        for glyph in [FULL, LOST, GROUND] {
            assert!(
                crate::cp437::is_renderable(glyph),
                "{glyph:?} is not in CP437 and will draw as `?`",
            );
        }
    }
}
