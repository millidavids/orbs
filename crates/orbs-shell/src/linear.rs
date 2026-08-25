//! The session pane, as a screen reader receives it.
//!
//! DESIGN.md §14 makes the linear stream a first-class view of the frame rather
//! than a debugging aid, and `orbs-render` captures one on **every** frame
//! whether or not a reader is attached — deliberately, so the path is exercised
//! by everyone rather than only by the players least able to report it broke.
//!
//! Nothing had ever displayed it. It was asserted by tests and printed by an
//! example, which is how a stream that is subtly wrong stays subtly wrong: the
//! failure mode is not a crash, it is a frame that describes itself *almost*
//! correctly, and only a person reading both can tell.
//!
//! `F5` swaps the session pane for what that pane says. The same rectangle, so
//! the comparison is one keypress rather than a squint, and it works at every
//! window size instead of only above the Deep-focus floor.
//!
//! # The pane is painted twice
//!
//! Reading the live frame's stream does not work, and the reason is worth
//! keeping: the mirror *replaces* the session pane, so by the time it is drawn
//! the frame no longer contains what it is supposed to be describing. The first
//! attempt showed a single empty `in` utterance and nothing else.
//!
//! So the session is painted into a scratch [`Frame`] that is never rasterised,
//! purely to capture its [`Speech`], and the utterances go on screen instead.
//! It costs one extra paint of one pane, only while the view is open, and it
//! buys the thing that matters: what appears is the stream of the pane it
//! replaced, not of the frame it is part of.

use bevy_ecs::prelude::*;
use orbs_render::{Frame, Pos, Rect, Speech, Style, UtteranceKind};
use orbs_sim::Sim;

use super::screen::Screen;

/// Whether the linear view is showing, and the frame it measures with.
#[derive(Resource, Debug, Default)]
pub struct Linear {
    showing: bool,
    /// Painted into and never drawn. Kept between frames so opening the view
    /// does not allocate a grid every time it is up.
    scratch: Frame,
}

impl Linear {
    /// Whether the session pane is currently showing speech instead of cells.
    #[must_use]
    pub const fn showing(&self) -> bool {
        self.showing
    }

    /// Show the linear stream, or stop showing it.
    ///
    /// **A method, not a system**, for the same reason [`Panel::refresh`] and
    /// [`Bench::advance`] are: §14's stream is the accessibility route, and the
    /// terminal build is the one DESIGN.md §14 calls *"the cheapest route to
    /// screen-reader support"* — so it is the last build that should be unable
    /// to reach it.
    ///
    /// [`Panel::refresh`]: crate::Panel::refresh
    /// [`Bench::advance`]: crate::Bench::advance
    pub fn toggle(&mut self) {
        self.showing = !self.showing;
        if !self.showing {
            // Return the grid rather than hold one for a pane nobody has open.
            self.scratch = Frame::default();
        }
        tracing::info!("linear view: {}", self.showing);
    }
}

/// Show or hide the linear view.
pub fn toggle(mut linear: ResMut<Linear>) {
    linear.toggle();
}

/// Draw what the session pane says, in the session pane's place.
pub fn paint(
    linear: &mut Linear,
    frame: &mut Frame,
    sim: &Sim,
    screen: &Screen,
    pane: Rect,
    carry_readings: bool,
    panel: &super::glance::Panel,
    scroll: &super::scrollback::Scroll,
    bench: &super::bench::Bench,
) {
    if pane.is_empty() {
        return;
    }

    // Paint the pane it is replacing, off screen, purely for its speech.
    //
    // With nothing revealing: this mirror exists so a sighted player and a
    // reader can be compared, and a half-arrived line has no speech at all by
    // design. Mirroring mid-reveal would show a stream with holes in it and
    // invite the conclusion that the stream is broken.
    linear.scratch.reset(frame.size());
    super::prompt::session(
        &mut linear.scratch,
        sim,
        screen,
        pane,
        carry_readings,
        &super::reveal::Reveal::default(),
        panel,
        scroll,
        // The real one, threaded rather than defaulted: `Bench::default()` reads
        // the environment, and this runs every frame F5 is up. It changes
        // nothing either way — an animated instrument has no speech, which is
        // the whole of what this mirror is for — so the cheap correct thing is
        // to draw the screen that is actually on screen.
        bench,
    );

    let mut painter = frame.painter(pane);
    // Untitled, then labelled with `glyphs`. `Painter::border` *announces* a
    // title as a heading — correct for every other pane and wrong for this one:
    // a mirror that spoke would put "linear" into the very stream it displays.
    // Caught by a test rather than by reading the code.
    painter.border(pane, None, Style::DIM);
    painter.glyphs(
        Pos::new(pane.col.saturating_add(1), pane.row),
        " linear  F5 back ",
        Style::DIM,
    );

    let body = pane.inset(1);
    let stream: &Speech = linear.scratch.speech();
    // The tail, matching what the session pane does with records — a reader
    // hears the end of a session, not the start of it.
    let skipped = stream.len().saturating_sub(usize::from(body.rows));

    for (row, utterance) in stream
        .utterances()
        .skip(skipped)
        .take(usize::from(body.rows))
        .enumerate()
    {
        let Ok(offset) = u16::try_from(row) else {
            break;
        };
        let at = Pos::new(body.col, body.row.saturating_add(offset));

        // `glyphs` throughout: silent, for the reason above.
        let written = painter.glyphs(at, kind_label(utterance.kind), Style::DIM);
        painter.glyphs(
            Pos::new(at.col.saturating_add(written).saturating_add(1), at.row),
            utterance.text,
            Style::NORMAL.with_role(utterance.role),
        );
    }
}

