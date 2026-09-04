//! The tower's two standings, as bars at the top of the pane (DESIGN.md §11.5).
//!
//! # Why these two and nothing else
//!
//! Experience and renown are the only numbers that are about the **tower over
//! its whole life** rather than about a room, a run or a fight. Everything else
//! on screen is local: the road is the room you are in, the panel is the
//! instruments in front of you, the rail is the seven domains. These two are the
//! game's long arc, and until now the only way to see either was to open the
//! weave or type `status`.
//!
//! # Toward the next tier, not toward the end
//!
//! Both bars fill from the tier *behind* to the tier *ahead*. The weave's own
//! bar measures the whole line to ten thousand, which is the right picture for a
//! screen about the whole line and the wrong one for a glance: at 8 experience
//! it is a sliver that does not move for an hour. Measured tier to tier it fills
//! visibly, and empties exactly once, when one is passed.
//!
//! # Two rows, and they are expensive
//!
//! §9's main window is *"fully rendered and fully functional"* and the road
//! already argued for one row. These take two more, so they yield before the
//! transcript does — [`split`] hands the body back untouched when a pane is
//! short, exactly as `road::split` does and for the same reason.

use orbs_render::{Painter, Pos, Rect, Span, Style};
use orbs_sim::{Ahead, Prose, Toward};

/// The rows the gauges took, and what is left for the rest of the pane.
#[derive(Debug, Clone, Copy)]
pub struct Split {
    /// Up to two rows, or empty when the gauges do not draw.
    pub area: Rect,
    /// The body below them.
    pub rest: Rect,
}

/// Rows the gauges want.
const ROWS: u16 = 2;

/// Rows the body must keep after they take theirs.
///
/// **Higher than the road's, because these cost twice as much.** A transcript
/// squeezed to nothing so two standing bars could draw would be the minimised
/// half winning over the main window, which §9 forbids.
///
/// **The guard subtracts [`ROWS`] and `road::split`'s does not, and that is the
/// difference rather than an inconsistency.** The road takes one row, so a body
/// of `KEEP + 1` leaves exactly `KEEP`; these take two, so the same shape of
/// test let a body of nine draw and kept seven — one under the floor this
/// constant declares, with the road then taking another and leaving six.
const KEEP: u16 = 8;

/// Cells the label takes, gap included: `ley line` is eight and `renown` six.
const LABEL: u16 = 9;

/// The widest a gauge is drawn, however wide the pane.
///
/// **A gauge is a column, not a banner.** Stretched to a 120-cell pane the bar
/// became a solid rule with a number at the far end, and the eye could not tell
/// a third full from a half — the thing a bar is for. htop sizes its bars to a
/// column and puts the reading against them; this is that number. What is left
/// over stays blank, which also keeps the two rows from reading as a wall.
const WIDEST: u16 = 40;

/// The narrowest bar there is: two brackets with one cell between them.
///
/// `Painter::gauge`'s own clip test, named here because this row has to make the
/// same judgement one level up — whether the rank fits is really *whether a bar
/// still fits after it*, and both questions want the same number.
const NARROWEST: u16 = 3;

/// The narrowest pane worth drawing a bracketed gauge in.
///
/// The label, a gauge with room to fill, and a reading wide enough for two
/// five-figure numbers and a slash. Below this the row would be brackets and a
/// truncated number, which says less than nothing.
const LEAST: u16 = 34;

/// Take the top rows for the gauges, if there is room.
#[must_use]
pub const fn split(body: Rect) -> Split {
    if body.rows.saturating_sub(ROWS) < KEEP || body.cols < LEAST {
        return Split {
            area: Rect::EMPTY,
            rest: body,
        };
    }
    Split {
        area: Rect::new(body.col, body.row, body.cols, ROWS),
        rest: Rect::new(
            body.col,
            body.row.saturating_add(ROWS),
            body.cols,
            body.rows.saturating_sub(ROWS),
        ),
    }
}

/// Draw both gauges into their rows.
///
/// `rank` is what the tower is called, drawn after renown's reading when there
/// is room — a title is the point of the second bar and the first thing to drop
/// when there is not. [`row`] is where that dropping happens, and it is measured
/// against whether a bar survives rather than guessed at from the pane's width.
pub fn paint(
    painter: &mut Painter<'_>,
    area: Rect,
    station: Toward,
    rankward: Toward,
    rank: Option<&str>,
    prose: &Prose,
) {
    if area.is_empty() {
        return;
    }
    row(
        painter,
        Rect::new(area.col, area.row, area.cols, 1),
        &prose.line("gauge_ley", &[]),
        station,
        None,
        prose,
    );
    if area.rows < 2 {
        return;
    }
    row(
        painter,
        Rect::new(area.col, area.row.saturating_add(1), area.cols, 1),
        &prose.line("gauge_renown", &[]),
        rankward,
        rank,
        prose,
    );
}

