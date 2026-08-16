//! The in-world manual (§6.1): how a thing is made, in the order you make it.
//!
//! Not part of §10.1's loop — it is what you read **before** committing an
//! instrument to a route, which is the half of the exit criterion that says the
//! same goal having two right answers is only a decision if the player can see
//! both.
//!
//! It is also where the retired brewing words land (§19). `make a potion of
//! clarity` resolves here, and for a while it resolved here and *acknowledged* —
//! a dead end, which §15 weighs above the raw resolution rate. This is what makes
//! the newcomer's most natural sentence the tutorial entry point rather than a
//! shrug.
//!
//! # It reads as instructions, not as a table
//!
//! The first version emitted the raw breadth-first walk as five bare columns per
//! row, tiled across the pane. Four things were wrong with it and they compounded:
//!
//! - **Backwards.** The walk starts at the goal, so the first line was the *last*
//!   thing to do. A player reading top-down got the recipe in reverse.
//! - **Unlabelled.** `clarity alembic clarified-draught phlegm 14` gives the
//!   reader five values and no way to tell an input from an output.
//! - **Clipped.** A two-input step does not fit ~60 cells, and tiling cut it.
//! - **Full of noise.** It expanded *every* route to everything, including three
//!   ways to make the `rock-salt` sitting in the dispensary.
//!
//! So: the primary route in **dependency order**, numbered, one authored sentence
//! per step (rule 6), stopping at what the player already has. Alternatives
//! follow, marked, with any step only they need. The fields stay on the record
//! (rule 4) so `sift` and a pipe still work on them.

use std::collections::BTreeSet;

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Prose, Recipe, Recipes};
use crate::parser::{Group, Intent, Verb, synonyms_of};
use crate::session::Scrollback;
use crate::tower::{self, Name, Store};

use super::missing;
use super::navigate::root;

/// How deep a route may go before the walk gives up.
///
/// The content's longest chain is four, and the `seen` set already stops a cycle
/// — this is the backstop for a content file that grows one, so a malformed
/// recipe table cannot recurse until the stack goes.
const MAX_DEPTH: usize = 16;

/// One command, at length.
///
/// # Sections, and what each is made of
///
/// The synopsis and description are authored; the ways of saying it are read off
/// `SYNONYMS`. That split is deliberate: `signature()` carries no connectives —
/// `move` is `[Reagent, Place?, Place]` with no `to` in it — so a generated
/// synopsis would read `move reagent place place`, which nobody types. A test
/// keeps the authored line honest instead, by asserting it names every required
/// slot.
///
/// `RecordKind::Message` throughout, not `Entry`: a page is instructions, and
/// `Message` wraps where `Entry` tiles. `recall.rs`'s route walk already makes
/// that argument. Long pages page for free, because `unfurl` searches by record.
fn page(world: &mut World, verb: Verb) {
    let canonical = verb.canonical();

    // The synopsis, and the one line saying what it is for. Both authored; a
    // missing key draws as the key itself, which is how `Prose::line` makes an
    // omission visible rather than blank.
    for key in [
        format!("man_{canonical}_use"),
        format!("man_{canonical}_gloss"),
    ] {
        say(world, canonical, &key);
    }

    // **Joined into one record, not one per authored line.** Prose is capped at
    // 70 cells by the width lint, so a description has to be written in pieces —
    // but emitting those pieces as separate records makes them *hard* line
    // breaks, and a page in a 100-column pane came out as ragged 50-cell strips
    // with a wall of text where its paragraphs should be. One record wraps to
    // whatever the pane actually is.
    section(world, "man_page_what");
    let body: Vec<String> = (1..=MAX_LINES)
        .map(|line| format!("man_{canonical}_{line}"))
        .take_while(|key| world.resource::<Prose>().has(key))
        .map(|key| world.resource::<Prose>().line(&key, &[]))
        .collect();
    if !body.is_empty() {
        line(world, canonical, &body.join(" "));
    }

    let examples: Vec<String> = (1..=MAX_LINES)
        .map(|n| format!("man_{canonical}_eg{n}"))
        .take_while(|key| world.resource::<Prose>().has(key))
        .collect();
    if !examples.is_empty() {
        section(world, "man_page_like");
        for key in examples {
            say(world, canonical, &key);
        }
    }

    // **One row per register, not one per phrase.** `attend` has five spellings
    // and four of them are plain, so a row each made the section longer than the
    // description it followed.
    //
    // Read off the table, so it cannot go stale. Arcane first, because that is
    // what the echo teaches and what an expert types.
    section(world, "man_page_said");
    let mut by_register: Vec<(crate::parser::Register, Vec<String>)> = Vec::new();
    for (register, phrase) in synonyms_of(verb) {
        match by_register.last_mut() {
            Some((last, phrases)) if *last == register => phrases.push(phrase),
            _ => by_register.push((register, vec![phrase])),
        }
    }
    for (register, phrases) in by_register {
        let phrase = phrases.join(", ");
        // **The arrangement is authored**, like every route step: a `Message`
        // draws its message and nothing else, so composing `"arcane  grind"` in
        // Rust would be a frontend decision made in the sim (rule 4) *and* a
        // sentence in source (rule 6). The fields stay fields, so `sift` still
        // works on them.
        let said = world
            .resource::<Prose>()
            .line("man_said", &[("kind", register.label()), ("name", &phrase)]);
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(FieldName::Name, canonical)
            .text(FieldName::Kind, register.label())
            .text(FieldName::Message, &said)
            .finish();
    }

    let also = format!("man_{canonical}_also");
    if world.resource::<Prose>().has(&also) {
        section(world, "man_page_also");
        say(world, canonical, &also);
    }
}

