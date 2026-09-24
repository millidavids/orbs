//! Drawing the weave screen (DESIGN.md §11.5, §19).
//!
//! Here rather than in `orbs-render`, for `sheet.rs`'s reason: what appears is
//! the sim's decision, arriving through `Tapestry`; only *where in the pane*
//! is decided here.
//!
//! Progression runs rightward on both tracks. One track draws below the
//! headings at a time — seven rooms' lines and a forked Ley Line do not both
//! fit in eighteen rows beside a details panel — with both headings always
//! drawn so the other track is visibly there. A fork's nodes stack downward:
//! rightward is progress, downward is a choice.
//!
//! The bar and the Ley Line share one scale, the last station's total, so a
//! node's position *is* its cost (§19).
//!
//! Names live in the details panel, not on the nodes: at 48 columns nothing
//! survives a sentence beside every node.

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
/// Bounded rather than proportional: three short authored lines, and a panel
/// that grew with the pane would be three quarters blank on a wide window.
const PANEL_ROWS: u16 = 5;
const PANEL_COLS: u16 = 30;

/// The smallest pane this can honestly be drawn in.
///
/// Two borders, the bar, the headings, the rooms' lines, a blank, the details
/// panel and the words. Below it the answer is to say so rather than draw
/// something misleading. §4's declared floor is 80×22.
///
/// Exact at seven rooms, one spare at six (§19), and left there: a floor that
/// tightened as rooms left would make the screen appear and vanish.
const MIN_ROWS: u16 = 18;
const MIN_COLS: u16 = 24;

/// Draw the weave screen into `pane`.
pub fn paint(frame: &mut Frame, screen: &Tapestry, pane: Rect, prose: &Prose) {
    if pane.is_empty() {
        return;
    }
    let mut painter = frame.painter(pane);
    let area = painter.area();

    // Too small says so: returning silently left a blank pane that still owned
    // the keyboard. The width needed is content rather than a constant —
    // sixteen stations cannot be drawn in a narrow pane however tightly packed,
    // and the honest answer is this message rather than a clipped tail.
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
/// Returns the next free row. A node's position *is* its cost only if the bar
/// and the line beneath share a scale.
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
    let label = gauge(earned, scale);
    let label_width = u16::try_from(label.chars().count()).unwrap_or(9);
    let width = inner.saturating_sub(label_width.saturating_add(1));

    // Spoken as a sentence rather than a row of blocks — §14 names progress
    // bars specifically.
    let spoken = prose.line(
        "weave_bar",
        &[
            ("quantity", &earned.to_string()),
            ("detail", &scale.to_string()),
        ],
    );
    // Filled to the cell the total stands at on the line below, on `along`'s
    // logarithmic scale. The label carries the numbers.
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

    // Renown shares the headings row: at the eighteen-row floor all but one row
    // is spoken for, and the two track names leave the right edge free. Drawn
    // only when it fits whole — a reader cannot tell 1,240 cut short from 12.
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
/// A line rather than separate glyphs: that says "here is a road". Stations sit
/// at their cost on [`along`]'s logarithmic scale; where cost and cells
/// disagree the cells win and [`pack`] says how.
///
/// The totals alternate between two rows, because a five-figure label is wider
/// than the three cells between stations and one row ran them together. Two
/// rows buy six cells — so six figures are abbreviated (see [`figures`]).
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

    // Every line first, then every node, so a frame overwrites the run cell
    // beside it rather than the other way round.
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
            &Span::new(&figures(station.at)).with_style(Style::DIM),
        );
    }
}

/// A threshold as the line draws it: exact below six figures, `250k` above.
///
/// Six figures is where a label stops fitting its `2 * GAP` = 6 cells; at a
/// long game the last two ran together into `121766250000`. Nothing below six
/// figures moves, so the dump captures do not shift.
///
/// Rounds up, never down — a lower label would say a station is nearer than it
/// is. The spoken form, the details pane and `weave_locked` take it unrounded.
fn figures(at: u64) -> String {
    if at < 100_000 {
        return at.to_string();
    }
    format!("{}k", at.div_ceil(1_000))
}

/// The bar's label: what is earned, of the line's scale.
///
/// One function because [`bar`] draws it and [`track_width`] subtracts its
/// width; two `format!`s left to drift would desync the fill from the stations.
fn gauge(earned: u64, scale: u64) -> String {
    format!("{} of {}", figures(earned), figures(scale))
}

