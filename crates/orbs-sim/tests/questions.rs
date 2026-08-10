//! The `if` grammar, exhaustively — every shape, every refusal, and the two
//! properties that catch the shapes nobody thought to write down.
//!
//! # Why this is a table and not thirty tests
//!
//! A grammar is a claim about *every* input, so the interesting failures are the
//! ones outside the cases anyone chose. Rows make the coverage countable and put
//! a wrong answer next to the right one in the failure message; the properties at
//! the bottom cover what rows cannot.
//!
//! # Never `a`, `b`, `c`
//!
//! `a` is on §6's filler list — `mix a potion` — so a grammar table written with
//! the usual metasyntactic names silently tests the empty question. The names
//! here are real instruments for exactly that reason, and it cost an afternoon
//! to learn.

use orbs_sim::parser::{Condition, SpellState, condition, write_condition};

/// The question `text` asks, written back out — or `-` for one the orb refuses.
///
/// Comparing written forms rather than trees keeps a fifty-row table readable,
/// and the round-trip property below is what makes that safe: the written form
/// is a faithful rendering of the tree, so two questions that write the same are
/// the same question.
fn read(text: &str) -> String {
    condition(text)
        .as_ref()
        .map_or_else(|| "-".to_owned(), write_condition)
}

/// Assert a table of `(input, expected)` in one go, reporting every failure.
fn table(rows: &[(&str, &str)]) {
    let wrong: Vec<String> = rows
        .iter()
        .filter_map(|(input, want)| {
            let got = read(input);
            (got != *want).then(|| format!("\n  {input:?}\n    want {want:?}\n    got  {got:?}"))
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "{} rows wrong:{}",
        wrong.len(),
        wrong.join("")
    );
}

// ---------------------------------------------------------------------------
// A1. Leaves and phrasings
// ---------------------------------------------------------------------------

#[test]
fn every_way_of_asking_one_question() {
    table(&[
        ("the mortar is idle", "mortar is idle"),
        ("mortar is free", "mortar is idle"),
        ("mortar is still", "mortar is idle"),
        ("mortar is working", "mortar is working"),
        ("mortar is busy", "mortar is working"),
        ("mortar is running", "mortar is working"),
        ("mortar is empty", "mortar is empty"),
        ("mortar is bare", "mortar is empty"),
        ("the dispensary has sage", "dispensary has sage"),
        // Negation, in the three spellings English offers.
        ("the mortar is not idle", "not mortar is idle"),
        ("not the mortar is idle", "not mortar is idle"),
        ("the dispensary has no sage", "not dispensary has sage"),
        // A name of several words, and two that contain the connective.
        ("the balneum mariae is idle", "balneum mariae is idle"),
        ("mortar_and_pestle is idle", "mortar_and_pestle is idle"),
        (
            "the flask_and_rod has ground-sage",
            "flask_and_rod has ground-sage",
        ),
    ]);
}

// ---------------------------------------------------------------------------
// A2. Precedence and grouping
// ---------------------------------------------------------------------------

#[test]
fn precedence_is_not_and_then_and_then_or() {
    table(&[
        (
            "mortar is idle and flask is idle",
            "mortar is idle and flask is idle",
        ),
        (
            "mortar is idle or flask is idle",
            "mortar is idle or flask is idle",
        ),
        // Flattened, not nested pairs: `and` of three is one question about three
        // things, and a tree that nested them would make `write` bracket itself.
        (
            "mortar is idle and flask is idle and alembic is idle",
            "mortar is idle and flask is idle and alembic is idle",
        ),
        (
            "mortar is idle or flask is idle or alembic is idle",
            "mortar is idle or flask is idle or alembic is idle",
        ),
        // The two rows precedence is *for*.
        (
            "mortar is idle and flask is idle or alembic is idle",
            "mortar is idle and flask is idle or alembic is idle",
        ),
        (
            "mortar is idle or flask is idle and alembic is idle",
            "mortar is idle or flask is idle and alembic is idle",
        ),
        (
            "not mortar is idle and flask is idle",
            "not mortar is idle and flask is idle",
        ),
        (
            "not mortar is idle or flask is idle",
            "not mortar is idle or flask is idle",
        ),
        ("not not mortar is idle", "not not mortar is idle"),
    ]);
}

#[test]
fn the_precedence_rows_are_the_trees_they_claim_to_be() {
    // **The written form is a rendering, so at least once it has to be checked
    // against the thing itself.** Otherwise a writer that dropped a bracket and
    // a reader that ignored one would agree with each other for ever.
    assert_eq!(
        condition("mortar is idle and flask is idle or alembic is idle"),
        Some(Condition::Any(vec![
            Condition::All(vec![idle("mortar"), idle("flask")]),
            idle("alembic"),
        ])),
    );
    assert_eq!(
        condition("mortar is idle or flask is idle and alembic is idle"),
        Some(Condition::Any(vec![
            idle("mortar"),
            Condition::All(vec![idle("flask"), idle("alembic")]),
        ])),
    );
}

#[test]
fn either_is_the_bracket_and_reaches_what_precedence_cannot() {
    // **The row this whole grammar turns on.** Give the bracket words full
    // sub-questions instead of operands and the inner disjunction swallows the
    // following `and` — `either … or … and …` then means exactly what it means
    // without the `either`, and the one grouping the language has stops
    // grouping.
    assert_eq!(
        condition("either mortar is idle or flask is idle and alembic is idle"),
        Some(Condition::All(vec![
            Condition::Any(vec![idle("mortar"), idle("flask")]),
            idle("alembic"),
        ])),
        "`either` stopped bracketing",
    );
    assert_eq!(
        condition("mortar is idle and either flask is idle or alembic is idle"),
        Some(Condition::All(vec![
            idle("mortar"),
            Condition::Any(vec![idle("flask"), idle("alembic")]),
        ])),
    );

    table(&[
        (
            "either mortar is idle or flask is idle or alembic is idle",
            "mortar is idle or flask is idle or alembic is idle",
        ),
        (
            "not either mortar is idle or flask is idle",
            "not either mortar is idle or flask is idle",
        ),
        (
            "either not mortar is idle or not flask is idle",
            "not mortar is idle or not flask is idle",
        ),
        // Three deep — and the inner `either` flattens into the outer one,
        // because `or` is associative and one question should have one tree.
        (
            "mortar is idle and either flask is idle or either alembic is idle or athanor is idle",
            "mortar is idle and either flask is idle or alembic is idle or athanor is idle",
        ),
    ]);
}

#[test]
fn both_is_a_courtesy_that_changes_no_meaning() {
    // Kept because a player who reaches for it should get what they meant, and
    // refusing a word someone would reasonably write is the dead end §6 forbids.
    // Pinned as **inert** so it cannot quietly acquire a meaning later.
    for (with, without) in [
        (
            "both mortar is idle and flask is idle",
            "mortar is idle and flask is idle",
        ),
        (
            "both mortar is idle and flask is idle or alembic is idle",
            "mortar is idle and flask is idle or alembic is idle",
        ),
        (
            "either mortar is idle or both flask is idle and alembic is idle",
            "mortar is idle or flask is idle and alembic is idle",
        ),
    ] {
        assert_eq!(
            condition(with),
            condition(without),
            "`both` changed the meaning of {with:?}",
        );
    }
}

// ---------------------------------------------------------------------------
// A3. A subject shared between operands
// ---------------------------------------------------------------------------

#[test]
fn a_subject_left_off_is_the_one_before_it() {
    table(&[
        (
            "the dispensary has sage and charcoal",
            "dispensary has sage and dispensary has charcoal",
        ),
        (
            "the dispensary has sage or charcoal",
            "dispensary has sage or dispensary has charcoal",
        ),
        (
            "the mortar is idle and empty",
            "mortar is idle and mortar is empty",
        ),
        (
            "the athanor is idle or working",
            "athanor is idle or athanor is working",
        ),
        // `no` and `not` attach to the operand they precede, and to no other.
        (
            "the dispensary has no sage and no charcoal",
            "not dispensary has sage and not dispensary has charcoal",
        ),
        (
            "the dispensary has no sage and charcoal",
            "not dispensary has sage and dispensary has charcoal",
        ),
        (
            "the mortar is not idle and working",
            "not mortar is idle and mortar is working",
        ),
        // ...and precedence applies to the short form exactly as to the long one,
        // which is what carrying the subject on the reader buys.
        (
            "the dispensary has sage and charcoal or rock-salt",
            "dispensary has sage and dispensary has charcoal or dispensary has rock-salt",
        ),
    ]);
}

#[test]
fn an_operand_that_brings_its_own_question_is_a_clause() {
    table(&[
        (
            "the dispensary has sage and the mortar is idle",
            "dispensary has sage and mortar is idle",
        ),
        (
            "the mortar is idle and the dispensary has sage",
            "mortar is idle and dispensary has sage",
        ),
        (
            "the mortar is idle and the mortar is empty",
            "mortar is idle and mortar is empty",
        ),
        (
            "the dispensary has sage and the flask has ground-sage and the mortar is idle",
            "dispensary has sage and flask has ground-sage and mortar is idle",
        ),
    ]);
}

// ---------------------------------------------------------------------------
// A4. Refusals
// ---------------------------------------------------------------------------

#[test]
fn everything_must_be_read_or_nothing_is() {
    // **The bug the whole rewrite exists to close.** The parser before this took
    // the first `is`, read one word after it and dropped the rest of the line —
    // so the first row here was saved into the player's file as
    // `if mortar is idle`, permanently, with the other half gone.
    table(&[
        ("the mortar is idle the athanor is working", "-"),
        ("the mortar is idle rubbish", "-"),
        ("the dispensary has sage the mortar is idle", "-"),
    ]);
}

#[test]
fn a_question_the_orb_cannot_read_is_refused_rather_than_half_read() {
    let refused = [
        "",
        "the",
        "and",
        "or",
        "not",
        "either",
        "both",
        "no",
        "mortar is idle and",
        "mortar is idle or",
        "or mortar is idle",
        "and mortar is idle",
        "mortar is idle and and flask is idle",
        "mortar is idle or or flask is idle",
        "either mortar is idle",
        "both mortar is idle",
        "either mortar is idle and flask is idle",
        "both mortar is idle or flask is idle",
        "mortar is gibbous",
        "mortar is",
        "dispensary has",
        "is idle",
        "has sage",
        "mortar is idle is working",
        "mortar has is",
        "not not",
        "either or",
        "either mortar is idle or",
        "mortar idle",
        "mortar",
        "not either mortar is idle",
    ];
    let read: Vec<(&str, String)> = refused
        .iter()
        .map(|input| (*input, read(input)))
        .filter(|(_, got)| got != "-")
        .collect();
    assert!(
        read.is_empty(),
        "the orb read something it should not have: {read:?}"
    );
}

// ---------------------------------------------------------------------------
// A5. Robustness — the inputs a file can hold that a person would not type
// ---------------------------------------------------------------------------

#[test]
fn layout_and_case_and_punctuation_do_not_change_a_question() {
    let want = read("the mortar is idle and the dispensary has sage");
    assert_ne!(want, "-");
    for spelling in [
        "   the mortar is idle and the dispensary has sage   ",
        "the  mortar   is idle  and the dispensary has  sage",
        "\tthe mortar is idle and the dispensary has sage",
        "THE MORTAR IS IDLE AND THE DISPENSARY HAS SAGE",
        "The Mortar Is Idle And The Dispensary Has Sage",
        "the mortar is idle and the dispensary has sage.",
        "the mortar is idle and the dispensary has sage?",
    ] {
        assert_eq!(read(spelling), want, "{spelling:?} read differently");
    }
}

#[test]
fn nothing_a_file_can_hold_makes_the_parser_panic() {
    // **§8.1: script text is a sabotage surface**, so the parser is something an
    // enemy writes into. It may refuse anything; it may not fall over, and it may
    // not go quadratic on a long word.
    let long = "x".repeat(4096);
    let deep = "not ".repeat(200) + "mortar is idle";
    let brackets = "either ".repeat(200) + "mortar is idle";
    for input in [
        long.as_str(),
        deep.as_str(),
        brackets.as_str(),
        "mortar is idlé",
        "mortar is 🜁 idle",
        "the 🜂 is idle",
        "mortar is idle and 🜃 has sage",
        &"mortar is idle and ".repeat(40),
        "is is is is",
        "has has has",
        "not not not not not",
        "either or either or",
    ] {
        // Whatever it answers, it answers.
        let _ = condition(input);
    }
}

#[test]
fn a_question_too_long_to_read_is_refused_rather_than_shortened() {
    // The tokeniser caps at `MAX_WORDS`, which is right for a typed command and
    // would be the original bug all over again here: a question quietly cut to
    // its first thirty-two words, with the rest deciding nothing.
    let long = std::iter::repeat_n("mortar is idle", 20)
        .collect::<Vec<_>>()
        .join(" and ");
    assert_eq!(read(&long), "-", "a long question was silently shortened");
}

// ---------------------------------------------------------------------------
// A6. Properties
// ---------------------------------------------------------------------------

/// Every question in this file's tables, as text.
fn every_shape() -> Vec<String> {
    let places = ["mortar", "flask", "balneum mariae", "athanor"];
    let states = ["idle", "working", "empty"];
    let things = ["sage", "charcoal", "ground-sage"];
    let mut out = Vec::new();
    for place in places {
        for state in states {
            out.push(format!("the {place} is {state}"));
            out.push(format!("the {place} is not {state}"));
        }
        for thing in things {
            out.push(format!("the {place} has {thing}"));
            out.push(format!("the {place} has no {thing}"));
        }
    }
    // ...and every join of two of them, both ways round, bracketed and not.
    let leaves: Vec<String> = out.clone();
    for left in leaves.iter().take(8) {
        for right in leaves.iter().skip(3).take(8) {
            out.push(format!("{left} and {right}"));
            out.push(format!("{left} or {right}"));
            out.push(format!("not {left} and {right}"));
            out.push(format!("either {left} or {right}"));
            out.push(format!("both {left} and {right}"));
            out.push(format!("{left} and either {right} or {left}"));
            out.push(format!("either {left} or {right} and {left}"));
        }
    }
    out
}

#[test]
fn every_question_the_orb_can_write_it_can_read_back() {
    // **The property that keeps the writer honest.** `interpret` and the line
    // that names a place it could not find both quote a question back, and a
    // written form the parser cannot read is a sentence the game shows a player
    // and then refuses.
    let mut checked = 0;
    for text in every_shape() {
        let Some(first) = condition(&text) else {
            continue;
        };
        let written = write_condition(&first);
        assert_eq!(
            condition(&written).as_ref(),
            Some(&first),
            "written back as {written:?}, which reads as something else",
        );
        // ...and writing it again gives the same text, so the form is a fixed
        // point rather than something that drifts each time it is quoted.
        let again = condition(&written).expect("it read back a moment ago");
        assert_eq!(write_condition(&again), written);
        checked += 1;
    }
    assert!(checked > 400, "the sweep only reached {checked} questions");
}

#[test]
fn no_name_in_a_question_is_ever_dropped() {
    // **The property that would have caught the original bug**, and the one
    // worth more than any single row above: every *name* the player typed must
    // survive into what the orb reads. A question that loses one is the failure
    // this whole change exists to prevent, whatever shape it arrives in.
    //
    // Names, not words — the notation is allowed to change, and does: `no`
    // becomes `not`, `free` becomes `idle`, `both` disappears because it never
    // meant anything. What may not change is what the question is *about*.
    let names = [
        "mortar",
        "flask",
        "balneum mariae",
        "athanor",
        "sage",
        "charcoal",
        "ground-sage",
    ];
    for text in every_shape() {
        let Some(question) = condition(&text) else {
            continue;
        };
        let written = write_condition(&question);
        for name in names {
            let typed = text.matches(name).count();
            assert!(
                typed == 0 || written.contains(name),
                "{name:?} vanished: {text:?} became {written:?}",
            );
        }
    }
}

#[test]
fn reading_a_question_is_deterministic() {
    // Twice from the same text is the same tree — pinned because a resolution
    // that depended on iteration order would make two runs from one seed differ,
    // which §13's determinism spine forbids and which no other test here would
    // notice.
    for text in every_shape() {
        assert_eq!(condition(&text), condition(&text), "{text:?}");
    }
}

fn idle(place: &str) -> Condition {
    Condition::Is {
        place: place.to_owned(),
        state: SpellState::Idle,
    }
}