/// How many description or example lines a page may carry.
///
/// A page is read in a transcript pane, not a browser. Eight is already more
/// than fits at the 80x22 floor without scrolling, so the cap is a backstop
/// against an authored run with no end rather than a budget anyone reaches.
const MAX_LINES: usize = 8;

/// A page heading.
fn section(world: &mut World, key: &str) {
    let heading = world.resource::<Prose>().line(key, &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Section)
        .text(FieldName::Kind, &heading)
        .finish();
}

/// One line of a page, already composed.
fn line(world: &mut World, canonical: &str, message: &str) {
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, canonical)
        .text(FieldName::Message, message)
        .finish();
}

/// One authored line of a page.
fn say(world: &mut World, canonical: &str, key: &str) {
    let message = world.resource::<Prose>().line(key, &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, canonical)
        .text(FieldName::Message, &message)
        .finish();
}

/// The subject that answers *what can I write in a spell, here*.
///
/// A `recall_` key of the same name is what makes it nameable, exactly as a
/// material's page does.
const SCRIPTING: &str = "scripting";

/// What a spell is made of, and what this room lets one ask about.
///
/// # Three sections, and only the last one moves
///
/// The **words** and the **question shapes** are the same in every room, because
/// the grammar is. What changes is what a question can *name*: the laboratory's
/// instruments answer `is idle`, the archive's four ways answer `has passage`,
/// and a player standing in one has no way to discover the other's vocabulary by
/// guessing at it.
///
/// So the third section is built from the room, the same way [`overview`] builds
/// its verb list from [`offered`](super::offered) — one rule, applied twice,
/// rather than a second idea of what *here* means.
fn scripting(world: &mut World) {
    let cwd = world.resource::<tower::Cwd>().0;
    let places: Vec<String> = tower::children_of(world, cwd)
        .into_iter()
        .filter(|node| world.get::<tower::Fixture>(*node).is_some())
        .filter_map(|node| world.get::<tower::Name>(node).map(|name| name.0.clone()))
        .collect();
    // The readings belong to a way, so they are listed where there are ways —
    // they resolve everywhere (a solver's `if` names them at cast, when no maze
    // is open), but naming them in the laboratory would be teaching a word the
    // room can never answer.
    let readings = tower::children_of(world, cwd)
        .into_iter()
        .any(|node| world.get::<tower::Reading>(node).is_some());

    say(world, SCRIPTING, "recall_scripting");

    section(world, "man_scripting_words");
    for word in crate::parser::SpellWord::ALL {
        entry(world, word.canonical(), spell_word_shape(word));
    }

    section(world, "man_scripting_asking");
    for key in SHAPES {
        let line = world.resource::<Prose>().line(key, &[]);
        entry(world, &line, "");
    }

    section(world, "man_scripting_here");
    for place in places {
        entry(world, &place, "");
    }
    if readings {
        for word in tower::maze::readings() {
            entry(world, word, "");
        }
    }
}

/// The shapes a question takes, in the order they are worth learning.
const SHAPES: [&str; 4] = [
    "man_scripting_shape_is",
    "man_scripting_shape_has",
    "man_scripting_shape_count",
    "man_scripting_shape_join",
];

