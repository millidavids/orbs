//! The tower rail — every domain at a glance, down the right (DESIGN.md §9).
//!
//! Awareness only, never commandable. It replaces the telemetry pane, which was
//! nine developer readings in fifty-eight columns; what survives of that pane is
//! the five rows at the rail's foot, which are the ones §15 made playability
//! gates and the ones §9 requires be readable.
//!
//! # It says what the domain's own pane would, in fewer words
//!
//! Rule 2 lets a frontend decide only *how* a cell is drawn, so every fact here
//! comes from [`Sim::briefs`] rather than from anything this module works out —
//! `orbs-tui` draws the same rail from the same `Vec<Brief>`. What is decided
//! here is layout and glyph, and nothing else.
//!
//! # Four of the seven are dark, and that is the point
//!
//! §10 fixes seven domains and the player starts with two. An unbuilt room draws
//! as a dim dotted row with no name: seven slots with four dark says *there is
//! more* without saying what, which is the foreshadowing §11's discovery loop
//! wants and which naming them would spend.
//!
//! # Each box is ruled off from the next
//!
//! A box's content is one to four rows, so consecutive domains ran together in a
//! column of whitespace and which line belonged to which room was left to the
//! reader. A horizontal rule closes every box but the last, whose boundary is the
//! foot's own rule. Both are silent — structure writes cells and no speech.

use orbs_render::{Frame, Pos, Rect, Span, Style};
use orbs_sim::Sim;
use orbs_sim::tower::{Brief, Mark, State};

use super::screen::Screen;

/// A domain with something to say, and a domain with something wrong.
///
/// **Glyphs, not colours.** §14 forbids meaning that lives only in hue, and a
/// rail is exactly where that rule is easiest to break — a red border and a
/// green exclamation are the obvious design and are invisible to a third of the
/// reasons §14 exists. The accent carries the same fact a second time, which is
/// what an accent is for.
const NEWS: &str = "!";
/// A fault. `‼` is CP437 0x13, and reads as trouble at one glyph.
const FAULT: &str = "‼";

/// A spell is running here.
///
/// **`►` is CP437 0x10; `▸` is not in the page at all and drew as `?`.** So the
/// row that says a room is automated said the orb did not know what was there —
/// the worst possible reading of it, in the pane whose whole job is *at a glance*.
/// The same defect the board's `▪` had (§19), found the same way: by looking.
const RUNNING: char = '►';

/// Draw the rail, if the layout found room for it.
///
/// `boxes` and `foot` come from [`ScreenLayout`](orbs_render::ScreenLayout) —
/// this module does no arithmetic about where the rail is, because §19 records
/// that transcribed geometry rots silently and the layout is the one place that
/// knows.
pub fn paint(
    frame: &mut Frame,
    sim: &Sim,
    screen: &Screen,
    at: Rect,
    boxes: &[Rect],
    foot: Rect,
    briefs: &[Brief],
) {
    if at.is_empty() {
        return;
    }
    let mut painter = frame.painter(at);
    painter.border(at, Some("tower"), Style::DIM);

    // **Handed in from `Panel`, never asked from the sim here.** `Sim::briefs`
    // walks every room and runs `instruments_in` per room; called from a painter
    // that is a 60 Hz sweep over 1 Hz data, which is the allocation `Panel`
    // exists to stop and says so in its own doc.
    for (index, (brief, slot)) in briefs.iter().zip(boxes).enumerate() {
        // **The rule belongs to the box above the boundary, and the last box has
        // none.** `lay_rail` hands out contiguous slots, so the foot begins on the
        // row after the seventh box ends — and the foot opens with a rule of its
        // own. A seventh rule here would draw two in adjacent rows.
        let ruled = index + 1 < boxes.len();
        domain(&mut painter, brief, *slot, ruled);
    }
    readings(&mut painter, sim, screen, foot);
}

