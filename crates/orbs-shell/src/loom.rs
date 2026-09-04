//! Drawing the weave screen (DESIGN.md §11.5, §19).
//!
//! # Why here rather than in `orbs-render`
//!
//! The same answer `sheet.rs` gives: `ScreenLayout` hands this function a
//! rectangle and it draws inside it. What appears is the sim's decision — the
//! tracks, the states and the totals all arrive through `Tapestry` — and only
//! *where in the pane* is decided here.
//!
//! # The shape: a bar, two headings, and one track at a time
//!
//! **Progression runs rightward**, and the screen says so on both tracks: the
//! experience bar fills to the right, the Ley Line runs right, and each room's
//! line runs right. A vertical list said none of it — it drew what you had as a
//! set of rows, and a player reading it could not see that the thing was a
//! *track* at all. That was the first version and it was replaced for that.
//!
//! **One track draws below the headings at a time**, and the word the player
//! typed decides which. Seven rooms' lines and a forked Ley Line do not both fit
//! in eighteen rows beside a details panel, and a screen that showed one of
//! them at half size would say the wrong thing about both. Both headings are
//! always drawn, so a player who has not asked for the other track can see it is
//! there.
//!
//! On the Ley Line a fork's nodes stack **downward**, which is the other axis
//! and the other meaning: rightward is progress, downward is a choice. On
//! Mastery there are no choices; downward is the next room.
//!
//! **The bar and the Ley Line share one scale** — the last station's total, so a
//! node's position *is* its cost, read against the same cells the bar fills.
//! §19 records that this was a fixed hundred and would become derived when the
//! curve reached it.
//!
//! # The names live in the details panel, not on the nodes
//!
//! The session pane is 48 columns in Deep focus. Nothing survives putting a
//! sentence beside every node at that width, so a node is a glyph, and what it
//! *is* goes in a boxed panel in the bottom right, for the one thing you are
//! aimed at.

use orbs_render::{Frame, Intensity, Painter, Pos, Rect, Role, Span, Style, UtteranceKind};
use orbs_sim::{Line, Node, Prose, Stop, Walk};

use super::Tapestry;
use super::stations;
use super::tapestry::{Aimed, Complaint, Mode, Track};

/// Cells a room's name takes on the mastery view, gap included.
///
/// `laboratory` is ten and `menagerie` nine; eleven leaves one cell before the
/// line begins.
const NAME: u16 = 11;

/// Cells between stations on a mastery line.
///
/// A station is three cells wide — frame, glyph, frame — so four keeps one run
/// cell between neighbours. Six stations is twenty-four cells, which with the
/// name fits the 46 a 48-column pane leaves inside its border.
const STRIDE: u16 = 4;

/// The details panel's size: a border, a sentence, a cost, and the two states.
///
/// **Bounded rather than proportional.** It holds three short authored lines,
/// and a panel that grew with the pane would leave three quarters of itself
/// blank on a wide window while the sentences stayed the same length.
const PANEL_ROWS: u16 = 5;
const PANEL_COLS: u16 = 30;

/// The smallest pane this can honestly be drawn in.
///
/// Two borders, the bar, the headings, the rooms' lines, a blank, the details
/// panel and the words. Below it the answer is to say so rather than draw
/// something misleading, exactly as the editor does. §4's declared floor is
/// 80×22, so this fits with room to spare.
///
/// **Eighteen was exactly right at seven rooms and is one spare at six** (§19).
/// Left where it is: a floor that tightened every time a room left would be a
/// screen that appeared and vanished as content moved, and the spare row is the
/// blank the details panel already sits under.
const MIN_ROWS: u16 = 18;
const MIN_COLS: u16 = 24;

