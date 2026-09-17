//! The small tables everything else stands on: the six humours against the
//! logician's gates, the glyphs' words, and a temper's rows — each checked over
//! every value it can take.

use std::collections::BTreeSet;

use super::super::temper::{ROWS, SENSES, Temper, sense_lit};
use super::super::{Glyph, Humour, LESSER_SENSES};

/// **Each humour is the gate its `recall` page names**, row for row: both dark,
/// one lit, the other lit, both lit.
#[test]
fn every_humour_answers_the_logicians_table_its_page_names() {
    let gates: [(Humour, fn(bool, bool) -> bool); 6] = [
        (Humour::Yoke, |a, b| a && b),
        (Humour::Spurn, |a, b| !(a && b)),
        (Humour::Heed, |a, b| a || b),
        (Humour::Eschew, |a, b| !(a || b)),
        (Humour::Oppose, |a, b| a != b),
        (Humour::Mirror, |a, b| a == b),
    ];
    for (humour, gate) in gates {
        for a in [false, true] {
            for b in [false, true] {
                assert_eq!(humour.answer(a, b), gate(a, b), "{humour:?} on {a} {b}");
            }
        }
    }
    let covered: BTreeSet<Humour> = gates.iter().map(|(humour, _)| *humour).collect();
    assert_eq!(covered.len(), Humour::ALL.len());
}

/// **Turning over and taking the dual are each their own undoing**, and the
/// pairs are the ones the pages teach: yoke and spurn, heed and eschew, oppose
/// and mirror; yoke's dual eschew, heed's spurn, oppose and mirror their own.
#[test]
fn negation_and_duality_pair_the_humours_the_pages_pair() {
    for humour in Humour::ALL {
        assert_eq!(humour.negated().negated(), humour);
        assert_eq!(humour.dual().dual(), humour);
        assert_ne!(humour.negated(), humour);
    }
    assert_eq!(Humour::Yoke.negated(), Humour::Spurn);
    assert_eq!(Humour::Heed.negated(), Humour::Eschew);
    assert_eq!(Humour::Oppose.negated(), Humour::Mirror);
    assert_eq!(Humour::Yoke.dual(), Humour::Eschew);
    assert_eq!(Humour::Heed.dual(), Humour::Spurn);
    assert_eq!(Humour::Oppose.dual(), Humour::Oppose);
    assert_eq!(Humour::Mirror.dual(), Humour::Mirror);
}

/// **No humour's word is a word the spell grammar owns, or a glyph's** — a
/// humour called `and` could never be asked about in an `if` line, and a humour
/// sharing a glyph's word would make `limn` ambiguous in either order.
#[test]
fn no_humour_word_is_grammar_or_a_glyph() {
    const GRAMMAR: [&str; 7] = ["and", "or", "not", "either", "both", "odd", "none"];
    let mut words = BTreeSet::new();
    for humour in Humour::ALL {
        let word = humour.word();
        assert!(words.insert(word), "{word} is two humours");
        assert!(!GRAMMAR.contains(&word), "{word} is the grammar's");
        assert!(Glyph::from_word(word).is_none(), "{word} is a glyph");
        assert_eq!(Humour::from_word(word), Some(humour));
    }
    for glyph in Glyph::ALL {
        assert!(words.insert(glyph.word()), "{} is a humour", glyph.word());
        assert!(Humour::from_word(glyph.word()).is_none());
    }
}

#[test]
fn every_glyph_reads_back_as_itself_and_sits_at_its_index() {
    for (index, glyph) in Glyph::ALL.into_iter().enumerate() {
        assert_eq!(Glyph::from_word(glyph.word()), Some(glyph));
        assert_eq!(glyph.index(), index);
    }
    assert_eq!(Glyph::from_word("crown"), None);
    assert_eq!(
        Glyph::from_word("Keystone"),
        None,
        "a glyph word is lower case"
    );
}

