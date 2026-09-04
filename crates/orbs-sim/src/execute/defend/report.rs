//! What a resolved round says, and what a finished siege pays (§5.1).
//!
//! **Three surfaces, and the split between them is rule 4 doing its job.** Every
//! roll goes to the log with its die and its face, `announce` puts one sentence
//! on the pane, and `settle` pays out — all three from the same records, read
//! three ways.

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
            // **Quiet**, like `follow`'s step: a round can be forty rolls and
            // the transcript would lose the player's own last line. The log is
            // where a postmortem is read; `announce` below is the one sentence
            // that reaches the pane.
            .quiet()
            // **`Normal`, not `Danger`, whichever way the die fell.** A roll
            // that told is the game working; the accent triad is for the tower
            // being in trouble, and forty red lines a round would spend it on
            // ordinary play. `settle` below is where a lost siege goes red.
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

    // **What the pledged dice actually came to**, said only when there were any.
    // The range was shown before the commitment; this is the other half of the
    // same rule — a gamble you cannot see the result of is not a gamble, it is a
    // die roll behind a curtain.
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
        // **What it cost, and only the sortie has one.** This handed
        // `round.spent` — the mettle the *sortie* took off the garrison — to all
        // four keys, so the moment anyone authored `{state}` into `area_line`,
        // `area_buckler` or `area_succour` those lines would quietly print the
        // sortie's number. Rule 6 makes that a content edit, done without
        // touching Rust and with nothing to catch it. `bought` above is the
        // per-area shape this follows.
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
/// **The barrier moves before the orb earns**, which is `muster`'s order and the
/// same reason: the work, and then what the work bought. §19 records the sanctum
/// shipping it the other way round and a finished course reading `integrity = 0`
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

    // **Escrow: progress-scaled, with a floor** (§11.5). Losing at 60% keeps
    // something worth having, which is what stops a lost siege being an evening
    // thrown away — *"effort is never wasted; only cynicism is"*.
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

    // **After the sentence about the siege, never before it**, which is the
    // order every other seam keeps and `done` documents: the wall holds, *and
    // then* the sanctum's line advances and the forge learns a charm. Counted
    // twice on a win — *survive a siege* and *win five* are different stations.
    tower::done(world, &tower::Work::event(tower::SIEGE), earned);
    if outcome == Outcome::Held {
        tower::note(world, tower::SIEGE_WON);
    }

    // **No rail mark, and that is a consequence of the bailey not being one of
    // the seven.** `briefs()` walks `DOMAINS` and the bailey is deliberately
    // absent from it (§10 fixes the count at seven), so a `Mark::News` written
    // here would be state nothing can ever draw — cleared only by an `attend`
    // that nobody was prompted to make.
    //
    // The siege says what happened on the transcript and in `bailey.log`, which
    // is where a postmortem is read anyway. **A finished siege also leaves the
    // board up**, deliberately: the last thing that happened is what a player
    // wants to see, and `defend` clears it.
}
