//! What a running spell must and must not do.

use crate::Sim;
use orbs_render::FieldName;

/// Every message the orb has said.
fn said(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| record.field(FieldName::Message))
        .filter_map(|value| match value {
            orbs_render::Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Whether anything the orb said contains `needle`.
fn mentioned(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

/// A sim with `lines` saved as a *laboratory* spell, ready to invoke.
///
/// A spell takes its domain from where it is written, and `Sim::new` stands at
/// `/tower`, which is not one — so every laboratory line would be flagged
/// unreadable and several tests here would pass proving nothing.
fn with_spell(name: &str, lines: &[&str]) -> Sim {
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    let lines: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    sim.write_spell(name, &lines);
    sim.step();
    sim
}

#[test]
fn a_guard_is_asked_before_the_first_pass_and_not_only_after_it() {
    // A guard, not a do-while: `repeat until <already true>` runs zero times.
    // The mortar is idle to begin with, so the guard holds on entry and the
    // grind never happens.
    let mut sim = with_spell(
        "check",
        &["repeat until the mortar is idle", "grind sage", "end"],
    );
    sim.submit("invoke check");
    sim.step_n(8);
    assert!(
        !mentioned(&sim, "dispensary to mortar"),
        "the body ran despite the guard holding on entry: {:?}",
        said(&sim),
    );
}

#[test]
fn a_guard_stops_the_loop_when_the_body_makes_it_true() {
    // The other half: asked again at the end of each pass, so a loop whose body
    // satisfies its own guard runs once and finishes.
    let mut sim = with_spell(
        "check",
        &["repeat until the mortar is working", "grind sage", "end"],
    );
    sim.submit("invoke check");
    sim.step_n(8);
    assert_eq!(
        said(&sim)
            .iter()
            .filter(|line| line.contains("dispensary to mortar"))
            .count(),
        1,
        "the guard did not stop the loop after one pass: {:?}",
        said(&sim),
    );
    assert!(
        mentioned(&sim, "is finished"),
        "the spell never ended: {:?}",
        said(&sim),
    );
}

#[test]
fn a_guard_that_cannot_be_answered_stops_the_loop_rather_than_spinning() {
    // The opposite of `if`'s rule: an unreadable `if` declines to act, but a
    // `repeat` that declined to *stop* would run for ever (§19). `mortr` names
    // no place, so `holds` has no answer at all.
    let mut sim = with_spell(
        "check",
        &["repeat until the mortr is working", "grind sage", "end"],
    );
    sim.submit("invoke check");
    sim.step_n(10);
    assert!(
        mentioned(&sim, "is finished"),
        "an unanswerable guard left the loop spinning: {:?}",
        said(&sim),
    );
}

#[test]
fn a_spell_runs_the_laboratory_and_leaves_a_product() {
    // Asserts a product, not a completion: the failure looks like success —
    // every line executes and the spell makes nothing, because each stage
    // siphoned an instrument whose run had not finished.
    let mut sim = with_spell(
        "brewing",
        &["kindle charcoal", "grind sage", "empty mortar_and_pestle"],
    );

    sim.submit("invoke brewing");
    sim.step_n(60);

    // Named, not counted: "a reagent that is not sage" is answered for free by
    // the dispensary's `rock-salt`, so it passed either way.
    sim.submit("attend dispensary");
    sim.step();
    let shelved: Vec<&str> = sim
        .scene()
        .nouns()
        .iter()
        .map(|noun| noun.name.as_str())
        .collect();
    assert!(
        shelved.contains(&"ground-sage"),
        "the spell finished without grinding anything: {shelved:?} / {:?}",
        said(&sim),
    );
}

#[test]
fn a_spell_waits_for_the_instrument_rather_than_being_refused() {
    // A script reaching a busy instrument is the next line of a recipe arriving
    // early, not a mistake. The refusal path would complain once per tick for
    // the whole of a run that is going fine.
    let mut sim = with_spell(
        "brewing",
        &[
            "attend laboratory",
            "kindle charcoal",
            "grind sage",
            "empty mortar_and_pestle",
        ],
    );
    sim.submit("invoke brewing");
    sim.step_n(4);

    let waits = said(&sim)
        .iter()
        .filter(|line| line.contains("waits"))
        .count();
    assert!(waits <= 1, "the wait was announced {waits} times, not once");
    assert!(
        !mentioned(&sim, "is working. wait"),
        "a script was refused as though it had typed the line: {:?}",
        said(&sim),
    );
}

#[test]
fn no_instruction_in_a_spell_is_ever_refused_for_being_busy() {
    // The predicate asked "does this verb start work", which `siphon` does not,
    // so a spell that ground then siphoned was refused. The question is "would
    // this be refused" — asserted on the refusal prose rather than a verb list,
    // so a new refusing verb fails here until the predicate learns it.
    let mut sim = with_spell(
        "brewing",
        &[
            "attend laboratory",
            "kindle charcoal",
            "grind sage",
            "empty mortar_and_pestle",
            "empty mortar_and_pestle",
        ],
    );
    sim.submit("invoke brewing");
    sim.step_n(60);

    for refusal in ["still at work", "is working. wait", "already scouring"] {
        assert!(
            !mentioned(&sim, refusal),
            "a spell was refused with {refusal:?} instead of waiting: {:?}",
            said(&sim),
        );
    }
    assert!(
        mentioned(&sim, "is finished"),
        "the spell never completed: {:?}",
        said(&sim),
    );
}

#[test]
fn an_endless_spell_can_be_called_off() {
    // `stop` took a `Place` and a spell is a `Script`, so an unbounded `repeat`
    // was unstoppable. The budget bounds a tick, not a session.
    let mut sim = with_spell("loop", &["repeat", "kindle charcoal", "end"]);
    sim.submit("invoke loop");
    sim.step_n(4);

    let before = sim.scrollback().records().len();
    sim.submit("stop loop");
    sim.step();
    assert!(mentioned(&sim, "you let"), "{:?}", said(&sim));

    // And it is actually stopped — not merely told so.
    let after_stop = sim.scrollback().records().len();
    sim.step_n(6);
    let quiet = sim.scrollback().records().len() - after_stop;
    assert!(
        quiet < 4,
        "the spell kept running after being stopped: {quiet} records in six ticks",
    );
    assert!(before < after_stop, "the spell was never running");
}

#[test]
fn stop_edit_restart_picks_up_the_new_text() {
    // The loop a person works in: cast, see it go wrong, call it off, change it,
    // cast again. Every step has to hold or the editor is write-once.
    let mut sim = with_spell("brewing", &["kindle charcoal"]);
    sim.submit("invoke brewing");
    sim.step_n(2);

    sim.submit("stop brewing");
    sim.step();

    sim.write_spell("brewing", &["grind sage".to_owned()]);
    sim.step();
    assert_eq!(sim.spell("brewing"), Some(vec!["grind sage".to_owned()]));

    sim.submit("invoke brewing");
    sim.step_n(40);
    assert!(
        mentioned(&sim, "yields ground-sage"),
        "the re-cast spell ran the old text: {:?}",
        said(&sim),
    );
}

#[test]
fn editing_a_running_spell_does_not_change_the_run_in_flight() {
    // §8: reloads queue to the next tick boundary, so a file cannot change under
    // a script mid-execution. The program is derived at cast, so the running
    // copy keeps its orders; `scribe` says so rather than seeming to ignore it.
    let mut sim = with_spell("loop", &["repeat", "kindle charcoal", "end"]);
    sim.submit("invoke loop");
    sim.step_n(2);

    sim.write_spell("loop", &["grind sage".to_owned()]);
    sim.step_n(4);

    assert!(
        !mentioned(&sim, "yields ground-sage"),
        "the text changed under a spell that was already running: {:?}",
        said(&sim),
    );
    // ...and the new text is what the *next* cast gets.
    sim.submit("stop loop");
    sim.step();
    sim.submit("invoke loop");
    sim.step_n(40);
    assert!(mentioned(&sim, "yields ground-sage"), "{:?}", said(&sim));
}

#[test]
fn stopping_a_spell_does_not_stop_what_it_started() {
    // Calling a spell off is walking away from it; the brew it began runs on,
    // as it would if you had typed the line by hand.
    let mut sim = with_spell("brewing", &["kindle charcoal", "grind sage"]);
    sim.submit("invoke brewing");
    sim.step_n(2);
    sim.submit("stop brewing");
    sim.step_n(40);

    assert!(
        mentioned(&sim, "yields ground-sage"),
        "stopping the spell also killed the grind it had started: {:?}",
        said(&sim),
    );
}

#[test]
fn a_spell_does_not_move_the_player() {
    // `attend` writes `Cwd`, so a spell containing one would teleport the player
    // and change what they can name from somewhere else.
    //
    // Bound, not invoked: an invocation ends the moment its caster leaves (§19),
    // so an invoked spell would stop before reaching the `attend`. A held spell
    // is the one that runs while the player is elsewhere.
    let mut sim = with_spell("wander", &["attend laboratory", "survey"]);
    crate::tower::credit(sim.world_mut(), 16);
    sim.submit("bind wander");
    sim.step();

    sim.submit("attend archive");
    sim.step();
    let before = sim.location();
    sim.step_n(6);

    assert_eq!(
        sim.location(),
        before,
        "the spell walked the player out of the archive",
    );
    assert!(
        mentioned(&sim, "will not do"),
        "the spell's `attend` was allowed: {:?}",
        said(&sim),
    );
}

#[test]
fn a_line_naming_something_gone_is_logged_and_the_spell_carries_on() {
    // §8's failure taxonomy, titled *"scripts always log and never halt"*.
    let mut sim = with_spell("rough", &["xyzzy plugh", "attend laboratory"]);
    sim.submit("invoke rough");
    sim.step_n(6);

    assert!(
        mentioned(&sim, "line 1"),
        "the bad line was not reported: {:?}",
        said(&sim),
    );
    assert!(
        mentioned(&sim, "is finished"),
        "the spell halted on a line it should have skipped: {:?}",
        said(&sim),
    );
}

#[test]
fn a_spell_may_not_open_the_editor_or_pass_an_hour() {
    // Three verbs are hazards rather than nonsense. `scribe` from inside a
    // script would put the *player's* next keystrokes into a spell they did not
    // open; `meditate` runs its whole count inside one `step()`.
    let mut sim = with_spell("bad", &["scribe elsewhere", "meditate 3600"]);
    sim.submit("invoke bad");
    sim.step_n(6);

    assert!(
        sim.opening().is_none(),
        "a spell opened the editor behind the player",
    );
    assert!(
        mentioned(&sim, "will not do"),
        "the forbidden verbs were not reported: {:?}",
        said(&sim),
    );
    // ...and the clock did not jump an hour.
    assert!(sim.tick().get() < 100, "a scripted meditate ran");
}

#[test]
fn a_spell_may_not_end_the_session() {
    // `may_issue` was the one place `quit` was missed when it was added, so a
    // spell could raise `Quitting`. The three verbs barred beside it seize the
    // keyboard; this one closes the game — and a bound spell re-casts every time
    // it runs off the end, so it would end the session on the orb's clock.
    let mut sim = with_spell("leaving", &["quit"]);
    sim.submit("invoke leaving");
    sim.step_n(6);

    assert!(!sim.quitting(), "a spell ended the player's session",);
    assert!(
        mentioned(&sim, "will not do"),
        "the scripted `quit` was allowed through: {:?}",
        said(&sim),
    );
}

#[test]
fn an_empty_or_unknown_spell_says_so_rather_than_running_nothing() {
    // §6 forbids a bare error, and a cheerful "begun" over a spell that does
    // nothing is worse than one.
    let mut sim = Sim::new(1);
    sim.submit("invoke first_light");
    sim.step();
    assert!(mentioned(&sim, "takes up"), "{:?}", said(&sim));

    let mut empty = with_spell("blank", &[""]);
    empty.submit("invoke blank");
    empty.step();
    assert!(mentioned(&empty, "nothing written"), "{:?}", said(&empty));
}

#[test]
fn invoking_a_running_spell_twice_does_not_start_it_twice() {
    // Long enough to outlive one tick's `SCRIPT_BUDGET`, so the second `invoke`
    // lands on a spell in flight. A shorter one finishes inside the first
    // `step`, where starting it again is the right answer.
    let mut sim = with_spell(
        "slow",
        &[
            "attend laboratory",
            "kindle charcoal",
            "grind sage",
            "empty mortar_and_pestle",
            "empty mortar_and_pestle",
            "survey",
        ],
    );
    sim.submit("invoke slow");
    sim.step();
    assert!(
        !mentioned(&sim, "is finished"),
        "the spell finished inside one tick, so nothing is being tested",
    );

    sim.submit("invoke slow");
    sim.step();
    assert!(mentioned(&sim, "already running"), "{:?}", said(&sim));
}

#[test]
fn a_spell_may_invoke_another_but_not_endlessly() {
    // §8 permits it to a call-depth limit of 3: exhausting the execution budget
    // instead logs at high verbosity only, so automation would stop silently.
    let mut sim = Sim::new(1);
    for (name, lines) in [
        ("one", vec!["invoke two"]),
        ("two", vec!["invoke three"]),
        ("three", vec!["invoke four"]),
        ("four", vec!["survey"]),
    ] {
        let lines: Vec<String> = lines.into_iter().map(str::to_owned).collect();
        sim.write_spell(name, &lines);
        sim.step();
    }

    sim.submit("invoke one");
    sim.step_n(10);

    assert!(
        mentioned(&sim, "takes up two.spell"),
        "a spell could not invoke another at all: {:?}",
        said(&sim),
    );
    assert!(
        mentioned(&sim, "too many spells deep"),
        "the depth limit never fired: {:?}",
        said(&sim),
    );
}

#[test]
fn a_spell_spends_a_tick_a_line() {
    // The mechanic, not a detail: at a budget of 4 a long spell and a tight one
    // cost the same, so there was nothing for an efficient script to be better
    // at. Asserted through `running_line`, which the editor's marker draws from.
    let mut sim = with_spell("slow", &["survey", "survey", "survey", "survey"]);
    sim.submit("invoke slow");
    sim.step();

    // `running_line` is where the orb goes next, and the casting tick already
    // runs line 1 — so a four-line spell stands on line 2 here.
    for expected in 2..=4 {
        assert_eq!(
            sim.running_line("slow"),
            Some(expected),
            "the spell was not on line {expected} after {} ticks",
            expected - 1,
        );
        sim.step();
    }
    assert_eq!(
        sim.running_line("slow"),
        None,
        "the spell outlived its lines"
    );
}

#[test]
fn saving_over_a_running_spell_changes_it_mid_flight() {
    // Watch a spell go wrong, fix the line, and the next pass takes it — no stop
    // and recast. The endless `repeat` is the only shape still running when the
    // edit lands, so this is about a live reload rather than the next cast.
    let mut sim = with_spell("watch", &["repeat", "survey", "end"]);
    sim.submit("invoke watch");
    sim.step_n(6);
    assert!(
        sim.running_line("watch").is_some(),
        "the spell was not still running when the edit landed",
    );

    let rewritten = [
        "repeat".to_owned(),
        "kindle charcoal".to_owned(),
        "end".to_owned(),
    ];
    sim.write_spell("watch", &rewritten);
    sim.step_n(6);

    assert!(
        mentioned(&sim, "the athanor takes light"),
        "the running spell kept its old orders: {:?}",
        said(&sim),
    );
    // ...and it is still the same invocation. A reload that stopped and recast
    // would look identical in the log above and be a different mechanic.
    assert!(
        !mentioned(&sim, "the orb takes up watch.spell")
            || said(&sim)
                .iter()
                .filter(|line| line.contains("the orb takes up watch.spell"))
                .count()
                == 1,
        "the spell was recast rather than reloaded: {:?}",
        said(&sim),
    );
}

#[test]
fn an_if_names_a_place_the_way_every_other_line_does() {
    // A control word's tail was the one thing the orb did not canonicalise, so
    // `if mortar is empty` never found `mortar_and_pestle` and took the `else`
    // for ever. Asserted on the branch that runs: a test checking `holds`
    // directly would have agreed with the bug.
    let mut sim = with_spell(
        "probe",
        &[
            "if mortar is empty",
            "grind sage",
            "else",
            "purge athanor",
            "end",
        ],
    );
    sim.submit("invoke probe");
    sim.step_n(30);

    assert!(
        mentioned(&sim, "yields ground-sage"),
        "the short name took the else: {:?}",
        said(&sim),
    );
    assert!(
        !mentioned(&sim, "scouring"),
        "the else ran as well: {:?}",
        said(&sim),
    );

    // ...and the file still says what the player wrote. The resolution is the
    // program's, made at cast, not written back into the spell.
    assert_eq!(
        sim.spell("probe").as_deref().and_then(<[String]>::first),
        Some(&"if mortar is empty".to_owned()),
        "the orb rewrote the line it read",
    );
}

#[test]
fn a_question_about_nowhere_says_so_rather_than_answering_no() {
    // §8's *Referent missing*, which is not the same as the answer being no — a
    // condition naming nowhere used to look exactly like a false one, so a spell
    // took the `else` for ever in silence.
    let mut sim = with_spell("probe", &["if the gatehouse is empty", "grind sage", "end"]);
    sim.submit("invoke probe");
    sim.step_n(6);

    assert!(
        mentioned(&sim, "there is no gatehouse here to ask about"),
        "a question about nowhere was answered silently: {:?}",
        said(&sim),
    );
    // Still §8: it logs, and does not halt.
    assert!(
        mentioned(&sim, "is finished"),
        "the spell halted on a bad question: {:?}",
        said(&sim),
    );
}

#[test]
fn both_shapes_of_question_reach_the_world() {
    // `has` and `is` resolve their names through the same pass, and the `has`
    // shape has a second name in it — the thing being looked for.
    let mut sim = with_spell(
        "probe",
        &["if the dispensary has sage", "grind sage", "end"],
    );
    sim.submit("invoke probe");
    sim.step_n(30);

    assert!(
        mentioned(&sim, "yields ground-sage"),
        "the dispensary's sage was not found: {:?}",
        said(&sim),
    );
}

#[test]
fn no_line_a_spell_can_say_has_a_hole_in_it() {
    // An unfilled placeholder draws as itself: `spell_gave_up` asked for
    // `{source}` where `say_failure` supplies `{detail}`. Asserted over the
    // whole stream rather than a list of keys, so a new failure line is covered
    // the day it is written. The spell drives every failure the runner has.
    let mut sim = with_spell(
        "broken",
        &[
            "wait for a moonrise",
            "if the gatehouse is empty",
            "grind moonstone",
            "end",
            "attend dispensary",
        ],
    );
    sim.submit("invoke broken");
    // Long enough for `PATIENCE` to run out on the wait, which is the line that
    // had the hole.
    sim.step_n(crate::tower::spell::PATIENCE + 20);

    let spoken = said(&sim);
    let holes: Vec<&String> = spoken.iter().filter(|line| line.contains('{')).collect();
    assert!(
        holes.is_empty(),
        "the orb said a line with an unfilled placeholder in it: {holes:?}",
    );
    // ...and it really did reach the failures, or the sweep above proves nothing.
    assert!(
        mentioned(&sim, "waited too long"),
        "the wait never gave up, so its line was never said: {:?}",
        said(&sim),
    );
}

#[test]
fn a_reagent_the_shelf_has_run_out_of_is_still_written_down() {
    // Every editor visit re-canonicalised the file against the shelf, and
    // §10.1's verbs take their reagent optionally — so with the sage spent
    // `grind sage` was written back as bare `grind`.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    // Spend the sage: the name is still real, nothing on the shelf answers it.
    sim.submit("grind sage");
    sim.step_n(20);

    sim.write_spell("keep", &["grind sage".to_owned()]);
    sim.step();
    assert_eq!(
        sim.spell("keep"),
        Some(vec!["grind sage".to_owned()]),
        "the orb wrote down a shorter command than it was given: {:?}",
        said(&sim),
    );
}

#[test]
fn loose_phrasing_is_kept_in_the_file_and_understood_when_it_runs() {
    // Both halves: plain-English phrasings must still run, and must no longer be
    // rewritten into the file to do it.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    let typed = vec![
        "make a potion of clarity".to_owned(),
        "look around".to_owned(),
        "grind the sage".to_owned(),
    ];
    sim.write_spell("phrase", &typed);
    sim.step();

    assert_eq!(sim.spell("phrase"), Some(typed), "the orb rewrote the file");

    sim.submit("invoke phrase");
    sim.step_n(30);
    assert!(
        mentioned(&sim, "yields ground-sage"),
        "the loose phrasing stopped working: {:?}",
        said(&sim),
    );
}

#[test]
fn saving_a_spell_says_nothing_at_all() {
    // The buffer autosaves after every pause in the typing, so a sentence per
    // save filled the transcript with copies of `5 lines, written down`.
    // Nothing was lost: an unreadable line is named, with its number, at cast.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    let before = sim.scrollback().records().len();

    sim.write_spell(
        "quiet",
        &["grind sage".to_owned(), "xyzzy plugh".to_owned()],
    );
    sim.step();

    assert_eq!(
        sim.spell("quiet"),
        Some(vec!["grind sage".to_owned(), "xyzzy plugh".to_owned()]),
        "the save did not land",
    );
    assert_eq!(
        sim.scrollback().records().len(),
        before,
        "a save put something in the transcript: {:?}",
        said(&sim),
    );
}

#[test]
fn a_running_spell_says_it_reloaded_once_per_session_however_often_it_is_saved() {
    // The one thing a save still says: editing a running spell lands now, which
    // is otherwise indistinguishable from being ignored. Once per session,
    // though, or the autosave restores the noise this item removed.
    // Unbounded, so it is still running when the second session opens — one that
    // had finished would report no reload for the wrong reason.
    let looping: Vec<String> = ["repeat", "survey", "end"]
        .iter()
        .map(|line| (*line).to_owned())
        .collect();
    let mut sim = with_spell("watched", &["repeat", "survey", "end"]);
    sim.submit("invoke watched");
    sim.step();
    sim.submit("scribe watched");
    sim.step();

    for _ in 0..3 {
        sim.write_spell("watched", &looping);
        sim.step();
    }
    let after_three_saves = said(&sim)
        .iter()
        .filter(|line| line.contains("under the orb's hand"))
        .count();
    assert_eq!(after_three_saves, 1, "{:?}", said(&sim));

    // ...and reopening the editor is a new session, so it says it again.
    sim.submit("scribe watched");
    sim.step();
    sim.write_spell("watched", &looping);
    sim.step();
    let after_reopening = said(&sim)
        .iter()
        .filter(|line| line.contains("under the orb's hand"))
        .count();
    assert_eq!(after_reopening, 2, "{:?}", said(&sim));
}

#[test]
fn the_indentation_is_the_players_too() {
    // The four branches that kept a line's words all wrote `line.trim()`, so the
    // orb re-indented what it rewrote and dropped the indent from what it did
    // not. Nothing re-indents now: the buffer indents as you type
    // (`editor::reindent`). This spell arrives through `write_spell` with
    // deliberately unhelpful layout, so tidy output means something tidied it.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    sim.submit("grind sage");
    sim.step_n(20);

    let typed = vec![
        "repeat".to_owned(),
        "attend dispensary".to_owned(), // refused at run time, kept here
        "        invoke somewhere_else".to_owned(), // over-indented, on purpose
        "grind sage".to_owned(),        // the shelf is bare; still the line
        "frobnicate the thing".to_owned(), // unreadable, still the line
        "end".to_owned(),
    ];
    sim.write_spell("probe", &typed);
    sim.step();

    assert_eq!(sim.spell("probe"), Some(typed), "something tidied the file");
}

#[test]
fn a_loop_over_a_base_reagent_never_runs_dry() {
    // §11.5's floor: there is always something to do. The tower held one sage,
    // so this spell ground once and then reported an empty mortar for ever.
    let mut sim = with_spell(
        "grinder",
        &[
            "repeat",
            "if mortar is empty",
            "grind sage",
            "else",
            "empty mortar_and_pestle",
            "end",
            "end",
        ],
    );
    sim.submit("invoke grinder");
    sim.step_n(80);

    let ground = said(&sim)
        .iter()
        .filter(|line| line.contains("yields ground-sage"))
        .count();
    assert!(
        ground >= 3,
        "the loop ground {ground} times before running dry: {:?}",
        said(&sim),
    );
}

#[test]
fn the_sage_survives_being_ground_and_what_it_makes_does_not_stack_up_twice() {
    // Two properties that broke together, and either covers for the other.
    //
    // The sage: `move` took a unit, but charging an instrument was a second copy
    // of the same three lines and re-parented the endless pile itself into the
    // mortar. The pile: `give` merges, so grinding twice adds to `ground-sage`
    // rather than standing a second node beside it under the same name.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    for _ in 0..2 {
        sim.submit("grind sage");
        sim.step_n(12);
        sim.submit("empty mortar_and_pestle");
        sim.step();
    }
    sim.submit("attend dispensary");
    sim.step();

    // Reagents only: every recipe output is also a manual `Topic`, so a bare
    // name count sees `ground-sage` twice however many are on the shelf.
    let shelf: Vec<&str> = sim
        .scene()
        .nouns()
        .iter()
        .filter(|noun| noun.kind == crate::parser::NounKind::Reagent)
        .map(|noun| noun.name.as_str())
        .collect();
    assert!(
        shelf.contains(&"sage"),
        "the endless sage was consumed: {shelf:?}",
    );
    assert_eq!(
        shelf.iter().filter(|name| **name == "ground-sage").count(),
        1,
        "the product stacked up as two nodes under one name: {shelf:?}",
    );
}

#[test]
fn everything_a_spell_does_is_credited_to_it() {
    // A spell emits what the same commands typed by hand emit, so a `repeat`
    // loop scrolled the player's own last line off in seconds. The pane draws
    // what is unattributed; the log keeps everything (§3, rule 4). Asserted over
    // the whole stream, because a spell reaches the ordinary emit sites —
    // including instrument completions landing ticks later, which was missed.
    let mut sim = with_spell(
        "grinder",
        &[
            "repeat",
            "if mortar is empty",
            "grind sage",
            "else",
            "empty mortar_and_pestle",
            "end",
            "end",
        ],
    );
    let before = sim.scrollback().records().len();
    sim.submit("invoke grinder");
    sim.step_n(60);

    let uncredited: Vec<String> = sim
        .scrollback()
        .records()
        .iter()
        .skip(before)
        // The player's own line stays: the `invoke`, its echo, and the orb
        // answering that it has taken the spell up. Those are the command, not
        // its output.
        .filter(|record| {
            !matches!(
                record.kind(),
                orbs_render::RecordKind::Input | orbs_render::RecordKind::Echo
            )
        })
        .filter(|record| record.field(FieldName::Spell).is_none())
        .filter_map(|record| record.field(FieldName::Message))
        .filter_map(|value| match value {
            orbs_render::Value::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .filter(|line| !line.contains("takes up"))
        .collect();

    assert!(
        uncredited.is_empty(),
        "a spell put these in the player's transcript: {uncredited:?}",
    );
    // ...and it really ran, or the sweep above proves nothing.
    assert!(
        mentioned(&sim, "yields ground-sage"),
        "the spell never did anything: {:?}",
        said(&sim),
    );
}

#[test]
fn two_runs_from_one_seed_execute_a_spell_identically() {
    // Rule 3. The runner orders spells by `NodeId` rather than query order —
    // `tower::node` records archetype order as a bug no test catches.
    let run = || {
        let mut sim = with_spell(
            "brewing",
            &[
                "attend laboratory",
                "kindle charcoal",
                "grind sage",
                "empty mortar_and_pestle",
            ],
        );
        sim.submit("invoke brewing");
        sim.step_n(40);
        said(&sim)
    };
    assert_eq!(run(), run(), "the same spell ran differently twice");
}

#[test]
fn a_spell_running_through_a_meditate_lands_where_it_would_have_watched() {
    // §19 chose an interval over a countdown so a skipped span behaves like a
    // watched one. A spell is the first thing that acts across that gap.
    let spell = ["attend laboratory", "kindle charcoal", "grind sage"];

    let mut watched = with_spell("brewing", &spell);
    watched.submit("invoke brewing");
    watched.step_n(40);

    let mut skipped = with_spell("brewing", &spell);
    skipped.submit("invoke brewing");
    skipped.submit("meditate 40");
    skipped.step();

    // The `meditate` echo itself is expected to differ — one session typed it
    // and the other did not. What must match is everything the *spell* did.
    let spell_output = |sim: &Sim| -> Vec<String> {
        said(sim)
            .into_iter()
            .filter(|line| !line.contains("meditate"))
            .collect()
    };
    assert_eq!(
        spell_output(&watched),
        spell_output(&skipped),
        "a spell watched tick by tick did not match one run through a meditate",
    );
}

// ---------------------------------------------------------------------------
// Parts — a named run of lines, and the frame stack that runs one
// ---------------------------------------------------------------------------

#[test]
fn a_part_runs_where_it_is_called_and_not_where_it_is_written() {
    // Reaching `part gathering()` in the ordinary top-to-bottom read must do
    // nothing; the body runs only where `gathering()` says so. The same file
    // twice, with and without the call, so anything the uncalled one does is the
    // definition running where it stands.
    let load = |lines: &[&str]| {
        let mut sim = with_spell("check", lines);
        sim.submit("invoke check");
        sim.step_n(20);
        said(&sim)
            .iter()
            .filter(|line| line.contains("dispensary to mortar"))
            .count()
    };

    let uncalled = load(&["part gathering()", "grind sage", "end"]);
    assert_eq!(uncalled, 0, "the definition ran where it was written");

    let called = load(&["part gathering()", "grind sage", "end", "gathering()"]);
    assert_eq!(called, 1, "the call did not run the part");
}

#[test]
fn one_part_called_twice_runs_twice() {
    // What a part is *for*, and the thing a definition executed in place could
    // never do. Two calls, one body, two grinds.
    let mut sim = with_spell(
        "check",
        &[
            "part gathering()",
            "grind sage",
            "empty mortar_and_pestle",
            "end",
            "repeat 2",
            "gathering()",
            "end",
        ],
    );
    sim.submit("invoke check");
    sim.step_n(60);

    let loads = said(&sim)
        .iter()
        .filter(|line| line.contains("dispensary to mortar"))
        .count();
    assert_eq!(loads, 2, "a part called twice ran {loads} times");
}

#[test]
fn a_part_returns_to_the_line_after_the_call() {
    // The half of a call that a jump would get wrong. The caller's `pc` is kept
    // whole while the callee walks its own tree, and stepping past the call
    // happens on the way *back* — so the line below it runs, exactly once.
    let mut sim = with_spell(
        "check",
        &[
            "part gathering()",
            "grind sage",
            "end",
            "gathering()",
            "empty mortar_and_pestle",
        ],
    );
    sim.submit("invoke check");
    sim.step_n(40);

    let emptied = said(&sim)
        .iter()
        .filter(|line| line.contains("turn the mortar_and_pestle out"))
        .count();
    assert_eq!(
        emptied,
        1,
        "the line after the call ran {emptied} times: {:?}",
        said(&sim),
    );
}

#[test]
fn a_call_inside_a_loop_keeps_the_loop_the_caller_was_in() {
    // A path addresses one tree, and a part is a different tree — so the
    // caller's open blocks have to survive the callee walking its own. Without a
    // stack the `repeat`'s turn count is whatever the part left behind.
    let mut sim = with_spell(
        "check",
        &[
            "part one()",
            "grind sage",
            "empty mortar_and_pestle",
            "end",
            "repeat 3",
            "one()",
            "end",
        ],
    );
    sim.submit("invoke check");
    sim.step_n(90);

    let loads = said(&sim)
        .iter()
        .filter(|line| line.contains("dispensary to mortar"))
        .count();
    assert_eq!(loads, 3, "the loop ran {loads} laps instead of three");
}

#[test]
fn a_part_that_calls_itself_stops_and_says_so() {
    // §8: scripts always log and never halt, so runaway recursion may neither
    // stop the spell nor be silent. Bounded at `MAX_PARTS`, reported by name.
    let mut sim = with_spell("check", &["part spiral()", "spiral()", "end", "spiral()"]);
    sim.submit("invoke check");
    sim.step_n(40);

    assert!(
        mentioned(&sim, "too deep to follow"),
        "runaway recursion was silent: {:?}",
        said(&sim),
    );
    assert!(
        mentioned(&sim, "is finished"),
        "the spell halted rather than carrying on: {:?}",
        said(&sim),
    );
}

#[test]
fn a_part_does_what_the_names_in_its_brackets_say() {
    // The whole feature in one file. `load(sage)` and `load(rock-salt)` are one
    // body doing two different things, which before parameters took a `let`
    // above each call and a shared store between them.
    let mut sim = with_spell(
        "check",
        &[
            "part load(what)",
            "grind what",
            "empty mortar_and_pestle",
            "end",
            "load(sage)",
            "load(rock-salt)",
        ],
    );
    sim.submit("invoke check");
    sim.step_n(60);

    let lines = said(&sim);
    assert!(
        lines.iter().any(|line| line.contains("sage")),
        "the first argument never reached the body: {lines:?}",
    );
    assert!(
        lines.iter().any(|line| line.contains("rock-salt")),
        "the second call ran with the first call's name: {lines:?}",
    );
}

#[test]
fn an_argument_is_what_the_caller_s_name_stands_for() {
    // One level of resolution, at the call, in the caller's store —
    // `substituted`'s rule everywhere else. A literal stands for itself.
    let mut sim = with_spell(
        "check",
        &[
            "part load(what)",
            "grind what",
            "end",
            "let herb be sage",
            "load(herb)",
        ],
    );
    sim.submit("invoke check");
    sim.step_n(30);

    assert!(
        said(&sim).iter().any(|line| line.contains("sage")),
        "a bound name was passed as the word rather than its value: {:?}",
        said(&sim),
    );
}

#[test]
fn a_part_cannot_see_a_name_it_was_not_given() {
    // `herb` is bound in the caller and never passed, so inside the part it
    // resolves against the room, finds nothing, and the line is reported missing
    // rather than quietly grinding sage. One shared store made this file work.
    let mut sim = with_spell(
        "check",
        &[
            "part load()",
            "grind herb",
            "end",
            "let herb be sage",
            "load()",
        ],
    );
    sim.submit("invoke check");
    sim.step_n(30);

    assert!(
        !said(&sim).iter().any(|line| line.contains("dispensary")),
        "the part read a binding it was never handed: {:?}",
        said(&sim),
    );
}

#[test]
fn what_a_part_binds_does_not_outlive_it() {
    // The other half, and what makes an accumulator safe: the caller binds
    // `herb` to sage, the part binds its own to rock-salt, and the grind after
    // the call must still be sage.
    let mut sim = with_spell(
        "check",
        &[
            "part load()",
            "let herb be rock-salt",
            "end",
            "let herb be sage",
            "load()",
            "grind herb",
        ],
    );
    sim.submit("invoke check");
    sim.step_n(30);

    let lines = said(&sim);
    assert!(
        lines.iter().any(|line| line.contains("sage")),
        "a part's binding leaked out and took the caller's with it: {lines:?}",
    );
    assert!(
        !lines.iter().any(|line| line.contains("rock-salt")),
        "the caller ground what the part had bound: {lines:?}",
    );
}

#[test]
fn a_for_each_inside_a_part_leaves_the_caller_s_cursor_alone() {
    // The stated cost of the shared store, now gone — [`Descent`]'s comment used
    // to name it. The caller walks a set, calls a part that walks the same set,
    // and must come back to the member it was on.
    //
    // In the lens rather than the archive: a `dial` always lands and names its
    // socket, where `follow` is refused by a wall and says nothing, so a maze
    // would make this depend on the seed's geometry.
    let mut sim = Sim::new(1);
    sim.submit("attend lens");
    sim.step();
    sim.submit("probe");
    sim.step();
    let lines: Vec<String> = [
        "part inner()",
        "for each socket",
        "end",
        "end",
        "for each socket",
        "inner()",
        "dial socket",
        "end",
    ]
    .iter()
    .map(|line| (*line).to_owned())
    .collect();
    sim.write_spell("check", &lines);
    sim.step();
    sim.submit("invoke check");
    sim.step_n(80);

    // Four sockets, so four dials — one of each. A clobbered cursor leaves every
    // lap dialling whatever the inner loop finished on, which is `fourth`.
    let dialled: Vec<&str> = ["first", "second", "third", "fourth"]
        .into_iter()
        .filter(|socket| mentioned(&sim, &format!("{socket} socket")))
        .collect();
    assert_eq!(
        dialled,
        ["first", "second", "third", "fourth"],
        "the caller's cursor did not survive the call: {:?}",
        said(&sim),
    );
}

#[test]
fn a_call_that_hands_over_the_wrong_number_is_said_at_cast() {
    // A question about the file, so it is answered once at cast rather than on
    // whichever tick the line is reached — possibly never.
    let mut sim = with_spell(
        "check",
        &["part between(here, there)", "end", "between(wellspring)"],
    );
    sim.submit("invoke check");
    sim.step_n(6);
    assert!(
        mentioned(&sim, "different number of names"),
        "a short call was accepted: {:?}",
        said(&sim),
    );
}

#[test]
fn a_heading_that_names_one_thing_twice_sets_nothing_aside() {
    // Binding the second over the first would leave the caller's first argument
    // unreachable. Refused as an unreadable heading, so the body is not set
    // aside under a name nothing can call.
    let mut sim = with_spell(
        "check",
        &["part between(here, here)", "end", "between(a, b)"],
    );
    sim.submit("invoke check");
    sim.step_n(6);
    assert!(
        mentioned(&sim, "part wants a name"),
        "a repeated parameter was accepted: {:?}",
        said(&sim),
    );
}

#[test]
fn the_painter_and_the_parser_agree_about_what_a_call_is() {
    // `lexeme` and `program` each decide independently whether a line is a call,
    // and they disagreed: the painter claimed any one-word head with a bracket
    // pair anywhere, so `move (sage) to mortar` drew magenta. Here rather than
    // in `lexeme` because `call_of` is `pub(super)`.
    for line in [
        "gathering()",
        "between(wellspring, near)",
        "gathering() # note",
        "move (sage) to mortar",
        "grind sage",
        "if the mortar_and_pestle is idle",
        "part between(here, there)",
    ] {
        let painted = crate::parser::lex(line)
            .iter()
            .any(|run| run.kind == orbs_render::Lexeme::Call);
        // A `part` heading carries a call-shaped tail and is not itself a call,
        // so it is asked of the argument, exactly as `read` asks it.
        let text =
            crate::parser::spell_word(line).map_or(line, |_| crate::parser::spell_argument(line));
        let parsed = super::program::call_of(text).is_some();
        assert_eq!(
            painted, parsed,
            "{line:?}: the painter says call={painted}, the parser says call={parsed}",
        );
    }
}

#[test]
fn a_call_to_a_part_nobody_wrote_is_said_at_cast() {
    // At cast rather than when the line is reached, which for a call inside a
    // branch may be never. `check_calls` asks the file, so the answer does not
    // depend on the tower at all.
    let mut sim = with_spell("check", &["missing()"]);
    sim.submit("invoke check");
    sim.step_n(4);
    assert!(
        mentioned(&sim, "no part of this spell is called that"),
        "a call to nothing was accepted: {:?}",
        said(&sim),
    );
}

#[test]
fn the_budget_is_the_floor_until_the_weave_says_otherwise() {
    // Nothing is takeable, so `Taken` is empty and the answer is
    // `SCRIPT_BUDGET`. What has to hold is that the number is read rather than
    // compiled in, and that an untrained orb still gets one.
    let sim = Sim::new(1);
    assert_eq!(super::budget(sim.world()), super::SCRIPT_BUDGET);
    assert_eq!(
        super::SCRIPT_BUDGET,
        1,
        "the floor moved without a decision"
    );
}

// ---------------------------------------------------------------------------
// What a spell may issue, and when it is told
// ---------------------------------------------------------------------------

#[test]
fn a_forbidden_verb_is_refused_when_the_spell_is_cast() {
    // `may_issue` is a security boundary and `run_line` asked it when the line
    // was reached — which inside a branch may be never. The guard here is never
    // taken: `quartz` is not a laboratory reagent, so the body never runs.
    let mut sim = with_spell(
        "risky",
        &[
            "if the dispensary has quartz",
            "meditate 3600",
            "end",
            "survey",
        ],
    );
    sim.submit("invoke risky");
    sim.step_n(4);

    assert!(
        mentioned(&sim, "will not take from a spell"),
        "a forbidden verb in an untaken branch was never mentioned: {:?}",
        said(&sim),
    );
}

#[test]
fn the_cast_check_does_not_fire_on_a_line_that_makes_its_own_input() {
    // Why a whole `Intent` cannot be frozen at cast: a spell makes its own
    // inputs, so at cast the room has no `ground-sage` and `analyse` drops the
    // argument. The verb survives, since a fixture in the room offers it — so
    // the cast check may look at the verb and must say nothing about arguments.
    let mut sim = with_spell(
        "brewing",
        &[
            "kindle charcoal",
            "grind sage",
            "empty mortar_and_pestle",
            "digest ground-sage",
        ],
    );
    sim.submit("invoke brewing");
    sim.step_n(4);

    assert!(
        !mentioned(&sim, "will not take from a spell"),
        "a pipeline line was refused at cast: {:?}",
        said(&sim),
    );
    // ...and it really does reach the world, rather than being quietly dropped.
    sim.step_n(40);
    assert!(
        mentioned(&sim, "dispensary to mortar"),
        "the spell never ran: {:?}",
        said(&sim),
    );
}

#[test]
fn a_spell_that_should_wait_still_waits_rather_than_being_refused() {
    // `would_block` filters `Intent`'s arguments on `NounKind::Place`, so
    // anything that flattens an argument list loses the kind and every waiting
    // spell starts being refused instead — with most of this file still green.
    // Asserted from the outside: a spell whose second line wants the instrument
    // its first just started must wait, and never see a refusal.
    let mut sim = with_spell("brewing", &["grind sage", "empty mortar_and_pestle"]);
    sim.submit("invoke brewing");
    sim.step_n(4);

    assert!(
        mentioned(&sim, "waits"),
        "the spell did not wait for the mortar: {:?}",
        said(&sim),
    );
    for refusal in ["still at work", "is working. wait", "already scouring"] {
        assert!(
            !mentioned(&sim, refusal),
            "the spell was refused with {refusal:?} instead of waiting: {:?}",
            said(&sim),
        );
    }
}

#[test]
fn a_part_is_not_reachable_from_another_spell() {
    // A spell is contained to a single `.spell` file (§19). Held here rather
    // than left as a property of how `program::tree` happens to be written.
    let mut sim = with_spell("lender", &["part gathering()", "grind sage", "end"]);
    let borrower: Vec<String> = ["gathering()".to_owned()].into();
    sim.write_spell("borrower", &borrower);
    sim.step();

    sim.submit("invoke borrower");
    sim.step_n(20);

    assert!(
        mentioned(&sim, "no part of this spell is called that"),
        "a spell reached a part defined in another file: {:?}",
        said(&sim),
    );
    assert!(
        !mentioned(&sim, "dispensary to mortar"),
        "the borrowed part actually ran: {:?}",
        said(&sim),
    );
}
