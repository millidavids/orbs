//! The laboratory's instrument panel — §10.1's permanent fixture.
//!
//! Four bars, drawn beside or above the transcript whenever the player is
//! standing in the laboratory. DESIGN.md §10.1: *"with four instruments running
//! you watch and respond"*, and a transcript that scrolls the state away is not
//! something anyone can watch.
//!
//! # Why it earns its space
//!
//! Twice a piece of laboratory state was reported as a bug because the only way
//! to see it was to touch it: banked fuel read as *"stopping got rid of the
//! charcoal"*, and a fouled instrument read as *"it will not start"*. Both were
//! answered with a sentence in `content/prose.toml`. A sentence tells you once,
//! when you ask. A bar tells you continuously, without asking.
//!
//! # It follows the shape of the pane
//!
//! A pane wider than it is tall gets the panel **down its side**, with bars that
//! grow upward; a pane taller than it is wide gets it **across the top**, with
//! bars that grow rightward. Cells are twice as tall as they are wide
//! ([`orbs_render::CELL_WIDTH`] against [`orbs_render::CELL_HEIGHT`]), so "wider
//! than tall" is measured in *pixels* — an 80×22 grid is a wide rectangle, not a
//! tall one.
//!
//! # §14
//!
//! Four bars drawn with `progress` would push four utterances **per frame**,
//! against §14's *"progress announcements: completion only"*. The bars are drawn
//! with `meter`, which is silent, and the panel says **one** line naming only
//! what is doing something.

use orbs_render::cp437::box_drawing;
use orbs_render::{
    CELL_HEIGHT, CELL_WIDTH, Motion, Painter, Pos, Rect, Style, UtteranceKind, Wash,
};
use orbs_sim::tower::{Craft, Instrument, Meter, State};

use super::bench::Bench;

/// Cells an instrument's name gets when the panel runs across the top.
///
/// `mortar_and_pestle` is **17** and the column reserves 18, because the last
/// cell is the gap before the state word. Reserving exactly 17 drew
/// `mortar_and_pestl` — a name the player cannot type, in the panel that exists
/// to tell them what to type.
const NAME: u16 = 18;

/// Cells for the state word, gap included.
///
/// **`gathering` is nine and the column reserved nine**, so the word ran
/// straight into the bar with nothing between them — the same off-by-the-gap
/// [`NAME`] records, in the column next to it. Ten is the longest word plus the
/// space after it.
const STATE: u16 = 10;

/// Cells one instrument's column takes when the panel runs down the side.
///
/// Two for the abbreviation, one for the gap.
const COLUMN: u16 = 3;

/// Which way the panel runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Along {
    /// Down the side, bars growing upward. A pane wider than it is tall.
    Side,
    /// Across the top, bars growing rightward. A pane taller than it is wide.
    Top,
}

impl Along {
    /// The way a pane of this shape wants it.
    fn of(area: Rect) -> Self {
        let wide = u32::from(area.cols) * u32::from(CELL_WIDTH);
        let tall = u32::from(area.rows) * u32::from(CELL_HEIGHT);
        if wide > tall { Self::Side } else { Self::Top }
    }
}

/// Where the panel sits, which way it runs, and what is left for the transcript.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Split {
    /// The panel's own rectangle. Empty when there is nothing to draw.
    pub area: Rect,
    /// Which way it runs — **carried, not re-derived.**
    ///
    /// `paint` used to read this back out of the panel's shape, by testing
    /// `rows == instruments.len() + 1`. A `Side` panel is full pane height, and
    /// full pane height can equal that too: five instruments and a six-row body
    /// made a vertical strip read as horizontal, so `top` drew 17-character names
    /// into a 16-column column. The function that *chose* the direction is the
    /// one that knows it.
    pub along: Along,
    /// What the transcript gets.
    pub rest: Rect,
}

/// The panel's footprint, and what is left for the transcript.
///
/// Both rectangles are empty when there is nothing to draw or no room to draw
/// it, so a caller can hand `rest` straight on.
pub(crate) fn split(area: Rect, instruments: &[Instrument]) -> Split {
    let nothing = Split {
        area: Rect::EMPTY,
        along: Along::Top,
        rest: area,
    };
    if instruments.is_empty() {
        return nothing;
    }
    let count = u16::try_from(instruments.len()).unwrap_or(u16::MAX);

    let along = Along::of(area);
    let (panel, rest) = match along {
        Along::Side => {
            // Columns, plus one cell for the rule between panel and transcript.
            let width = count.saturating_mul(COLUMN).saturating_add(1);
            // Refuse rather than squeeze: a panel with no room for its bars is
            // worse than no panel, because it looks like the instruments are
            // idle.
            if area.cols <= width.saturating_add(20) || area.rows < 4 {
                return nothing;
            }
            let rest = Rect::new(area.col, area.row, area.cols - width, area.rows);
            let panel = Rect::new(area.col + area.cols - width, area.row, width, area.rows);
            (panel, rest)
        }
        Along::Top => {
            // A row each, plus one for the rule.
            let height = count.saturating_add(1);
            if area.rows <= height.saturating_add(3) {
                return nothing;
            }
            let panel = Rect::new(area.col, area.row, area.cols, height);
            let rest = Rect::new(area.col, area.row + height, area.cols, area.rows - height);
            (panel, rest)
        }
    };
    Split {
        area: panel,
        along,
        rest,
    }
}

