//! A beast: arriving, being limned and called, its par and troops, and the save
//! it travels in.

use super::super::circuit::{self, Turned};
use super::super::temper::{self, OPENING, PAR, Temper};
use super::super::{Beast, Call, Glyph, Humour, Shape, lesser_answer};
use crate::rng::Rngs;

fn waiting() -> Beast {
    Beast::draw(&mut Rngs::from_seed(3), true)
}

fn lesser() -> Beast {
    Beast::draw(&mut Rngs::from_seed(3), false)
}

#[test]
fn a_beast_arrives_at_the_opening_uncalled() {
    let beast = waiting();
    assert_eq!(beast.glyphs(), OPENING);
    assert_eq!(beast.calls(), 0);
    assert_eq!(beast.answer(), None);
    assert_eq!(beast.agreeing(), 0);
}

#[test]
fn the_opening_never_holds_and_a_call_counts() {
    for mut beast in [waiting(), lesser()] {
        assert!(matches!(beast.call(), Call::Balked { rows } if rows > 0));
        assert_eq!(beast.calls(), 1);
        assert!(beast.answer().is_some());
    }
}

#[test]
fn limning_the_solution_holds_and_says_every_row_agreed() {
    for (mut beast, rows) in [(waiting(), 8), (lesser(), 4)] {
        let Some(solution) = beast.solution() else {
            panic!("a drawn beast has no solution");
        };
        for (glyph, humour) in Glyph::ALL.into_iter().zip(solution) {
            beast.limn(glyph, humour);
        }
        assert_eq!(beast.call(), Call::Held);
        assert_eq!(beast.agreeing(), rows);
        assert!(beast.within_par());
    }
}

/// A call answers what the glyphs as they stand answer, turned wires and all,
/// and a balk counts the rows that differ — over every limning.
#[test]
fn a_call_answers_what_the_circuit_answers_under_its_turned_wires() {
    let (turned, _) = turned_and_untouched();
    let Shape::Whole(puzzle) = turned.shape() else {
        panic!("a whole draw was lesser");
    };
    assert!(!puzzle.turned.is_none());
    for glyphs in circuit::limnings() {
        let mut beast = turned.clone();
        for (glyph, humour) in Glyph::ALL.into_iter().zip(glyphs) {
            beast.limn(glyph, humour);
        }
        let expected = circuit::answer(puzzle.wiring, puzzle.turned, glyphs);
        let call = beast.call();
        assert_eq!(beast.answer(), Some(expected));
        let differing = 8 - expected.agreeing(puzzle.temper);
        match call {
            Call::Held => assert_eq!(differing, 0, "{glyphs:?}"),
            Call::Balked { rows } => assert_eq!(rows, differing, "{glyphs:?}"),
        }
    }
}

#[test]
fn stepping_moves_one_glyph_and_leaves_the_others() {
    let mut beast = waiting();
    assert_eq!(beast.step(Glyph::Sunwise), Humour::Spurn);
    assert_eq!(beast.humour(Glyph::Sunwise), Humour::Spurn);
    assert_eq!(beast.humour(Glyph::Keystone), Humour::Yoke);
    assert_eq!(beast.humour(Glyph::Widdershins), Humour::Yoke);
}

#[test]
fn par_is_a_count_of_calls_and_the_fourth_is_past_it() {
    let mut beast = waiting();
    assert_eq!(beast.troops(), 4);
    for _ in 0..PAR {
        beast.call();
    }
    assert!(beast.within_par());
    assert_eq!(beast.troops(), 4);
    beast.call();
    assert!(!beast.within_par());
    assert_eq!(beast.troops(), 3);
}

/// One call at a lesser circle, and one troop whichever side of it.
#[test]
fn a_lesser_beast_pays_one_troop_and_its_par_is_one_call() {
    let mut beast = lesser();
    assert_eq!(beast.troops(), 1);
    beast.call();
    assert!(beast.within_par());
    beast.call();
    assert!(!beast.within_par());
    assert_eq!(beast.troops(), 1);
}

#[test]
fn a_beast_saved_mid_search_comes_back_the_same() {
    let mut beast = waiting();
    beast.step(Glyph::Keystone);
    beast.limn(Glyph::Widdershins, Humour::Oppose);
    beast.call();
    let back = Beast::from_save(&beast.to_save());
    assert_eq!(back, Some(beast));

    let mut beast = lesser();
    beast.limn(Glyph::Keystone, Humour::Heed);
    beast.call();
    let saved = beast.to_save();
    assert_eq!(saved.temper.len(), 4);
    assert!(saved.sunwise.is_none(), "a lesser beast wrote a wiring");
    assert_eq!(Beast::from_save(&saved), Some(beast));
}

#[test]
fn a_save_with_a_word_no_humour_has_is_no_beast() {
    let mut save = waiting().to_save();
    save.glyphs[1] = "and".to_owned();
    assert!(Beast::from_save(&save).is_none());
    let mut save = waiting().to_save();
    save.temper = "01011".to_owned();
    assert!(Beast::from_save(&save).is_none());
    let mut save = waiting().to_save();
    save.glyphs.pop();
    assert!(
        Beast::from_save(&save).is_none(),
        "two glyphs read as three"
    );
}

