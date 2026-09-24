//! What a resolved round says, and what a finished siege pays (§5.1).
//!
//! Three surfaces, and the split between them is rule 4 doing its job: every
//! roll goes to the log with its die and its face, `announce` puts one sentence
//! on the pane, and `settle` pays out — all from the same records, read three
//! ways.

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use super::shared::RAMPART;
use crate::content::Prose;
use crate::parser::Verb;
use crate::session::Scrollback;
use crate::tower::{
    self, siege,
    siege::{Outcome, Round, Siege},
};

/// Write every roll of a round to the log.
pub(super) fn log_rolls(world: &mut World, round: &Round) {
    for (landed, who) in round
        .struck
        .iter()
        .map(|l| (l, siege::ENEMY))
        .chain(round.answered.iter().map(|l| (l, siege::GARRISON)))
    {
        let faces = landed
            .faces
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(" and ");
        let message = world.resource::<Prose>().line(
            if landed.tells {
                "roll_tells"
            } else {
                "roll_misses"
            },
            &[
                ("name", who),
                ("kind", landed.die.word()),
                ("detail", &faces),
                ("quantity", &landed.total.to_string()),
                ("state", &landed.against.to_string()),
            ],
        );
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Hold.canonical())
            .text(FieldName::Source, RAMPART)
            .text(FieldName::Kind, landed.die.word())
            .count(FieldName::Quantity, u64::from(landed.face))
            .text(
                FieldName::State,
                if landed.tells { "tells" } else { "misses" },
            )
            .text(FieldName::Message, &message)
            // Quiet, like `follow`'s step: a round can be forty rolls and the
            // transcript would lose the player's own last line. The log is where
            // a postmortem is read.
            .quiet()
            // `Normal`, not `Danger`, whichever way the die fell: a roll that
            // told is the game working, and forty red lines a round would spend
            // the accent triad on ordinary play. `settle` is where a lost siege
            // goes red.
            .role(Role::Normal)
            .finish();
    }
}