/// Draw the panel where [`split`] put it.
///
/// `domain` is where the player is standing, for the spoken summary — passed in
/// rather than written here, because a frontend must not be the thing that
/// decides a place name (rule 2). It was the literal `"laboratory"`, next to a
/// pane title drawn from the real location: the moment §10's Phase 9a adds a
/// second instrumented room, a sighted player would read `/tower/workshop` in the
/// border while a screen-reader user heard "laboratory: forge burning".
pub(crate) fn paint(
    painter: &mut Painter<'_>,
    split: Split,
    instruments: &[Instrument],
    domain: &str,
    bench: &Bench,
) {
    if instruments.is_empty() || split.area.is_empty() {
        return;
    }
    match split.along {
        Along::Side => side(painter, split.area, instruments, bench),
        Along::Top => top(painter, split.area, instruments, bench),
    }
    speak(painter, instruments, domain);
}

/// How an instrument's bar is drawn.
///
/// # The animated bars do not take the instrument's accent, deliberately
///
/// [`style_of`] colours the label and the plain gauge; the three picture bars
/// get [`Style::NORMAL`] and their own vocabulary instead. That looks like a
/// channel being dropped and is not:
///
/// - **On the fire it is not available.** `Style::depicted` yields no depiction
///   on an accented cell — §4 reserves the triad strictly for meaning and a
///   picture means nothing — so passing `Style::COST` to `fire_meter` would not
///   tint the flame, it would delete it.
/// - **Nothing is lost.** The accent is on the label in both layouts (`side`
///   draws the abbreviation in it, `top` draws the name and the state word), and
///   `speak` puts the state in the linear stream. §14 asks that colour never be
///   the sole carrier, which is satisfied twice over.
/// - **The picture already says it.** A bar drawn as fire is what `Burning`
///   means; a full bed of broken material is what `Ready` means. Accenting them
///   as well would be the same fact in two channels, and §3 keeps the triad
///   meaningful by not spending it on things that are already legible.
enum Bar {
    /// The plain `█`/`░` gauge. Anything with no picture of its own yet.
    Plain,
    /// A hearth alight: flame, plume and sparks.
    Fire,
    /// A hearth gone out: a wisp of smoke off the bottom and nothing else.
    Cold,
    /// The stacks open, and how much of them has been walked.
    ///
    /// **Meterless**, unlike [`Plain`](Self::Plain): a lectern with no maze open
    /// reports no meter at all, and `Plain` in that state draws *nothing* — the
    /// defect this file records shipping twice already. A reading that has not
    /// begun is an empty gauge, not an absent row.
    Read,
    /// A mortar: a block being broken down, or a bowl standing.
    Grind {
        /// Whether the block is being worked.
        working: bool,
        /// Whether what is in the bowl is only the last run's leavings.
        spent: bool,
    },
    /// A water bath: a vessel of liquid, rolling as it digests.
    Bath {
        /// What the liquid is doing.
        motion: Motion,
        /// Whether the vessel holds only the last run's sediment.
        spent: bool,
        /// Whether a run has settled sediment in its bottom, under the liquid.
        leavings: bool,
        /// Whether bubbles break at the surface — the alembic, not the balneum.
        breaking: bool,
    },
    /// A flask: two bands of ingredient, and the mixture growing under them.
    Mix {
        /// What the mixture is doing.
        motion: Motion,
        /// Whether the vessel holds only the last run's dregs.
        spent: bool,
        /// Whether a run has settled dregs in its bottom, under the mixture.
        leavings: bool,
    },
}

impl Bar {
    /// Whether this bar draws without a quantity behind it.
    ///
    /// **The states the sim reports no meter for.** A cold hearth has nothing
    /// left to measure; a loaded or finished mortar has nothing *in progress*.
    /// All three are pictures worth drawing, and all three are reached past the
    /// check that skips every other meterless instrument.
    const fn meterless(&self) -> bool {
        matches!(
            self,
            Self::Cold | Self::Read | Self::Grind { .. } | Self::Bath { .. } | Self::Mix { .. }
        )
    }
}

