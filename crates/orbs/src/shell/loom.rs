//! Drawing the weave screen (DESIGN.md §11.5, §19).
//!
//! # Why here rather than in `orbs-render`
//!
//! The same answer `sheet.rs` gives: `ScreenLayout` hands this function a
//! rectangle and it draws inside it. What appears is the sim's decision — the
//! tracks, the states and the totals all arrive through `Tapestry` — and only
//! *where in the pane* is decided here.
//!
//! # The shape: a bar, then two tracks running right
//!
//! **Progression runs rightward**, and the screen says so three times over: the
//! experience bar fills to the right, the Ley Line runs right, and Mastery's
//! tiers run right. A vertical list said none of it — it drew what you had as a
//! set of rows, and a player reading it could not see that the thing was a
//! *track* at all. That was the first version and it was replaced for that.
//!
//! A tier's nodes stack **downward**, which is the other axis and the other
//! meaning: rightward is progress and downward is a choice. The Ley Line, which
//! has no choices, is therefore one node tall everywhere.
//!
//! **All three share one scale** — [`SCALE`], a fixed hundred — so a node's
//! position *is* its cost, read against the same cells the bar fills.
//!
//! # The names live in the details panel, not on the nodes
//!
//! The session pane is 60 columns in Deep focus — the grid is 120×45 and two
//! panes tile side by side — and the 80×22 authoring floor is narrower still.
//! Nothing survives putting a sentence beside every node at that width.
//!
//! So a node is a glyph and its total, and what it *is* goes in a boxed panel in
//! the bottom right, for the one thing you are aimed at. That is what makes the
//! picture fit and the words readable at the same time.

use orbs_render::{Frame, Intensity, Painter, Pos, Rect, Role, Span, Style, UtteranceKind};
use orbs_sim::{Node, Prose, Standing};

use super::Tapestry;
use super::tapestry::{Complaint, Mode, Track};

/// The glyph for each state.
///
/// **All three are in CP437 and were checked**, which is not a formality: `●`
/// (U+25CF) is *not* in the table, and the renderer skips what it cannot draw —
/// so "taken" would have rendered as nothing at all, collapsing the one
/// distinction §14 says must not be carried by colour alone. That is the same
/// class of defect as the em-dash CLAUDE.md records. `•` is 0x07, `○` is 0x09,
/// `·` is 0xFA, and `─` is 0xC4.
const TAKEN: char = '\u{2022}';
const OPEN: char = '\u{25cb}';
const LOCKED: char = '\u{b7}';
const RUN: char = '\u{2500}';

/// The frame around a node sitting on a track, and the frame around the aimed
/// one.
///
/// **Two pairs, because brightness could not do it.** An aimed `○` drawn Bright
/// is identical to the sibling beside it — `Open` is already Bright — and §14
/// forbids the difference being colour. So the aimed node changes its *cells*.
/// `»` is CP437 0xAF, which the editor already uses to mark the line an
/// invocation has reached; `«` is 0xAE beside it.
const FRAME: (char, char) = ('[', ']');
const AIMED: (char, char) = ('\u{ab}', '\u{bb}');

/// Where the trunk splits into a tier's nodes.
///
/// `┬` on the first row, `├` on any between, `└` on the last — so one trunk
/// arrives from the left and every node of the first tier hangs off it. All
/// three are CP437 (0xC2, 0xC3, 0xC0), like the run they join.
const FORK_TOP: char = '\u{252c}';
const FORK_MID: char = '\u{251c}';
const FORK_END: char = '\u{2514}';

/// What the experience bar is measured against.
///
/// **A fixed hundred, not the next threshold.** The bar used to retarget every
/// time a step was passed, so it emptied itself at the moment the player earned
/// something — the one instant it should have looked like progress. A constant
/// scale means the fill only ever grows, and it is what lets the Ley Line be
/// drawn *underneath* it at the same scale: a node's position on the line is its
/// cost, read against the same hundred cells.
///
/// Provisional, and deliberately round. The curve does not reach 100 yet; when
/// it does, this becomes a number derived from the content rather than chosen.
const SCALE: u64 = 100;

