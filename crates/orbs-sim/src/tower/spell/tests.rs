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

/// A sim with `lines` saved as `name`, ready to invoke.
/// A sim with `lines` saved as a **laboratory** spell, ready to invoke.
///
/// **Standing in the laboratory first is not decoration.** A spell takes its
/// domain from where it is written, and `Sim::new` stands at `/tower` — which is
/// not a domain, so every laboratory line would be flagged unreadable and the
/// spell would run doing nothing at all. Several tests here would then pass
/// while proving nothing.
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
    // **The difference between a guard and a do-while**, and Autonauts' rule:
    // `repeat until <already true>` runs **zero** times. The first pass is where
    // a spell does damage, so a guard that cannot prevent it is not a guard.
    //
    // The mortar is idle to begin with, so `until … is idle` holds on entry and
    // the grind never happens.
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
    // And the other half: asked again at the end of each pass, so a loop whose
    // body satisfies its own guard runs **once** and finishes — rather than for
    // ever, which is what an unbounded `repeat` would have done here and is
    // exactly the guessed-bound problem `until` exists to remove.
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
    // **The opposite of `if`'s rule, deliberately.** An `if` whose question
    // cannot be read declines to act, which is safe. A `repeat` that declined to
    // *stop* would run for ever on a question nobody can answer — §19's "a spell
    // that has stopped describing the world it runs in", left running instead of
    // caught. `mortr` names no place, so `holds` has no answer at all.
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
    // **The test the whole item turns on, and it asserts a *product* rather
    // than a completion.** The failure this exists to catch looks exactly like
    // success: `invoke` runs, every line executes, records appear — and the
    // spell makes nothing, because each stage siphoned an instrument whose run
    // had not finished. A test that checked "the script ran" would pass.
    let mut sim = with_spell(
        "brewing",
        &["kindle charcoal", "grind sage", "empty mortar_and_pestle"],
    );

    sim.submit("invoke brewing");
    sim.step_n(60);

    // **Named, not counted.** This asked whether the scene held "a reagent that
    // is not sage" — which the dispensary answers for free with `rock-salt` and
    // `charcoal`, so it passed whether or not the spell had done anything at
    // all. A test for a product has to name the product.
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
    // A script reaching a busy instrument is not making a mistake — it is the
    // next line of a recipe arriving before the last one finished. Running it
    // through the refusal path would emit one complaint per tick for the whole
    // duration of a run that is going perfectly.
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
    // **Found by looking, not by testing.** The predicate originally asked
    // "does this verb start work", which `siphon` does not — so a spell that
    // ground and then siphoned was told *"the mortar_and_pestle is still at
    // work"*, as though the player had typed the line. `move`, `empty` and
    // `purge` were the same shape of hole.
    //
    // The question is **"would this be refused"**. Asserting on the refusal
    // *prose* rather than on the verb list is what makes this survive a new
    // verb: any future command that refuses on a busy instrument fails here
    // until the predicate learns about it.
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
    // **The player's way back out, and it did not exist.** `stop` took a
    // `Place`, a spell is a `Script`, so `stop loop` never resolved to the thing
    // it named — and nothing but running out of program removes `Running`, which
    // an unbounded `repeat` never does. A player who wrote one had a spell
    // working the laboratory for ever with no way to reach it.
    //
    // The budget bounds a *tick*, which is what stops the game hanging. It does
    // nothing at all about the session.
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
    // **The loop a person actually works in**: cast it, see it do the wrong
    // thing, call it off, change it, cast it again. Every step of that has to
    // hold or the editor is a thing you write into once.
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
    // §8: *"reloads queue to the next tick boundary, so a file cannot change
    // under a script mid-execution."* The program is derived when the spell is
    // **cast**, so saving over it is safe — and the running copy keeps doing
    // what it was told. `scribe` says so, because a player who edits a running
    // spell and sees nothing change has been silently ignored.
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
    // Two different things to stop, and `stop` reaching both must not conflate
    // them. Calling a spell off is walking away from it; the brew it began runs
    // on, exactly as it would if you had typed the line by hand.
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
    // **`attend` writes `Cwd`.** A spell containing one would otherwise teleport
    // the player, change their prompt, and change what they can name — while
    // they were standing somewhere else doing something else.
    //
    // **Bound, not invoked.** It used to invoke from the archive and assert the
    // player stayed there — which now passes for a reason that is not this one:
    // an invocation ends the moment its caster leaves (§19), so the spell would
    // stop before it reached the `attend` and the test would prove nothing. A
    // *held* spell is the one that runs while the player is elsewhere, so it is
    // the one that could move them.
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
    // genuinely lands on a spell in flight. A shorter one finishes inside the
    // first `step` and starting it again is then the *right* answer — which is
    // what this test asserted by accident before.
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
    // §8 permits it to a **call-depth limit of 3**, and argues why the execution
    // budget alone is not a sufficient guard: exhausting it makes every
    // instruction *Budget starved*, which logs at high verbosity only, so all
    // automation would stop **silently**. Depth-limiting is loud.
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
    // **The mechanic, not an implementation detail.** The budget was 4, which
    // made a long spell and a tight one cost the same and left nothing for an
    // efficient script to *be better at*. At one line per tick a wasted line is
    // a wasted second of the tower's time, which is the whole reason to care
    // how a spell is written.
    //
    // Asserted through `running_line`, which is what the editor's marker draws
    // from: a budget that quietly went back to 4 would take the marker with it.
    let mut sim = with_spell("slow", &["survey", "survey", "survey", "survey"]);
    sim.submit("invoke slow");
    sim.step();

    // `running_line` is where the orb goes **next**, and the tick that casts a
    // spell already runs its first line — so a four-line spell stands on line 2
    // here, and one further tick per line after that. At the old budget of 4 the
    // whole thing would be over before this loop starts.
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
    // The loop this whole surface exists for: you watch a spell go wrong, fix
    // the line, and the next pass takes it — without stopping and recasting.
    //
    // The spell loops forever on purpose. A `repeat` with no count is the only
    // shape that is still running by the time the edit lands, which is what
    // makes the assertion about a *live* reload rather than about the next cast.
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
    // **The bug this exists for looked like an inverted condition.** `holds`
    // compares a place name exactly, and a control word's tail was the one thing
    // the orb did not canonicalise — so `if mortar is empty` never found
    // `mortar_and_pestle`, answered no on every pass, and took the `else` for
    // ever. Reported from a screenshot; no test in the suite could see it,
    // because `if` was only ever tested as a *parse*.
    //
    // Asserted on the branch that runs, not on the condition: a test that
    // checked `holds` directly would have agreed with the bug.
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
    // **program's**, made at cast; it used to be written into the spell, which
    // taught the name at the cost of the file being the player's.
    assert_eq!(
        sim.spell("probe").as_deref().and_then(<[String]>::first),
        Some(&"if mortar is empty".to_owned()),
        "the orb rewrote the line it read",
    );
}