/// Which bar an instrument gets.
///
/// **Keyed on [`Craft`], not on the name.** The sim says what a thing *does*
/// (`tower::panel::craft_of`) for the same reason it says what its two-letter
/// form is: a frontend comparing against the literal `"mortar_and_pestle"` would
/// be re-deriving in its own source what the recipe table already knows, and
/// `orbs-tui` would have to derive it a second time.
///
/// **`Banked` keeps the plain gauge**, deliberately. It is damped with its fuel
/// *kept*, and that quantity is the single most valuable thing this panel shows —
/// banked fuel was one of the two states reported as a bug for being invisible
/// (see this module's header). A hearth drawn as smoke would lose the number;
/// drawn as fire it would lie about being alight.
/// **Motion is not consulted here, deliberately.** This used to short-circuit to
/// `Bar::Plain` when animation was off — and `Bar::Plain` is not
/// [`Bar::meterless`], so the three states the sim reports no quantity for drew
/// *nothing at all*. Turning motion off blanked the cold hearth, the loaded bowl
/// and the fouled one, which is §14's accessibility switch costing a player
/// information rather than movement, and it put back both of the invisible-state
/// bugs this panel was built to answer.
///
/// A picture at rest is a picture, not an absence. `Bench` zeroes the phase and
/// the pour when motion is off, so the same picture simply stops moving.
const fn bar_of(craft: Craft, state: State, heat: bool) -> Bar {
    // **One arm per craft, and no wildcard.** `Craft` is declared closed for
    // exactly this: *"a view that fell through to a default for an unrecognised
    // craft would draw a working instrument as an idle one"* — and here it is
    // worse than that, because `Bar::Plain` is not `meterless()`, so a craft
    // that fell through would draw *nothing* for its at-rest states. Naming
    // every variant makes the next instrument's picture a compile error in this
    // file rather than a blank column in the game.
    match craft {
        // Every state, one bar. A maze has no stages — it is open or it is not,
        // and the gauge says how much of it has been seen either way.
        Craft::Reading => Bar::Read,
        // **The prism, and a plain gauge is right here where it was wrong for
        // the lectern.** A press has a duration the tower knows before it starts
        // — twelve ticks, every time — which is exactly what the plain meter is
        // for, and exactly what a maze does *not* have (hence `Bar::Read`, which
        // is meterless).
        //
        // What a plain gauge cannot show is the ward itself: how many sigils are
        // aligned, what has been tried, which sockets have settled. That is the
        // **board**, which is its own item and draws beside the transcript the
        // way the maze map does — a panel row is a progress bar, and this is the
        // one instrument whose interesting state is not progress.
        //
        // Not `Bar::Plain` by omission, which is the defect `Bar::meterless`
        // records shipping three times: a prism between presses is `Empty` and a
        // gauge with no meter behind it draws nothing at all. `Empty` here means
        // *no reading open*, and the board is what says so.
        Craft::Scrying => Bar::Plain,
        // **A plain gauge, and it is an improvement on what it replaced.** The
        // lectern used to be `Craft::Reading` because it carried the maze's verb,
        // so a *scroll coming together* drew the stacks' explored-cells gauge
        // — a picture of a different thing entirely. A twenty-tick run against a
        // known duration is exactly what the plain meter is for.
        //
        // No picture of its own yet: §10.1 gives each laboratory instrument one,
        // and the archive's would be its own item rather than a line here.
        Craft::Assembling => Bar::Plain,
        Craft::Heating => match state {
            State::Burning => Bar::Fire,
            State::Cold => Bar::Cold,
            // **`Banked` keeps the gauge.** It is damped with its fuel *kept*,
            // and that quantity is the most valuable thing this panel shows.
            _ => Bar::Plain,
        },
        Craft::Grinding => match state {
            State::Working => Bar::Grind {
                working: true,
                spent: false,
            },
            State::Charged | State::Gathering | State::Ready => Bar::Grind {
                working: false,
                spent: false,
            },
            // **Leavings, and they must not look like a loaded bowl.** At the
            // side layout there is no state word on screen, so drawing husks as
            // a solid block is the panel saying the mortar is ready when it is
            // jammed — the exact complaint (*"it will not start"*) this panel
            // was built to answer.
            State::Fouled => Bar::Grind {
                working: false,
                spent: true,
            },
            // **`Scouring` stays plain.** It is the triage slot being cleared,
            // not the instrument doing its own work, and a grind there would say
            // the mortar was grinding when it is being scrubbed out.
            _ => Bar::Plain,
        },
        Craft::Digesting => match state {
            // **Bubbles mean the fire is in.** A bath whose athanor has gone out
            // is still working — §10.1 checks heat when a run *starts* and lets
            // it finish — so the picture is the only place that fact appears
            // without a `survey`. Losing the bubbles is the panel saying *your
            // fire died* about the resource the whole timing loop turns on.
            State::Working if heat => Bar::Bath {
                motion: Motion::Bubbling,
                spent: false,
                leavings: true,
                breaking: false,
            },
            State::Working => Bar::Bath {
                motion: Motion::Drifting,
                spent: false,
                leavings: true,
                breaking: false,
            },
            // **A finished bath goes on turning over, gently.** It has just
            // spent its run over a lit athanor and is still hot; a charged one
            // has not been heated at all and is dead flat. That is the
            // difference between "waiting for you" and "not started", which the
            // meter cannot express because the sim reports no quantity for
            // either.
            State::Ready => Bar::Bath {
                motion: Motion::Drifting,
                spent: false,
                leavings: true,
                breaking: false,
            },
            State::Charged | State::Gathering => Bar::Bath {
                motion: Motion::Standing,
                spent: false,
                leavings: false,
                breaking: false,
            },
            // **Sediment, and it must not look like a charged vessel.** The
            // same rule the mortar's husks follow, and for the same reason:
            // at the side layout there is no state word on screen.
            State::Fouled => Bar::Bath {
                motion: Motion::Standing,
                spent: true,
                leavings: true,
                breaking: false,
            },
            // `Scouring` stays plain — the triage slot being cleared, not the
            // bath doing its own work.
            _ => Bar::Plain,
        },
        Craft::Combining => match state {
            // **Never `Bubbling`, whatever the athanor is doing.** Nothing heats
            // a flask (§10.1 lists it as needing no heat), and bubbles are what
            // heat looks like.
            State::Working => Bar::Mix {
                motion: Motion::Stirring,
                spent: false,
                leavings: true,
            },
            // A finished flask holds liquid, and liquid moves. Slowly.
            State::Ready => Bar::Mix {
                motion: Motion::Drifting,
                spent: false,
                leavings: true,
            },
            State::Charged | State::Gathering => Bar::Mix {
                motion: Motion::Standing,
                spent: false,
                leavings: false,
            },
            State::Fouled => Bar::Mix {
                motion: Motion::Standing,
                spent: true,
                leavings: true,
            },
            _ => Bar::Plain,
        },
        // **The alembic is a vessel too**, and draws the balneum's picture with
        // its bubbles breaking at the face — and, where the bar runs upward,
        // getting out: a distillation is a harder boil than a digestion, so some
        // of what rises leaves the liquid (`Steep::upward`, §19). §10.1's five
        // instruments now have five pictures; what distinguishes these two is
        // the break, the escape and the colour of what is in them, not a second
        // liquid vocabulary.
        //
        // Until this arm existed it fell to `Bar::Plain` — which is not
        // `meterless()`, so a *charged* or *ready* alembic drew *nothing at
        // all*, at the instrument the whole pipeline ends at. That is the third
        // time this exact defect has shipped; see `Bar::meterless`.
        Craft::Distilling => match state {
            State::Working if heat => Bar::Bath {
                motion: Motion::Bubbling,
                spent: false,
                leavings: true,
                breaking: true,
            },
            State::Working => Bar::Bath {
                motion: Motion::Drifting,
                spent: false,
                leavings: true,
                breaking: true,
            },
            State::Ready => Bar::Bath {
                motion: Motion::Drifting,
                spent: false,
                leavings: true,
                breaking: true,
            },
            State::Charged | State::Gathering => Bar::Bath {
                motion: Motion::Standing,
                spent: false,
                leavings: false,
                breaking: true,
            },
            State::Fouled => Bar::Bath {
                motion: Motion::Standing,
                spent: true,
                leavings: true,
                breaking: true,
            },
            _ => Bar::Plain,
        },
        // No picture yet — §10.1 promises one each. Listed rather than swept
        // into a wildcard so that writing one is a change *here*.
        Craft::Idle => Bar::Plain,
    }
}

