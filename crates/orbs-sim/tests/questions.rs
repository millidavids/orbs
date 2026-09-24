//! The `if` grammar, exhaustively — every shape, every refusal, and the two
//! properties that catch the shapes nobody thought to write down.
//!
//! A table rather than thirty tests: a grammar is a claim about *every* input,
//! so rows make the coverage countable and the properties at the bottom cover
//! what rows cannot.
//!
//! Never `a`, `b`, `c`: `a` is on §6's filler list — `mix a potion` — so the
//! usual metasyntactic names silently test the empty question.

use orbs_sim::parser::{Bound, Condition, SpellState, condition, write_condition};

/// The question `text` asks, written back out — or `-` for one the orb refuses.
///
/// Comparing written forms rather than trees keeps the table readable; the
/// round-trip property below is what makes that safe.
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
    // Checked against the tree at least once: a writer that dropped a bracket
    // and a reader that ignored one would agree with each other for ever.
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
    // The row the grammar turns on: give the bracket words full sub-questions
    // instead of operands and the inner disjunction swallows the following
    // `and`, so `either` stops grouping.
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
    // Kept because refusing a word someone would reasonably write is §6's dead
    // end. Pinned as inert so it cannot quietly acquire a meaning.
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
    // The bug the rewrite exists to close: the old parser took the first `is`,
    // read one word after it and dropped the rest of the line into the player's
    // file for good.
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
    // §8.1: script text is a sabotage surface. The parser may refuse anything;
    // it may not fall over or go quadratic on a long word.
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
    // The tokeniser caps at `MAX_WORDS`, which here would be the original bug
    // again: a question cut to thirty-two words with the rest deciding nothing.
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
            // All three properties run off this generator, so a shape missing
            // here is one nothing checks while all three stay green. `2` and
            // `4` because 1 is the default and writes back bare.
            out.push(format!("the {place} has 2 {thing}"));
            out.push(format!("the {place} has 4 {thing}"));
            out.push(format!("the {place} has no 2 {thing}"));
            // Both spellings. `or more` round-trips through the bare form, not
            // back to itself — the property compares conditions, not text.
            out.push(format!("the {place} has 2 or more {thing}"));
            out.push(format!("the {place} has 2 or fewer {thing}"));
            out.push(format!("the {place} has 0 or fewer {thing}"));
            // The row that was missing: `0 or more` is kept uncollapsed by the
            // reader, so the writer must keep its words — and no row generated
            // the shape, so the round-trip could not see it.
            out.push(format!("the {place} has 0 or more {thing}"));
            out.push(format!("the {place} has no 2 or fewer {thing}"));
        }
        // `has 1 X` and `has 0 X` do not round-trip literally by design: 1 is
        // written back bare and 0 is `no`. Asserted by name in A7 instead.
    }
    // The far side, in every shape it can take: these properties are all that
    // stands between a new `Quantity` variant and a question the orb can read
    // and cannot write back. Multi-word places on both sides deliberately —
    // `balneum mariae` proves the far side stops at a stopper, not a space.
    for near in ["mortar", "balneum mariae"] {
        for far in ["flask", "balneum mariae"] {
            for thing in ["sage", "ground-sage"] {
                for comparative in ["more", "fewer", "as many"] {
                    let closer = if comparative == "as many" {
                        "as"
                    } else {
                        "than"
                    };
                    let head = format!("the {near} has {comparative} {thing} {closer}");
                    // A bare place — the shape that predates the operators.
                    out.push(format!("{head} the {far}"));
                    // ...its own reading over there.
                    out.push(format!("{head} the {far} has charcoal"));
                    // ...doubled, and doubled with a different reading.
                    out.push(format!("{head} double the {far}"));
                    out.push(format!("{head} double the {far} has charcoal"));
                    // ...and `plus`, which binds last and so wraps the rest.
                    out.push(format!("{head} the {far} plus 2"));
                    out.push(format!("{head} the {far} has charcoal plus 3"));
                    out.push(format!("{head} double the {far} plus 4"));
                    out.push(format!("{head} double the {far} has charcoal plus 5"));
                    // `plus 0` on its own: the value that would be invisible if
                    // `strict` were derived from the variant, not the grammar.
                    out.push(format!("{head} the {far} plus 0"));
                    // Negated, so the tree survives being wrapped.
                    out.push(format!("not {head} double the {far} plus 1"));
                }
            }
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
    // Keeps the writer honest: `interpret` quotes a question back, and a
    // written form the parser cannot read is a sentence shown and then refused.
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
    // The property that would have caught the original bug: every name the
    // player typed must survive into what the orb reads.
    //
    // Names, not words — the notation may change (`no` becomes `not`, `free`
    // becomes `idle`), but not what the question is about.
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
    // Twice from the same text is the same tree: a resolution depending on
    // iteration order would make two runs from one seed differ (§13).
    for text in every_shape() {
        assert_eq!(condition(&text), condition(&text), "{text:?}");
    }
}

// ---------------------------------------------------------------------------
// A7. Counting — the two spellings that deliberately do not round-trip
// ---------------------------------------------------------------------------

#[test]
fn a_bare_has_is_a_count_of_one_and_writes_back_bare() {
    // The compatibility claim, both ways: every spell written before counting
    // says `has sage`, so the default is 1 in and 1 is invisible out. Quoting
    // `has 1 sage` back would be the orb inventing a notation.
    assert_eq!(
        condition("the dispensary has sage"),
        Some(counted("dispensary", "sage", 1)),
    );
    assert_eq!(read("the dispensary has 1 sage"), "dispensary has sage");
}

#[test]
fn a_count_of_nought_is_the_way_no_is_spelled_with_a_digit() {
    // `has 0 sage` is `has no sage`, a decision rather than a reading: "at
    // least nought" is satisfied by an empty shelf, so it always fires.
    assert_eq!(
        condition("the dispensary has 0 sage"),
        condition("the dispensary has no sage"),
    );
    assert_eq!(read("the dispensary has 0 sage"), "not dispensary has sage");
}

#[test]
fn a_number_the_orb_cannot_count_to_is_not_a_count() {
    // Past `u32` it is not a count and is not clamped: it stays part of the
    // thing's name, so `compile` reports a name it cannot place and §8.1 gets
    // its culprit. Saturating would turn a typo into a silent guard.
    assert_eq!(
        condition("the dispensary has 99999999999999 sage"),
        Some(counted("dispensary", "99999999999999 sage", 1)),
    );
    // A count with nothing after it is not a question at all: the span is empty,
    // so the line is refused rather than read as a shelf holding a number.
    assert_eq!(condition("the dispensary has 4"), None);
}

#[test]
fn or_more_is_the_default_said_out_loud_and_or_fewer_is_not() {
    // `2` and `2 or more` are one question, so the bare form is canonical. `or
    // fewer` has no other spelling and keeps its words, and that asymmetry is
    // what tells a player which direction a bare count means.
    assert_eq!(
        condition("the cabinet has 2 or more fragment"),
        condition("the cabinet has 2 fragment"),
    );
    assert_eq!(
        read("the cabinet has 2 or more fragment"),
        "cabinet has 2 fragment"
    );
    assert_eq!(
        condition("the cabinet has 2 or fewer fragment"),
        Some(bounded("cabinet", "fragment", 2, Bound::AtMost)),
    );
    assert_eq!(
        read("the cabinet has 2 or fewer fragment"),
        "cabinet has 2 or fewer fragment",
    );
}

#[test]
fn every_spelling_of_a_comparison_reads_and_none_is_swallowed() {
    // `has at least 2 X` used to become `has X` — `at` is §6 filler and the
    // rest resolved down to the noun, so the count and the words went with no
    // fault raised. The right-hand side is the canonical form, so the collapses
    // are visible.
    table(&[
        // At least, six ways.
        ("the cabinet has 2 fragment", "cabinet has 2 fragment"),
        (
            "the cabinet has at least 2 fragment",
            "cabinet has 2 fragment",
        ),
        (
            "the cabinet has 2 or more fragment",
            "cabinet has 2 fragment",
        ),
        (
            "the cabinet has more than 1 fragment",
            "cabinet has 2 fragment",
        ),
        (
            "the cabinet has greater than 1 fragment",
            "cabinet has 2 fragment",
        ),
        ("the cabinet has >= 2 fragment", "cabinet has 2 fragment"),
        ("the cabinet has >=2 fragment", "cabinet has 2 fragment"),
        ("the cabinet has > 1 fragment", "cabinet has 2 fragment"),
        // At most, five.
        (
            "the cabinet has at most 2 fragment",
            "cabinet has 2 or fewer fragment",
        ),
        (
            "the cabinet has 2 or fewer fragment",
            "cabinet has 2 or fewer fragment",
        ),
        (
            "the cabinet has fewer than 3 fragment",
            "cabinet has 2 or fewer fragment",
        ),
        (
            "the cabinet has <= 2 fragment",
            "cabinet has 2 or fewer fragment",
        ),
        (
            "the cabinet has <3 fragment",
            "cabinet has 2 or fewer fragment",
        ),
        // Exactly, which arrived with the symbols and has no other spelling.
        (
            "the cabinet has exactly 2 fragment",
            "cabinet has exactly 2 fragment",
        ),
        (
            "the cabinet has = 2 fragment",
            "cabinet has exactly 2 fragment",
        ),
        (
            "the cabinet has ==2 fragment",
            "cabinet has exactly 2 fragment",
        ),
    ]);
}

#[test]
fn the_far_sides_operators_bind_left_to_right_and_only_once() {
    use orbs_sim::parser::Quantity;

    // Round-tripping does not prove this: `double east plus 2` writes back as
    // itself whichever way it nested, and the two trees are different
    // arithmetic — `2·east + 2` against `2·(east + 2)`. Left to right is what
    // the module doc claims, so `plus` must be the outer node.
    let read = |text: &str| match condition(text) {
        Some(Condition::Has { count, .. }) => count,
        other => panic!("{text:?} did not read as a comparison: {other:?}"),
    };

    assert_eq!(
        read("mortar has more sage than double flask plus 2"),
        Quantity::Plus {
            of: Box::new(Quantity::Doubled(Box::new(Quantity::Elsewhere(
                "flask".to_owned()
            )))),
            by: 2,
        },
        "`double … plus n` nested the wrong way, so it means 2(x+n) not 2x+n",
    );

    // ...and with a reading of its own on the far side, the same order holds.
    assert_eq!(
        read("mortar has more sage than double flask has charcoal plus 3"),
        Quantity::Plus {
            of: Box::new(Quantity::Doubled(Box::new(Quantity::Of {
                place: "flask".to_owned(),
                thing: "charcoal".to_owned(),
            }))),
            by: 3,
        },
    );

    // One `plus`, and a second is refused rather than dropped: there is no
    // associativity to learn, because `condition`'s all-or-nothing rule fails
    // the whole line.
    assert_eq!(
        condition("mortar has more sage than flask plus 2 plus 3"),
        None,
        "a chained `plus` was accepted, which needs a precedence answer",
    );

    // A bare `plus` with no number is not a far side either.
    assert_eq!(condition("mortar has more sage than flask plus"), None);
}

#[test]
fn a_comparison_can_name_another_place_instead_of_a_number() {
    // The sentence the language could not say, and why `threading` is six tiers
    // unrolled by hand: only one side of a comparison could be a world read.
    // The canonical form is the words, so it round-trips as the numbers do.
    table(&[
        (
            "north has fewer marks than east",
            "north has fewer marks than east",
        ),
        (
            "north has more marks than east",
            "north has more marks than east",
        ),
        (
            "north has less marks than east",
            "north has fewer marks than east",
        ),
        (
            "north has as many marks as east",
            "north has as many marks as east",
        ),
        (
            "north has as much marks as east",
            "north has as many marks as east",
        ),
        // The place compared against may be several words. `balneum mariae`
        // and not `mortar and pestle`: a name carrying an `and` is one token in
        // the tower, because `and` is a connective here.
        (
            "the mortar_and_pestle has more sage than the balneum mariae",
            "mortar_and_pestle has more sage than balneum mariae",
        ),
        // It composes with everything else, because it is an operand rather
        // than a clause of its own.
        (
            "north has fewer marks than east and the mortar is idle",
            "north has fewer marks than east and mortar is idle",
        ),
        (
            "not north has fewer marks than east",
            "not north has fewer marks than east",
        ),
    ]);
}

#[test]
fn a_comparative_exists_for_every_bound() {
    // What holds the table complete, and why the writer falls back instead of
    // panicking: a `Bound` with no comparative row writes a question the reader
    // cannot read, breaking the round-trip silently.
    for bound in [Bound::AtLeast, Bound::AtMost, Bound::Exactly] {
        let question = Condition::Has {
            place: "north".to_owned(),
            thing: "marks".to_owned(),
            count: orbs_sim::parser::Quantity::Elsewhere("east".to_owned()),
            bound,
        };
        let written = write_condition(&question);
        assert_eq!(
            condition(&written).as_ref(),
            Some(&question),
            "{bound:?} wrote {written:?}, which does not read back as itself",
        );
    }
}

#[test]
fn a_comparative_with_nothing_to_compare_against_refuses_the_line() {
    // The half-written one must not fall through to the count path: `fewer`
    // would go to the thing's name and `spell::compile` would drop it, so
    // `north has fewer marks` becomes `north has marks` and answers the
    // opposite (§19).
    for half in [
        "north has fewer marks",
        "north has more marks",
        "north has as many marks",
        "north has fewer marks than",
    ] {
        assert_eq!(read(half), "-", "{half:?} was read as something");
    }

    // And the numeric spellings sharing their first word still read: `more than
    // 1 fragment` is `BOUNDS`, with nothing between comparative and closer.
    assert_eq!(
        read("the cabinet has more than 1 fragment"),
        "cabinet has 2 fragment",
    );
    assert_eq!(
        read("the cabinet has fewer than 3 fragment"),
        "cabinet has 2 or fewer fragment",
    );
}

#[test]
fn a_bound_with_no_number_keeps_its_words_rather_than_dropping_them() {
    // Nothing vanishes: a bound with no number is not a comparison, so the
    // words go back into the thing's name and `compile` reports it (§8.1). What
    // must not happen is `has fragment`, which is what `at least 2 fragment`
    // did before comparators were spelled out.
    for (text, kept) in [
        ("the cabinet has at least fragment", "least fragment"),
        ("the cabinet has >= fragment", ">= fragment"),
    ] {
        assert_eq!(read(text), format!("cabinet has {kept}"), "{text:?}");
    }
    // And with nothing after it at all, the word is the thing it looked for.
    assert_eq!(read("the cabinet has exactly"), "cabinet has exactly");
}

#[test]
fn a_bare_or_is_still_a_disjunction() {
    // The lookahead takes two tokens or neither: `or` followed by anything but
    // `more`/`fewer`/`less` is left where it was.
    assert_eq!(
        condition("the dispensary has 2 sage or the mortar is idle"),
        Some(Condition::Any(vec![
            counted("dispensary", "sage", 2),
            idle("mortar"),
        ])),
    );
}

#[test]
fn a_count_of_nought_collapses_only_when_it_is_not_a_comparison() {
    // `has 0 X` is `has no X` — "at least nought" always fires. `has 0 or fewer
    // X` is a comparison asking for an empty shelf, and collapsing it would
    // make the explicit spelling pointless.
    assert_eq!(
        condition("the dispensary has 0 sage"),
        condition("the dispensary has no sage"),
    );
    assert_eq!(
        condition("the dispensary has 0 or fewer sage"),
        Some(bounded("dispensary", "sage", 0, Bound::AtMost)),
    );
    assert_ne!(
        condition("the dispensary has 0 or fewer sage"),
        condition("the dispensary has no sage"),
    );
}

#[test]
fn a_count_survives_a_short_form_and_a_join() {
    // The short form looks ahead for a question word, not a number, so a
    // counted operand right of an `and` must still find its subject.
    assert_eq!(
        condition("the cabinet has 4 fragment and 2 sage"),
        Some(Condition::All(vec![
            counted("cabinet", "fragment", 4),
            counted("cabinet", "sage", 2),
        ])),
    );
}

fn counted(place: &str, thing: &str, count: u32) -> Condition {
    bounded(place, thing, count, Bound::AtLeast)
}

/// `has <count> or fewer <thing>`, and its at-least twin.
fn bounded(place: &str, thing: &str, count: u32, bound: Bound) -> Condition {
    Condition::Has {
        place: place.to_owned(),
        thing: thing.to_owned(),
        count: orbs_sim::parser::Quantity::Count(count),
        bound,
    }
}

fn idle(place: &str) -> Condition {
    Condition::Is {
        place: place.to_owned(),
        state: SpellState::Idle,
    }
}
