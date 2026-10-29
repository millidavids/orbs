//! A room's wait: which instrument it is reported as, and why.

use bevy_ecs::prelude::*;

use crate::content::Lack;
use crate::tower::node::Name;
use crate::tower::panel::{Instrument, State, Unit};
use crate::tower::spell::Hold;
use crate::tower::{Cast, Craft};

/// How a room is reported: what is running there, or what wants the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wait {
    /// An instrument and its state, ticks left on a timed meter, and what it
    /// lacks if it is gathering.
    At {
        /// What it is called.
        instrument: String,
        /// Its two letters, for a narrow column.
        short: String,
        /// Its condition.
        state: State,
        /// Ticks left, when its meter counts ticks and has some left.
        left: Option<u64>,
        /// What a gathering instrument still lacks.
        lack: Option<Lack>,
    },
    /// Nothing here is running, and a spell here is held.
    Held {
        /// Which spell.
        spell: String,
        /// On what.
        hold: Hold,
    },
    /// A room with nothing in it to report.
    Nothing,
}

impl Wait {
    /// The one word a table column shows.
    #[must_use]
    pub const fn word(&self) -> &'static str {
        match self {
            Self::At { state, .. } => state.label(),
            Self::Held { .. } => "held",
            Self::Nothing => State::Empty.label(),
        }
    }
}

/// How much an instrument in `state` has to say about its room, most first.
///
/// A run in hand first, as the rail always drew it; then what stops the room
/// until the player acts; a fire after those, since a hearth alight is not a
/// job. A cold hearth last: it is a wait only when something wants heat, and
/// then the thing wanting it is reported. Exhaustive, so a new state has to be
/// placed here.
const fn rank(state: State) -> u8 {
    match state {
        State::Working | State::Scouring => 0,
        State::Ready => 1,
        State::Fouled => 2,
        State::Charged => 3,
        State::Gathering => 4,
        State::Burning => 5,
        State::Banked => 6,
        State::Empty => 7,
        State::Cold => 8,
    }
}

/// The instrument a room is reported as. Ties go to the first raised, which
/// is how the panel lists them.
///
/// A charged run that wants a fire the room lacks is not one keystroke from
/// running: the hearth is, and it takes the charged rank in its place.
#[must_use]
pub fn reported(instruments: &[Instrument]) -> Option<&Instrument> {
    let wanting = instruments.iter().any(|instrument| instrument.wants_fire);
    instruments.iter().min_by_key(|instrument| {
        if instrument.wants_fire {
            rank(State::Cold)
        } else if wanting && instrument.craft == Craft::Heating {
            rank(State::Charged)
        } else {
            rank(instrument.state)
        }
    })
}

/// Ticks left on an instrument's meter, when it counts ticks.
fn left(instrument: &Instrument) -> Option<u64> {
    let ticks = instrument.meter?.unit == Unit::Ticks;
    instrument.remaining().filter(|left| ticks && *left > 0)
}

/// What a room called `room` is waiting on, given its instruments.
///
/// A run in hand outranks a spell's hold, and a held spell outranks an idle
/// instrument: when automation is stuck, why is the useful answer. `casts` is
/// [`running_spells`](crate::tower::running_spells), walked once by the
/// caller.
#[must_use]
pub fn wait_in(instruments: &[Instrument], room: &str, casts: &[Cast]) -> Wait {
    let first = reported(instruments);
    let at = |instrument: &Instrument| Wait::At {
        instrument: instrument.name.clone(),
        short: instrument.short.clone(),
        state: instrument.state,
        left: left(instrument),
        lack: instrument.wants.clone(),
    };
    if let Some(running) =
        first.filter(|instrument| matches!(instrument.state, State::Working | State::Scouring))
    {
        return at(running);
    }
    let held = casts.iter().find_map(|cast| {
        (cast.domain == room)
            .then_some(cast)
            .zip(cast.held.as_ref())
    });
    if let Some((cast, hold)) = held {
        return Wait::Held {
            spell: cast.spell.clone(),
            hold: hold.clone(),
        };
    }
    first.map_or(Wait::Nothing, at)
}