/// The reading a meterless bar stands in with.
///
/// A loaded mortar is nothing-ground and a finished one is everything-ground;
/// both are the same picture at its two ends rather than special cases. A cold
/// hearth has no quantity at all, so it takes the empty end.
const fn stand_in(state: State) -> Meter {
    match state {
        State::Ready => Meter { done: 1, total: 1 },
        _ => Meter { done: 0, total: 1 },
    }
}

/// Whether the athanor is alight.
///
/// **Read off the panel's own slice, not the world.** The sim already reports
/// every instrument's state; a frontend reaching past that for the heat source
/// would be the second place that decides what "lit" means. Found by
/// [`Craft::Heating`] rather than by name or by state, for the reason
/// `bench::hearth` records: `Burning` is the heat source's word today and a
/// forge in Phase 9a would claim it too.
///
/// It is the bath's whole distinction — see `bar_of`.
fn burning(instruments: &[Instrument]) -> bool {
    instruments
        .iter()
        .any(|instrument| instrument.craft == Craft::Heating && instrument.state == State::Burning)
}

/// The reading an instrument's bar draws, or `None` if it gets no bar at all.
///
/// **Shared by both layouts**, which each had their own copy — one written with
/// `continue` and one with `Option`, doing the same thing by different means.
/// The fallback is the subtle part: three states report no meter and are still
/// pictures worth drawing, and a layout that forgot it would blank a cold hearth
/// or a fouled bowl in one orientation and not the other.
fn reading(instrument: &Instrument, kind: &Bar) -> Option<(u32, u32)> {
    let meter = match instrument.meter {
        Some(meter) => meter,
        None if kind.meterless() => stand_in(instrument.state),
        None => return None,
    };
    Some((to_u32(meter.done), to_u32(meter.total)))
}

/// Draw one instrument's bar, whichever way the panel runs.
///
/// One dispatch rather than one per layout: the pairing of a [`Bar`] with its
/// painter is the thing that must not drift, and it was written twice.
fn draw(
    painter: &mut Painter<'_>,
    at: Rect,
    kind: &Bar,
    (done, total): (u32, u32),
    style: Style,
    bench: &Bench,
    along: Along,
    tints: [Option<Wash>; 2],
) {
    // **The bar only, never the label.** A tint says what is *in* the
    // instrument, and the label is the instrument's name — colouring it would
    // make `mp` change colour with its contents, which says the tool changed.
    // It also keeps the two channels from colliding: a fouled instrument's
    // label is `Role::Danger` red, and `tint::resolve` declines on an accent,
    // so the rule is enforced in both places rather than relied on in one.
    //
    // **The flask is the exception and paints its own.** Its bar is three
    // bands, and only the painter knows where they fall at a given fill — so it
    // writes all three itself and this must not lay a fourth over the top.
    if !matches!(kind, Bar::Mix { .. })
        && let Some(wash) = tints[0]
    {
        painter.tint(at, wash);
    }
    let upward = along == Along::Side;
    match (kind, upward) {
        (Bar::Plain, true) => painter.meter_upward(at, done, total, style),
        (Bar::Plain, false) => painter.meter(at, done, total, style),
        // **The plain gauge, deliberately.** The stacks' picture is the map
        // (its own item); what belongs on the panel is *how much has been
        // walked*, and a bespoke glyph vocabulary here would be a second, worse
        // drawing of the same fact in a column two cells wide.
        (Bar::Read, true) => painter.meter_upward(at, done, total, style),
        (Bar::Read, false) => painter.meter(at, done, total, style),
        (Bar::Fire, true) => painter.fire_meter_upward(at, done, total, bench.burn(true)),
        (Bar::Fire, false) => painter.fire_meter(at, done, total, bench.burn(true)),
        (Bar::Cold, true) => painter.fire_meter_upward(at, done, total, bench.burn(false)),
        (Bar::Cold, false) => painter.fire_meter(at, done, total, bench.burn(false)),
        (Bar::Grind { working, spent }, true) => {
            painter.grind_meter_upward(at, done, total, bench.grind(*working, *spent));
        }
        (Bar::Grind { working, spent }, false) => {
            painter.grind_meter(at, done, total, bench.grind(*working, *spent));
        }
        (
            Bar::Bath {
                motion,
                spent,
                leavings,
                breaking,
            },
            true,
        ) => {
            let work = bench.steep(*motion, *spent, *leavings, *breaking);
            painter.bath_meter_upward(at, done, total, work);
        }
        (
            Bar::Bath {
                motion,
                spent,
                leavings,
                breaking,
            },
            false,
        ) => {
            let work = bench.steep(*motion, *spent, *leavings, *breaking);
            painter.bath_meter(at, done, total, work);
        }
        (
            Bar::Mix {
                motion,
                spent,
                leavings,
            },
            true,
        ) => {
            painter.mix_meter_upward(
                at,
                done,
                total,
                bench.stir(*motion, *spent, *leavings),
                tints,
            );
        }
        (
            Bar::Mix {
                motion,
                spent,
                leavings,
            },
            false,
        ) => {
            painter.mix_meter(
                at,
                done,
                total,
                bench.stir(*motion, *spent, *leavings),
                tints,
            );
        }
    }
}