/// The details panel's size: a border, a sentence, a cost, and the two states.
///
/// **Bounded rather than proportional.** It holds four short authored lines, and
/// a panel that grew with the pane would leave three quarters of itself blank on
/// a wide window while the sentences stayed the same length.
const PANEL_ROWS: u16 = 5;
const PANEL_COLS: u16 = 30;

/// The smallest pane this can honestly be drawn in.
///
/// The bar, a blank, two headings with two rows of nodes and a row of totals
/// each, the details panel and the words — plus two borders. Below it the answer
/// is to say so rather than draw something misleading, exactly as the editor
/// does. §4's declared floor is 80×22, so this fits with room to spare.
const MIN_ROWS: u16 = 18;
const MIN_COLS: u16 = 24;

/// Draw the weave screen into `pane`.
pub(crate) fn paint(frame: &mut Frame, screen: &Tapestry, pane: Rect, prose: &Prose) {
    if pane.is_empty() {
        return;
    }
    let mut painter = frame.painter(pane);
    let area = painter.area();

    // **Too small says so.** Returning silently left a wholly blank pane while
    // this screen still owned the keyboard — and `Tapestry::escape` deliberately
    // does not close, so the only way out was typing `quit` at nothing. The doc
    // above already claimed this refused rather than drawing something
    // misleading; a blank pane with the keys held is the most misleading thing
    // it could draw.
    if area.rows < MIN_ROWS || area.cols < MIN_COLS {
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
    y = y.saturating_add(1);

    y = line_track(
        &mut painter,
        screen,
        left,
        y,
        inner,
        &prose.line("weave_ley", &[]),
    );
    y = y.saturating_add(1);
    tree_track(
        &mut painter,
        screen,
        left,
        y,
        inner,
        &prose.line("weave_mastery", &[]),
    );

    details(&mut painter, screen, area, prose);
    status(&mut painter, screen, area, prose);
}

/// The experience bar, filling rightward against a fixed [`SCALE`].
///
/// Returns the width the bar occupied, so the Ley Line beneath it can place its
/// nodes against the same cells — a node's position *is* its cost, which is only
/// true if the two rows share a scale.
fn bar(
    painter: &mut Painter<'_>,
    screen: &Tapestry,
    left: u16,
    y: u16,
    inner: u16,
    prose: &Prose,
) -> u16 {
    let earned = screen.experience();
    let label = format!("{earned} of {SCALE}");
    let label_width = u16::try_from(label.chars().count()).unwrap_or(9);
    let width = inner.saturating_sub(label_width.saturating_add(1));

    // Spoken as a sentence rather than as a row of blocks — §14 names progress
    // bars specifically, and `progress` takes the sentence for exactly this.
    let spoken = prose.line(
        "weave_bar",
        &[
            ("quantity", &earned.to_string()),
            ("detail", &SCALE.to_string()),
        ],
    );
    painter.progress(
        Rect::new(left, y, width, 1),
        u32::try_from(earned).unwrap_or(u32::MAX),
        u32::try_from(SCALE).unwrap_or(u32::MAX),
        Style::default().with_role(Role::Success),
        &spoken,
    );
    painter.span(
        Pos::new(left.saturating_add(width).saturating_add(1), y),
        &Span::new(&label).with_style(Style::default().with_intensity(Intensity::Bright)),
    );
    y.saturating_add(1)
}

/// The Ley Line: one unbroken line, with its steps standing on it.
///
/// **A line, because that is what a ley line is** — and because the track has no
/// choices in it, so there is nothing for a column to hold. Drawing it as a row
/// of separate glyphs said "here are some things"; drawing it as a line with
/// stations on it says "here is a road, and these are the places along it".
///
/// **Nodes sit at their cost.** The run spans the same cells the bar above does,
/// so a step at 16 stands one sixth of the way along and the fill either has
/// reached it or has not. The two rows are one picture, which is the whole
/// reason [`SCALE`] is fixed.
fn line_track(
    painter: &mut Painter<'_>,
    screen: &Tapestry,
    left: u16,
    y: u16,
    inner: u16,
    name: &str,
) -> u16 {
    let looking = screen.track() == Track::LeyLine && screen.mode() == Mode::Browsing;
    painter.span(
        Pos::new(left, y),
        &Span::new(name).with_style(if looking {
            Style::default().with_intensity(Intensity::Bright)
        } else {
            Style::DIM
        }),
    );

    let row = y.saturating_add(1);
    let width = track_width(screen, inner);
    // Silent: the road is a line, not a fact. Every fact on it is a station.
    painter.glyphs(
        Pos::new(left, row),
        &RUN.to_string().repeat(usize::from(width)),
        Style::DIM,
    );

    // Through `stations` like Mastery's, so two steps authored close together
    // cannot draw one over the other — see there.
    let steps: Vec<Node> = screen.ley_line_columns().into_iter().flatten().collect();
    let at = stations(left, width, steps.iter().map(Some));
    for (node, x) in steps.iter().zip(at) {
        glyph(painter, screen, x, row, node);
        painter.span(
            Pos::new(x.saturating_sub(1), row.saturating_add(1)),
            &Span::new(&node.at.to_string()).with_style(Style::DIM),
        );
    }
    row.saturating_add(2)
}

/// How wide a track's line is: exactly the cells the bar above fills.
///
/// **One number, read by all three rows.** The bar, the Ley Line and Mastery are
/// the same scale drawn three ways, and they only are if they measure the same
/// span — so the label's width is subtracted here rather than in each of them.
fn track_width(screen: &Tapestry, inner: u16) -> u16 {
    let label = format!("{} of {SCALE}", screen.experience());
    inner.saturating_sub(u16::try_from(label.chars().count()).unwrap_or(9) + 1)
}

/// Where a run of stations stands, in order, never closer than they can be drawn.
///
/// **Cost decides the position; the pane decides the floor.** Two totals three
/// experience apart land on the same cell of a 36-wide track, and a node writes
/// its frame at `x-1` and `x+1` *around* its glyph — so the second one's `[`
/// landed on the first one's mark and erased it. `Progression::check` cannot
/// prevent that: how many cells a hundred experience spans is a fact about a
/// pane, and `boundaries.rs` keeps panes out of the sim entirely.
///
/// So the painter guarantees what the content cannot. `GAP` is the width of a
/// drawn station — frame, glyph, frame — so consecutive stations never share a
/// cell, and the picture degrades to *evenly spaced* rather than to *one node
/// eating another*. Being slightly wrong about position is a picture; a missing
/// node is a lie.
fn stations<'a>(left: u16, width: u16, nodes: impl Iterator<Item = Option<&'a Node>>) -> Vec<u16> {
    const GAP: u16 = 3;
    let mut placed: Vec<u16> = Vec::new();
    for node in nodes {
        let want = station(left, width, node.map_or(0, |node| node.at));
        let floor = placed
            .last()
            .map_or(want, |last| last.saturating_add(GAP).max(want));
        placed.push(floor);
    }
    placed
}