/// A fixed-width tag, so the text starts in the same column on every row.
const fn kind_label(kind: UtteranceKind) -> &'static str {
    match kind {
        UtteranceKind::Heading => "head",
        UtteranceKind::Text => "text",
        UtteranceKind::TableRow => "row ",
        UtteranceKind::Progress => "prog",
        UtteranceKind::Echo => "echo",
        UtteranceKind::Input => "in  ",
        UtteranceKind::Hint => "hint",
        UtteranceKind::Guide => "aide",
        UtteranceKind::Completion => "done",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbs_render::GridSize;

    fn session_with(lines: &[&str]) -> (Sim, Screen, Frame, Rect) {
        let mut sim = Sim::new(1);
        for line in lines {
            sim.submit(line);
            sim.step();
        }
        let grid = GridSize::new(80, 22);
        let frame = Frame::new(grid);
        let screen = Screen::default();
        (sim, screen, frame, Rect::new(0, 0, 80, 20))
    }

    #[test]
    fn every_tag_is_the_same_width() {
        // The text column must not jitter row to row, or the pane reads as
        // ragged noise rather than as a transcript.
        let widths: Vec<usize> = [
            UtteranceKind::Heading,
            UtteranceKind::Text,
            UtteranceKind::TableRow,
            UtteranceKind::Progress,
            UtteranceKind::Echo,
            UtteranceKind::Input,
            UtteranceKind::Completion,
        ]
        .into_iter()
        .map(|kind| kind_label(kind).chars().count())
        .collect();
        assert!(
            widths.iter().all(|width| *width == widths[0]),
            "ragged tags: {widths:?}",
        );
    }

    #[test]
    fn it_shows_what_the_pane_it_replaced_says() {
        // The defect this shape exists for: reading the live frame's stream
        // showed one empty `in` utterance, because the mirror had already
        // replaced the pane it was supposed to be describing.
        let (sim, screen, mut frame, pane) = session_with(&["look around", "xyzzy"]);
        let mut linear = Linear::default();

        paint(
            &mut linear,
            &mut frame,
            &sim,
            &screen,
            pane,
            true,
            &crate::Panel::default(),
            &crate::Scroll::default(),
            &crate::Bench::default(),
        );

        let drawn = frame.to_text();
        assert!(drawn.contains("look around"), "{drawn}");
        assert!(drawn.contains("survey"), "{drawn}");
        assert!(drawn.contains("xyzzy"), "{drawn}");
        assert!(drawn.contains("echo"), "utterance kinds are not tagged");
    }

    #[test]
    fn the_mirror_never_speaks() {
        // Anything it said would land in the stream it is displaying.
        let (sim, screen, mut frame, pane) = session_with(&["look around"]);
        let mut linear = Linear::default();

        paint(
            &mut linear,
            &mut frame,
            &sim,
            &screen,
            pane,
            true,
            &crate::Panel::default(),
            &crate::Scroll::default(),
            &crate::Bench::default(),
        );

        assert!(
            frame.speech().is_empty(),
            "the mirror spoke: {:?}",
            frame.speech().to_transcript(),
        );
    }

    #[test]
    fn closing_it_gives_the_grid_back() {
        let mut linear = Linear {
            showing: true,
            ..Default::default()
        };
        linear.scratch.reset(GridSize::new(160, 45));

        // A bare `World` rather than an `App`: this crate depends on `bevy_ecs`
        // and not on the engine, and running one system needs nothing more.
        let mut world = World::new();
        world.insert_resource(linear);
        world.run_system_cached(toggle).expect("toggle");

        let linear = world.resource::<Linear>();
        assert!(!linear.showing);
        assert_eq!(linear.scratch.size(), GridSize::default());
    }
}