/// Down the side: a column per instrument, bars growing upward.
fn side(painter: &mut Painter<'_>, area: Rect, instruments: &[Instrument], bench: &Bench) {
    let heat = burning(instruments);
    // The rule sits on the panel's first column, between it and the transcript.
    painter.fill(
        Rect::new(area.col, area.row, 1, area.rows),
        box_drawing::VERTICAL,
        Style::DIM,
    );

    let body = Rect::new(area.col + 1, area.row, area.cols - 1, area.rows);
    for (index, instrument) in instruments.iter().enumerate() {
        let at = body
            .col
            .saturating_add(u16::try_from(index).unwrap_or(0).saturating_mul(COLUMN));
        if at >= body.right() {
            break;
        }
        let style = style_of(instrument.state);
        painter.glyphs(Pos::new(at, body.row), &instrument.short, style);

        // Bars grow up from the bottom, which is what a level reads as, and are
        // as wide as the label above them — a one-cell bar under a two-cell
        // abbreviation reads as a stray column rather than as that tool's meter.
        let kind = bar_of(instrument.craft, instrument.state, heat);
        let Some(reading) = reading(instrument, &kind) else {
            continue;
        };
        let width = (COLUMN - 1).min(body.right().saturating_sub(at));
        let bar = Rect::new(at, body.row + 1, width, body.rows.saturating_sub(1));
        draw(
            painter,
            bar,
            &kind,
            reading,
            style,
            bench,
            Along::Side,
            instrument.tints,
        );
    }
}

/// Across the top: a row per instrument, bars growing rightward.
fn top(painter: &mut Painter<'_>, area: Rect, instruments: &[Instrument], bench: &Bench) {
    let heat = burning(instruments);
    for (index, instrument) in instruments.iter().enumerate() {
        let row = area.row.saturating_add(u16::try_from(index).unwrap_or(0));
        if row >= area.bottom() {
            break;
        }
        let style = style_of(instrument.state);
        painter.glyphs(
            Pos::new(area.col, row),
            orbs_render::arriving(&instrument.name, u32::from(NAME) - 1),
            style,
        );
        painter.glyphs(
            Pos::new(area.col.saturating_add(NAME), row),
            instrument.state.label(),
            style,
        );

        let used = NAME.saturating_add(STATE);
        let kind = bar_of(instrument.craft, instrument.state, heat);
        if let Some(reading) = reading(instrument, &kind)
            && area.cols > used
        {
            let bar = Rect::new(
                area.col.saturating_add(used),
                row,
                area.cols.saturating_sub(used),
                1,
            );
            draw(
                painter,
                bar,
                &kind,
                reading,
                style,
                bench,
                Along::Top,
                instrument.tints,
            );
        }
    }

    // The rule sits on the panel's last row, between it and the transcript.
    painter.fill(
        Rect::new(area.col, area.bottom().saturating_sub(1), area.cols, 1),
        box_drawing::HORIZONTAL,
        Style::DIM,
    );
}

/// One utterance for the whole panel, naming only what is doing something.
///
/// Four per frame is what §14 forbids, and "four instruments idle" is not news a
/// listener needs repeated.
fn speak(painter: &mut Painter<'_>, instruments: &[Instrument], domain: &str) {
    use core::fmt::Write as _;

    // One buffer written into, rather than a `format!` per instrument plus a
    // `Vec<String>` plus a `join` plus an outer `format!` — six allocations a
    // frame for a sentence that changes at most once a second.
    //
    // **Only `Empty` is dropped, and `Cold` no longer is.** The rule this
    // follows is §19's: a visual constraint must not become an informational
    // one. `Cold` failed it in the direction nobody checks — the `Top` layout
    // draws the word *cold* on its row and `bar_of` gives it a picture of its
    // own, a wisp of smoke, so a sighted player was told the athanor had gone
    // out and a listener was told nothing at all. It is also the most actionable
    // state the athanor has: cold means kindle it, and nothing else in the
    // laboratory will run until you do.
    //
    // `Empty` stays out because it is the one state that genuinely means
    // *nothing is there* — the panel's whole utterance is what is doing
    // something, and four idle instruments is not news a listener needs
    // repeated once a second.
    let mut spoken = String::new();
    for instrument in instruments
        .iter()
        .filter(|instrument| instrument.state != State::Empty)
    {
        if spoken.is_empty() {
            spoken.push_str(domain);
            spoken.push_str(": ");
        } else {
            spoken.push_str(", ");
        }
        // Infallible into a `String`; the `Result` is `fmt`'s signature, not a
        // failure mode.
        let _ = write!(spoken, "{} {}", instrument.name, instrument.state.label());
    }
    if !spoken.is_empty() {
        painter.announce(UtteranceKind::Progress, Style::DIM.role, &spoken);
    }
}

/// The accent an instrument's state draws in.
///
/// §3 keeps the triad meaningful: `Cost` for work under way and fuel being
/// spent, `Success` for something finished and waiting, `Danger` for an
/// instrument that will not start, plain for at rest.
///
/// **`Fouled` is the third accent, and it is the one the panel was built for.**
/// *"A fouled instrument read as it will not start"* is one of the two defects
/// §10.1's panel exists to answer, and it is the only state here a player has to
/// *act* on before anything else can happen — the tool is jammed and the loop is
/// stopped until it is cleared. `Cost` and `Success` both describe things going
/// right; this is the one that does not, which is exactly §4's line for the
/// accent.
///
/// It reaches a listener too: `Role::Danger` travels into the linear stream
/// beside the text, so the red is reinforcement rather than the carrier. The
/// picture bars already say it a second way — the mortar's sparse husks, the
/// bath's low band of sediment — and `speak` says it a third.
const fn style_of(state: State) -> Style {
    match state {
        State::Working | State::Scouring | State::Burning => Style::COST,
        State::Ready => Style::SUCCESS,
        State::Fouled => Style::DANGER,
        _ => Style::DIM,
    }
}

