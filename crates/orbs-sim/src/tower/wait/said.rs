//! A wait as one sentence — what `status rates` prints and the rail's detail
//! speaks, so the two say the same.

use super::room::Wait;
use crate::content::{Lack, Prose};
use crate::tower::panel::State;
use crate::tower::spell::Hold;

/// What a room waits on, as one sentence.
#[must_use]
pub fn sentence(prose: &Prose, wait: &Wait) -> String {
    match wait {
        Wait::At {
            instrument,
            state,
            left,
            lack,
            ..
        } => at(prose, instrument, *state, *left, lack.as_ref()),
        Wait::Held { spell, hold } => held(prose, spell, hold),
        Wait::Nothing => prose.line("rates_wait_empty", &[]),
    }
}

/// An instrument's sentence, by its state.
fn at(prose: &Prose, name: &str, state: State, left: Option<u64>, lack: Option<&Lack>) -> String {
    let count = left.map(|left| left.to_string()).unwrap_or_default();
    let key = match (state, left) {
        (State::Working, Some(_)) => "rates_wait_working",
        (State::Working, None) => "rates_wait_busy",
        (State::Scouring, Some(_)) => "rates_wait_scouring",
        (State::Scouring, None) => "rates_wait_scrubbing",
        (State::Ready, _) => "rates_wait_ready",
        (State::Fouled, _) => "rates_wait_fouled",
        (State::Charged, _) => "rates_wait_charged",
        (State::Gathering, _) => match lack {
            Some(Lack::More(_)) => "rates_wait_more",
            _ => "rates_wait_gathering",
        },
        (State::Cold, _) => "rates_wait_cold",
        (State::Burning, Some(_)) => "rates_wait_burning",
        (State::Burning, None) => "rates_wait_alight",
        (State::Banked, _) => "rates_wait_banked",
        (State::Empty, _) => "rates_wait_empty",
    };
    prose.line(
        key,
        &[
            ("name", name),
            ("count", &count),
            ("detail", lack.map_or("", Lack::material)),
        ],
    )
}

/// A held spell's sentence — the one its log already says, where there is one.
fn held(prose: &Prose, spell: &str, hold: &Hold) -> String {
    match hold {
        Hold::Busy(blocked) => prose.line(
            "spell_waiting",
            &[
                ("name", spell),
                ("source", blocked.name()),
                ("state", blocked.doing()),
            ],
        ),
        Hold::Watching(wanted) => {
            prose.line("spell_watching", &[("name", spell), ("source", wanted)])
        }
        Hold::Pulling(from) => {
            prose.line("rates_hold_pulling", &[("name", spell), ("source", from)])
        }
        Hold::Biding(left) => prose.line(
            "rates_hold_biding",
            &[("name", spell), ("count", &left.to_string())],
        ),
    }
}