/// Draw the weave screen into `pane`.
pub fn paint(frame: &mut Frame, screen: &Tapestry, pane: Rect, prose: &Prose) {
    if pane.is_empty() {
        return;
    }
    let mut painter = frame.painter(pane);
    let area = painter.area();

    // **Too small says so.** Returning silently left a wholly blank pane while
    // this screen still owned the keyboard — and `Tapestry::escape` deliberately
    // does not close, so the only way out was typing `quit` at nothing.
    //
    // **The width the Ley Line needs is content, not a constant.** Sixteen
    // stations cannot be drawn in a narrow pane however tightly they are packed,
    // and the honest answer there is this message rather than a line with its
    // tail clipped off — see `pack`.
    let narrow =
        track_width(screen, area.cols.saturating_sub(2)) < track_needs(screen.ley_line().len());
    if area.rows < MIN_ROWS || area.cols < MIN_COLS || narrow {
        painter.border(area, Some(&prose.line("weave_title", &[])), Style::DIM);
        let inside = area.inset(1);
        if !inside.is_empty() {
            painter.paragraph(inside, &Span::new(&prose.line("weave_too_small", &[])));
        }
        return;
    }
    painter.border(area, Some(&prose.line("weave_title", &[])), Style::DIM);

    let left = area.col.saturating_add(1);
    let inner = area.cols.saturating_sub(2);
    let mut y = area.row.saturating_add(1);

    y = bar(&mut painter, screen, left, y, inner, prose);
    y = headings(&mut painter, screen, left, y, inner, prose);
    match screen.track() {
        Track::LeyLine => line_track(&mut painter, screen, left, y, inner),
        Track::Mastery => room_lines(&mut painter, screen, left, y, inner),
    }

    details(&mut painter, screen, area, prose);
    status(&mut painter, screen, area, prose);
}

/// The experience bar, filling rightward against the line's scale.
///
/// Returns the next free row. The Ley Line beneath it places its stations
/// against the same cells — a node's position *is* its cost, which is only true
/// if the two rows share a scale.
fn bar(
    painter: &mut Painter<'_>,
    screen: &Tapestry,
    left: u16,
    y: u16,
    inner: u16,
    prose: &Prose,
) -> u16 {
    let earned = screen.experience();
    let scale = screen.scale().max(1);
    let label = format!("{earned} of {scale}");
    let label_width = u16::try_from(label.chars().count()).unwrap_or(9);
    let width = inner.saturating_sub(label_width.saturating_add(1));

    // Spoken as a sentence rather than as a row of blocks — §14 names progress
    // bars specifically, and `progress` takes the sentence for exactly this.
    let spoken = prose.line(
        "weave_bar",
        &[
            ("quantity", &earned.to_string()),
            ("detail", &scale.to_string()),
        ],
    );
    // **Filled to the cell the total stands at on the line below**, so the
    // fill's edge and the stations read against one scale — which since the
    // line runs to ten thousand is the logarithmic one `along` draws; see
    // there. The label carries the plain numbers.
    let filled = if earned >= scale {
        width
    } else {
        along(width, scale, earned).saturating_add(1)
    };
    painter.progress(
        Rect::new(left, y, width, 1),
        u32::from(filled),
        u32::from(width.max(1)),
        Style::default().with_role(Role::Success),
        &spoken,
    );
    painter.span(
        Pos::new(left.saturating_add(width).saturating_add(1), y),
        &Span::new(&label).with_style(Style::default().with_intensity(Intensity::Bright)),
    );
    y.saturating_add(1)
}

