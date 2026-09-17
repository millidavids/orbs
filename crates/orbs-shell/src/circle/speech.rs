//! What the circle says, for a reader who cannot see it.
//!
//! **Spoken once as a summary, never cell by cell**, and in words rather than the
//! drawn rows — the lattice's `residue_spoken` lesson, and here the table *is*
//! the puzzle, so there is no shape to fall back on. The painted sense rows are
//! silent, so everything they show has to be in these words: each glyph and what
//! it is given (turned or not), the temper by the senses lit on each of its lit
//! rows, and where the last call balked.

use orbs_render::{Circle, CircleGiven, Painter, Role, UtteranceKind};
use orbs_sim::content::Prose;

/// Announce the board.
pub(super) fn speak(painter: &mut Painter<'_>, circle: &Circle, prose: &Prose) {
    painter.announce(
        UtteranceKind::Progress,
        Role::Normal,
        &summary(circle, prose),
    );
}

/// The whole summary, as one sentence a reader hears.
pub(super) fn summary(circle: &Circle, prose: &Prose) -> String {
    // **Only when there are other glyphs to feed it.** At a lesser circle the
    // keystone is the one line and is given two senses, which a reader has not
    // heard yet — so it is spoken as an outer glyph is, with what it is given.
    let keystone = (circle.lines.len() > 1).then(|| circle.lines.len() - 1);
    let glyphs: Vec<String> = circle
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if Some(index) == keystone {
                // The keystone is last, and is given the other two — which a
                // reader has just heard named, so saying so again is length
                // without information.
                prose.line("circle_keystone_spoken", &[("detail", &line.humour)])
            } else {
                // Two things given, always — two senses or two glyphs — so the
                // `and` between them is the template's, not a literal here.
                let given = |index: usize| {
                    line.given
                        .get(index)
                        .map(|given| named(given, prose))
                        .unwrap_or_default()
                };
                prose.line(
                    "circle_glyph_spoken",
                    &[
                        ("name", &line.glyph),
                        ("detail", &line.humour),
                        ("quantity", &given(0)),
                        ("count", &given(1)),
                    ],
                )
            }
        })
        .collect();
    let balking = circle.balking();
    let detail = if circle.answer.is_none() {
        prose.line("circle_uncalled_spoken", &[])
    } else {
        let rows: Vec<String> = balking.iter().map(ToString::to_string).collect();
        prose.line(
            &prose.counted_key("circle_balks_spoken", counted(rows.len())),
            &[("quantity", &rows.join(" "))],
        )
    };
    let lit: Vec<String> = circle
        .temper
        .iter()
        .enumerate()
        .filter(|(_, lit)| **lit)
        .map(|(row, _)| lit_row(circle, row, prose))
        .collect();
    prose.line(
        &prose.counted_key("circle_spoken", counted(lit.len())),
        &[
            ("name", &glyphs.join(". ")),
            ("quantity", &lit.join(", ")),
            ("detail", &detail),
        ],
    )
}

/// One thing a glyph is given, as a reader hears it: *"breath"*, or *"turned
/// breath"*.
fn named(given: &CircleGiven, prose: &Prose) -> String {
    if given.turned {
        prose.line("circle_turned_spoken", &[("name", &given.name)])
    } else {
        given.name.clone()
    }
}

/// A lit row by its number **and the senses lit on it**: *"4 (bone breath)"*.
///
/// **The senses, not only the number**, because the number alone asks a reader to
/// know how the rows are counted — the painted sense rows say it to an eye and
/// are silent to an ear. With the senses named, the temper is a list of which
/// senses make the beast answer, and nothing else is needed to solve it.
fn lit_row(circle: &Circle, row: usize, prose: &Prose) -> String {
    let senses: Vec<&str> = circle
        .senses
        .iter()
        .zip(&circle.lit)
        .filter(|(_, lit)| lit.get(row).copied().unwrap_or(false))
        .map(|(name, _)| name.as_str())
        .collect();
    let count = (row + 1).to_string();
    if senses.is_empty() {
        prose.line("circle_row_none_spoken", &[("count", &count)])
    } else {
        prose.line(
            "circle_row_spoken",
            &[("count", &count), ("name", &senses.join(" "))],
        )
    }
}

/// A row count as `Prose::counted_key` takes one.
fn counted(rows: usize) -> u64 {
    u64::try_from(rows).unwrap_or(u64::MAX)
}