/// Where a step of cost `at` stands along a line `width` cells wide.
///
/// Kept one cell inside each end, because a station is drawn with a frame either
/// side of it and a node at zero or at the far edge would lose one of them.
fn station(left: u16, width: u16, at: u64) -> u16 {
    let span = u64::from(width.saturating_sub(3));
    let along = u16::try_from(at.min(SCALE) * span / SCALE.max(1)).unwrap_or(0);
    left.saturating_add(1).saturating_add(along)
}

/// Mastery: one trunk, forking into the first tier, then a line from each node
/// to its own successor in the next.
///
/// **Placed at cost, like the Ley Line** — a tier at 24 stands a quarter of the
/// way along the same cells the bar fills, so the two tracks and the bar are one
/// scale read three times. Drawing tiers at a fixed stride put a tier at 24 and a
/// tier at 40 five cells apart, which said they were adjacent when they are not.
///
/// **And a line rather than loose glyphs**, for the reason the Ley Line is one:
/// what a tree draws is *reachability*, and reachability is the lines. A node
/// with no line into it is a node nothing says how to get to.
fn tree_track(
    painter: &mut Painter<'_>,
    screen: &Tapestry,
    left: u16,
    y: u16,
    inner: u16,
    name: &str,
) -> u16 {
    let looking = screen.track() == Track::Mastery && screen.mode() == Mode::Browsing;
    painter.span(
        Pos::new(left, y),
        &Span::new(name).with_style(if looking {
            Style::default().with_intensity(Intensity::Bright)
        } else {
            Style::DIM
        }),
    );

    let top = y.saturating_add(1);
    let tiers = screen.mastery();
    let depth = tiers.iter().map(Vec::len).max().unwrap_or(0);
    let Some(first) = tiers.first().filter(|tier| !tier.is_empty()) else {
        return top;
    };

    let width = track_width(screen, inner);
    let stations = stations(left, width, tiers.iter().map(|tier| tier.first()));

    // **Every line first, then every node**, so a frame can overwrite the cell of
    // a run beside it rather than being overwritten by one drawn later.
    let fork = stations[0].saturating_sub(2);
    painter.glyphs(
        Pos::new(left, top),
        &RUN.to_string()
            .repeat(usize::from(fork.saturating_sub(left))),
        Style::DIM,
    );
    for row in 0..first.len() {
        let at = top.saturating_add(u16::try_from(row).unwrap_or(0));
        let corner = if first.len() == 1 {
            RUN
        } else if row == 0 {
            FORK_TOP
        } else if row + 1 == first.len() {
            FORK_END
        } else {
            FORK_MID
        };
        painter.glyphs(Pos::new(fork, at), &corner.to_string(), Style::DIM);
    }
    // One line per sibling, from a node to **its own** successor — which is what
    // makes the picture a tree rather than two rows of unrelated marks.
    //
    // **Only the rows both tiers actually have.** This ran to the *deepest*
    // tier's height, so a tier authored narrower than its neighbour drew runs
    // leaving from empty cells — a line from nothing to something, which is the
    // one thing a tree must not draw. Invisible in the shipped 2×2 content and
    // unconstrained by `Progression::check`, which places no rule on tier sizes.
    for (index, pair) in stations.windows(2).enumerate() {
        let (from, to) = (pair[0], pair[1]);
        let start = from.saturating_add(2);
        let stop = to.saturating_sub(1);
        if stop <= start {
            continue;
        }
        let joined = tiers[index].len().min(tiers[index + 1].len());
        for row in 0..joined {
            let at = top.saturating_add(u16::try_from(row).unwrap_or(0));
            painter.glyphs(
                Pos::new(start, at),
                &RUN.to_string().repeat(usize::from(stop - start)),
                Style::DIM,
            );
        }
    }

    for (index, tier) in tiers.iter().enumerate() {
        let x = stations[index];
        for (row, node) in tier.iter().enumerate() {
            glyph(
                painter,
                screen,
                x,
                top.saturating_add(u16::try_from(row).unwrap_or(0)),
                node,
            );
        }
        if let Some(node) = tier.first() {
            painter.span(
                Pos::new(
                    x.saturating_sub(1),
                    top.saturating_add(u16::try_from(depth).unwrap_or(1)),
                ),
                &Span::new(&node.at.to_string()).with_style(Style::DIM),
            );
        }
    }
    top.saturating_add(u16::try_from(depth).unwrap_or(1))
        .saturating_add(1)
}