/// The two track names on one row, the one being walked drawn bright.
///
/// Both always, so a player who has only ever typed `ley` can see there is a
/// second track to ask for.
fn headings(
    painter: &mut Painter<'_>,
    screen: &Tapestry,
    left: u16,
    y: u16,
    inner: u16,
    prose: &Prose,
) -> u16 {
    let looking = screen.mode() == Mode::Browsing;
    let ley = prose.line("weave_ley", &[]);
    let mastery = prose.line("weave_mastery", &[]);
    let style = |track: Track| {
        if screen.track() == track && looking {
            Style::default().with_intensity(Intensity::Bright)
        } else {
            Style::DIM
        }
    };
    painter.span(
        Pos::new(left, y),
        &Span::new(&ley).with_style(style(Track::LeyLine)),
    );
    let after = left
        .saturating_add(u16::try_from(ley.chars().count()).unwrap_or(8))
        .saturating_add(3);
    painter.span(
        Pos::new(after, y),
        &Span::new(&mastery).with_style(style(Track::Mastery)),
    );

    // **Renown shares the headings row rather than taking one of its own.** The
    // pane is eighteen rows at its floor and all but one are spoken for — bar,
    // headings, the rooms' lines, the panel and the words. The two track names take
    // under twenty cells, so the tower's other number sits at the right edge of
    // the same row, where the bar's own total sits one row up.
    //
    // Drawn only when there is room for it whole: a truncated number is worse
    // than none, because a reader cannot tell 1,240 cut short from 12.
    let renown = prose.line("weave_renown", &[("count", &screen.renown().to_string())]);
    let width = u16::try_from(renown.chars().count()).unwrap_or(0);
    let taken = after
        .saturating_add(u16::try_from(mastery.chars().count()).unwrap_or(7))
        .saturating_sub(left);
    if width > 0 && taken.saturating_add(width).saturating_add(2) <= inner {
        painter.span(
            Pos::new(left.saturating_add(inner).saturating_sub(width), y),
            &Span::new(&renown).with_style(Style::DIM),
        );
    }
    y.saturating_add(1)
}

/// The Ley Line: one unbroken line with its stations standing on it, and a
/// fork's siblings hanging below their station.
///
/// **A line, because that is what a ley line is.** Drawing it as a row of
/// separate glyphs said "here are some things"; drawing it as a line with
/// stations on it says "here is a road, and these are the places along it".
///
/// **Stations sit at their cost**, as near as the cells allow. The run spans the
/// same cells the bar above does, so a station stands where the fill will reach
/// it — on the logarithmic scale [`along`] draws, since the line runs to ten
/// thousand and its first five stations are all inside the first hundred. Where
/// cost and cells disagree the cells win and [`pack`] says how, because a
/// station drawn slightly wrong is a picture and a station not drawn is a lie.
///
/// **The totals alternate between two rows**, because a five-figure label is
/// wider than the three cells a station keeps between itself and the next, and
/// one row of them ran together into a number nobody authored.
fn line_track(painter: &mut Painter<'_>, screen: &Tapestry, left: u16, y: u16, inner: u16) {
    let width = track_width(screen, inner);
    stations::run(painter, Pos::new(left, y), width);

    let track = screen.ley_line();
    let totals: Vec<u64> = track.iter().map(|station| station.at).collect();
    let packed = pack(left, width, screen.scale(), &totals);
    let depth = track
        .iter()
        .map(|station| station.nodes.len())
        .max()
        .unwrap_or(1);

    // **Every line first, then every node**, so a frame can overwrite the cell
    // of a run beside it rather than being overwritten by one drawn later. A
    // fork's siblings stack under its station; the first sits on the run.
    for (index, (station, x)) in track.iter().zip(&packed.at).enumerate() {
        for (row, node) in station.nodes.iter().enumerate() {
            let row = y.saturating_add(u16::try_from(row).unwrap_or(0));
            glyph(painter, screen, *x, row, node, packed.framed);
        }
        let stagger = u16::try_from(index % 2).unwrap_or(0);
        painter.span(
            Pos::new(
                x.saturating_sub(1),
                y.saturating_add(u16::try_from(depth).unwrap_or(1))
                    .saturating_add(stagger),
            ),
            &Span::new(&station.at.to_string()).with_style(Style::DIM),
        );
    }
}

/// How wide the Ley Line's run is: exactly the cells the bar above fills.
///
/// **One number, read by both rows.** The bar and the line are the same scale
/// drawn two ways, and they only are if they measure the same span — so the
/// label's width is subtracted here rather than in each of them.
fn track_width(screen: &Tapestry, inner: u16) -> u16 {
    let label = format!("{} of {}", screen.experience(), screen.scale().max(1));
    inner.saturating_sub(u16::try_from(label.chars().count()).unwrap_or(9) + 1)
}