/// How wide the Ley Line's run is: exactly the cells the bar above fills.
///
/// One number, read by both rows, so the label's width is subtracted here
/// rather than in each of them.
fn track_width(screen: &Tapestry, inner: u16) -> u16 {
    let label = gauge(screen.experience(), screen.scale().max(1));
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
/// Cost decides the position; the pane decides the rest. Two close totals land
/// on the same cell of a narrow track and a framed node writes at `x-1` and
/// `x+1`, so the second one's `[` erased the first one's mark — which
/// `Progression::check` cannot prevent, since cells are a fact about a pane and
/// `boundaries.rs` keeps panes out of the sim. Three moves:
///
/// - Frames come off before positions go wrong. Sixteen framed stations want
///   48 cells; bare marks need two each, and a road of `·` still reads as one.
/// - A forward pass pushes right, so consecutive stations never share a cell.
/// - A backward pass pulls left, so the tail cannot march off the end — pushing
///   alone clipped the last stations silently while `announce` still spoke them.
///
/// A split pane leaves 39 cells, which is where the frames come off. `paint`
/// refuses anything too narrow for even the tight packing ([`track_needs`]), so
/// the two passes never have to overlap stations to satisfy each other.
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
/// A station at each end and the tightest gap between. Below this the screen
/// says the window is too small rather than dropping stations. Derived from the
/// content, so a line that outgrows the pane is caught by the picture refusing.
fn track_needs(count: usize) -> u16 {
    u16::try_from(count.saturating_sub(1))
        .unwrap_or(u16::MAX)
        .saturating_mul(TIGHT)
        .saturating_add(GAP)
}

/// How many cells along a line `width` cells wide a total of `at` stands, on
/// a **logarithmic** scale to `scale`.
///
/// Position is still cost, read the way the curve grows. The stations grow by
/// about half each — 16, 24, 40, 56, 96 … — so linearly the first five stand
/// inside the first cell. Equal cells for equal *ratios* spreads them as the
/// curve does (§19).
///
/// A longer game moves every station *left*, `scale` being the denominator: the
/// first sits at `ln(17)/ln(10001)` = 0.31 of the run at baseline and
/// `ln(17)/ln(250001)` = 0.23 at long.
///
/// [`pack`] handles crowding, so nothing here needs a length of its own; the
/// *label* did — see [`figures`]. A float, in a painter, which is not the sim.
/// The last three cells are kept for the far end's frame.
fn along(width: u16, scale: u64, at: u64) -> u16 {
    let span = f64::from(width.saturating_sub(3));
    let scale = scale.max(2);
    // `ln(1 + x)` rather than `ln(x)`, so nought is nought rather than negative
    // infinity and the first station is still visibly along the line.
    #[allow(clippy::cast_precision_loss)]
    let fraction = ((at.min(scale) + 1) as f64).ln() / ((scale + 1) as f64).ln();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cells = (fraction.clamp(0.0, 1.0) * span).round() as u16;
    cells
}

/// The rooms' lines, one row each: the name, then a run with its
/// stations standing on it.
///
/// Evenly spaced, not at cost: a mastery station's deed counts a different
/// thing on every line, so position says *order* instead.
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
/// `framed` is off when the line is packed too tightly to frame every station
/// (see [`pack`]). The aimed one keeps its frame either way — that frame is the
/// only thing carrying "aimed" without colour (§14).
fn glyph(painter: &mut Painter<'_>, screen: &Tapestry, x: u16, y: u16, node: &Node, framed: bool) {
    let (mark, style) = stations::standing_mark(node.standing);
    let here = screen.cursor().is_some_and(|aimed| aimed == node.mark());
    if framed {
        stations::framed(painter, x, y, mark, style, here);
    } else {
        stations::bare(painter, x, y, mark, style, here);
    }
    // Drawn silently, then said properly: `Painter::span` would push "`○`" into
    // the speech stream, and §14 forbids meaning carried by a mark.
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
/// One panel for the one thing you are pointed at, since at 48 columns a
/// sentence cannot sit beside every node. Under the tracks rather than beside
/// them, because a track runs the full width.
///
/// A node says two things: *unlocked* is whether the tower has earned enough to
/// reach it, *active* whether what it grants is in effect. A step is both at
/// once; a fork node can be unlocked and idle, or idle for ever once a sibling
/// took the choice. A station says how far and what it opens.
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
            // Not blank: §6 forbids a dead end, and a sometimes-empty panel
            // reads as a screen that has broken.
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
            // What it costs, so a locked node says how much more — and which
            // lane it is, for a fork.
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
            // Pinned right, so the two facts read as columns rather than one
            // run-on phrase.
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
                    &prose.counted(&format!("mastery_{}", stop.id), stop.needed),
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
/// Always says something (§6 forbids a dead end): at the command line this row
/// is the whole interface.
fn status(painter: &mut Painter<'_>, screen: &Tapestry, area: Rect, prose: &Prose) {
    let y = area.row.saturating_add(area.rows).saturating_sub(2);
    let width = u32::from(area.cols.saturating_sub(2));

    // A half-typed word outranks everything, then a complaint, then help.
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
    /// Written out rather than read from `Progression`: these properties are
    /// about *sixteen of anything* in a narrow pane, and a test that moved with
    /// the content would stop testing the geometry.
    const CURVE: [u64; 16] = [
        16, 24, 40, 56, 96, 160, 256, 400, 640, 1000, 1600, 2500, 4000, 6400, 8000, 10_000,
    ];

    /// The same line at a long game, which is where six figures arrive.
    ///
    /// Hand-copied like [`CURVE`] and deliberately a stale mirror: it pins the
    /// geometry, a fact about the pane rather than the tier, so retuning `Long`
    /// does not move it.
    const LONG: [u64; 16] = [
        16, 26, 57, 109, 259, 586, 1_239, 2_490, 5_008, 9_640, 18_665, 34_765, 65_440, 121_766,
        175_248, 250_000,
    ];

    /// Where the run starts, in a pane that begins somewhere.
    const LEFT: u16 = 4;

    #[test]
    fn every_station_is_drawn_inside_the_run_it_stands_on() {
        // Pushing with nothing pulling back painted the tail outside the
        // painter's area, clipped in silence while `announce` still spoke it.
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
        // A whole pane frames every station; a split pane leaves 39 cells,
        // where sixteen framed stations want 48.
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
    fn a_six_figure_total_is_abbreviated_and_a_five_figure_one_is_not() {
        // Nothing baseline draws may move, or 138 dump captures move with it.
        assert_eq!(figures(0), "0");
        assert_eq!(figures(10_000), "10000");
        assert_eq!(figures(60_000), "60000");
        assert_eq!(figures(99_999), "99999");

        // Rounded up, so a label never says a station is nearer than it is.
        assert_eq!(figures(100_000), "100k");
        assert_eq!(figures(100_001), "101k");
        assert_eq!(figures(121_766), "122k");
        assert_eq!(figures(175_248), "176k");
        assert_eq!(figures(250_000), "250k");
    }

    #[test]
    fn no_total_runs_into_the_one_beside_it_on_its_row() {
        // At long the last two odd stations printed `121766250000`. A total is
        // drawn at `x - 1` and the rows alternate, so the one it can touch is
        // two stations along. Asserted only where the line is framed: below
        // that the pane is narrower than the game hands out.
        for curve in [&CURVE, &LONG] {
            let scale = curve[curve.len() - 1];
            for width in track_needs(curve.len())..=120 {
                let packed = pack(LEFT, width, scale, curve);
                if !packed.framed {
                    continue;
                }
                for index in 0..curve.len().saturating_sub(2) {
                    let label = figures(curve[index]);
                    let cells = u16::try_from(label.chars().count()).unwrap_or(u16::MAX);
                    let ends = packed.at[index].saturating_sub(1).saturating_add(cells);
                    let next = packed.at[index + 2].saturating_sub(1);
                    assert!(
                        ends <= next,
                        "`{label}` runs into `{}` at width {width}",
                        figures(curve[index + 2]),
                    );
                }
            }
        }
    }

    #[test]
    fn the_bar_and_the_track_measure_the_same_label() {
        // Two `format!`s one refactor apart from disagreeing would put the
        // fill's edge cells away from the station it is meant to reach.
        assert_eq!(gauge(0, 10_000), "0 of 10000");
        assert_eq!(gauge(250_000, 250_000), "250k of 250k");
        assert_eq!(gauge(123_456, 250_000), "124k of 250k");
    }

    #[test]
    fn the_run_stays_in_the_order_the_line_is_authored_in() {
        // Both passes move stations, and one standing left of its predecessor
        // would say the line runs the other way.
        for width in [track_needs(CURVE.len()), 39, 51, 87, 120] {
            let packed = pack(LEFT, width, 10_000, &CURVE);
            assert!(
                packed.at.windows(2).all(|pair| pair[0] < pair[1]),
                "the line doubled back at width {width}: {packed:?}",
            );
        }
    }
}