#[test]
fn a_question_about_nowhere_says_so_rather_than_answering_no() {
    // §8's *Referent missing*, and **not the same as the answer being no** —
    // which is the distinction the bug above turned on. A condition naming a
    // place the tower does not have used to be indistinguishable from one that
    // was simply false, so a spell could take the `else` for ever in silence.
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
    // **A placeholder no emit site fills draws as itself**, which `prose.toml`
    // documents as deliberate — *"visible on screen, so a typo is caught by
    // looking"*. It works: `spell_gave_up` asked for `{source}` where
    // `say_failure` supplies `{detail}`, and a player watching a spell give up
    // read `waited too long on the {source}`.
    //
    // Looking caught it. This is so looking does not have to. Asserted over the
    // **whole stream** rather than against a list of keys, because a list is the
    // thing that goes stale — a new failure line with a new placeholder is
    // covered here the day it is written.
    //
    // The spell drives every failure the runner has: a wait that never lands, a
    // question about nowhere, a name that is not there, and a forbidden verb.
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
    // **Reported as "`grind sage` is truncated to `grind` when I close and
    // reopen the editor."** `quit` saves, so every visit re-canonicalised the
    // file against whatever happened to be on the shelf — and §10.1's verbs take
    // their reagent *optionally*, so with the sage spent `grind sage` resolved
    // to bare `grind` and the orb wrote that down. Two different commands,
    // swapped in silence, in a file the player had already finished writing.
    let mut sim = Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();
    // Spend the sage: the **name** is still real, and nothing on the shelf
    // answers it. That gap is the whole bug.
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
    // **Both halves, because the fix could have broken either.** The game's
    // flagship plain-English phrasings — `make a potion of clarity`,
    // `look around` — must still *run*, and they must no longer be **rewritten**
    // into the file to do it. This asserted the rewrite until the file became
    // the player's; what it asserts now is that giving that up cost the loose
    // phrasing nothing.
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
    // **Reported from a screenshot of sixteen identical lines.** The buffer
    // writes itself out after every pause in the typing, so a sentence per save
    // is a sentence every second or two — one editing session filled the
    // transcript behind the modal with copies of `5 lines, written down`.
    //
    // Nothing was lost with it: a line the orb could not read is named
    // individually, with its number, when the spell is cast — which the tests
    // below this one hold to.
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
    // The one thing a save still says, and the reason it survived: editing a
    // spell while it runs lands *now*, which is safe, invisible, and otherwise
    // indistinguishable from being ignored.
    //
    // Once per session, though. The autosave fires on every pause, so a notice
    // per write is the same noise this item removed, wearing a different
    // sentence.
    // Unbounded, so it is still running when the second session opens — a spell
    // that had finished by then would report no reload for the honest reason,
    // and the test would pass on the wrong evidence.
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
    // **Reported from a screenshot**: `grind sage` flush with `repeat` while
    // everything around it was indented, because the four branches that kept a
    // line's words all wrote `line.trim()`. The orb re-indented what it rewrote
    // and dropped the indent from what it did not, so the file disagreed with
    // itself about where a line sat.
    //
    // Nothing re-indents now, which settles it in the other direction: the
    // buffer indents as you type (`editor::reindent`), and what the buffer holds
    // is what the file holds. This spell arrives through `write_spell` with no
    // editor involved and with *deliberately unhelpful* layout, so the only way
    // it can come back tidy is if something tidied it.
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
    // **§11.5's floor**: there is always something to do. The tower held one
    // sage, so this exact spell ground once and then reported an empty mortar
    // for ever — correct behaviour with nothing behind it.
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
    // Two properties that broke together, and one covers for the other if you
    // only check one.
    //
    // **The sage.** `move` took a unit, but charging an instrument was a second
    // copy of the same three lines and re-parented the endless pile itself into
    // the mortar — where the run spent it and `empty` swept it into the store.
    // Testing `move` alone said everything was fine.
    //
    // **The pile.** `give` merges, so grinding twice adds to `ground-sage`
    // rather than standing a second node beside it under the same name. That was
    // wrong before counts existed too; counts merely made it visible.
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

    // **Reagents only.** Every recipe output is also a manual `Topic`, nameable
    // from anywhere — so a bare name count sees `ground-sage` twice however many
    // are on the shelf, and this test failed for a reason that had nothing to do
    // with what it is about.
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
    // **What keeps the transcript readable.** A spell emits exactly what the
    // same commands typed by hand emit, so a `repeat` loop pushed several
    // records every few ticks and the player's own last line scrolled off in
    // seconds. The pane draws what is *unattributed*; the log keeps everything,
    // because a log is already a view over this one stream (§3, rule 4).
    //
    // Asserted over the whole stream rather than on one record: the sites a
    // spell reaches are the ordinary ones, so this has to hold for records
    // nobody thought about — including the instrument completions that land
    // ticks later, in a different system, which is the case that was missed the
    // first time.
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
        // **The player's own line stays**: the `invoke` they typed, its echo,
        // and the orb answering that it has taken the spell up. Those are the
        // command, not its output — and a filter that hid them would have made
        // casting a spell look like nothing happened.
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
    // Rule 3. The runner orders spells by `NodeId` rather than by query order
    // precisely so this holds — `tower::node` records archetype order as a bug
    // that changes what a phrase resolves to with no test catching it.
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
    // §19 chose an interval over a countdown so hundreds of ticks inside one
    // `step()` behave identically to being watched. A spell is the first thing
    // that *acts* across that gap, so the property needs asserting again here.
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