/// Cells a framed station occupies: frame, mark, frame.
const GAP: u16 = 3;

/// Cells a station occupies when the line is too long to frame every one: the
/// mark, and one run cell before the next.
const TIGHT: u16 = 2;

/// Where a run of stations stands, and whether there was room to frame them.
#[derive(Debug)]
struct Packed {
    /// One column per station, left to right, all inside the run.
    at: Vec<u16>,
    /// Whether a station is drawn `[·]` rather than as a bare mark.
    framed: bool,
}

/// Where a run of stations stands: never closer than they can be drawn, and
/// never past the end of the track.
///
/// **Cost decides the position; the pane decides the rest.** Two totals a few
/// experience apart land on the same cell of a 36-wide track, and a framed node
/// writes at `x-1` and `x+1` *around* its glyph — so the second one's `[` landed
/// on the first one's mark and erased it. `Progression::check` cannot prevent
/// that: how many cells a hundred experience spans is a fact about a pane, and
/// `boundaries.rs` keeps panes out of the sim entirely.
///
/// So the painter guarantees what the content cannot, in three moves:
///
/// - **Frames come off before positions go wrong.** Sixteen stations at three
///   cells each want 48. The bare mark needs two, and a road of `·` marks still
///   reads as a road.
/// - **A forward pass pushes right**, so consecutive stations never share a
///   cell.
/// - **A backward pass pulls left**, so the tail cannot march off the end. It is
///   the move that was missing: the pushing pass alone put the last stations
///   outside the painter's area, where they were clipped away silently while
///   `announce` still spoke them and the cursor still walked onto them. Being
///   slightly wrong about position is a picture; a missing node is a lie.
///
/// **The narrow pane is Phase 11a's, not today's.** `PANES` is 1, and every grid
/// the game accepts leaves the one pane at least sixty columns, where the framed
/// chain fits. A second pane halves the main window to 52 and leaves this 39,
/// which is where the frames come off — and where, before the backward pass, the
/// last stations went missing.
///
/// `paint` refuses a pane too narrow for even the tight packing ([`track_needs`]),
/// so the two passes never have to overlap two stations to satisfy each other.
fn pack(left: u16, width: u16, scale: u64, totals: &[u64]) -> Packed {
    // A station may write a frame at `x-1` and `x+1`, so the cells it can stand
    // on are one inside each end of the run.
    let first = left.saturating_add(1);
    let last = left.saturating_add(width.saturating_sub(2)).max(first);
    let steps = u16::try_from(totals.len().saturating_sub(1)).unwrap_or(u16::MAX);
    let framed = u32::from(steps) * u32::from(GAP) <= u32::from(last - first);
    let gap = if framed { GAP } else { TIGHT };

    let mut at: Vec<u16> = Vec::with_capacity(totals.len());
    for total in totals {
        let want = first.saturating_add(along(width, scale, *total));
        let floor = at
            .last()
            .map_or(want, |placed| placed.saturating_add(gap).max(want));
        at.push(floor);
    }
    for index in (0..at.len()).rev() {
        let behind = u16::try_from(at.len() - 1 - index).unwrap_or(u16::MAX);
        let ceiling = last.saturating_sub(behind.saturating_mul(gap));
        at[index] = at[index].min(ceiling).max(first);
    }
    Packed { at, framed }
}

/// The cells the Ley Line needs before it can be drawn honestly at all.
///
/// A station at each end of the run and the tightest gap between them. Below
/// this the screen says the window is too small rather than drawing a line with
/// stations missing from it — the answer it already gives for a pane too short,
/// and the same reason.
///
/// **Derived from the content**, so a line that grows past what a pane can hold
/// is caught by the picture refusing rather than by nobody noticing.
fn track_needs(count: usize) -> u16 {
    u16::try_from(count.saturating_sub(1))
        .unwrap_or(u16::MAX)
        .saturating_mul(TIGHT)
        .saturating_add(GAP)
}

