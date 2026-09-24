//! The session pane, as a screen reader receives it.
//!
//! DESIGN.md §14 makes the linear stream a first-class view of the frame, and
//! `orbs-render` captures one every frame whether or not a reader is attached,
//! so the path is exercised by everyone rather than only by the players least
//! able to report it broke.
//!
//! Nothing had ever displayed it: a stream that is subtly wrong does not crash,
//! it describes the frame *almost* correctly, and only a person reading both
//! can tell. `F5` swaps the session pane for what that pane says, in the same
//! rectangle and at every window size.
//!
//! The pane is painted twice. The mirror *replaces* the session pane, so
//! reading the live frame's stream describes a frame that no longer holds what
//! it is describing — the first attempt showed one empty `in` utterance. So the
//! session is painted into a scratch [`Frame`] that is never rasterised, purely
//! to capture its [`Speech`], and those utterances go on screen.

use bevy_ecs::prelude::*;
use orbs_render::{Frame, Pos, Rect, Speech, Style, UtteranceKind};
use orbs_sim::Sim;

use super::screen::Screen;

/// Whether the linear view is showing, and the frame it measures with.
#[derive(Resource, Debug)]
pub struct Linear {
    showing: bool,
    /// Painted into and never drawn. Kept between frames so opening the view
    /// does not allocate a grid every time it is up.
    scratch: Frame,
}

impl Default for Linear {
    /// Showing if the player left it showing: §14's accessibility route is a
    /// worse one if it has to be found again every launch.
    fn default() -> Self {
        Self {
            showing: crate::settings::get(LINEAR).is_some_and(|word| crate::settings::is_on(&word)),
            scratch: Frame::default(),
        }
    }
}

impl Linear {
    /// Whether the session pane is currently showing speech instead of cells.
    #[must_use]
    pub const fn showing(&self) -> bool {
        self.showing
    }

    /// Show the linear stream, or stop showing it.
    ///
    /// A method, not a system, for the same reason [`Panel::refresh`] and
    /// [`Bench::advance`] are: the terminal build is §14's *"cheapest route to
    /// screen-reader support"* and must be able to reach it.
    ///
    /// [`Panel::refresh`]: crate::Panel::refresh
    /// [`Bench::advance`]: crate::Bench::advance
    pub fn toggle(&mut self) {
        self.show(!self.showing);
    }

    /// Show the linear stream, or not, whichever `showing` says.
    ///
    /// The half a settings page needs: `F5` flips, the page names the state it
    /// wants. Flipping until it agreed does the wrong thing when something else
    /// changed the state in between.
    pub fn show(&mut self, showing: bool) {
        self.showing = showing;
        if !self.showing {
            // Return the grid rather than hold one for a pane nobody has open.
            self.scratch = Frame::default();
        }
        tracing::info!("linear view: {}", self.showing);
        // One setting seen twice, and the shell's to remember because `F5` is
        // bound in both builds. §14 makes this the accessibility route.
        crate::settings::set(LINEAR, crate::settings::switched(self.showing));
    }
}

/// What the linear stream is called in the settings file.
///
/// Here rather than in a frontend because both builds bind `F5` and must write
/// the same key — a second spelling is a setting that half persists.
pub const LINEAR: &str = "linear";

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
    // With nothing revealing: a half-arrived line has no speech by design, so
    // mirroring mid-reveal would show a stream with holes in it.
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
        // The real one, not a default: `Bench::default()` reads the
        // environment, and this runs every frame F5 is up.
        bench,
    );

    let mut painter = frame.painter(pane);
    // Untitled, then labelled with `glyphs`. `Painter::border` announces a
    // title as a heading, and a mirror that spoke would put "linear" into the
    // stream it displays.
    painter.border(pane, None, Style::DIM);
    painter.glyphs(
        Pos::new(pane.col.saturating_add(1), pane.row),
        " linear  F5 back ",
        Style::DIM,
    );

    let body = pane.inset(1);
    let stream: &Speech = linear.scratch.speech();
    // The tail, as the session pane does: a reader hears the end of a session.
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
        // The text column must not jitter row to row.
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
        // Reading the live frame's stream showed one empty `in` utterance: the
        // mirror had already replaced the pane it was describing.
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

        // A bare `World`, not an `App`: this crate depends on `bevy_ecs` only.
        let mut world = World::new();
        world.insert_resource(linear);
        world.run_system_cached(toggle).expect("toggle");

        let linear = world.resource::<Linear>();
        assert!(!linear.showing);
        assert_eq!(linear.scratch.size(), GridSize::default());
    }
}
