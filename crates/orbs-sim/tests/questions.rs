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

use orbs_sim::parser::{Bound, Condition, SpellState, condition, write_condition};

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
            // **Counted, or the writer half of `has <count>` is unverified.**
            // These three properties all run off this generator, so a shape
            // missing here is a shape nothing round-trips, nothing checks for a
            // dropped name, and nothing pins as deterministic — while all three
            // stay green. `2` and `4` because 1 is the default and writes back
            // *without* the number, which is its own case below.
            out.push(format!("the {place} has 2 {thing}"));
            out.push(format!("the {place} has 4 {thing}"));
            out.push(format!("the {place} has no 2 {thing}"));
            // Both comparator spellings. `or more` round-trips **through** the
            // bare form rather than back to itself, which the property allows
            // because it compares conditions and not text.
            out.push(format!("the {place} has 2 or more {thing}"));
            out.push(format!("the {place} has 2 or fewer {thing}"));
            out.push(format!("the {place} has 0 or fewer {thing}"));
            // **The row that was missing.** `0 or more` is kept uncollapsed by
            // the reader, so the writer has to keep its words too — writing
            // `has 0 X` handed it back as `not has X`, and the round-trip
            // property could not see it because no row generated the shape.
            out.push(format!("the {place} has 0 or more {thing}"));
            out.push(format!("the {place} has no 2 or fewer {thing}"));
        }
        // `has 1 X` and `has 0 X` are the two that do **not** round-trip
        // literally, by design: 1 is written back bare and 0 is `no`. They are
        // asserted by name in A7 rather than fed through the round-trip, which
        // would only be able to say they differ.
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

// ---------------------------------------------------------------------------
// A7. Counting — the two spellings that deliberately do not round-trip
// ---------------------------------------------------------------------------

#[test]
fn a_bare_has_is_a_count_of_one_and_writes_back_bare() {
    // **The compatibility claim, both ways.** Every spell written before
    // counting existed says `has sage`, and it has to keep meaning exactly what
    // it meant — so the default is 1 on the way in, and 1 is invisible on the
    // way out. Quoting `has 1 sage` back at a player who typed `has sage` would
    // be the orb inventing a notation, which is what `write_condition` exists
    // not to do.
    assert_eq!(
        condition("the dispensary has sage"),
        Some(counted("dispensary", "sage", 1)),
    );
    assert_eq!(read("the dispensary has 1 sage"), "dispensary has sage");
}

#[test]
fn a_count_of_nought_is_the_way_no_is_spelled_with_a_digit() {
    // **`has 0 sage` is `has no sage`**, and it is a decision rather than a
    // reading. Taken literally, "at least nought" is satisfied by an empty
    // shelf — a guard that always fires, which is the last thing a player
    // expects from a number they wrote to restrict something.
    assert_eq!(
        condition("the dispensary has 0 sage"),
        condition("the dispensary has no sage"),
    );
    assert_eq!(read("the dispensary has 0 sage"), "not dispensary has sage");
}

#[test]
fn a_number_the_orb_cannot_count_to_is_not_a_count() {
    // Past `u32` it is not a count, and it is **not** silently clamped either.
    // It stays where it was written — part of the thing's name — so the question
    // is about something called `99999999999999 sage`, which nothing is. The
    // spell's compile step then reports that name as one it cannot place, and
    // §8.1 gets its culprit. Saturating to `u32::MAX` would turn a typo into a
    // guard that never fires and never says why.
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
    // **`2` and `2 or more` are one question**, so the bare form is canonical
    // and the spoken one collapses to it. `or fewer` has no other spelling, so
    // it keeps its words — and that asymmetry is what `interpret` shows a player
    // to tell them which direction a bare count means.
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
    // **The table this feature is for.** `has at least 2 X` used to become
    // `has X` — the count *and* the words gone, no fault raised — because `at`
    // is §6 filler and the rest resolved down to the noun. Every row here was a
    // sentence a player would reasonably type and the orb silently rewrote.
    //
    // The right-hand side is the canonical form, so the collapses are visible:
    // at-least writes bare, and a strict comparator becomes the count it means.
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
fn a_bound_with_no_number_keeps_its_words_rather_than_dropping_them() {
    // **Nothing vanishes, which is the whole property.** A bound with no number
    // is not a comparison, so the words go back and `span` takes them into the
    // thing's name — a question about something called `least fragment`, which
    // nothing is, so `compile` reports that name and §8.1 gets its culprit.
    //
    // The same shape as a number too large to count. What must **not** happen is
    // the words disappearing and leaving `has fragment`, which is what
    // `at least 2 fragment` did before comparators were spelled out.
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
    // **The lookahead takes two tokens or neither.** `or` followed by anything
    // but `more`/`fewer`/`less` is left exactly where it was, so a question with
    // a count on its left half still joins.
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
    // `has 0 X` is `has no X` — "at least nought" is satisfied by an empty shelf
    // and is a guard that always fires. `has 0 or fewer X` is a **comparison**
    // asking for an empty shelf and saying so, and collapsing it would make the
    // explicit spelling pointless.
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
    // The short form (`shared`) looks ahead for a question word, not for a
    // number, so a counted operand on the right of an `and` must still find its
    // subject. This is the interaction the count's parse position was chosen for.
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
        count,
        bound,
    }
}

fn idle(place: &str) -> Condition {
    Condition::Is {
        place: place.to_owned(),
        state: SpellState::Idle,
    }
}