/// One label, one gauge, one reading, and optionally a name after it.
fn row(
    painter: &mut Painter<'_>,
    at: Rect,
    label: &str,
    toward: Toward,
    name: Option<&str>,
    prose: &Prose,
) {
    // **The reading is measured before the gauge is placed**, so the bar takes
    // whatever is left rather than the reading being truncated by it. A number
    // cut in half is unreadable; a bar two cells shorter is not.
    //
    // **Relative on both sides of the slash.** `done` counts from the tier
    // behind, so pairing it with the *absolute* total of the tier ahead would
    // read as two different scales — `8/16` is honest where `8/10000` is not.
    //
    // **An unmeasured track draws no row at all**, rather than guessing. It is
    // the state a `Panel` holds before its first refresh, and the two honest
    // readings are both wrong for it: `0/1` claims a tier is one step away and
    // `nothing more authored` claims the game is over.
    let reading = match toward.ahead {
        Ahead::Tier(_) => prose.line(
            "gauge_toward",
            &[
                ("count", &toward.done.to_string()),
                ("quantity", &toward.span.to_string()),
            ],
        ),
        Ahead::Nothing => prose.line("gauge_topped", &[]),
        Ahead::Unasked => return,
    };
    let start = at.col.saturating_add(LABEL);
    let room = at.right().saturating_sub(start);
    let bar_after = |tail: &str| {
        let width = u16::try_from(tail.chars().count()).unwrap_or(u16::MAX);
        room.saturating_sub(width.saturating_add(1)).min(WIDEST)
    };

    // **The title goes before the bar does**, which is what this function's own
    // doc promised and did not do. The rank was appended unconditionally and
    // then charged to the bar, so the longest titles starved the thing they were
    // annotating — at the endgame `nothing more authored  remembered` is 33
    // cells and left no bar at all in a `LEAST`-wide pane. A bar with no title
    // still says where the tower stands; a title with no bar is a word in a gap.
    let titled = name.map(|name| {
        let named = prose.line(&format!("renown_{name}"), &[]);
        format!("{reading}  {named}")
    });
    let tail = titled
        .filter(|titled| bar_after(titled) >= NARROWEST)
        .unwrap_or(reading);
    let bar = bar_after(&tail);

    // Spoken as a sentence rather than as a row of pipes — §14 names progress
    // bars specifically, and `gauge` takes the sentence for exactly this.
    //
    // **A topped track speaks what it draws.** `Toward::FULL` is `1/1` — a
    // rendering convenience that fills the bar, not a count of anything — so the
    // ordinary sentence told a listener the tower stood *one short of a next
    // tier* on a track whose screen said `nothing more authored`. §14's stream
    // has to carry the same fact the screen does, and those are opposite facts.
    let spoken = if matches!(toward.ahead, Ahead::Nothing) {
        prose.line("gauge_spoken_topped", &[("name", label)])
    } else {
        prose.line(
            "gauge_spoken",
            &[
                ("name", label),
                ("count", &toward.done.to_string()),
                ("quantity", &toward.span.to_string()),
            ],
        )
    };

    // **The gauge is called even when it cannot draw, and that is the point.**
    // `Painter::gauge` pushes its utterance *ahead of its own clip test* so
    // §14's stream does not depend on what happened to fit; returning here on a
    // narrow row would defeat that one level up, and silently — a reader would
    // simply stop being told either standing. So the bar is offered whatever
    // width is left and clips itself, and what a narrow row loses is ink.
    //
    // **No accent, and that is load-bearing.** The fill warms red through
    // yellow to green as it climbs, and it carries that as a *depiction* — the
    // only channel a colour may travel on, since this crate is forbidden to
    // resolve one and a boundary test enforces it. `Style::depicted` drops a
    // picture on any accented cell, so `Role::Success` here would have painted
    // every gauge one flat green and swallowed the ramp.
    painter.gauge(
        Rect::new(start, at.row, bar, 1),
        u32::try_from(toward.done).unwrap_or(u32::MAX),
        u32::try_from(toward.span).unwrap_or(u32::MAX),
        Style::default(),
        &spoken,
    );

    // **Nothing is drawn at all below a drawable bar**, rather than a label with
    // empty space after it. `LEAST` already says a row of brackets and a
    // truncated number "says less than nothing"; a lone dim `ley line` with no
    // reading and no bar says the same, and it drew before this whenever a
    // second pane halved the body or the endgame reading took the width.
    if bar < NARROWEST {
        return;
    }
    painter.span(
        Pos::new(at.col, at.row),
        &Span::new(label).with_style(Style::DIM),
    );
    painter.span(
        Pos::new(start.saturating_add(bar).saturating_add(1), at.row),
        &Span::new(&tail).with_style(Style::DIM),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::{Frame, GridSize};

    #[test]
    fn the_gauges_yield_before_the_transcript_does() {
        let roomy = split(Rect::new(1, 1, 60, 20));
        assert_eq!(roomy.area.rows, ROWS);
        assert_eq!(roomy.rest.rows, 18);
        assert_eq!(roomy.rest.row, 3);

        // A short pane keeps its whole body: §9's main window does not yield.
        let short = split(Rect::new(1, 1, 60, KEEP));
        assert!(short.area.is_empty(), "the gauges squeezed a short pane");
        assert_eq!(short.rest.rows, KEEP);

        // ...and so does a narrow one, where the reading could not be read.
        let narrow = split(Rect::new(1, 1, LEAST - 1, 20));
        assert!(narrow.area.is_empty(), "the gauges drew in a narrow pane");
    }

    /// **`KEEP` means what it says, on the row either side of the edge.**
    ///
    /// The guard was `body.rows <= KEEP` — the shape `road::split` uses, which
    /// is exact there because the road takes one row. These take two, so a body
    /// of `KEEP + 1` passed it and kept `KEEP - 1`, and the road then took
    /// another and kept `KEEP - 2`. The test the fix needed is not that a short
    /// pane bails but that the *first pane the gauges draw in* still leaves the
    /// floor standing.
    #[test]
    fn the_body_keeps_its_floor_on_the_first_row_the_gauges_draw_in() {
        for rows in 0..=KEEP + ROWS - 1 {
            let split = split(Rect::new(1, 1, 60, rows));
            assert!(
                split.area.is_empty(),
                "the gauges drew in a {rows}-row body and left {}, under {KEEP}",
                split.rest.rows,
            );
            assert_eq!(split.rest.rows, rows, "a bailed split kept the body");
        }

        let least = split(Rect::new(1, 1, 60, KEEP + ROWS));
        assert_eq!(least.area.rows, ROWS, "the gauges never draw at all");
        assert_eq!(least.rest.rows, KEEP, "the body kept less than its floor");
    }

    /// **A row too narrow to draw a bar still says what the bar would have.**
    ///
    /// §14's rule, and `Painter::gauge` already keeps it by pushing its
    /// utterance ahead of its own clip test. This row defeated that by returning
    /// before calling it — so a reader lost both standings entirely, and a
    /// sighted player got a dim orphan label with nothing after it. Reachable
    /// today at the endgame reading, where `gauge_topped` plus a rank takes the
    /// width a `LEAST`-wide pane has.
    #[test]
    fn a_row_with_no_room_for_a_bar_still_speaks_its_reading() {
        let prose = Prose::builtin();
        let toward = Toward {
            done: 3,
            span: 16,
            ahead: Ahead::Tier(16),
        };

        for cols in [LEAST, LABEL + 4, LABEL, 0] {
            let mut frame = Frame::new(GridSize::new(80, 8));
            let mut painter = frame.painter(Rect::new(0, 0, 80, 8));
            row(
                &mut painter,
                Rect::new(0, 0, cols, 1),
                &prose.line("gauge_ley", &[]),
                toward,
                // The longest tail there is: the endgame reading and a rank.
                Some("archmage"),
                &prose,
            );
            assert!(
                frame
                    .speech()
                    .utterances()
                    .any(|utterance| utterance.text.contains("3 of 16")),
                "a {cols}-cell row said nothing about where the tower stands",
            );
        }

        // ...and the label does not draw alone when the bar could not.
        let mut frame = Frame::new(GridSize::new(80, 8));
        let mut painter = frame.painter(Rect::new(0, 0, 80, 8));
        row(
            &mut painter,
            Rect::new(0, 0, LABEL, 1),
            &prose.line("gauge_ley", &[]),
            toward,
            None,
            &prose,
        );
        assert!(
            !frame.to_text().contains("ley line"),
            "a bar-less row drew an orphan label:\n{}",
            frame.to_text(),
        );
    }

    /// **A track with nothing left to reach must not speak of a next tier.**
    ///
    /// `Toward::FULL` is `1/1` because that fills the bar, not because anything
    /// is one short of anything. The sentence read those two numbers anyway, so
    /// the screen said `nothing more authored` and a listener was told the tower
    /// stood one step from a tier that does not exist — the two halves of §14's
    /// one stream disagreeing about a fact.
    #[test]
    fn a_topped_gauge_speaks_what_it_draws() {
        let prose = Prose::builtin();
        let topped = Toward {
            done: 1,
            span: 1,
            ahead: Ahead::Nothing,
        };

        let mut frame = Frame::new(GridSize::new(80, 8));
        let mut painter = frame.painter(Rect::new(0, 0, 80, 8));
        row(
            &mut painter,
            Rect::new(0, 0, 80, 1),
            &prose.line("gauge_ley", &[]),
            topped,
            None,
            &prose,
        );

        let said = frame.speech().to_transcript();
        assert!(
            said.contains("nothing more authored"),
            "a topped gauge did not say it had topped: {said}",
        );
        assert!(
            !said.contains("toward the next"),
            "a topped gauge spoke of a tier that is not authored: {said}",
        );
        assert!(
            frame.to_text().contains("nothing more authored"),
            "the row and its speech disagree:\n{}",
            frame.to_text(),
        );
    }

    /// **An unmeasured track says nothing rather than saying it has finished.**
    ///
    /// `Toward::default` is what a `Panel` holds before its first refresh, and
    /// it used to be indistinguishable from a topped track — both were `at:
    /// None` — so the emptiest possible tower drew and spoke as the most
    /// finished one. Every frontend refreshes before it paints, so this was
    /// never on screen; the type simply could not tell the two apart, and one
    /// forgotten refresh was all it would have taken.
    #[test]
    fn an_unmeasured_track_draws_and_says_nothing() {
        let prose = Prose::builtin();

        let mut frame = Frame::new(GridSize::new(80, 8));
        let mut painter = frame.painter(Rect::new(0, 0, 80, 8));
        row(
            &mut painter,
            Rect::new(0, 0, 80, 1),
            &prose.line("gauge_ley", &[]),
            Toward::default(),
            None,
            &prose,
        );

        assert_eq!(
            frame.to_text().trim(),
            "",
            "an unasked gauge drew a reading"
        );
        assert!(
            frame.speech().to_transcript().is_empty(),
            "an unasked gauge spoke: {}",
            frame.speech().to_transcript(),
        );
    }

    /// **The title yields to the bar, which is what the doc always promised.**
    ///
    /// The rank was appended to the reading and the whole tail charged to the
    /// bar, so the longest titles starved it: at the endgame the renown tail is
    /// `nothing more authored  remembered`, 33 cells against the 25 a
    /// `LEAST`-wide pane leaves, and the row drew nothing at all.
    #[test]
    fn a_long_title_yields_before_the_bar_does() {
        let prose = Prose::builtin();
        let topped = Toward {
            done: 1,
            span: 1,
            ahead: Ahead::Nothing,
        };

        let mut frame = Frame::new(GridSize::new(80, 8));
        let mut painter = frame.painter(Rect::new(0, 0, 80, 8));
        row(
            &mut painter,
            Rect::new(0, 0, LEAST, 1),
            &prose.line("gauge_renown", &[]),
            topped,
            Some("remembered"),
            &prose,
        );

        let drawn = frame.to_text();
        assert!(
            drawn.contains('['),
            "the bar was starved by the title:\n{drawn}"
        );
        assert!(
            !drawn.contains("remembered"),
            "the title was kept at the bar's expense:\n{drawn}",
        );

        // ...and where both fit, the title is still drawn.
        let mut roomy = Frame::new(GridSize::new(80, 8));
        let mut painter = roomy.painter(Rect::new(0, 0, 80, 8));
        row(
            &mut painter,
            Rect::new(0, 0, 80, 1),
            &prose.line("gauge_renown", &[]),
            topped,
            Some("remembered"),
            &prose,
        );
        assert!(
            roomy.to_text().contains("remembered"),
            "a wide pane dropped a title that fitted:\n{}",
            roomy.to_text(),
        );
    }
}