/// One node, and the cursor if it is on this one.
fn glyph(painter: &mut Painter<'_>, screen: &Tapestry, x: u16, y: u16, node: &Node) {
    let mark = match node.standing {
        Standing::Taken => TAKEN,
        Standing::Open => OPEN,
        Standing::Locked => LOCKED,
    };
    let style = match node.standing {
        Standing::Taken => Style::default().with_role(Role::Success),
        Standing::Open => Style::default().with_intensity(Intensity::Bright),
        Standing::Locked => Style::DIM,
    };
    // **Drawn silently, then said properly.** `Painter::span` would push the
    // glyph's own text into the speech stream, so a reader would hear "`○`" and
    // be told nothing — §14 forbids meaning carried by a mark. `glyphs` writes
    // no speech, and the `announce` below is the row a listener actually needs:
    // the total and the state, as words. That is the same division
    // `Painter::meter` makes, whose doc says a silent caller *owes* the listener
    // an utterance.
    // **A frame either side, always** — a node standing on a line needs to read
    // as a station rather than as a break in it. The aimed one swaps the pair
    // for `«»`, which changes the *cells*: brightness could not carry it,
    // because an aimed `○` is already Bright and identical to its sibling, and
    // §14 forbids the difference being colour. The frame overwrites one cell of
    // the run on each side, which is why the run is drawn first.
    let here = screen.cursor() == Some(node.id.as_str());
    let (open, close) = if here { AIMED } else { FRAME };
    let frame = if here {
        Style::default().with_intensity(Intensity::Bright)
    } else {
        Style::DIM
    };
    painter.glyphs(Pos::new(x.saturating_sub(1), y), &open.to_string(), frame);
    painter.glyphs(Pos::new(x.saturating_add(1), y), &close.to_string(), frame);
    // **Bright as well as framed.** The frame is what survives greyscale and
    // what makes an aimed `○` tell apart from an open one; the brightness is
    // what the eye finds first. Two carriers for one fact is what §14 asks for —
    // neither is doing it alone.
    painter.glyphs(
        Pos::new(x, y),
        &mark.to_string(),
        if here {
            style.with_intensity(Intensity::Bright)
        } else {
            style
        },
    );
    painter.announce(
        UtteranceKind::TableRow,
        Role::Normal,
        &format!("{}: {}", node.at, node.standing.word()),
    );
}

