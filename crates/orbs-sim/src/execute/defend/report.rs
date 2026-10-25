//! What a resolved round says, and what a finished siege pays (§5.1).
//!
//! Three surfaces from one set of records (rule 4): every roll to the log,
//! one sentence on the pane, and `settle`'s pay.

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
            // `Normal` whichever way the die fell: a roll is ordinary play. A
            // lost siege goes red at `settle`.
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
        // What it cost, which only the sortie has, so no other key can print
        // the sortie's number.
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
/// The barrier moves before the orb earns — the work, then what it bought —
/// `muster`'s order (§19).
pub(super) fn settle(world: &mut World, rampart: Entity, outcome: Outcome) {
    let completion = world.get::<Siege>(rampart).map_or(0, Siege::completion);

    // The road stays empty for a while — §11.5's cadence, and what stops a
    // siege being a faucet.
    let now = world.resource::<crate::tick::Tick>().get();
    if let Some(mut siege) = world.get_mut::<Siege>(rampart) {
        siege.clear_at = Some(now.saturating_add(siege::CADENCE));
    }

    // The line comes home: survivors up to what was levied, and whatever the
    // menagerie brought while it fought (§19, *Supply*).
    if let Some((survivors, levied)) = world
        .get::<Siege>(rampart)
        .map(|siege| (siege.garrison.count, siege.levied))
    {
        siege::disband(world, survivors, levied);
    }

    // Stilled touches the barrier neither way: no penalty, and no victory.
    let _ = match outcome {
        Outcome::Held => tower::mend_by(world, siege::VICTORY_MEND),
        Outcome::Fallen => tower::wear_by(world, siege::DEFEAT_WEAR),
        Outcome::Stilled => 0,
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
        Outcome::Stilled => "siege_stilled",
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
                Outcome::Stilled => "stilled",
            },
        )
        .count(FieldName::Quantity, u64::from(completion))
        .text(FieldName::Message, &message)
        .role(match outcome {
            Outcome::Held => Role::Success,
            Outcome::Fallen => Role::Danger,
            Outcome::Stilled => Role::Normal,
        })
        .finish();

    // What the whole fight did to the tower's standing (§11.5, §19), said once
    // after the siege's sentence: the rounds move renown quietly, and a short
    // pane drops the gauges. Measured from `Siege::standing`, the opening
    // snapshot, so the rounds and the outcome are one number.
    let before = world.get::<Siege>(rampart).and_then(|siege| siege.standing);
    let stake = siege::renown_stake(arrived, completion, outcome);
    match outcome {
        Outcome::Held => tower::renown::earn(world, stake),
        Outcome::Fallen => tower::renown::slip(world, stake),
        Outcome::Stilled => {}
    }
    // No snapshot (an older save), nothing said: from nought, a lost fight
    // would announce winning the tower's whole total.
    if let Some(before) = before {
        standing(world, before);
    }

    // After the siege's sentence, as every seam keeps. Counted twice on a win:
    // *survive one* and *win five* are different stations.
    tower::done(world, &tower::Work::event(tower::SIEGE), earned);
    if outcome == Outcome::Held {
        tower::note(world, tower::SIEGE_WON);
    }
    // Every supply the siege had open stops with it (§19, *Supply*).
    super::spending::close_siege(world);

    // No rail mark: the bailey is not in `DOMAINS`, so nothing could draw it.
    // The board stays up until `defend` clears it.
}

/// Say what the whole fight did to the tower's standing, measured from `before`.
///
/// One record for the siege, since the rounds move renown quietly
/// (`renown::slip`). Measured at both ends rather than summed, so nothing is
/// saved and the floor comes free. Silent when nothing moved
/// (`a_move_of_nought_says_nothing`).
fn standing(world: &mut World, before: u64) {
    let now = world.resource::<tower::Renown>().get();
    let (key, moved, role) = match now.cmp(&before) {
        std::cmp::Ordering::Greater => ("siege_standing_won", now - before, Role::Success),
        std::cmp::Ordering::Less => ("siege_standing_lost", before - now, Role::Danger),
        std::cmp::Ordering::Equal => return,
    };
    tower::renown::say(world, key, moved, now, role, Some(RAMPART));
}
