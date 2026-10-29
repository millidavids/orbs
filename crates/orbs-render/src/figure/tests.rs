//! The rules every figure keeps, swept over all five.

use super::lectern::KNIT_EVERY;
use super::prism::FOCUS_EVERY;
use super::{Figure, Rite, cell};
use crate::FLIP_HZ;
use crate::pulse::harness::sweep;
use crate::pulse::{CYCLE_SECS, TICKS_PER_CYCLE};
use crate::style::{Depiction, Fill};

const TOTAL: u16 = 16;
const LANES: u16 = 2;

fn rite(figure: Figure, phase: f32, working: bool) -> Rite {
    Rite {
        phase,
        figure,
        working,
        settling: false,
        fill: Fill::Middle,
    }
}

fn bar(filled: u16, rite: Rite) -> Vec<(char, Depiction)> {
    (0..LANES)
        .flat_map(|lane| (0..TOTAL).map(move |step| cell(lane, step, filled, TOTAL, rite)))
        .collect()
}

/// Every working pose a figure has: the lattice also settles.
fn poses(figure: Figure, phase: f32) -> Vec<Rite> {
    let working = rite(figure, phase, true);
    let mut all = vec![working];
    if figure == Figure::Lattice {
        all.push(Rite {
            settling: true,
            ..working
        });
    }
    all
}

#[test]
fn the_value_survives_in_greyscale() {
    // §14. Whatever moves, a filled cell is `█` or `▓` and an empty one `░` or
    // `▒`, so the fill reads by coverage alone.
    for figure in Figure::ALL {
        for phase in sweep().step_by(7) {
            for pose in poses(figure, phase) {
                for filled in 0..=TOTAL {
                    for step in 0..TOTAL {
                        for lane in 0..LANES {
                            let (glyph, _) = cell(lane, step, filled, TOTAL, pose);
                            let solid = matches!(glyph, '█' | '▓');
                            assert_eq!(
                                solid,
                                step < filled,
                                "{figure:?} drew {glyph} at {step} of fill {filled}",
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn a_figure_at_rest_is_perfectly_still() {
    // §10.1's at-rest rule, and what motion off reduces every figure to.
    for figure in Figure::ALL {
        for filled in [0, 1, 8, TOTAL] {
            let first = bar(filled, rite(figure, 0.0, false));
            for phase in sweep() {
                assert_eq!(
                    bar(filled, rite(figure, phase, false)),
                    first,
                    "a resting {figure:?} moved at {phase}",
                );
            }
        }
    }
}

#[test]
fn a_working_figure_moves() {
    for figure in Figure::ALL {
        for pose in poses(figure, 0.0) {
            let first = bar(8, pose);
            assert!(
                sweep().any(|phase| bar(8, Rite { phase, ..pose }) != first),
                "a working {figure:?} never moved",
            );
        }
    }
}

#[test]
fn no_cell_flashes_at_the_photosensitive_floor_or_turns_the_bar_over() {
    // Measured, not read off the constants: three flashes a second is the
    // floor (§19), a flash being two changes, counted per cell over a cycle.
    // And a picture is a shimmer, not a pulse: no moment turns half the bar.
    let cells = usize::from(LANES * TOTAL);
    for figure in Figure::ALL {
        for pose in poses(figure, 0.0) {
            let at = |phase: f32| bar(8, Rite { phase, ..pose });
            let mut changes = vec![0u16; cells];
            let mut before = at(0.0);
            for phase in sweep().skip(1) {
                let now = at(phase);
                let moved = before.iter().zip(&now).filter(|(a, b)| a != b).count();
                assert!(moved < cells / 2, "{figure:?} turned {moved} cells at once");
                for (count, (a, b)) in changes.iter_mut().zip(before.iter().zip(&now)) {
                    *count += u16::from(a != b);
                }
                before = now;
            }
            let busiest = changes.into_iter().max().unwrap_or(0);
            // Every tempo here is four ticks or slower, so a third of the floor.
            let flashes = f32::from(busiest) / 2.0 / CYCLE_SECS;
            assert!(
                flashes <= 1.0,
                "a {figure:?} cell flashed {flashes} times a second",
            );
        }
    }
}

#[test]
fn past_the_bar_is_blank() {
    for figure in Figure::ALL {
        assert_eq!(cell(0, TOTAL, TOTAL, TOTAL, rite(figure, 3.0, true)).0, ' ');
    }
}

/// The phase at a beat of `every` ticks, counted on past the cycle's wrap.
fn at_beat(beat: u16, every: u16) -> f32 {
    f32::from((beat * every) % TICKS_PER_CYCLE) / FLIP_HZ
}

#[test]
fn the_prisms_focus_climbs_to_the_top_through_the_cycle_wrap() {
    // `beat % open` cut a climb short at the wrap unless the cycle's beats
    // were a whole number of `open`: twenty open sockets never reached 16-19.
    let beats = TICKS_PER_CYCLE / FOCUS_EVERY;
    for filled in 0..TOTAL {
        let open = TOTAL - filled;
        let focus = |beat: u16| {
            let rite = rite(Figure::Prism, at_beat(beat, FOCUS_EVERY), true);
            (filled..TOTAL)
                .find(|step| cell(0, *step, filled, TOTAL, rite).0 == '▒')
                .map(|step| step - filled)
        };
        let mut last = focus(0);
        for beat in 1..2 * beats {
            let now = focus(beat);
            let fine = match (last, now) {
                (Some(before), Some(now)) => now == before + 1 || (now == 0 && before == open - 1),
                (Some(before), None) => before == open - 1,
                (None, Some(now)) => now == 0,
                (None, None) => true,
            };
            assert!(fine, "{open} open: {last:?} then {now:?} at beat {beat}");
            last = now;
        }
    }
}

#[test]
fn the_lecterns_seams_knit_one_a_beat_through_the_cycle_wrap() {
    // Nine beats to a cycle held the open seams twice as long at the wrap.
    let beats = TICKS_PER_CYCLE / KNIT_EVERY;
    for beat in 0..2 * beats {
        let rite = rite(Figure::Lectern, at_beat(beat, KNIT_EVERY), true);
        let open = (0..TOTAL)
            .filter(|step| cell(0, *step, TOTAL, TOTAL, rite).0 == '▓')
            .count();
        assert_eq!(open, 3 - usize::from(beat % 4), "beat {beat}");
    }
}

#[test]
fn the_lattice_settles_cooler_than_it_catches() {
    // An anneal is the fire going down; binding is the fire catching.
    let heats = |settling: bool| {
        bar(
            TOTAL,
            Rite {
                settling,
                ..rite(Figure::Lattice, 0.0, false)
            },
        )
        .into_iter()
        .filter_map(|(_, depiction)| depiction.heat())
        .max()
    };
    assert!(heats(true) < heats(false));
}