/// [`wait_in`], reading the room's instruments and name from the world.
#[must_use]
pub fn wait_of(world: &World, room: Entity, casts: &[Cast]) -> Wait {
    let instruments = crate::tower::instruments_in(world, room);
    let name = world.get::<Name>(room).map_or("", |name| name.0.as_str());
    wait_in(&instruments, name, casts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tower::panel::Meter;

    fn instrument(name: &str, state: State) -> Instrument {
        Instrument {
            name: name.to_owned(),
            craft: if name == "athanor" {
                Craft::Heating
            } else {
                Craft::Idle
            },
            short: name.chars().take(2).collect(),
            state,
            meter: None,
            tints: [None; 2],
            wants: None,
            wants_fire: false,
        }
    }

    #[test]
    fn a_charged_run_wanting_heat_waits_on_the_hearth() {
        let mut alembic = instrument("alembic", State::Charged);
        alembic.wants_fire = true;
        for hearth in [State::Cold, State::Banked, State::Charged] {
            let room = [
                instrument("mortar_and_pestle", State::Empty),
                alembic.clone(),
                instrument("athanor", hearth),
            ];
            let got = reported(&room).map(|it| it.name.as_str());
            assert_eq!(got, Some("athanor"), "a {hearth:?} hearth");
        }
        // A charged run that needs no fire is still one keystroke away.
        let room = [
            instrument("mortar_and_pestle", State::Charged),
            alembic,
            instrument("athanor", State::Cold),
        ];
        assert_eq!(
            reported(&room).map(|it| it.name.as_str()),
            Some("mortar_and_pestle")
        );
    }

    #[test]
    fn a_fire_counts_what_is_left_and_a_job_what_is_to_come() {
        let metered = |state, done, total| Instrument {
            meter: Some(Meter {
                done,
                total,
                unit: Unit::Ticks,
            }),
            ..instrument("athanor", state)
        };
        assert_eq!(left(&metered(State::Burning, 592, 600)), Some(592));
        assert_eq!(left(&metered(State::Banked, 40, 40)), Some(40));
        assert_eq!(left(&metered(State::Working, 3, 10)), Some(7));
        assert_eq!(left(&metered(State::Scouring, 10, 10)), None);
    }

    #[test]
    fn a_room_reports_its_most_pressing_instrument() {
        use State::{Burning, Cold, Empty, Fouled, Ready, Working};
        // (the room, the instrument it reports). Work first; then a product
        // waiting, then a fault; a cold hearth is a wait only when something
        // wants heat; a resting room names its first instrument.
        let rooms: [(&[(&str, State)], &str); 5] = [
            (
                &[
                    ("mortar_and_pestle", Empty),
                    ("alembic", Fouled),
                    ("athanor", Cold),
                ],
                "alembic",
            ),
            (
                &[
                    ("alembic", Ready),
                    ("mortar_and_pestle", Working),
                    ("athanor", Burning),
                ],
                "mortar_and_pestle",
            ),
            (&[("alembic", Ready), ("athanor", Burning)], "alembic"),
            (
                &[("mortar_and_pestle", Empty), ("athanor", Cold)],
                "mortar_and_pestle",
            ),
            (
                &[("mortar_and_pestle", Empty), ("alembic", Empty)],
                "mortar_and_pestle",
            ),
        ];
        for (room, want) in rooms {
            let room: Vec<_> = room
                .iter()
                .map(|(name, state)| instrument(name, *state))
                .collect();
            let got = reported(&room).map(|it| (it.name.as_str(), it.state));
            assert_eq!(got.map(|(name, _)| name), Some(want), "{got:?} of {room:?}");
        }
    }

    #[test]
    fn a_held_spell_outranks_an_idle_tool_but_not_a_running_one() {
        let cast = Cast {
            spell: "tending".to_owned(),
            domain: "laboratory".to_owned(),
            cursors: 1,
            held: Some(Hold::Biding(5)),
        };
        let idle = [instrument("alembic", State::Ready)];
        assert!(matches!(
            wait_in(&idle, "laboratory", std::slice::from_ref(&cast)),
            Wait::Held { .. }
        ));
        let busy = [instrument("alembic", State::Working)];
        assert!(matches!(
            wait_in(&busy, "laboratory", std::slice::from_ref(&cast)),
            Wait::At {
                state: State::Working,
                ..
            }
        ));
        // A spell in another room holds nothing here.
        assert!(matches!(
            wait_in(&idle, "archive", std::slice::from_ref(&cast)),
            Wait::At { .. }
        ));
    }
}