/// **Row nought is every sense dark and the last every sense lit**, the first
/// sense the highest bit — the order a truth table is written and the board's
/// columns are drawn, for both circles.
#[test]
fn rows_count_up_with_the_first_sense_highest() {
    for (senses, rows) in [(SENSES, ROWS), (LESSER_SENSES, 4)] {
        for sense in 0..senses {
            assert!(!sense_lit(0, sense, senses), "row nought lights {sense}");
            assert!(
                sense_lit(rows - 1, sense, senses),
                "the last row darkens {sense}"
            );
        }
        for row in 0..rows {
            let rebuilt = (0..senses)
                .filter(|sense| sense_lit(row, *sense, senses))
                .map(|sense| 1 << (senses - 1 - sense))
                .sum::<usize>();
            assert_eq!(rebuilt, row, "{senses} senses, row {row}");
        }
    }
}

/// **Every temper of both sizes reads back as itself**, and a string of the wrong
/// length or with anything but `0` and `1` in it is refused.
#[test]
fn every_temper_round_trips_through_its_rows_and_nothing_else_reads() {
    for bits in 0..=u8::MAX {
        let whole = Temper::from_bits(bits, SENSES);
        assert_eq!(Temper::from_rows(&whole.to_rows()), Some(whole));
        assert_eq!(whole.to_rows().len(), ROWS);
        let lesser = Temper::from_bits(bits, LESSER_SENSES);
        assert_eq!(Temper::from_rows(&lesser.to_rows()), Some(lesser));
        assert_eq!(
            lesser.bits(),
            bits & 0b1111,
            "high rows kept on a lesser temper"
        );
        assert_eq!(lesser.fervour(), (bits & 0b1111).count_ones());
    }
    for bad in [
        "",
        "0",
        "010",
        "01010",
        "0101010",
        "010101010",
        "0101a101",
        "01 1",
    ] {
        assert!(Temper::from_rows(bad).is_none(), "{bad:?} read as a temper");
    }
}

/// **Agreement is symmetric, a temper agrees with itself on every row, and a
/// lesser temper never counts rows it does not have.**
#[test]
fn agreement_is_symmetric_whole_with_itself_and_masked_when_lesser() {
    for a in 0..=u8::MAX {
        for b in [0, 1, 0b1010_0101, 0b1111_0000, u8::MAX, a.rotate_left(3)] {
            for senses in [SENSES, LESSER_SENSES] {
                let (one, other) = (Temper::from_bits(a, senses), Temper::from_bits(b, senses));
                assert_eq!(one.agreeing(other), other.agreeing(one));
                let rows = u32::try_from(one.rows()).unwrap_or(u32::MAX);
                assert_eq!(one.agreeing(one), rows);
                assert!(one.agreeing(other) <= rows);
                let differing = (0..one.rows())
                    .filter(|row| one.lit(*row) != other.lit(*row))
                    .count();
                assert_eq!(
                    one.agreeing(other),
                    rows - u32::try_from(differing).unwrap_or(0),
                    "{a:08b} {b:08b} over {senses}",
                );
            }
        }
    }
}

/// **`turns_on_every_sense` is its definition**, checked by brute force over all
/// 256 eight-row tempers and all 16 four-row ones: every sense has some row
/// whose answer changes when that sense alone is turned over.
#[test]
fn turning_on_every_sense_is_what_it_says_for_every_temper() {
    for senses in [SENSES, LESSER_SENSES] {
        let rows = 1usize << senses;
        let mut turning = 0;
        for bits in 0..=u8::MAX {
            let temper = Temper::from_bits(bits, senses);
            let brute = (0..senses).all(|sense| {
                (0..rows).any(|row| {
                    let other = (0..senses)
                        .map(|each| sense_lit(row, each, senses) != (each == sense))
                        .fold(0, |row, lit| row * 2 + usize::from(lit));
                    temper.lit(row) != temper.lit(other)
                })
            });
            assert_eq!(
                temper.turns_on_every_sense(),
                brute,
                "{bits:08b} over {senses}"
            );
            turning += usize::from(brute && usize::from(bits) < (1 << rows));
        }
        // The known counts of boolean functions depending on all their inputs.
        let expected = if senses == SENSES { 218 } else { 10 };
        assert_eq!(turning, expected, "over {senses} senses");
    }
}