/// The temper's length and the wiring must agree: a lesser temper with wiring,
/// or a whole one without, is no beast rather than a guess.
#[test]
fn a_save_whose_temper_and_wiring_disagree_is_no_beast() {
    let mut save = waiting().to_save();
    save.temper = lesser_answer(Humour::Heed).to_rows();
    assert!(Beast::from_save(&save).is_none());
    let mut save = lesser().to_save();
    save.sunwise = Some([0, 1]);
    save.widdershins = Some([1, 2]);
    assert!(Beast::from_save(&save).is_none());
    let mut save = waiting().to_save();
    save.widdershins = save.sunwise;
    assert!(Beast::from_save(&save).is_none(), "one pair wired twice");
}

/// A beast drawn with a turned wire, and one with none, from the same seeds
/// the draw test sweeps — so the round trip is not only the untouched case.
fn turned_and_untouched() -> (Beast, Beast) {
    let mut rngs = Rngs::from_seed(3);
    let mut turned = None;
    let mut untouched = None;
    while turned.is_none() || untouched.is_none() {
        let beast = Beast::draw(&mut rngs, true);
        let slot = if beast.shape().turned().is_none() {
            &mut untouched
        } else {
            &mut turned
        };
        slot.get_or_insert(beast);
    }
    (
        turned.unwrap_or_else(waiting),
        untouched.unwrap_or_else(waiting),
    )
}

/// The turned wires travel, and a beast with none writes none — so a format-13
/// document, which never had the field, reads as nothing turned.
#[test]
fn a_turned_beast_comes_back_turned_and_an_untouched_one_writes_no_mask() {
    let (mut turned, mut untouched) = turned_and_untouched();
    turned.limn(Glyph::Sunwise, Humour::Mirror);
    turned.call();
    let saved = turned.to_save();
    assert!(saved.turned.is_some(), "a turned beast wrote no mask");
    assert_eq!(Beast::from_save(&saved), Some(turned));

    untouched.call();
    let saved = untouched.to_save();
    assert!(saved.turned.is_none(), "an untouched beast wrote a mask");
    assert_eq!(Beast::from_save(&saved).as_ref(), Some(&untouched));
    let mut spelled = saved;
    spelled.turned = Some("0000".to_owned());
    assert_eq!(Beast::from_save(&spelled), Some(untouched));
}

/// A mask is refused unless a circuit could carry it, on a beast that could:
/// malformed, both of a glyph's wires, a lesser beast, a temper it disagrees
/// with.
#[test]
fn a_mask_the_beast_could_not_have_is_no_beast() {
    let (turned, untouched) = turned_and_untouched();
    for bad in ["01", "1100", "0011", "1x00"] {
        let mut save = untouched.to_save();
        save.turned = Some(bad.to_owned());
        assert!(Beast::from_save(&save).is_none(), "{bad:?} was read");
    }
    let mut save = lesser().to_save();
    save.turned = Some("0000".to_owned());
    assert!(
        Beast::from_save(&save).is_none(),
        "a lesser beast held a mask"
    );

    // Either the plain circuit was never drawn, or it was and accepting it is
    // right; `is_puzzle` decides which.
    let mut save = turned.to_save();
    save.turned = None;
    let Shape::Whole(puzzle) = turned.shape() else {
        panic!("a whole draw was lesser");
    };
    let plain = circuit::Puzzle {
        turned: Turned::NONE,
        ..puzzle
    };
    assert_eq!(Beast::from_save(&save).is_some(), circuit::is_puzzle(plain));
}

#[test]
fn a_restore_refuses_what_no_circle_answers() {
    let beast = waiting();
    let Shape::Whole(mut puzzle) = beast.shape() else {
        panic!("an open draw was lesser");
    };
    assert!(Beast::restored(Shape::Whole(puzzle), OPENING, 0, None).is_some());
    // Lit on every row: a temper no circuit is drawn with.
    puzzle.temper = Temper::from_bits(u8::MAX, temper::SENSES);
    assert!(Beast::restored(Shape::Whole(puzzle), OPENING, 0, None).is_none());
    // The opening keystone's own table: a lesser beast held uncalled.
    let opening = lesser_answer(OPENING[Glyph::Keystone.index()]);
    assert!(Beast::restored(Shape::Lesser(opening), OPENING, 0, None).is_none());
}

/// A wrong-length answer is forgotten, not fatal — the next call rewrites it.
#[test]
fn a_restored_answer_of_the_wrong_length_is_forgotten() {
    let beast = waiting();
    let wrong = lesser_answer(Humour::Heed);
    let restored = Beast::restored(beast.shape(), OPENING, 2, Some(wrong));
    let Some(restored) = restored else {
        panic!("a whole beast with a four-row answer was refused outright");
    };
    assert_eq!(restored.answer(), None);
    assert_eq!(restored.calls(), 2);
}
