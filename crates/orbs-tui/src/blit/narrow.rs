//! ASCII for the glyphs a wide-ambiguous terminal would give two columns.

/// An ASCII stand-in for a glyph a wide-ambiguous terminal would give two
/// columns to.
///
/// Only the symbols the painters actually use are listed; anything else is left
/// alone.
///
/// The borders and the bars stay out although `─ │ ═ ║ ╔ ╗ ╚ ╝` and `█ ▓ ▒` all
/// measure East Asian Ambiguous (only `░` is Neutral), on the bet that a
/// terminal widening box drawing would break every full-screen program on the
/// system. So the wide-ambiguous path is partial: `term::PROBE` firing means
/// the symbols are safe, not the screen.
///
/// Every substitution is distinct (§14): the glyph carries the identity where
/// the colour cannot. The first version mapped `•` and `·` both to `.`, merging
/// *taken* into *locked* on a surface made entirely of those glyphs, and
/// invisibly — the row still lines up.
/// `every_substitution_is_its_own_character` holds the map injective, and
/// `blit::scan` holds that every Ambiguous glyph a painter draws is in it.
const SUBSTITUTIONS: [(char, char); 26] = [
    // The rail's fault marker and its "this room is automated" arrow.
    ('‼', '!'),
    ('►', '>'),
    ('◄', '<'),
    // The ward's six sigils.
    ('☼', '*'),
    ('♂', 'm'),
    ('♀', 'f'),
    ('♦', 'd'),
    ('♠', 's'),
    // The three states, kept three: filled, hollow, faint — taken from open
    // from locked, aligned from astray from loose.
    ('•', '+'),
    ('○', 'o'),
    ('·', '.'),
    ('■', '#'),
    // The maze's way out, and the transcript's outcome markers. `»` is
    // deliberately not `>`: that is `►`, and they share a screen.
    ('Ω', 'U'),
    ('√', 'v'),
    ('→', '-'),
    ('≈', '~'),
    ('¿', '?'),
    ('»', '}'),
    // The fire's sparks, and the boot card's furniture.
    ('∙', ','),
    ('°', '^'),
    ('⌂', 'A'),
    ('⌐', '='),
    ('∟', 'L'),
    // Endless stock, and it was missing: `Stock::label` draws `∞` for every
    // inexhaustible pile, and it is Ambiguous, so it took two columns and
    // shifted the row. `8` for the shape; nothing else claims it.
    ('∞', '8'),
    // The circle's marks row and its wiring, missing until a scan of the
    // painters found them: a balked row shifted one column, under the wrong
    // number. `{` beside `»`'s `}`, the other way.
    ('▲', 'x'),
    ('←', '{'),
];

pub(super) const fn narrowed(glyph: char) -> char {
    // One table, not two literals: as a match of 23 arms beside an array of 21,
    // `⌐` and `∟` were covered by no test at all.
    let mut index = 0;
    while index < SUBSTITUTIONS.len() {
        if SUBSTITUTIONS[index].0 == glyph {
            return SUBSTITUTIONS[index].1;
        }
        index += 1;
    }
    glyph
}

/// Every glyph the table covers — derived, not retyped.
///
/// `●` is deliberately absent: not in CP437, so `Cell::new` replaces it with `?`
/// before a frame reaches here (§19). A stand-in for a glyph that cannot occur
/// is a row nobody can reach.
#[cfg(test)]
pub(super) fn substituted() -> Vec<char> {
    SUBSTITUTIONS.iter().map(|(from, _)| *from).collect()
}

#[cfg(test)]
mod widths {
    use super::{narrowed, substituted};

    #[test]
    fn a_wide_terminal_gets_glyphs_it_can_draw_in_one_column() {
        // Or the table makes the problem it exists to solve.
        for glyph in substituted() {
            let swapped = narrowed(glyph);
            assert!(
                swapped.is_ascii_graphic(),
                "{glyph:?} was swapped for {swapped:?}, which is not plain ASCII",
            );
            assert_ne!(swapped, glyph, "{glyph:?} was left as itself");
        }
    }

    #[test]
    fn the_width_probe_is_a_glyph_this_table_substitutes() {
        // `term::PROBE` asks the question and this table is the only thing that
        // acts on the answer, so a probe glyph absent from here fires correctly
        // and changes nothing on screen.
        let probe = crate::term::PROBE;
        assert!(
            substituted().contains(&probe),
            "{probe:?} is what the terminal is measured with, but this table \
             leaves it alone — so a wide terminal would be detected and then \
             drawn to exactly as if it were narrow",
        );
    }

    #[test]
    fn every_substitution_is_its_own_character() {
        // §14. Two glyphs sharing a stand-in merges two meanings invisibly, the
        // row still lining up: `•` and `·` both went to `.`, making a *taken*
        // node identical to a *locked* one.
        let mut seen: Vec<char> = Vec::new();
        for glyph in substituted() {
            let swapped = narrowed(glyph);
            assert!(
                !seen.contains(&swapped),
                "{glyph:?} was swapped for {swapped:?}, which another glyph already uses",
            );
            seen.push(swapped);
        }
    }

    #[test]
    fn the_states_a_surface_is_made_of_stay_distinct() {
        // The rule above aimed at the two surfaces made *entirely* of these
        // glyphs. `loom.rs`: taken / open / locked. `board.rs`: held / loose /
        // aligned / astray.
        let weave = ['\u{2022}', '\u{25cb}', '\u{b7}'].map(narrowed);
        assert_eq!(
            weave.iter().collect::<std::collections::HashSet<_>>().len(),
            3,
            "two progression states became the same character: {weave:?}",
        );

        let ward = ['\u{25a0}', '\u{b7}', '\u{2022}', '\u{25cb}'].map(narrowed);
        assert_eq!(
            ward.iter().collect::<std::collections::HashSet<_>>().len(),
            4,
            "two ward states became the same character: {ward:?}",
        );
    }

    #[test]
    fn the_wards_six_sigils_stay_six() {
        // `board.rs`: *"six glyphs, never six colours"*. A terminal that
        // widens them still has to hand back six distinguishable things.
        let sigils = orbs_render::SIGILS.map(narrowed);
        assert_eq!(
            sigils
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            orbs_render::SIGILS.len(),
            "two sigils became the same character: {sigils:?}",
        );
    }

    #[test]
    fn the_substitutions_are_drawable_by_the_bevy_build_too() {
        // A stand-in outside CP437 is a glyph the *other* frontend cannot draw,
        // which is the boundary running backwards.
        for glyph in substituted() {
            let swapped = narrowed(glyph);
            assert!(
                orbs_render::is_renderable(swapped),
                "{glyph:?} is swapped for {swapped:?}, which the window cannot draw",
            );
        }
    }
}