/// How many cells along a line `width` cells wide a total of `at` stands, on
/// a **logarithmic** scale to `scale`.
///
/// **Position is still cost, read the way the curve grows.** The line's
/// stations grow by about half each — 16, 24, 40, 56, 96 … 10000 — so on a
/// linear scale the first five stand inside the first cell and the picture is
/// a knot at the left and a road with nothing on it. Equal cells for equal
/// *ratios* spreads them as the curve spreads them: a station is still further
/// right than a cheaper one, and the bar above fills to exactly the cell the
/// total has reached. §19 said the fixed hundred would become derived once the
/// curve reached it; what it became is this.
///
/// A float, in a painter — the sim is integer throughout and this crate is not
/// the sim. The last three cells are kept for the far end's frame.
fn along(width: u16, scale: u64, at: u64) -> u16 {
    let span = f64::from(width.saturating_sub(3));
    let scale = scale.max(2);
    // `ln(1 + x)` rather than `ln(x)`, so nought is nought rather than
    // negative infinity and the first station is still visibly along the line.
    #[allow(clippy::cast_precision_loss)]
    let fraction = ((at.min(scale) + 1) as f64).ln() / ((scale + 1) as f64).ln();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cells = (fraction.clamp(0.0, 1.0) * span).round() as u16;
    cells
}

/// The rooms' lines, one row each: the name, then a run with its
/// stations standing on it.
///
/// **Evenly spaced, not at cost.** A mastery station has no total — its deed is
/// a count of a different thing on every line — so position cannot mean cost
/// here, and pretending it did would put "five potions" and "five walks" at the
/// same place for no reason. What position says on this track is *order*.
///
/// A room that is not open draws as the rail draws it: a dim dotted run and no
/// name, so the row says *there is more* without saying what.
fn room_lines(painter: &mut Painter<'_>, screen: &Tapestry, left: u16, y: u16, inner: u16) {
    let start = left.saturating_add(NAME);
    let width = inner.saturating_sub(NAME);
    for (row, line) in screen.mastery().iter().enumerate() {
        let row = y.saturating_add(u16::try_from(row).unwrap_or(0));
        if !line.open {
            painter.glyphs(
                Pos::new(left, row),
                &stations::LATER.to_string().repeat(usize::from(inner)),
                Style::DIM,
            );
            continue;
        }
        let here = line
            .stops
            .iter()
            .any(|stop| screen.cursor() == Some(stop.mark()));
        painter.span(
            Pos::new(left, row),
            &Span::new(line.domain).with_style(if here {
                Style::default().with_intensity(Intensity::Bright)
            } else {
                Style::DIM
            }),
        );
        stations::run(painter, Pos::new(start, row), width);
        for (index, stop) in line.stops.iter().enumerate() {
            let x = start
                .saturating_add(1)
                .saturating_add(STRIDE.saturating_mul(u16::try_from(index).unwrap_or(0)));
            if x.saturating_add(1) >= start.saturating_add(width) {
                break;
            }
            station(painter, screen, x, row, line, stop);
        }
    }
}

/// One node of the Ley Line, and the cursor if it is on this one.
///
/// `framed` is off when the line is packed too tightly to give every station a
/// frame of its own — see [`pack`], and the pane that does it is a phase away.
/// The aimed one keeps its frame either way, because that frame is the only
/// thing carrying "aimed" without colour (§14), and at the tight gap it lands on
/// run cells rather than on a neighbour.
fn glyph(painter: &mut Painter<'_>, screen: &Tapestry, x: u16, y: u16, node: &Node, framed: bool) {
    let (mark, style) = stations::standing_mark(node.standing);
    let here = screen.cursor().is_some_and(|aimed| aimed == node.mark());
    if framed {
        stations::framed(painter, x, y, mark, style, here);
    } else {
        stations::bare(painter, x, y, mark, style, here);
    }
    // **Drawn silently, then said properly.** `Painter::span` would push the
    // glyph's own text into the speech stream, so a reader would hear "`○`" and
    // be told nothing — §14 forbids meaning carried by a mark. The `announce` is
    // the row a listener actually needs: the total and the state, as words.
    painter.announce(
        UtteranceKind::TableRow,
        Role::Normal,
        &format!("{}: {}", node.at, node.standing.word()),
    );
}