/// What a control word takes after it, for the listing.
const fn spell_word_shape(word: crate::parser::SpellWord) -> &'static str {
    match word {
        crate::parser::SpellWord::Repeat => "<count>",
        crate::parser::SpellWord::Until | crate::parser::SpellWord::If => "<question>",
        crate::parser::SpellWord::Wait => "<thing>",
        crate::parser::SpellWord::Else | crate::parser::SpellWord::End => "",
    }
}

/// One row under a section: a name, and what follows it.
fn entry(world: &mut World, name: &str, shape: &str) {
    let mut records = world.resource_mut::<Scrollback>();
    let records = records.records_mut();
    let row = records.push(RecordKind::Entry).text(FieldName::Name, name);
    // An empty field is not an absent one — it draws as trailing blanks and
    // speaks as a labelled silence, which is why `overview` guards the same way.
    if shape.is_empty() {
        row.finish();
    } else {
        row.text(FieldName::Kind, shape).finish();
    }
}

/// Everything you can type where you are standing, grouped.
///
/// # Why this is `Section` + `Entry` and not a formatted string
///
/// `RecordKind::Section` stacks and draws as a `[heading]`; `RecordKind::Entry`
/// **tiles**, packing across whatever width the pane has. `survey` already emits
/// exactly this pair, so a clap-shaped listing needed no render code at all —
/// and because the entries are records rather than a string, `sift` still works
/// on them and §14 hears one utterance per verb rather than a wall of spacing.
///
/// # It lists what works *here*
///
/// [`offered`](super::offered) is the boot report's own filter: live, ungated,
/// and in scope. `grind` appears in the laboratory and not in the archive, which
/// is §7 and is the rule Tab already follows — offering a word the parser would
/// refuse is the dead end §15 weighs above the raw resolution rate.
///
/// So the listing is *narrower* than the manual: `recall grind` will answer from
/// anywhere, because a manual you can only read in the right room has a lock on
/// it. That asymmetry is deliberate and recorded in §19.
fn overview(world: &mut World) {
    let offered = super::offered(world);

    for group in Group::ALL {
        let members: Vec<Verb> = offered
            .iter()
            .copied()
            .filter(|verb| verb.group() == group)
            .collect();
        // A heading over nothing is furniture. The archive has no spells in it
        // and no destructive verb the room has earned, and printing empty
        // sections would make the overview mostly headings.
        if members.is_empty() {
            continue;
        }

        section(world, group.key());
        // Through the shared [`entry`], which carries the empty-field rule —
        // `status` and `undo` take no argument, and an empty `Kind` draws as
        // trailing blanks and speaks as a labelled silence. It was written out
        // here *and* in `entry`, which is two homes for one rule.
        for verb in members {
            entry(world, verb.canonical(), verb.signature_label());
        }
    }

    // The pointer out. Without it the overview reads as *this is the vocabulary*
    // rather than *this is the vocabulary here*, and a player standing in the
    // archive would never learn the laboratory has words of its own.
    let message = world.resource::<Prose>().line("man_elsewhere", &[]);
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, Verb::Recall.canonical())
        .text(FieldName::Message, &message)
        .finish();
}

/// Show how a thing is made, and every way there is to make it.
pub(super) fn recall(intent: &Intent, world: &mut World) {
    let Some(topic) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        // **The overview, and this branch was unreachable until the slot became
        // optional.** A required slot with fillers never yields an argument-less
        // intent — every filler ties and `analyse` returns `Ambiguous` — so bare
        // `recall` opened a numbered prompt offering the four
        // alphabetically-first subjects. `help`, `man` and `?` all land here, so
        // that was §6's no-bare-error rule failing at the one command whose job
        // is answering the question. See `TOPIC_OPTIONAL`.
        overview(world);
        return;
    };

    // **The manual's own pages, before the recipe walk.** A verb canonical is a
    // `NounKind::Command` and nothing else, so this cannot shadow a recipe — but
    // it is checked first anyway, because the day a recipe is named after a verb
    // the page is what the player meant.
    if let Some(verb) = Verb::ALL.into_iter().find(|verb| verb.canonical() == topic) {
        page(world, verb);
        return;
    }

    // **The one page about the language rather than about the tower**, and it is
    // scoped like the overview above: the grammar is the same everywhere, what a
    // question can *name* is not. Before this, nothing in the game taught the
    // spell vocabulary at all — control words are outside `Verb::ALL`, so
    // `recall repeat` reached nothing and a player had no way to find out what an
    // `if` could ask.
    if topic == SCRIPTING {
        scripting(world);
        return;
    }

    // **What it is, before how it is made.** A route answers *how do I get one*;
    // a player holding a potion or a scroll is asking *what is this for*, and the
    // two are different questions. This used to be an `else`: anything with a
    // recipe got the walk and nothing else, so every finished product in the game
    // could tell you its five steps and not one word about what it did.
    let said = describe(world, &topic);

    let plan = plan(world, &topic);
    if plan.is_empty() {
        // Not a recipe either. §6.1 makes `recall` the **in-world manual**, so a
        // subject like `brewing` is answered with authored prose (rule 6) rather
        // than treated as a thing that does not exist.
        if !said {
            missing(Verb::Recall, &topic, world);
        }
        return;
    }

    say_plan(world, &topic, &plan);
}