/// A tick count as a meter value, saturating rather than wrapping.
fn to_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instruments(names: &[&str]) -> Vec<Instrument> {
        names
            .iter()
            .map(|name| Instrument {
                name: (*name).to_owned(),
                short: name.chars().take(2).collect(),
                // These exercise layout, not pictures — `Empty` draws a gauge
                // whatever the craft is.
                craft: Craft::Idle,
                state: State::Empty,
                meter: None,
                tints: [None; 2],
            })
            .collect()
    }

    const LABORATORY: [&str; 5] = [
        "mortar_and_pestle",
        "balneum_mariae",
        "flask_and_rod",
        "alembic",
        "athanor",
    ];

    #[test]
    fn only_a_burning_athanor_burns_and_only_a_cold_one_smokes() {
        // The panel exists because laboratory state was twice reported as a bug
        // for being invisible (see this module's header), so the one thing it
        // must never do is *misreport*. Flames on a banked hearth would claim it
        // was alight; flames on a cold one would claim it had fuel.
        let hearth = |state| bar_of(Craft::Heating, state, true);
        assert!(matches!(hearth(State::Burning), Bar::Fire));
        assert!(matches!(hearth(State::Cold), Bar::Cold));

        // **The mortar draws through its whole lifecycle**, including the three
        // states the sim reports no meter for — which is what stops a loaded or
        // finished tool looking like a row the panel forgot.
        let mortar = |state| bar_of(Craft::Grinding, state, true);
        assert!(matches!(
            mortar(State::Working),
            Bar::Grind {
                working: true,
                spent: false
            }
        ));
        for state in [State::Charged, State::Ready] {
            assert!(
                matches!(
                    mortar(state),
                    Bar::Grind {
                        working: false,
                        spent: false
                    }
                ),
                "{} did not draw a bowl at rest",
                state.label(),
            );
        }
        // **Fouled is its own picture.** Husks drawn as a loaded bowl is the
        // panel saying an instrument is ready when it is jammed, and at the side
        // layout there is no state word on screen to correct it.
        assert!(matches!(
            mortar(State::Fouled),
            Bar::Grind {
                working: false,
                spent: true
            }
        ));
        // Scouring is the triage slot being cleared, not the mortar working.
        assert!(matches!(mortar(State::Scouring), Bar::Plain));
        // ...and a craft with no picture of its own still gets a gauge. All
        // five of §10.1's instruments now have one, so `Idle` is what is left —
        // the dispensary and anything else with no recipe.
        assert!(matches!(
            bar_of(Craft::Idle, State::Working, true),
            Bar::Plain
        ));

        // **The alembic draws, and until it did it drew nothing.** It fell to
        // `Bar::Plain`, which is not `meterless()` — so a *charged* or *ready*
        // alembic showed no bar at all, at the instrument the whole pipeline
        // ends at. Third time this exact defect has shipped; the assertion
        // above used to be the one enforcing it.
        let still = |state| bar_of(Craft::Distilling, state, true);
        for state in [State::Charged, State::Working, State::Ready, State::Fouled] {
            assert!(
                still(state).meterless(),
                "a {} alembic would draw nothing",
                state.label(),
            );
        }
        // ...and its bubbles break at the face, which is what separates its
        // picture from the balneum's.
        assert!(matches!(
            still(State::Working),
            Bar::Bath { breaking: true, .. }
        ));
        assert!(matches!(
            bar_of(Craft::Digesting, State::Working, true),
            Bar::Bath {
                breaking: false,
                ..
            }
        ));

        // **`Banked` keeps the gauge.** Its fuel is the number the bar is for,
        // and it was one of the two states reported as a bug for being
        // invisible — smoke would lose it, fire would lie about it.
        for state in [
            State::Banked,
            State::Empty,
            State::Charged,
            State::Working,
            State::Scouring,
            State::Ready,
            State::Fouled,
        ] {
            assert!(
                matches!(hearth(state), Bar::Plain),
                "the hearth's {} did not draw a plain gauge",
                state.label(),
            );
        }

        // **Motion off keeps the picture and stills it.** This used to assert
        // the opposite — that reduce-motion fell back to `Bar::Plain` — and
        // that was the bug: `Bar::Plain` is not `meterless()`, so the three
        // states with no quantity behind them drew *nothing*, and turning
        // motion off blanked the cold hearth and both resting bowls. The switch
        // must cost movement, never information.
        let mut doused = Bench::default();
        doused.set_enabled(false);
        for (craft, state) in [
            (Craft::Heating, State::Cold),
            (Craft::Grinding, State::Charged),
            (Craft::Grinding, State::Fouled),
        ] {
            assert!(
                bar_of(craft, state, true).meterless(),
                "{craft:?} {} lost its picture",
                state.label(),
            );
        }
        assert_eq!(doused.grind(true, false).phase, 0.0, "it still moves");
        assert!(!doused.grind(true, false).working, "the block still breaks");
        assert_eq!(doused.burn(true).flare, 0.0, "it still flares");
    }

    /// The tint, from the sim's own world through to a resolved colour.
    ///
    /// **The only test that crosses all three crates**, which is where this can
    /// fail with every single-crate test still green: `materials.toml` says sage
    /// is green, `render::tint` says what green is, and this module is the one
    /// thing that connects them. Each half passes on its own while the bar draws
    /// in the base hue.
    #[test]
    fn what_is_in_an_instrument_colours_its_bar() {
        use crate::render::{palette, tint};
        use orbs_render::{Depiction, Frame, GridSize, Intensity, Pos, Role, Tint};
        use orbs_sim::Sim;

        let mut sim = Sim::new(1);
        for line in ["attend laboratory", "move sage to mortar_and_pestle"] {
            sim.submit(line);
            sim.step();
        }
        let instruments = sim.instruments();
        let mortar = instruments
            .iter()
            .find(|instrument| instrument.name == "mortar_and_pestle")
            .expect("the laboratory has one");
        assert_eq!(
            mortar.tint(),
            Some(Wash::plain(Tint::Green)),
            "the sim did not report what is in the bowl",
        );

        // Paint the panel the way the shell does, then resolve every cell the
        // way the grid renderer does.
        let area = Rect::new(0, 0, 80, 22);
        let mut frame = Frame::new(GridSize::new(80, 22));
        let split = split(area, &instruments);
        paint(
            &mut frame.painter(area),
            split,
            &instruments,
            "laboratory",
            &Bench::default(),
        );

        let theme = palette::ALL[0];
        let sage = tint::resolve(
            Wash::plain(Tint::Green),
            Role::Normal,
            Intensity::Bright,
            Depiction::None,
        )
        .expect("green resolves");

        let tinted = (0..22)
            .flat_map(|row| (0..80).map(move |col| Pos::new(col, row)))
            .filter(|at| {
                frame.cell(*at).is_some_and(|cell| {
                    theme.resolve_tinted(cell.style, frame.tint_at(*at))
                        == bevy::prelude::Color::from(sage)
                })
            })
            .count();
        assert!(
            tinted > 0,
            "nothing drew in the sage's own colour — the sim reports the tint \
             and it never reaches a cell",
        );
    }

    #[test]
    fn a_flask_draws_its_two_ingredients_and_their_mixture() {
        // The flask is the only instrument whose bar is three regions, and the
        // only one whose tint carries a *pair* — so it is the only place the
        // `Wash` payload is exercised end to end. Everything below is reachable
        // by a player, which is why the setup is eleven commands long: the flask
        // combines what the mortar and the bath hand it.
        use orbs_render::{Frame, GridSize, Tint, Wash};
        use orbs_sim::Sim;

        let mut sim = Sim::new(1);
        for line in [
            "attend laboratory",
            "kindle charcoal",
            "grind sage",
            "meditate 9",
            "empty mortar_and_pestle",
            "digest ground-sage",
            "meditate 14",
            "siphon balneum_mariae",
            "grind rock-salt",
            "meditate 9",
            "empty mortar_and_pestle",
            "mix sage-tincture with ground-salt",
            "meditate 3",
        ] {
            sim.submit(line);
            sim.step();
        }

        let instruments = sim.instruments();
        let flask = instruments
            .iter()
            .find(|instrument| instrument.name == "flask_and_rod")
            .expect("the laboratory has one");
        assert_eq!(flask.state, State::Working, "the flask is not combining");
        assert_eq!(
            flask.tints,
            [
                Some(Wash::plain(Tint::Green)),
                Some(Wash::plain(Tint::Bone))
            ],
            "the sim did not report both ingredients",
        );

        // The panel, painted the way the shell paints it.
        let area = Rect::new(0, 0, 80, 60);
        let mut frame = Frame::new(GridSize::new(80, 60));
        let split = split(area, &instruments);
        paint(
            &mut frame.painter(area),
            split,
            &instruments,
            "laboratory",
            &Bench::default(),
        );

        // Three regions, and the mixture is the one carrying two families.
        let washes: Vec<Wash> = frame.tints().iter().map(|(_, wash)| *wash).collect();
        assert!(
            washes.contains(&Wash::plain(Tint::Green)),
            "the first ingredient's band is missing: {washes:?}",
        );
        assert!(
            washes.contains(&Wash::plain(Tint::Bone)),
            "the second ingredient's band is missing: {washes:?}",
        );
        assert!(
            washes.contains(&Wash::mixing(Tint::Green, Tint::Bone)),
            "the mixture is not drawn as two things combining: {washes:?}",
        );

        // ...and the bands do not overlap, or one would silently cover another
        // and the flask would look like an ordinary one-colour bar.
        let regions: Vec<Rect> = frame.tints().iter().map(|(area, _)| *area).collect();
        for (index, first) in regions.iter().enumerate() {
            for second in regions.iter().skip(index + 1) {
                assert!(
                    first.intersection(*second).is_empty(),
                    "two bands overlap: {first:?} and {second:?}",
                );
            }
        }
    }

    #[test]
    fn a_jammed_instrument_is_the_one_that_reads_as_a_problem() {
        // The triad, and the line §4 draws through it: `Cost` and `Success` both
        // describe things going right, and `Fouled` is the only state on this
        // panel a player has to *act* on before the loop can continue.
        assert_eq!(style_of(State::Fouled).role, orbs_render::Role::Danger);

        // Nothing else claims it. A second danger state would make the accent a
        // category rather than a signal — §3's whole argument for keeping the
        // triad narrow.
        for state in [
            State::Empty,
            State::Charged,
            State::Working,
            State::Scouring,
            State::Ready,
            State::Burning,
            State::Banked,
            State::Cold,
        ] {
            assert_ne!(
                style_of(state).role,
                orbs_render::Role::Danger,
                "{} also reads as a problem",
                state.label(),
            );
        }

        // ...and the accent stays at ordinary weight, so it reads as a *kind*
        // rather than as emphasis — the rule `accent_constants_keep_default_
        // intensity` holds for the constants themselves.
        assert_eq!(
            style_of(State::Fouled).intensity,
            orbs_render::Intensity::Normal,
        );
    }

    #[test]
    fn a_listener_hears_every_state_the_screen_draws() {
        // §14 and §19: what a sighted player reads must reach a listener too.
        // Both layouts draw *something* for every state but `Empty` — the `Top`
        // one draws the state word on its row, and `bar_of` gives the cold
        // hearth and all three resting bowls pictures of their own — so any
        // state filtered out of this utterance is information the screen has
        // and the stream does not.
        //
        // `Cold` was filtered, and it is the most actionable state the athanor
        // has: nothing in the laboratory runs until it is kindled.
        let mut frame = orbs_render::Frame::new(orbs_render::GridSize::new(80, 22));
        let area = Rect::new(0, 0, 80, 22);
        let states = [
            State::Cold,
            State::Burning,
            State::Banked,
            State::Charged,
            State::Working,
            State::Scouring,
            State::Ready,
            State::Fouled,
        ];
        let drawn: Vec<Instrument> = states
            .iter()
            .enumerate()
            .map(|(index, state)| Instrument {
                name: format!("tool_{index}"),
                short: format!("t{index}"),
                craft: Craft::Idle,
                state: *state,
                meter: None,
                tints: [None; 2],
            })
            .collect();

        speak(&mut frame.painter(area), &drawn, "laboratory");
        let said: String = frame
            .speech()
            .utterances()
            .map(|utterance| utterance.text)
            .collect();
        for state in states {
            assert!(
                said.contains(state.label()),
                "`{}` is drawn but never spoken: {said}",
                state.label(),
            );
        }

        // ...and `Empty` still is not, because "four instruments idle" is not
        // news a listener needs once a second.
        let idle = instruments(&["mortar_and_pestle"]);
        let mut quiet = orbs_render::Frame::new(orbs_render::GridSize::new(80, 22));
        speak(&mut quiet.painter(area), &idle, "laboratory");
        assert_eq!(quiet.speech().utterances().count(), 0, "idle was announced");
    }

    #[test]
    fn an_empty_laboratory_takes_no_space() {
        let split = split(Rect::new(0, 0, 80, 22), &[]);
        assert!(split.area.is_empty());
        assert_eq!(split.rest, Rect::new(0, 0, 80, 22));
    }

    #[test]
    fn a_wide_pane_puts_the_panel_down_the_side() {
        // 80×22 cells is 640×352 pixels — a wide rectangle, because cells are
        // twice as tall as they are wide. Measuring in cells would call this
        // tall and put the panel on top.
        let area = Rect::new(0, 0, 80, 22);
        assert_eq!(Along::of(area), Along::Side);

        let split = split(area, &instruments(&LABORATORY));
        assert_eq!(split.along, Along::Side);
        assert_eq!(split.area.rows, area.rows, "a side panel is full height");
        assert_eq!(split.rest.rows, area.rows);
        assert_eq!(
            split.area.cols + split.rest.cols,
            area.cols,
            "they tile the pane"
        );
    }

    #[test]
    fn a_tall_pane_puts_the_panel_across_the_top() {
        let area = Rect::new(0, 0, 40, 60);
        assert_eq!(Along::of(area), Along::Top);

        let split = split(area, &instruments(&LABORATORY));
        assert_eq!(split.along, Along::Top);
        assert_eq!(split.area.cols, area.cols, "a top panel is full width");
        assert_eq!(
            split.area.rows + split.rest.rows,
            area.rows,
            "they tile the pane"
        );
    }

    #[test]
    fn the_direction_is_carried_rather_than_read_back_from_the_shape() {
        // `paint` must not disagree with `split` about which way the panel runs,
        // or it draws columns into a strip laid out as rows. It used to re-derive
        // this from `rows == instruments.len() + 1` — and a **`Side`** panel is
        // full pane height, which for five instruments and a six-row body is
        // exactly six. Carrying the answer makes the ambiguity unrepresentable;
        // this pins the shape that used to trip it.
        let ambiguous = Rect::new(0, 0, 80, 6);
        let split = split(ambiguous, &instruments(&LABORATORY));
        assert_eq!(Along::of(ambiguous), Along::Side);
        if !split.area.is_empty() {
            assert_eq!(
                split.along,
                Along::Side,
                "a six-row body read back as a top panel"
            );
        }
    }

    #[test]
    fn a_pane_with_no_room_gets_no_panel_rather_than_a_squeezed_one() {
        // A panel with no room for its bars looks like four idle instruments,
        // which is worse than no panel at all.
        let split = split(Rect::new(0, 0, 24, 8), &instruments(&LABORATORY));
        assert!(split.area.is_empty());
        assert_eq!(split.rest.cols, 24);
    }

    #[test]
    fn every_instrument_name_survives_the_top_layout_whole() {
        // A truncated name is a name the player cannot type, in the panel whose
        // job is telling them what to type. `mortar_and_pestl` shipped once.
        for name in LABORATORY {
            assert_eq!(
                orbs_render::arriving(name, u32::from(NAME) - 1),
                name,
                "{name} is cut by its own column"
            );
        }
    }

    #[test]
    fn every_state_word_survives_its_column_whole() {
        for state in [
            State::Empty,
            State::Charged,
            State::Working,
            State::Scouring,
            State::Ready,
            State::Fouled,
            State::Burning,
            State::Banked,
            State::Cold,
        ] {
            let label = state.label();
            assert!(
                u16::try_from(label.len()).unwrap_or(u16::MAX) < STATE,
                "{label} fills its column with no gap before the bar"
            );
        }
    }
}