/// One station of a room's line, and the cursor if it is on this one.
fn station(painter: &mut Painter<'_>, screen: &Tapestry, x: u16, y: u16, line: &Line, stop: &Stop) {
    let (mark, style) = stations::walk_mark(stop.walk);
    let here = screen.cursor() == Some(stop.mark());
    stations::framed(painter, x, y, mark, style, here);
    painter.announce(
        UtteranceKind::TableRow,
        Role::Normal,
        &format!(
            "{} {}: {}, {} of {}",
            line.domain,
            stop.id.rsplit('_').next().unwrap_or_default(),
            stop.walk.word(),
            stop.done,
            stop.needed,
        ),
    );
}

/// The details panel: what the cursor is on, and the facts about it.
///
/// **The whole reason the nodes are bare glyphs.** At 48 columns a sentence
/// cannot sit beside every node, and abbreviating them all would make the screen
/// a puzzle. One panel, for the one thing you are pointed at.
///
/// **Bottom right, and boxed.** It sits under the tracks rather than beside
/// them, because a track runs the full width of the pane and anything alongside
/// one would be sharing cells with the road.
///
/// # A node says two things, and they are not the same thing
///
/// **Unlocked** is whether the tower has earned enough to reach it. **Active** is
/// whether what it grants is in effect. A step is both together — passing one
/// *is* taking it — but a fork node can be unlocked and idle (nobody has chosen
/// it) or unlocked and idle for ever (a sibling took the fork's one choice).
///
/// # A station says how far, and what it opens
///
/// Its deed as a sentence, `3 of 5` toward it, and the room, recipe or charm on
/// the far side of it — which is what makes a line worth walking.
fn details(painter: &mut Painter<'_>, screen: &Tapestry, area: Rect, prose: &Prose) {
    let width = area.cols.saturating_sub(2).min(PANEL_COLS);
    let col = area
        .col
        .saturating_add(area.cols)
        .saturating_sub(width)
        .saturating_sub(1);
    let row = area
        .row
        .saturating_add(area.rows)
        .saturating_sub(PANEL_ROWS)
        .saturating_sub(2);
    let box_area = Rect::new(col, row, width, PANEL_ROWS);
    painter.border(
        box_area,
        Some(&prose.line("weave_details", &[])),
        Style::DIM,
    );

    let text_col = col.saturating_add(1);
    let room = u32::from(width.saturating_sub(2));
    let right = |text: &str| {
        col.saturating_add(width)
            .saturating_sub(u16::try_from(text.chars().count()).unwrap_or(8))
            .saturating_sub(1)
            .max(text_col)
    };

    match screen.aimed() {
        None => {
            // Not blank: §6 forbids a dead end, and a panel that is sometimes
            // full and sometimes empty reads as a screen that has broken.
            painter.span(
                Pos::new(text_col, row.saturating_add(1)),
                &Span::new(orbs_render::arriving(
                    &prose.line("weave_aim_first", &[]),
                    room,
                ))
                .with_style(Style::DIM),
            );
        }
        Some(Aimed::Node(node)) => {
            painter.span(
                Pos::new(text_col, row.saturating_add(1)),
                &Span::new(orbs_render::arriving(
                    &prose.line(&format!("weave_node_{}", node.id), &[]),
                    room,
                ))
                .with_style(Style::default().with_intensity(Intensity::Bright)),
            );
            // What it costs, so a locked node says how much more rather than
            // only that it is shut — and which lane it is, for a fork.
            painter.span(
                Pos::new(text_col, row.saturating_add(2)),
                &Span::new(orbs_render::arriving(
                    &prose.line("weave_at", &[("count", &node.at.to_string())]),
                    room,
                ))
                .with_style(Style::DIM),
            );
            if let Some(lane) = node.lane {
                let word = prose.line(&format!("weave_lane_{}", lane.word()), &[]);
                painter.span(
                    Pos::new(right(&word), row.saturating_add(2)),
                    &Span::new(&word).with_style(Style::DIM),
                );
            }

            let reach = prose.line(
                if node.unlocked {
                    "weave_unlocked"
                } else {
                    "weave_locked_state"
                },
                &[],
            );
            let live = prose.line(
                if node.standing.active() {
                    "weave_active"
                } else {
                    "weave_inactive"
                },
                &[],
            );
            let states = row.saturating_add(3);
            painter.span(
                Pos::new(text_col, states),
                &Span::new(&reach).with_style(if node.unlocked {
                    Style::default().with_role(Role::Success)
                } else {
                    Style::DIM
                }),
            );
            // Pinned right inside the box, so the two facts read as a pair of
            // columns rather than as one run-on phrase.
            painter.span(
                Pos::new(right(&live), states),
                &Span::new(&live).with_style(if node.standing.active() {
                    Style::default().with_role(Role::Success)
                } else {
                    Style::DIM
                }),
            );
        }
        Some(Aimed::Stop { stop, .. }) => {
            painter.span(
                Pos::new(text_col, row.saturating_add(1)),
                &Span::new(orbs_render::arriving(
                    &prose.line(&format!("mastery_{}", stop.id), &[]),
                    room,
                ))
                .with_style(Style::default().with_intensity(Intensity::Bright)),
            );
            painter.span(
                Pos::new(text_col, row.saturating_add(2)),
                &Span::new(orbs_render::arriving(
                    &prose.line(
                        "weave_progress",
                        &[
                            ("count", &stop.done.to_string()),
                            ("quantity", &stop.needed.to_string()),
                        ],
                    ),
                    room,
                ))
                .with_style(Style::DIM),
            );
            let walk = prose.line(&format!("weave_walk_{}", stop.walk.word()), &[]);
            let states = row.saturating_add(3);
            painter.span(
                Pos::new(text_col, states),
                &Span::new(&walk).with_style(if stop.walk == Walk::Reached {
                    Style::default().with_role(Role::Success)
                } else {
                    Style::DIM
                }),
            );
            if let Some(key) = stop.opens.first() {
                let name = key.rsplit(':').next().unwrap_or(key);
                let opens = prose.line("weave_opens", &[("name", name)]);
                painter.span(
                    Pos::new(right(&opens), states),
                    &Span::new(&opens).with_style(Style::DIM),
                );
            }
        }
    }
}

