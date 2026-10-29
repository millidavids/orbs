//! A wait in twelve cells, for the rail's detail row.

use super::room::Wait;
use crate::content::Prose;
use crate::tower::panel::{State, abbreviate, short_material};
use crate::tower::spell::Hold;

/// What a wait draws on the rail's detail row, or `None` when the state row
/// already says it. A running meter's own detail (`al 34t`) outranks this, and
/// the reader hears [`sentence`](super::sentence) in its place.
#[must_use]
pub fn short(prose: &Prose, wait: &Wait) -> Option<String> {
    match wait {
        Wait::At {
            instrument,
            short,
            state,
            lack,
            ..
        } => match state {
            State::Gathering => lack.as_ref().map(|lack| {
                prose.line(
                    "brief_wants",
                    &[("name", short), ("detail", short_material(lack.material()))],
                )
            }),
            State::Ready | State::Fouled | State::Charged => Some(at(prose, short, instrument)),
            State::Working
            | State::Scouring
            | State::Burning
            | State::Banked
            | State::Empty
            | State::Cold => None,
        },
        Wait::Held { hold, .. } => Some(match hold {
            Hold::Busy(blocked) => {
                prose.line("brief_after", &[("name", &abbreviate(blocked.name()))])
            }
            Hold::Watching(_) => prose.line("brief_watching", &[]),
            Hold::Pulling(_) => prose.line("brief_pulling", &[]),
            Hold::Biding(left) => prose.line("brief_biding", &[("count", &left.to_string())]),
        }),
        Wait::Nothing => None,
    }
}

/// `at al`, or the whole name where the two letters would read as the word
/// before them: the athanor's `at` drew `at at`.
fn at(prose: &Prose, short: &str, name: &str) -> String {
    let drawn = prose.line("brief_at", &[("name", short)]);
    let mut words = drawn.split_whitespace();
    let doubled = words
        .next()
        .is_some_and(|first| words.next() == Some(first));
    if doubled {
        prose.line("brief_at", &[("name", name)])
    } else {
        drawn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Lack;

    /// What the rail has for a detail: sixteen columns, an inset of one each
    /// side, and two cells of indent.
    const ROOM: usize = 12;

    #[test]
    fn every_input_a_gathering_tool_could_lack_fits_the_rail() {
        // Cut in half, `fr +tincture` would be the `tending.spe` defect again.
        // A lack is always some recipe's input, at the instrument it feeds.
        let prose = Prose::builtin();
        let recipes = crate::content::Recipes::builtin();
        for instrument in recipes.instruments() {
            let inputs = recipes
                .for_instrument(instrument)
                .iter()
                .flat_map(crate::content::Recipe::inputs);
            for (material, lack) in inputs.flat_map(|material| {
                [
                    (material, Lack::Kind(material.to_owned())),
                    (material, Lack::More(material.to_owned())),
                ]
            }) {
                let wait = Wait::At {
                    instrument: instrument.to_owned(),
                    short: crate::tower::panel::abbreviate(instrument),
                    state: State::Gathering,
                    left: None,
                    lack: Some(lack),
                };
                let drawn = short(&prose, &wait).expect("a gathering tool says what it lacks");
                assert!(
                    drawn.chars().count() <= ROOM,
                    "`{drawn}` is {} cells for {material}",
                    drawn.chars().count()
                );
            }
        }
    }

    #[test]
    fn a_detail_never_reads_one_word_twice() {
        let prose = Prose::builtin();
        let recipes = crate::content::Recipes::builtin();
        let mut names = recipes.instruments();
        names.push(crate::tower::ATHANOR);
        for name in names {
            let wait = Wait::At {
                instrument: name.to_owned(),
                short: crate::tower::panel::abbreviate(name),
                state: State::Charged,
                left: None,
                lack: None,
            };
            let drawn = short(&prose, &wait).expect("a charged tool is named");
            let words: Vec<&str> = drawn.split_whitespace().collect();
            assert!(words.windows(2).all(|pair| pair[0] != pair[1]), "`{drawn}`");
            assert!(drawn.chars().count() <= ROOM, "`{drawn}`");
        }
    }

    #[test]
    fn every_hold_fits_the_rail() {
        let prose = Prose::builtin();
        let blocked = crate::tower::spell::Blocked::Slot {
            name: "mortar_and_pestle".to_owned(),
            doing: "grinding".to_owned(),
        };
        for hold in [
            Hold::Busy(blocked),
            Hold::Watching("the_longest_name_a_spell_could_watch".to_owned()),
            Hold::Pulling("satchel".to_owned()),
            Hold::Biding(u64::from(crate::tower::spell::LONGEST_BIDE)),
        ] {
            let wait = Wait::Held {
                spell: "tending".to_owned(),
                hold,
            };
            let drawn = short(&prose, &wait).expect("a held spell says so");
            assert!(drawn.chars().count() <= ROOM, "`{drawn}`");
        }
    }
}