/// Say what `topic` is, and how it is used. Whether anything was said.
///
/// Two keys, and the second is deliberately **not** `recall_<topic>_use`:
/// [`Prose::topics`](crate::content::Prose::topics) decides what is *nameable* by
/// stripping `recall_`, so that spelling would register `clarity_use` as a
/// subject to ask the orb about. It is the trap the route templates already
/// record paying for, where `grimoire_step_or` made `step_or` a topic.
///
/// `using_` is optional and often absent — a byproduct is a thing you have
/// rather than a thing you do — and where it *is* present it may honestly say
/// the use is not built. `undo`'s page does the same, and §15 wants that: a page
/// admitting a word does nothing is the cheapest way to keep a player out of a
/// dead end.
fn describe(world: &mut World, topic: &str) -> bool {
    let prose = world.resource::<Prose>();
    let what = format!("recall_{topic}");
    let how = format!("using_{topic}");
    let lines: Vec<String> = [what, how]
        .into_iter()
        .filter(|key| prose.has(key))
        .map(|key| prose.line(&key, &[]))
        .collect();
    if lines.is_empty() {
        return false;
    }
    for message in lines {
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Message)
            .text(FieldName::Name, topic)
            .text(FieldName::Message, &message)
            .finish();
    }
    true
}

/// Write the plan out, a record per step.
fn say_plan(world: &mut World, goal: &str, plan: &Plan) {
    let steps = plan.steps.len();
    let ticks: u64 = plan.steps.iter().map(|step| step.ticks).sum();

    let heading = world.resource::<Prose>().line(
        "route_line",
        &[
            ("name", goal),
            ("count", &steps.to_string()),
            ("ticks", &ticks.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, goal)
        .count(FieldName::Quantity, to_count(steps))
        .count(FieldName::Remaining, ticks)
        .text(FieldName::Message, &heading)
        .role(Role::Success)
        .finish();

    for (index, step) in plan.steps.iter().enumerate() {
        say_step(world, step, Some(index + 1));
    }
    for step in &plan.alternates {
        say_step(world, step, None);
    }
}

/// One step, as a record carrying both its facts and its sentence.
///
/// `RecordKind::Message`, not `Entry`. `Entry` speaks as a `TableRow` — which is
/// right for `survey`, where §14 needs the column labels — and it **tiles**,
/// packing rows across the pane. Both are wrong for an instruction: the sentence
/// *is* the content, it has to wrap rather than be cut, and a reader wants the
/// line rather than five `label: value` pairs. A `Message` carrying fields keeps
/// `sift` working on them (rule 4) while drawing and speaking as prose.
fn say_step(world: &mut World, step: &Step, index: Option<usize>) {
    let inputs = step.inputs.join(" + ");
    let prose = world.resource::<Prose>();
    let heat = if step.heat {
        prose.line("route_heat", &[])
    } else {
        String::new()
    };
    // Composed in like the heat clause beside it, and for the same reason: not
    // every step has one. The `+` lives in `route_leaves` rather than in the
    // step line, so a step that leaves nothing does not print a plus with
    // nothing after it.
    let leaves = if step.leaves.is_empty() {
        String::new()
    } else {
        prose.line("route_leaves", &[("state", &step.leaves)])
    };
    let message = prose.line(
        if index.is_some() {
            "route_step"
        } else {
            "route_step_or"
        },
        &[
            (
                "index",
                &index.map_or_else(String::new, |n| format!("{n}.")),
            ),
            ("detail", &inputs),
            ("name", &step.output),
            ("state", &leaves),
            ("source", &step.instrument),
            ("ticks", &step.ticks.to_string()),
            ("kind", &heat),
        ],
    );

    // The facts ride along for `sift` and a pipe, under the same names the
    // pipeline's own records use: the instrument is a `Path`, what comes out is
    // the `Name`, what went in is the `Origin`.
    //
    // **Never `Detail`.** A record carrying prose draws its `Message` *and* its
    // `Detail`, by design — `Detail` is secondary prose subordinate to the
    // message — so putting the inputs there printed them in front of the sentence
    // written to explain them. That is the trap `wield`'s refusal fell into, and
    // `Origin` (where a thing came *from*) is what an input actually is.
    let mut scrollback = world.resource_mut::<Scrollback>();
    let mut record = scrollback
        .records_mut()
        .push(RecordKind::Message)
        .text(FieldName::Name, &step.output)
        .text(FieldName::Origin, &inputs)
        .text(FieldName::Path, &step.instrument)
        .text(FieldName::State, &step.leaves)
        .count(FieldName::Remaining, step.ticks)
        .text(FieldName::Message, &message);
    if let Some(index) = index {
        record = record.count(FieldName::Quantity, to_count(index));
    }
    record.finish();
}

/// One thing to do, in the order it is done.
struct Step {
    /// What this step makes.
    output: String,
    /// Where it happens.
    instrument: String,
    /// What goes in.
    inputs: Vec<String>,
    /// The byproduct it also leaves — §10.1 gives every one of them a use, and
    /// showing it here is how a player finds the alternative route below.
    leaves: String,
    /// How long it takes.
    ticks: u64,
    /// Whether the athanor must be lit, from the recipe's own `heat` key.
    heat: bool,
}

impl Step {
    fn of(output: &str, instrument: &str, recipe: &Recipe) -> Self {
        Self {
            output: output.to_owned(),
            instrument: instrument.to_owned(),
            inputs: recipe.inputs().into_iter().map(ToOwned::to_owned).collect(),
            leaves: recipe.leaves.clone().unwrap_or_default(),
            ticks: recipe.ticks,
            heat: recipe.heat,
        }
    }
}

/// A route to something, and the other ways there are to reach it.
#[derive(Default)]
struct Plan {
    /// The primary route, in the order the steps are performed.
    steps: Vec<Step>,
    /// Every other route to a step in it, plus anything only they need.
    alternates: Vec<Step>,
}

impl Plan {
    const fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Work out how to make `goal` from what the tower already stocks.
fn plan(world: &World, goal: &str) -> Plan {
    let stocked = stocked(world);
    let recipes = world.resource::<Recipes>();
    let mut plan = Plan::default();
    let mut seen = BTreeSet::new();

    // **The goal itself is never treated as stock.** `recall rock-salt` should
    // answer with how to make it even though the dispensary holds some, which is
    // what the `depth > 0` guard inside `build` is for.
    build(recipes, &stocked, goal, 0, &mut plan, &mut seen, true);
    plan
}

/// Push the steps that make `output`, deepest first.
///
/// **Post-order**, which is what puts the walk in *doing* order: a step's inputs
/// are emitted before the step that consumes them, so reading top-down is the
/// order a player types. The breadth-first walk this replaced emitted the goal's
/// own step first, which is the last thing you do.
///
/// `primary` says whether these steps belong to the route being recommended or
/// to an alternative — an alternative's own sub-steps are only needed if you take
/// it, so they go with it rather than into the numbered list.
fn build(
    recipes: &Recipes,
    stocked: &BTreeSet<String>,
    output: &str,
    depth: usize,
    plan: &mut Plan,
    seen: &mut BTreeSet<String>,
    primary: bool,
) {
    if depth >= MAX_DEPTH || !seen.insert(output.to_owned()) {
        return;
    }
    // Stop at what the player already has. Without this the walk expanded three
    // ways to make the `rock-salt` sitting in the dispensary, which is true and
    // useless — the manual is for what you cannot simply pick up.
    if depth > 0 && stocked.contains(output) {
        return;
    }

    let routes = recipes.routes(output);
    let Some((instrument, recipe)) = routes.first() else {
        return;
    };

    for input in recipe.inputs() {
        build(recipes, stocked, input, depth + 1, plan, seen, primary);
    }
    let step = Step::of(output, instrument, recipe);
    if primary {
        plan.steps.push(step);
    } else {
        plan.alternates.push(step);
    }

    // Every other way to reach the same thing, with whatever only it needs.
    // **File order decides which is primary**: `recipes.toml` is authored, and
    // its first entry for an output is the designer's recommendation.
    for (instrument, recipe) in routes.iter().skip(1) {
        for input in recipe.inputs() {
            build(recipes, stocked, input, depth + 1, plan, seen, false);
        }
        plan.alternates.push(Step::of(output, instrument, recipe));
    }
}

/// Everything a store in the tower holds.
///
/// Walked from the root rather than taken from a global query: `tower::node`
/// requires anything a player can see be derived by walking `Children`, because
/// archetype order is not insertion order. The result here is an order-free set,
/// but the walk keeps the rule unbroken rather than arguing the exception.
fn stocked(world: &World) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut stack = vec![root(world)];
    while let Some(node) = stack.pop() {
        if world.get::<Store>(node).is_some() {
            for held in tower::children_of(world, node) {
                if let Some(name) = world.get::<Name>(held) {
                    names.insert(name.0.clone());
                }
            }
        }
        stack.extend(tower::children_of(world, node));
    }
    names
}

/// A count as a record value, saturating rather than wrapping.
fn to_count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    /// Every record the orb has drawn, as its rendered line.
    fn drawn(sim: &Sim) -> Vec<(RecordKind, String)> {
        sim.scrollback()
            .records()
            .iter()
            .map(|record| (record.kind(), record.to_line()))
            .collect()
    }

    fn run(sim: &mut Sim, line: &str) {
        sim.submit(line);
        sim.step();
    }

    #[test]
    fn a_bare_recall_lists_the_vocabulary_rather_than_asking_which_topic() {
        // **The defect the manual starts from, and nothing caught it.** A
        // required slot with fillers never yields an argument-less intent, so
        // bare `recall` returned `Ambiguous` and offered the four
        // alphabetically-first subjects — and `help`, `man` and `?` all land
        // here. §6 forbids a bare error; asking a lost player to choose between
        // `archive`, `brewing`, `clarified-draught` and `clarity` is that rule
        // failing at the one command whose job is answering the question.
        let mut sim = Sim::new(1);
        run(&mut sim, "attend laboratory");
        let before = sim.scrollback().records().len();
        run(&mut sim, "help");

        assert!(
            sim.choices().is_empty(),
            "`help` asked which topic instead of answering",
        );
        let after: Vec<_> = drawn(&sim).split_off(before);
        assert!(
            after.iter().any(|(kind, _)| *kind == RecordKind::Section),
            "no headings: {after:?}",
        );
        assert!(
            after.iter().any(|(_, line)| line.starts_with("grind")),
            "the laboratory's own work was not listed: {after:?}",
        );
    }

    #[test]
    fn the_overview_lists_what_resolves_here_and_nothing_else() {
        // Both directions, in two rooms. The listing is `execute::offered`, which
        // is the boot report's filter — so this is also what stops the tutorial a
        // player reads at launch disagreeing with the manual a minute later.
        for (place, wanted, unwanted) in [
            ("laboratory", "grind", "kindle-nothing"),
            ("archive", "research", "grind"),
        ] {
            let mut sim = Sim::new(1);
            run(&mut sim, &format!("attend {place}"));
            let before = sim.scrollback().records().len();
            run(&mut sim, "recall");
            let after: Vec<_> = drawn(&sim).split_off(before);

            let listed: Vec<&str> = after
                .iter()
                .filter(|(kind, _)| *kind == RecordKind::Entry)
                .filter_map(|(_, line)| line.split_whitespace().next())
                .collect();
            assert!(listed.contains(&wanted), "{place}: no {wanted}: {listed:?}");
            assert!(
                !listed.contains(&unwanted),
                "{place}: offered {unwanted}, which would not resolve here",
            );
            // Nothing gated and nothing dark. `bind` refuses at concentration 0
            // and `undo` is not built; either would be a dead end in the first
            // thing a lost player reads.
            assert!(!listed.contains(&"bind"), "{place}: offered a gated verb");
            assert!(!listed.contains(&"undo"), "{place}: offered a dark verb");
        }
    }

    #[test]
    fn every_word_for_the_manual_reaches_the_same_answer() {
        // `help`, `man` and `?` are synonyms of `recall` (§6.1's registers), and
        // the overview is the answer a player gets from all four spellings.
        let mut baseline = Sim::new(1);
        run(&mut baseline, "recall");
        let expected = drawn(&baseline);

        for word in ["help", "man", "?"] {
            let mut sim = Sim::new(1);
            run(&mut sim, word);
            assert_eq!(
                drawn(&sim).len(),
                expected.len(),
                "`{word}` answered differently from `recall`",
            );
        }
    }

    #[test]
    fn every_group_the_table_names_has_a_heading_authored() {
        // A group with no prose draws its own key — `Prose::line` returns the key
        // for a miss, deliberately, so the failure is visible rather than blank.
        // This makes it a test failure instead.
        let prose = crate::content::Prose::builtin();
        for group in Group::ALL {
            assert!(
                prose.has(group.key()),
                "{} has no authored heading",
                group.key(),
            );
        }
        assert!(prose.has("man_elsewhere"));
    }

    #[test]
    fn the_manual_registers_no_subjects_of_its_own() {
        // `Prose::topics` strips `recall_` to decide what is *nameable*, which is
        // how `step_or` once became a subject nobody authored (§19). The manual's
        // own furniture is under `man_`, so none of it may appear.
        let prose = crate::content::Prose::builtin();
        let leaked: Vec<&str> = prose
            .topics()
            .into_iter()
            .filter(|topic| topic.starts_with("group_") || *topic == "elsewhere")
            .collect();
        assert!(leaked.is_empty(), "the manual leaked subjects: {leaked:?}");
    }

    #[test]
    fn a_page_a_verb_has_is_the_page_it_gets() {
        // Only three are written so far, so this is not the completeness lint —
        // it is the shape. A page names itself, says what it is for, and reads
        // its synonyms off the table rather than out of prose.
        let mut sim = Sim::new(1);
        run(&mut sim, "attend archive");
        let before = sim.scrollback().records().len();
        run(&mut sim, "recall grind");
        let lines: Vec<String> = drawn(&sim)
            .split_off(before)
            .into_iter()
            .map(|(_, l)| l)
            .collect();
        let page = lines.join("\n");

        // Readable from the archive, where `grind` itself does not resolve. A
        // manual you can only read in the right room has a lock on it.
        assert!(page.contains("grind <reagent>"), "no synopsis: {page}");
        assert!(page.contains("crush a reagent"), "no gloss: {page}");
        // Off `SYNONYMS`, so it cannot go stale when a word is added.
        assert!(page.contains("arcane: grind"), "no registers: {page}");
        assert!(page.contains("plain: crush"), "no plain form: {page}");
    }

    #[test]
    fn a_synopsis_names_every_slot_its_signature_requires() {
        // **The authored line, kept honest.** `signature()` carries no
        // connectives — `move` is `[Reagent, Place?, Place]` with no `to` — so a
        // generated synopsis would read `move reagent place place`, which nobody
        // types. It is written instead, and this is what stops it drifting from
        // the signature it describes.
        let prose = crate::content::Prose::builtin();
        for verb in Verb::ALL {
            let key = format!("man_{}_use", verb.canonical());
            let line = prose.line(&key, &[]);
            assert!(
                line.starts_with(verb.canonical()),
                "{key} does not open with the word it documents: {line:?}",
            );
            // **`follow` is exempt, and the exemption is the recorded debt.**
            // Its slot is a `Place` because the place half of a spell's
            // condition resolves against exactly that kind, which is why the
            // four ways are places you cannot stand in (§19). `follow <place>`
            // would be honest about the implementation and wrong for a player,
            // who is choosing a direction. The exemption goes when the ways stop
            // needing to be places.
            if verb == Verb::Follow {
                assert!(line.contains("way"), "{key} lost its direction: {line:?}");
                continue;
            }
            for slot in verb.signature().iter().filter(|slot| slot.required) {
                assert!(
                    line.contains(slot.kind.label()),
                    "{key} never names its required {}: {line:?}",
                    slot.kind.label(),
                );
            }
        }
    }

    #[test]
    fn every_word_a_spell_is_written_with_has_a_page() {
        // **The gap an audit against seven other languages found**, and it was
        // in the docs rather than the code: every one of them ships a reference,
        // and ours had none. Control words are outside `Verb::ALL`, so the verb
        // lint below could never have covered them — `recall repeat` reached
        // nothing at all, and a player had no way to discover what an `if` could
        // ask.
        //
        // The readings are the same shape of hole one noun space over: they are
        // `NounKind::Sense` rather than materials, so `every_material_has_a_page`
        // does not see them either, and `recall marks` answered with a *scoping*
        // message about other rooms — a dead end wearing a wrong reason.
        let prose = crate::content::Prose::builtin();
        for word in crate::parser::SpellWord::ALL {
            assert!(
                prose.has(&format!("recall_{}", word.canonical())),
                "`{}` is a word a spell is written with and the manual cannot \
                 say what it does",
                word.canonical(),
            );
        }
        for reading in crate::tower::maze::readings() {
            assert!(
                prose.has(&format!("recall_{reading}")),
                "`{reading}` is a word a solver's `if` names and the manual \
                 cannot say what it means",
            );
        }
        // And the page that ties them to a room.
        assert!(prose.has("recall_scripting"));
    }

    #[test]
    fn every_material_has_a_page() {
        // **The same completeness lint, one noun space over.** Verbs have had one
        // since the manual was written; materials had none, so a reagent could be
        // authored with a colour, a recipe and a route and never a word saying
        // what it *was*. §19 records that failing twice already — the four shard
        // names, and the `dust` a byproduct field invented — and both times what
        // found it was a person asking rather than a test.
        //
        // One key required, not three: a material is a thing, and most of them
        // want a sentence rather than a page. `using_` is where a *use* goes and
        // is optional, because a byproduct is something you have rather than
        // something you do.
        let prose = crate::content::Prose::builtin();
        for material in crate::content::Materials::builtin().names() {
            assert!(
                prose.has(&format!("recall_{material}")),
                "`{material}` is authored in materials.toml and the manual cannot \
                 say what it is",
            );
        }
    }

    #[test]
    fn a_finished_product_says_what_it_is_for() {
        // **The pages this change was for.** A potion and a scroll are the two
        // things a player *holds*, and a route told them five steps and nothing
        // about the thing in their hand. Every one of them now has a `using_`
        // line — which for a potion honestly says the drinking is not built, on
        // `undo`'s precedent.
        let prose = crate::content::Prose::builtin();
        let recipes = crate::content::Recipes::builtin();
        let mut checked = 0;
        for made in recipes.outputs() {
            if !matches!(
                recipes.kind_of(made),
                crate::parser::NounKind::Essence | crate::parser::NounKind::Scroll
            ) {
                continue;
            }
            checked += 1;
            assert!(
                prose.has(&format!("using_{made}")),
                "`{made}` is finished work and its page does not say how it is used",
            );
        }
        assert!(checked > 0, "no finished product was actually checked");
    }

    #[test]
    fn every_verb_has_a_page() {
        // **The completeness lint**, and what stops the manual rotting as verbs
        // are added: a new verb is already a compile error in `Verb::group`, and
        // this makes it a test failure in the prose too.
        //
        // Three keys are required and the rest are optional, because a page's
        // length should follow what there is to say. `_use` and `_gloss` are the
        // two lines the overview and the header both need; `_1` is the shortest
        // honest answer to *what does it do*.
        let prose = crate::content::Prose::builtin();
        for verb in Verb::ALL {
            let canonical = verb.canonical();
            for suffix in ["use", "gloss", "1"] {
                let key = format!("man_{canonical}_{suffix}");
                assert!(prose.has(&key), "{canonical} has no {key}");
            }
        }
    }

    #[test]
    fn a_page_never_claims_a_word_that_is_not_there() {
        // Every `see also` names real vocabulary. A manual pointing at a word
        // the parser does not have is worse than one that points nowhere,
        // because the player types it and lands in the dead end §15 weighs
        // heaviest.
        let prose = crate::content::Prose::builtin();
        let known: Vec<&str> = Verb::ALL.iter().map(|verb| verb.canonical()).collect();
        for verb in Verb::ALL {
            let key = format!("man_{}_also", verb.canonical());
            if !prose.has(&key) {
                continue;
            }
            for word in prose.line(&key, &[]).split(',') {
                // `recall <topic>` entries point at a subject, not a verb; the
                // first word is what has to exist.
                let Some(first) = word.split_whitespace().next() else {
                    continue;
                };
                assert!(
                    known.contains(&first),
                    "{key} points at {first:?}, which is not a word",
                );
            }
        }
    }

    #[test]
    fn the_careful_group_is_exactly_the_destructive_verbs() {
        // `is_destructive` had no reader at all until the manual wanted one. Two
        // lists that must not drift, so they are checked against each other
        // rather than one being derived and the other trusted.
        for verb in Verb::ALL {
            assert_eq!(
                verb.group() == Group::Careful,
                verb.is_destructive(),
                "{} is grouped and marked differently",
                verb.canonical(),
            );
        }
    }

    #[test]
    fn no_verb_is_left_out_of_the_overview() {
        // `Verb::group` has no wildcard, so a new verb is a compile error there —
        // but a verb put in a group nobody prints would compile and simply not be
        // there. This is what catches that.
        for verb in Verb::ALL {
            assert!(
                Group::ALL.contains(&verb.group()),
                "{} is in a group the overview never prints",
                verb.canonical(),
            );
        }
    }
}