/// The bottom row: the words, or what the last one was refused for.
///
/// **It always says something.** §6 forbids a dead end, and at the command line
/// this row is the whole interface — a player who does not know the words has
/// nowhere else to look.
fn status(painter: &mut Painter<'_>, screen: &Tapestry, area: Rect, prose: &Prose) {
    let y = area.row.saturating_add(area.rows).saturating_sub(2);
    let width = u32::from(area.cols.saturating_sub(2));

    // A half-typed word outranks everything — you should always be able to see
    // what you are typing — then a complaint the player just caused, then help.
    let (text, style) = match (screen.mode(), screen.complaint()) {
        (Mode::Command, _) if !screen.command().is_empty() => {
            (format!("> {}", screen.command()), Style::default())
        }
        (_, Some(complaint)) => (
            refusal(complaint, prose),
            Style::default().with_role(Role::Danger),
        ),
        (Mode::Command, None) => (prose.line("weave_words", &[]), Style::DIM),
        (Mode::Browsing, None) => (
            prose.line(
                match screen.track() {
                    Track::LeyLine => "weave_browsing",
                    Track::Mastery => "weave_browsing_mastery",
                },
                &[],
            ),
            Style::DIM,
        ),
    };

    painter.span(
        Pos::new(area.col.saturating_add(1), y),
        &Span::new(orbs_render::arriving(&text, width)).with_style(style),
    );
}

