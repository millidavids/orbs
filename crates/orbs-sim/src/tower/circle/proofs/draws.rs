//! The draw: deterministic from the seed, one stream value each, and over enough
//! beasts every kind of circuit a player can meet.

use std::collections::BTreeSet;

use rand::Rng;

use super::super::circuit::{self, Puzzle, Turned, Wiring};
use super::super::temper::Temper;
use super::super::{Beast, Shape};
use super::oracle::puzzles;
use crate::rng::{RngStream, Rngs};

/// **One seed is one sequence of beasts**, however many are drawn — a replay's
/// whole claim, one room down.
#[test]
fn the_same_seed_draws_the_same_beasts_in_the_same_order() {
    let sequence = |seed| {
        let mut rngs = Rngs::from_seed(seed);
        (0..40)
            .map(|_| circuit::draw(&mut rngs))
            .collect::<Vec<_>>()
    };
    for seed in [0, 11, 181, u64::MAX] {
        assert_eq!(sequence(seed), sequence(seed), "seed {seed}");
    }
    assert_ne!(sequence(3), sequence(4), "two seeds drew one sequence");
}

/// **One value from the menagerie's stream, and nothing from any other** —
/// checked on every stream's position rather than on the next value alone, so a
/// draw that nudged a second stream fails too.
#[test]
fn a_draw_moves_the_menagerie_stream_by_one_value_and_no_other_stream() {
    for seed in 0..200 {
        for whole in [true, false] {
            let mut drawn = Rngs::from_seed(seed);
            let _ = Shape::draw(&mut drawn, whole);
            let mut stepped = Rngs::from_seed(seed);
            let _: u64 = stepped.stream(RngStream::Menagerie).random();
            assert_eq!(
                drawn.positions(),
                stepped.positions(),
                "seed {seed}, whole {whole}"
            );
        }
    }
}

/// **Every draw is a beast the rules accept**, over twenty thousand of them.
#[test]
fn every_drawn_beast_is_a_puzzle_and_arrives_uncalled_at_the_opening() {
    let mut rngs = Rngs::from_seed(7);
    for _ in 0..20_000 {
        let beast = Beast::draw(&mut rngs, true);
        let Shape::Whole(puzzle) = beast.shape() else {
            panic!("a whole draw was lesser");
        };
        assert!(circuit::is_puzzle(puzzle), "{puzzle:?}");
        assert_eq!(beast.calls(), 0);
        assert!(beast.answer().is_none());
    }
}

/// **Variety, as a player meets it.** A hundred thousand draws from one tower
/// reach every one of the 2,994 puzzles, every wiring, every mask and all 192
/// tempers — the rarest puzzle is two circuits in 9,756, so missing one would be a
/// broken draw rather than bad luck — and each wiring and mask arrives about as
/// often as its circuits say.
#[test]
fn enough_draws_reach_every_puzzle_every_wiring_and_every_turned_mask() {
    let mut rngs = Rngs::from_seed(1);
    let drawn: Vec<Puzzle> = (0..100_000).map(|_| circuit::draw(&mut rngs)).collect();
    let distinct: BTreeSet<Puzzle> = drawn.iter().copied().collect();
    assert_eq!(distinct, puzzles(), "a puzzle was never drawn");
    let tempers: BTreeSet<Temper> = drawn.iter().map(|one| one.temper).collect();
    assert_eq!(tempers.len(), 192);

    let share = |count: usize| count * 1000 / drawn.len();
    for wiring in Wiring::all() {
        let seen = share(drawn.iter().filter(|one| one.wiring == wiring).count());
        // A sixth each: every wiring has the same circuits.
        assert!((150..=183).contains(&seen), "{wiring:?} at {seen}‰");
    }
    for turned in Turned::ALL {
        let seen = share(drawn.iter().filter(|one| one.turned == turned).count());
        let expected = share(
            circuit::drawable()
                .iter()
                .filter(|one| one.turned == turned)
                .count()
                * drawn.len()
                / circuit::drawable().len(),
        );
        assert!(
            seen.abs_diff(expected) <= 10,
            "{turned:?} at {seen}‰ against {expected}‰",
        );
    }
    let untouched = drawn.iter().filter(|one| one.turned.is_none()).count();
    assert!(
        share(untouched) < 150,
        "{untouched} of {} draws turned nothing",
        drawn.len()
    );
}

/// **The first beast of a hundred towers is a hundred towers' worth of
/// variety**, which is what a player starting a new game meets.
#[test]
fn a_hundred_seeds_open_on_many_different_beasts() {
    let first: BTreeSet<Puzzle> = (0..100)
        .map(|seed| circuit::draw(&mut Rngs::from_seed(seed)))
        .collect();
    assert!(
        first.len() >= 90,
        "a hundred seeds opened on {} beasts",
        first.len()
    );
}
