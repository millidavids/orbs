//! The circle as the board needs it: cells from the beast, words from content.

use super::temper;
use super::{Beast, Glyph, Shape};
use crate::content::Prose;

/// A beast at the circle, drawn.
///
/// Every word the board shows is resolved here, because `orbs-render` may never
/// depend on `orbs-sim` and holds no authored English (rule 6). The senses'
/// names and the tally are prose, the glyphs and humours the parser's own
/// words, so the picture names exactly what `limn` takes.
///
/// A lesser circle is the same board with less on it — one line, two senses,
/// four columns — since the board draws what it is handed rather than knowing
/// which circle it is.
///
/// A turned wire is carried beside the sense's name rather than folded into it,
/// so the drawn mark and the spoken *turned* come from one fact.
#[must_use]
pub fn view(beast: &Beast, prose: &Prose) -> orbs_render::Circle {
    let shape = beast.shape();
    let turned = shape.turned();
    let (senses_drawn, rows) = (shape.senses(), shape.rows());
    let senses: Vec<String> = (0..senses_drawn)
        .map(|sense| prose.line(&format!("circle_sense_{sense}"), &[]))
        .collect();
    let given = |name: &str, turned: bool| orbs_render::CircleGiven {
        name: name.to_owned(),
        turned,
    };
    let named = |glyph: Glyph, pair: [usize; 2]| -> Vec<orbs_render::CircleGiven> {
        pair.into_iter()
            .enumerate()
            .map(|(input, sense)| {
                given(
                    senses.get(sense).map_or("", String::as_str),
                    turned.wire(glyph, input),
                )
            })
            .collect()
    };
    let line = |glyph: Glyph, given: Vec<orbs_render::CircleGiven>| orbs_render::CircleLine {
        glyph: glyph.word().to_owned(),
        humour: beast.humour(glyph).word().to_owned(),
        given,
    };
    let lines = match shape {
        // The keystone alone, given the two senses as the outer glyphs would be,
        // and never turned.
        Shape::Lesser(_) => vec![line(Glyph::Keystone, named(Glyph::Keystone, [0, 1]))],
        // The outer two first, then the keystone they feed, so the board reads
        // in the direction a sense travels: into a glyph, out through the
        // keystone to the beast.
        Shape::Whole(puzzle) => vec![
            line(Glyph::Sunwise, named(Glyph::Sunwise, puzzle.wiring.sunwise)),
            line(
                Glyph::Widdershins,
                named(Glyph::Widdershins, puzzle.wiring.widdershins),
            ),
            line(
                Glyph::Keystone,
                vec![
                    given(Glyph::Sunwise.word(), false),
                    given(Glyph::Widdershins.word(), false),
                ],
            ),
        ],
    };
    let lit = (0..senses_drawn)
        .map(|sense| {
            (0..rows)
                .map(|row| temper::sense_lit(row, sense, senses_drawn))
                .collect()
        })
        .collect();
    let drawn = |temper: temper::Temper| (0..rows).map(|row| temper.lit(row)).collect();
    let tally = match beast.answer() {
        None => prose.line("circle_tally_uncalled", &[]),
        Some(_) => {
            let balking = u32::try_from(rows)
                .unwrap_or(u32::MAX)
                .saturating_sub(beast.agreeing());
            prose.line(
                "circle_tally",
                &[
                    ("name", &prose.counted("circle_calls", beast.calls())),
                    ("detail", &prose.counted("circle_balks", balking)),
                ],
            )
        }
    };
    orbs_render::Circle {
        lines,
        senses,
        lit,
        temper: drawn(shape.temper()),
        answer: beast.answer().map(drawn),
        labels: [
            prose.line("circle_temper_label", &[]),
            prose.line("circle_answer_label", &[]),
        ],
        tally,
    }
}