/// The authored sentence for a refusal.
fn refusal(complaint: &Complaint, prose: &Prose) -> String {
    match complaint {
        Complaint::Unknown(word) => prose.line("weave_unknown", &[("name", word)]),
        Complaint::Nothing => prose.line("weave_nothing_here", &[]),
        Complaint::Already(id) => prose.line("weave_already", &[("name", id)]),
        Complaint::NotAChoice(id) => prose.line("weave_not_a_choice", &[("name", id)]),
        Complaint::Locked(id, at) => {
            prose.line("weave_locked", &[("name", id), ("count", &at.to_string())])
        }
        Complaint::Spent(id) => prose.line("weave_spent", &[("name", id)]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped curve, to the soft ending.
    ///
    /// Written out rather than read from `Progression`, because what these
    /// properties are about is *sixteen of anything* in a pane that narrow —
    /// and a test that moved with the content would stop testing the geometry
    /// the day someone shortened the line.
    const CURVE: [u64; 16] = [
        16, 24, 40, 56, 96, 160, 256, 400, 640, 1000, 1600, 2500, 4000, 6400, 8000, 10_000,
    ];

    /// Where the run starts, in a pane that begins somewhere.
    const LEFT: u16 = 4;

    #[test]
    fn every_station_is_drawn_inside_the_run_it_stands_on() {
        // **The defect this replaces.** The pass that pushed stations apart had
        // nothing pulling them back, so at a split pane's width the tail of the
        // line was painted outside the painter's area and clipped away in
        // silence — while `announce` still spoke it and the cursor still walked
        // onto it.
        for width in track_needs(CURVE.len())..=120 {
            let packed = pack(LEFT, width, 10_000, &CURVE);
            assert_eq!(packed.at.len(), CURVE.len(), "a station went missing");
            for (index, x) in packed.at.iter().enumerate() {
                assert!(
                    *x > LEFT,
                    "station {index} stands off the left of a {width}-cell run",
                );
                assert!(
                    x.saturating_add(1) < LEFT.saturating_add(width),
                    "station {index} stands past the end of a {width}-cell run",
                );
            }
        }
    }

    #[test]
    fn no_station_is_drawn_on_top_of_another() {
        for width in track_needs(CURVE.len())..=120 {
            let packed = pack(LEFT, width, 10_000, &CURVE);
            let gap = if packed.framed { GAP } else { TIGHT };
            for pair in packed.at.windows(2) {
                assert!(
                    pair[1].saturating_sub(pair[0]) >= gap,
                    "two stations share a cell at width {width}: {packed:?}",
                );
            }
        }
    }

    #[test]
    fn the_frames_come_off_before_a_station_goes_missing() {
        // A whole pane frames every station; a split pane in the Bevy build
        // leaves 39 cells, where sixteen framed stations want 48.
        assert!(
            pack(LEFT, 87, 10_000, &CURVE).framed,
            "a whole pane dropped the frames it had room for",
        );
        assert!(
            !pack(LEFT, 39, 10_000, &CURVE).framed,
            "a split pane kept frames it has no room for",
        );
    }

    #[test]
    fn the_run_stays_in_the_order_the_line_is_authored_in() {
        // Both passes move stations, and a picture whose second station stood
        // left of its first would say the line runs the other way.
        for width in [track_needs(CURVE.len()), 39, 51, 87, 120] {
            let packed = pack(LEFT, width, 10_000, &CURVE);
            assert!(
                packed.at.windows(2).all(|pair| pair[0] < pair[1]),
                "the line doubled back at width {width}: {packed:?}",
            );
        }
    }
}