/// One domain's box: what it is, what it is doing, and what is running there.
///
/// `ruled` closes the box with a horizontal line, which is what separates one
/// domain from the next. It is drawn **first**, so a box whose content would have
/// reached the last row is cut by the rule rather than drawing over it.
fn domain(painter: &mut orbs_render::Painter<'_>, brief: &Brief, at: Rect, ruled: bool) {
    if at.is_empty() {
        return;
    }

    // Where content must stop. Without a rule that is the box's own bottom; with
    // one it is the rule's row, so every `row >=` guard below asks this rather
    // than `at.bottom()`.
    //
    // Silent, like the border and the foot's rule: §19's frame rule is that
    // structure writes cells and no speech, and a reader hearing six horizontal
    // lines read out between seven domains would get box-drawing noise where the
    // sighted player gets separation for free.
    let floor = if ruled && at.rows > 1 {
        let row = at.bottom().saturating_sub(1);
        painter.glyphs(
            Pos::new(at.col, row),
            &"─".repeat(usize::from(at.cols)),
            Style::DIM,
        );
        row
    } else {
        at.bottom()
    };

    if !brief.built {
        // **A rule, and no name.** Drawn rather than skipped so the seven slots
        // keep fixed positions — a box that appeared and pushed the others down
        // would make the rail unreadable at the one moment it has news.
        //
        // `glyphs` rather than `span`, so it is **silent**: §19's frame rule is
        // that structure writes cells and no speech, and a reader being read
        // fourteen middle dots four times is the noisiest possible way to say
        // *there is nothing here yet*.
        painter.glyphs(at.origin(), &"·".repeat(usize::from(at.cols)), Style::DIM);
        return;
    }

    let mark = match brief.mark {
        Some(Mark::Fault) => FAULT,
        Some(Mark::News) => NEWS,
        None => "",
    };
    let style = match brief.mark {
        Some(Mark::Fault) => Style::DANGER,
        Some(Mark::News) => Style::SUCCESS,
        None => Style::NORMAL,
    };

    // The name, with its mark pushed to the right edge so a glance down the rail
    // finds every mark in one column.
    //
    // **`chars().count()`, never `len()`.** `‼` is one column and three bytes, so
    // a byte count padded a fault two columns short of the edge while `!` — one
    // byte — landed exactly on it. Faults and news then appeared in *different*
    // columns, breaking the one property this arithmetic exists for. The test
    // below measured the mark correctly all along, which is why it could not
    // catch it.
    let width = usize::from(at.cols);
    let wide = mark.chars().count();
    let name = truncate(brief.name, width.saturating_sub(wide));
    let pad = width.saturating_sub(name.chars().count() + wide);
    painter.span(
        at.origin(),
        &Span::new(&format!("{name}{}{mark}", " ".repeat(pad))).with_style(style),
    );

    let mut row = at.row.saturating_add(1);
    if row >= floor {
        return;
    }
    painter.span(
        Pos::new(at.col, row),
        &Span::new(&format!("  {}", word(brief.state))).with_style(Style::DIM),
    );

    // What is working, and what is automating it. Both are optional and both
    // are the reason a box beats a row: §9's sidebar could hold one or the
    // other, and a player wants to know that the alembic is busy *and* that
    // `tending` is the thing keeping it busy.
    if let Some(detail) = &brief.detail {
        row = row.saturating_add(1);
        if row >= floor {
            return;
        }
        painter.span(
            Pos::new(at.col, row),
            &Span::new(&format!("  {}", truncate(detail, width.saturating_sub(2))))
                .with_style(Style::DIM),
        );
    }
    if let Some(spell) = &brief.spell {
        row = row.saturating_add(1);
        if row >= floor {
            return;
        }
        painter.span(
            Pos::new(at.col, row),
            &Span::new(&format!(
                "  {RUNNING}{}",
                truncate(spell, width.saturating_sub(3))
            ))
            .with_style(Style::COST),
        );
    }
}

/// The readings the telemetry pane used to carry.
///
/// **These five and no more.** `tick` is the only visible proof the sim runs,
/// `held` is §8's *"surfaced in `status` and in the sidebar"*, `scale` against a
/// fixed `grid` is §19's whole change in two rows, and `focus` is §9's setting.
/// Everything else the pane had — `seed`, `queued`, `logged` — is already in
/// `status`, and `status` is where it belongs: it is the full answer, and this is
/// the glance.
fn readings(painter: &mut orbs_render::Painter<'_>, sim: &Sim, screen: &Screen, at: Rect) {
    if at.is_empty() {
        return;
    }
    let held = sim.concentration();
    let rows = [
        format!("tick  {}", sim.tick().get()),
        // **Absent until the orb has been taught to hold one** — §8 is explicit
        // that concentration 0 is the starting state and not an exhaustion, so a
        // row reading `held 0 of 0` would spend a line saying nothing for the
        // first two minutes of the game.
        if held > 0 {
            format!("held  {} of {held}", sim.bound().len())
        } else {
            String::new()
        },
        format!("scale {:.2}x", screen.scale()),
        format!("grid  {}x{}", screen.grid.cols, screen.grid.rows),
        format!("focus {}", focus(screen)),
    ];

    let mut row = at.row;
    // A rule between the domains and the readings, so the eye knows the numbers
    // below it are about the orb rather than about the seventh domain. Silent,
    // for the reason a border is: structure writes cells and no speech.
    painter.glyphs(
        Pos::new(at.col, row),
        &"─".repeat(usize::from(at.cols)),
        Style::DIM,
    );
    row = row.saturating_add(1);

    for text in rows.iter().filter(|text| !text.is_empty()) {
        if row >= at.bottom() {
            return;
        }
        painter.span(
            Pos::new(at.col, row),
            &Span::new(truncate(text, usize::from(at.cols))).with_style(Style::DIM),
        );
        row = row.saturating_add(1);
    }
}

/// §9's focus mode, as a word.
const fn focus(screen: &Screen) -> &'static str {
    screen.mode.word()
}