/// Say what the round did, in one sentence.
pub(super) fn announce(world: &mut World, round: &Round) {
    let message = world.resource::<Prose>().line(
        "hold_round",
        &[
            ("state", round.intent.word()),
            ("quantity", &round.taken.to_string()),
            ("detail", &round.dealt.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, Verb::Hold.canonical())
        .text(FieldName::Source, RAMPART)
        .text(FieldName::Message, &message)
        .role(Role::Success)
        .finish();

    // What the pledged dice came to, said only when there were any. The range
    // was shown before the commitment and this is the other half of that rule —
    // a gamble you cannot see the result of is a die roll behind a curtain.
    for (area, strength) in siege::Area::ALL
        .into_iter()
        .map(|area| (area, round.strengths.of(area)))
        .filter(|(_, strength)| *strength > 0)
    {
        // Each area says what its strength *bought*, not merely what it was: a
        // succour of 12 into a line missing 3 put back 3, and the number that
        // matters is the second one.
        let bought = match area {
            siege::Area::Succour => round.mended.to_string(),
            siege::Area::Sortie => round.sortied.to_string(),
            _ => strength.to_string(),
        };
        // What it cost, and only the sortie has one. This handed `round.spent` —
        // the mettle the *sortie* took off the garrison — to all four keys, so
        // authoring `{state}` into `area_line` would quietly print the sortie's
        // number. Rule 6 makes that a content edit with nothing to catch it.
        let cost = match area {
            siege::Area::Sortie => round.spent.to_string(),
            _ => String::new(),
        };
        let message = world.resource::<Prose>().line(
            match area {
                siege::Area::Line => "area_line",
                siege::Area::Buckler => "area_buckler",
                siege::Area::Succour => "area_succour",
                siege::Area::Sortie => "area_sortie",
            },
            &[
                ("quantity", &strength.to_string()),
                ("detail", &bought),
                ("state", &cost),
            ],
        );
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, Verb::Pledge.canonical())
            .text(FieldName::Source, RAMPART)
            .text(FieldName::Kind, area.word())
            .count(FieldName::Quantity, u64::from(strength))
            .text(FieldName::Message, &message)
            .role(Role::Normal)
            .finish();
    }
}

/// Pay out a finished siege.
///
/// The barrier moves before the orb earns, which is `muster`'s order and its
/// reason — the work, then what the work bought. §19 records the sanctum
/// shipping it the other way round, so a finished course read `integrity = 0`
/// on the transcript while the rail already said otherwise.
pub(super) fn settle(world: &mut World, rampart: Entity, outcome: Outcome) {
    let completion = world.get::<Siege>(rampart).map_or(0, Siege::completion);

    // The road stays empty for a while — §11.5's cadence, and what stops a
    // siege being a faucet.
    let now = world.resource::<crate::tick::Tick>().get();
    if let Some(mut siege) = world.get_mut::<Siege>(rampart) {
        siege.clear_at = Some(now.saturating_add(siege::CADENCE));
    }

    let _ = match outcome {
        Outcome::Held => tower::mend_by(world, siege::VICTORY_MEND),
        Outcome::Fallen => tower::wear_by(world, siege::DEFEAT_WEAR),
    };

    // Escrow: progress-scaled, with a floor (§11.5). Losing at 60% keeps
    // something worth having, which stops a lost siege being an evening thrown
    // away.
    let arrived = world.get::<Siege>(rampart).map_or(0, |siege| siege.arrived);
    let earned = siege::escrow(
        arrived,
        completion,
        outcome,
        tower::grant::escrow_percent(world),
    );
    let key = match outcome {
        Outcome::Held => "siege_held",
        Outcome::Fallen => "siege_fallen",
    };
    let message = world.resource::<Prose>().line(
        key,
        &[
            ("quantity", &completion.to_string()),
            ("detail", &earned.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Status)
        .text(FieldName::Name, Verb::Defend.canonical())
        .text(FieldName::Source, RAMPART)
        .text(
            FieldName::State,
            match outcome {
                Outcome::Held => "held",
                Outcome::Fallen => "fallen",
            },
        )
        .count(FieldName::Quantity, u64::from(completion))
        .text(FieldName::Message, &message)
        .role(match outcome {
            Outcome::Held => Role::Success,
            Outcome::Fallen => Role::Danger,
        })
        .finish();

    // What the whole fight did to the tower's standing (§11.5, §19), said once
    // and after the siege's own sentence.
    //
    // Once rather than per round: the rounds move renown quietly, because a
    // round narrates itself and `renown::lose`'s sentence would be a second
    // telling thirteen times over. Silence *and* no summary would leave the
    // gauge as the only signal, and `gauges::split` drops both gauge rows on a
    // short pane — so this is the one record that answers *what did that fight
    // cost me*.
    //
    // Measured from what the tower was worth when the enemy arrived, not from a
    // line ago: `Siege::standing` is the opening snapshot, so this reports the
    // rounds *and* the outcome as one number. Read here it would say only what
    // the stake did and omit what the exchanges cost — the half a player wants
    // explained when a title has just gone.
    let before = world.get::<Siege>(rampart).and_then(|siege| siege.standing);
    let stake = siege::renown_stake(arrived, completion, outcome);
    match outcome {
        Outcome::Held => tower::renown::earn(world, stake),
        Outcome::Fallen => tower::renown::slip(world, stake),
    }
    // No snapshot, nothing said. A siege carried from a save written before the
    // snapshot has nothing to measure from, and nought is not a safe stand-in:
    // `now - 0` is the tower's whole total, so a lost fight would announce
    // having won every renown the player has, in the winning voice.
    if let Some(before) = before {
        standing(world, before);
    }

    // After the sentence about the siege, never before it — the order every
    // other seam keeps: the wall holds, *and then* the sanctum's line advances.
    // Counted twice on a win, because *survive a siege* and *win five* are
    // different stations.
    tower::done(world, &tower::Work::event(tower::SIEGE), earned);
    if outcome == Outcome::Held {
        tower::note(world, tower::SIEGE_WON);
    }

    // No rail mark, because `briefs()` walks `DOMAINS` and the bailey is
    // deliberately absent from it — a `Mark::News` here would be state nothing
    // can draw, cleared only by an `attend` nobody was prompted to make. The
    // transcript and `bailey.log` say what happened. A finished siege also
    // leaves the board up deliberately, and `defend` clears it.
}

/// Say what the whole fight did to the tower's standing, measured from `before`.
///
/// One record for the siege, not one per round: the rounds move renown quietly
/// (`renown::slip`), so this is the only thing that says the movement happened.
///
/// Measured rather than accumulated, because a total read at both ends cannot
/// fall out of step with what the rounds did — no running sum on the `Siege`,
/// nothing to save, and the saturating floor for free, so a tower that could
/// only fall to nought reports the fall it took rather than the one it was owed.
///
/// Silent when nothing moved, which is a real case: an unpledged fight that
/// trades evenly and is lost at a completion the stake rounds away comes to
/// nought (`a_move_of_nought_says_nothing`).
fn standing(world: &mut World, before: u64) {
    let now = world.resource::<tower::Renown>().get();
    let (key, moved, role) = match now.cmp(&before) {
        std::cmp::Ordering::Greater => ("siege_standing_won", now - before, Role::Success),
        std::cmp::Ordering::Less => ("siege_standing_lost", before - now, Role::Danger),
        std::cmp::Ordering::Equal => return,
    };
    let message = world.resource::<Prose>().line(
        key,
        &[
            ("quantity", &moved.to_string()),
            ("count", &now.to_string()),
        ],
    );
    world
        .resource_mut::<Scrollback>()
        .records_mut()
        .push(RecordKind::Completion)
        .text(FieldName::Name, "renown")
        .text(FieldName::Source, RAMPART)
        .count(FieldName::Quantity, moved)
        .text(FieldName::Message, &message)
        .role(role)
        .finish();
}
