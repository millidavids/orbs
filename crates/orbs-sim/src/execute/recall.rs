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
use crate::parser::{Intent, Verb};
use crate::session::Scrollback;
use crate::tower::{self, Name, Store};

use super::navigate::root;
use super::{acknowledge, missing};

/// How deep a route may go before the walk gives up.
///
/// The content's longest chain is four, and the `seen` set already stops a cycle
/// — this is the backstop for a content file that grows one, so a malformed
/// recipe table cannot recurse until the stack goes.
const MAX_DEPTH: usize = 16;

/// Show how a thing is made, and every way there is to make it.
pub(super) fn recall(intent: &Intent, world: &mut World) {
    let Some(topic) = intent
        .arguments
        .first()
        .map(|argument| argument.value.clone())
    else {
        acknowledge(Verb::Recall, world);
        return;
    };

    let plan = plan(world, &topic);
    if plan.is_empty() {
        // Not a recipe. §6.1 makes `recall` the **in-world manual**, so a
        // subject like `brewing` is answered with authored prose (rule 6) rather
        // than treated as a thing that does not exist.
        //
        // `recall_` is also what [`Prose::topics`](crate::content::Prose::topics)
        // strips to decide what is *nameable*, which is why the route templates
        // below are `route_` and not `recall_`: a template is not a subject, and
        // sharing the prefix registered `step_or` as something to ask about.
        let key = format!("recall_{topic}");
        if world.resource::<Prose>().has(&key) {
            let message = world.resource::<Prose>().line(&key, &[]);
            world
                .resource_mut::<Scrollback>()
                .records_mut()
                .push(RecordKind::Message)
                .text(FieldName::Name, &topic)
                .text(FieldName::Message, &message)
                .finish();
            return;
        }
        missing(Verb::Recall, &topic, world);
        return;
    }

    say_plan(world, &topic, &plan);
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
            ("state", &step.leaves),
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
            leaves: recipe.leaves.clone(),
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