/// The details panel: what the cursor is on, and the two facts about it.
///
/// **The whole reason the nodes are bare glyphs.** At 48 columns a sentence
/// cannot sit beside every node, and abbreviating them all would make the screen
/// a puzzle. One panel, for the one thing you are pointed at.
///
/// **Bottom right, and boxed.** It sits under the tracks rather than beside
/// them, because a track runs the full width of the pane and anything alongside
/// one would be sharing cells with the road. The border is what makes it a panel
/// rather than two loose rows that happen to be near each other.
///
/// # It says two things, and they are not the same thing
///
/// **Unlocked** is whether the tower has earned enough to reach it. **Active** is
/// whether what it grants is in effect. A Ley Line step is both together —
/// passing one *is* taking it — but a Mastery node can be unlocked and idle
/// (nobody has chosen it) or unlocked and idle for ever (a sibling took the
/// tier's one choice). One line could not have carried that, and the glyph
/// cannot: `Locked` deliberately draws the same for both.
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

    let Some(node) = screen.aimed() else {
        // Not blank: §6 forbids a dead end, and a panel that is sometimes full
        // and sometimes empty reads as a screen that has broken.
        painter.span(
            Pos::new(text_col, row.saturating_add(1)),
            &Span::new(orbs_render::arriving(
                &prose.line("weave_aim_first", &[]),
                room,
            ))
            .with_style(Style::DIM),
        );
        return;
    };

    painter.span(
        Pos::new(text_col, row.saturating_add(1)),
        &Span::new(orbs_render::arriving(
            &prose.line(&format!("weave_node_{}", node.id), &[]),
            room,
        ))
        .with_style(Style::default().with_intensity(Intensity::Bright)),
    );
    // What it costs, so a locked node says how much more rather than only that
    // it is shut.
    painter.span(
        Pos::new(text_col, row.saturating_add(2)),
        &Span::new(orbs_render::arriving(
            &prose.line("weave_at", &[("count", &node.at.to_string())]),
            room,
        ))
        .with_style(Style::DIM),
    );

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
    // Pinned right inside the box, so the two facts read as a pair of columns
    // rather than as one run-on phrase.
    let live_col = col
        .saturating_add(width)
        .saturating_sub(u16::try_from(live.chars().count()).unwrap_or(8))
        .saturating_sub(1);
    painter.span(
        Pos::new(live_col.max(text_col), states),
        &Span::new(&live).with_style(if node.standing.active() {
            Style::default().with_role(Role::Success)
        } else {
            Style::DIM
        }),
    );
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
        (Mode::Browsing, None) => (prose.line("weave_browsing", &[]), Style::DIM),
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
        Complaint::NothingBehind(id) => prose.line("weave_nothing_behind", &[("name", id)]),
        Complaint::Locked(id, at) => {
            prose.line("weave_locked", &[("name", id), ("count", &at.to_string())])
        }
        Complaint::Spent(id) => prose.line("weave_spent", &[("name", id)]),
    }
}