/// What a domain is doing, in the panel's own vocabulary.
///
/// **[`State::label`] for nine of the ten, and one deliberate difference.**
///
/// This used to be a hand-written second copy of all ten arms, under a comment
/// claiming *"the same words the instrument panel prints, so the rail and the
/// pane cannot describe one room two ways"* — and they already did: `Empty` read
/// `idle` here and `empty` there, so an untouched laboratory was `idle` on the
/// rail and `empty` in the pane and to a screen reader.
///
/// The word is kept, because the two surfaces are describing different subjects.
/// The panel labels an **instrument**, and an instrument with nothing in it is
/// *empty*. The rail labels a **room** — `Brief::state` is what its busiest
/// instrument is doing — and a room is not empty, it is idle. What was wrong was
/// writing the other nine out again beside it, which is what let the difference
/// become accidental instead of stated.
const fn word(state: State) -> &'static str {
    match state {
        State::Empty => "idle",
        other => other.label(),
    }
}

/// Cut a word to fit, rather than letting it run into the border.
///
/// §19's frame rule is that **truncation is visual only** — the linear stream
/// still carries the full text, so a narrow rail is a visual constraint and never
/// an informational one.
fn truncate(text: &str, width: usize) -> &str {
    // **`orbs_render::arriving`, which the instrument panel one file over has
    // been calling all along.** This was a third hand-rolled char-safe cut in
    // the workspace, and `char_index` — the helper `arriving` is built on —
    // exists precisely because two private copies of that idiom in different
    // crates desynchronised Tab completion once already.
    //
    // It also allocated: a `String` per call, four calls a box, seven boxes,
    // every frame, for text that changes at 1 Hz.
    orbs_render::arriving(text, u32::try_from(width).unwrap_or(u32::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::RAIL_COLS;

    #[test]
    fn both_marks_land_in_the_same_column() {
        // The property the padding exists for, and the one a byte count broke:
        // `‼` is three bytes and one column, so `len()` pushed a fault two short
        // of the edge while `!` sat on it.
        let width = usize::from(RAIL_COLS) - 2;
        let row = |name: &str, mark: &str| {
            let wide = mark.chars().count();
            let cut = truncate(name, width.saturating_sub(wide));
            let pad = width.saturating_sub(cut.chars().count() + wide);
            format!("{cut}{}{mark}", " ".repeat(pad))
        };
        for name in orbs_sim::tower::DOMAINS {
            for mark in [FAULT, NEWS, ""] {
                assert_eq!(
                    row(name, mark).chars().count(),
                    width,
                    "`{name}` with {mark:?} does not fill the box",
                );
            }
        }
    }

    #[test]
    fn every_glyph_the_rail_draws_is_in_the_code_page() {
        // **The check `▸` needed and did not have.** It is U+25B8, is not in CP437,
        // and so drew as `?` — the row saying a room is automated instead read as
        // the orb not knowing what was there. Nothing failed: `is_renderable` is
        // called on authored *prose*, and these are Rust literals in a painter.
        //
        // Listed rather than derived, because a painter's glyphs are exactly the
        // strings on the other side of these constants — adding one and not adding
        // it here is the failure this cannot catch, and a lint that scanned the
        // source would be a different tool.
        let drawn = [RUNNING, '·', '─'];
        for glyph in drawn {
            assert!(
                orbs_render::cp437::is_renderable(glyph),
                "{glyph:?} (U+{:04X}) is not in CP437 and will draw as `?`",
                glyph as u32,
            );
        }
        for mark in [FAULT, NEWS] {
            for glyph in mark.chars() {
                assert!(
                    orbs_render::cp437::is_renderable(glyph),
                    "the mark {mark:?} holds {glyph:?}, which is not in CP437",
                );
            }
        }
    }

    #[test]
    fn a_name_that_does_not_fit_is_cut_rather_than_wrapped() {
        assert_eq!(truncate("laboratory", 14), "laboratory");
        assert_eq!(truncate("laboratory", 5), "labor");
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn the_longest_domain_name_fits_beside_its_mark() {
        // The arithmetic `RAIL_COLS` was chosen against, as an assertion rather
        // than a comment — §19 records that a transcribed measurement rots
        // silently, and this is the measurement the whole width rests on.
        let inside = usize::from(RAIL_COLS) - 2;
        for name in orbs_sim::tower::DOMAINS {
            assert!(
                name.len() + FAULT.chars().count() <= inside,
                "`{name}` plus a mark does not fit {inside} columns",
            );
        }
    }

    #[test]
    fn every_panel_state_has_a_word() {
        // `State` is a closed enum on purpose, so a new one is a compile error
        // here rather than a domain that silently reads `idle`. This asserts the
        // other half: that no word is empty.
        for state in [
            State::Empty,
            State::Charged,
            State::Gathering,
            State::Working,
            State::Scouring,
            State::Ready,
            State::Fouled,
            State::Burning,
            State::Banked,
            State::Cold,
        ] {
            assert!(!word(state).is_empty(), "{state:?} has no word");
        }
    }
}
