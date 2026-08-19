//! The loop: a clock, a keyboard, and a repaint.
//!
//! What the Bevy build spends a system graph and four `SystemSet` edges on is
//! straight-line statement order here, and the ordering hazards those sets exist
//! to prevent stop being hazards. One of them does not, and it is the reason
//! this module has a long comment in it — see [`run`]'s first step, and the two
//! tests below that hold it.

use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use orbs_render::{DisplayMode, Frame, GridSize};
use orbs_shell::{
    Bench, Boot, Key, Line, Linear, Offered, PaneTransition, Panel, Reveal, Screen, Scroll,
};
use orbs_sim::Sim;

use crate::blit;
use crate::surfaces::{Owner, Surfaces};
use crate::term;

/// One world tick, which DESIGN.md §5.0 makes one real second.
const TICK: Duration = Duration::from_secs(1);

/// How often the screen is redrawn at most.
///
/// Thirty rather than sixty: the animations are meters and a fire, none of which
/// reads differently at the higher rate, and every frame is escape sequences
/// down a pipe that may be a network. The diff in [`blit`] does the real work.
const FRAME: Duration = Duration::from_millis(33);

/// Everything the loop keeps between frames.
struct Session {
    sim: Sim,
    line: Line,
    offered: Offered,
    ghost: String,
    panel: Panel,
    scroll: Scroll,
    bench: Bench,
    linear: Linear,
    reveal: Reveal,
    panes: PaneTransition,
    frame: Frame,
    screen: blit::Screen,
    grid: GridSize,
    /// Whichever of the four has the keyboard, if any has.
    surfaces: Surfaces,
    /// How the main window divides among its panes — `F4`.
    ///
    /// **Held here rather than rebuilt per frame**, because it is a *setting*
    /// (§9: the player may override it at any time) and `Screen` is assembled
    /// fresh in [`Session::draw`]. Without somewhere to keep it, `F4` would flip
    /// the mode and the next frame would flip it back.
    mode: DisplayMode,
    /// How many records the transcript last fitted, for `PageUp`.
    ///
    /// Measured by the painter, because how many records fit is a fact only it
    /// has — a record is not one row, and paging by rows moves more than a
    /// screenful and drops the lines in between.
    page: usize,
    /// How far §4's boot sequence has got.
    ///
    /// **The clock is `orbs-shell`'s and only the driving is here**, which is the
    /// same split the Bevy build has: `Boot::advance` walks the stages, `Stage`
    /// owns their durations, and each frontend supplies a `Time` and a frame to
    /// paint into. `Boot::default` reads `ORBS_BOOT=0` itself, so the skip works
    /// here for free and identically.
    boot: Boot,
    /// When a surface last handed the keyboard back, while arrows keep arriving.
    ///
    /// **`HeldOver`, for a terminal that has no key-up to hang it on.** The Bevy
    /// build spends a resource, a system and a test on this: escape the maze
    /// with a finger still on an arrow and key repeat keeps delivering presses
    /// after the maze has let go, and an arrow at the prompt means *recall
    /// history* — so `wander` reappears on the command line.
    ///
    /// The winit fix drops keys whose *press* went to another surface until they
    /// are *released*. A terminal sends no release event at all, so that shape
    /// is unavailable and the comment below claiming none of the machinery was
    /// needed conflated key repeat with modifiers — they are different problems
    /// and only the second one a terminal solves for free.
    ///
    /// What a terminal does give is *timing*: auto-repeat arrives every 30-40 ms
    /// and a person cannot press an arrow twice inside [`HELD_OVER`]. So an
    /// unbroken run of arrows straight out of a handover is a held key, and the
    /// first gap is the release.
    held_over: Option<Instant>,
    /// What this binary is built out of, for the card's third line.
    ///
    /// §4's card is a **diegetic inventory of the machine**, so it has to
    /// describe *this* machine — `crossterm 0.29` where the other build says
    /// `bevy 0.19.0`. It is the frontend's to know and the shell's to place,
    /// exactly as the wizard's name is.
    engine: String,
}

impl Session {
    fn new(sim: Sim, grid: GridSize, narrow: bool, engine: String) -> Self {
        let mut panel = Panel::default();
        panel.refresh(&sim);
        Self {
            boot: Boot::default(),
            held_over: None,
            engine,
            sim,
            line: Line::default(),
            offered: Offered::default(),
            ghost: String::new(),
            panel,
            scroll: Scroll::default(),
            bench: Bench::default(),
            linear: Linear::default(),
            reveal: Reveal::default(),
            // A terminal has one pane and never animates between counts, so this
            // is settled from the first frame and stays there. `transition.rs`
            // says as much: *"`orbs-tui` is free to ignore all of it."*
            panes: PaneTransition::settled(1),
            frame: Frame::new(grid),
            screen: blit::Screen::new(grid, narrow),
            grid,
            surfaces: Surfaces::default(),
            // §9: **Wide by default, always** — the same opening mode the Bevy
            // build takes from `Screen::for_window`.
            mode: DisplayMode::Wide,
            // A sane floor until the first paint measures the real one.
            page: 1,
        }
    }

    /// Advance the world by one tick, and re-read what the painters cache.
    ///
    /// `Panel` is rebuilt here rather than every frame for the same reason the
    /// Bevy build guards it on `resource_changed::<Tower>`: it walks every room
    /// and allocates per recipe, for a table that changes at 1 Hz. A hand-rolled
    /// loop has no change detection, so the tick boundary *is* the guard.
    /// Returns `false` when the world says the session is over.
    fn tick(&mut self) -> bool {
        self.sim.step();
        // **`quit` lands here, not at `submit`**, and that is not a delay worth
        // engineering away. `submit` echoes and queues; every verb's *effect*
        // runs at the next `step`, which is what keeps effects tick-aligned
        // (§19). So the player sees the echo, then `quit_begins` on the tick,
        // then the terminal comes back — the same beat every other word has.
        if self.sim.quitting() {
            return false;
        }
        self.panel.refresh(&self.sim);
        // A verb may have asked for a surface, and an open one may have been
        // closed from under the player by a spell.
        self.surfaces
            .open(&mut self.sim, &mut self.scroll, self.page);
        self.surfaces.tick(&self.sim);
        self.ghost = self
            .line
            .ghost(self.sim.scene(), !self.sim.choices().is_empty());
        true
    }

    /// Take one keystroke. Returns `false` when the player asked to leave.
    fn typed(&mut self, event: KeyEvent) -> bool {
        // **F10 leaves, from anywhere — the Bevy build's binding, ungated there
        // too.** Not Escape: the moment there is a text field, Escape is what a
        // player presses to get out of something *smaller*, and every one of the
        // four surfaces uses it that way. Matching the key matters more than it
        // looks: this is the same game, and a player who learns one build's exit
        // must not be trapped in the other's.
        if event.code == KeyCode::F(10) {
            return false;
        }
        // **The rest of the function keys, ungated by surface**, exactly as the
        // Bevy build has them: they are `input_just_pressed` in `Update` there
        // with no surface condition, so `F5` works from inside the editor and
        // must here too.
        if self.shortcut(event.code) {
            return true;
        }
        // **The chord guard, and it is one field read.** Under winit this needs
        // `HeldOver`, `Quiet` and `chord_is_stale` — a press/release model, key
        // repeat, and a ghost modifier left behind when a key-up went to the
        // screenshot overlay instead of the window. A terminal carries the
        // modifiers on the event itself, so none of that machinery has anything
        // to be a workaround for.
        //
        // Alt is deliberately absent, exactly as it is on the other side: AltGr
        // is how European layouts type `@`, `#` and `\`.
        let chord = event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER);
        if chord {
            // Raw mode means the terminal does not turn this into SIGINT, so it
            // is ours to answer — and a full-screen application that cannot be
            // left by the one chord everybody tries is a trap.
            //
            // **One accepted divergence**: the Bevy build lets `Cmd+←/→` mean
            // Home/End, *"which a Mac keyboard has no other key for"*. A
            // terminal is not in that position — `Fn+←/→` sends real Home and
            // End, and every macOS terminal emulator does so — and adding a
            // second chord path here would mean a second place for a surface to
            // be forgotten. Every other chord is swallowed, as it is there.
            return !matches!(event.code, KeyCode::Char('c' | 'd'));
        }

        // **Exactly one surface takes a keystroke**, and the failure where two
        // consume a key is invisible until a player types `:wq` and finds it in
        // their command history.
        // **The transcript scrolls without `unfurl`, and from behind a surface.**
        // The word exists because the *key* could not be discovered — the border
        // only advertises `PgDn newest` once you are already scrolled back — not
        // because the key needs permission.
        //
        // **Answered before the surface dispatch, which is where this was
        // wrong.** These arms used to sit below it, so the editor, the weave
        // screen and the maze swallowed both keys through their `_ => {}` and
        // the arms were unreachable whenever anything was open. The Bevy build
        // has `scroll_back`/`scroll_forward` in `Update` gated only on `booted`,
        // so the transcript pages behind the editor there — and the comment
        // claiming parity sat twenty lines under the code that broke it.
        let total = self.sim.scrollback().records().drawn_len();
        match event.code {
            KeyCode::PageUp => {
                self.scroll.page(self.page, true, total);
                return true;
            }
            KeyCode::PageDown => {
                self.scroll.page(self.page, false, total);
                return true;
            }
            _ => {}
        }

        let owner = self.surfaces.owner(&self.scroll);
        if owner == Owner::Prompt {
            // The prompt has the keys — unless a surface only just gave them up
            // and the player has not let go yet.
            if self.still_held(event.code) {
                return true;
            }
        } else {
            self.held_over = Some(Instant::now());
        }
        if owner != Owner::Prompt {
            self.surfaces
                .typed(owner, event.code, &mut self.sim, &mut self.scroll);
            // **The one keystroke that reaches the world without a tick.**
            //
            // `Sim::walk` is the third entry point (§19): an arrow moves the
            // reading *now* and spends no world time, so it lands between the
            // ticks `Panel` is rebuilt on. `Session::tick`'s "the tick boundary
            // is the guard" is true of everything a `step` changes and false of
            // the one thing a walk does — and `panel.stacks` is what both the
            // inline map and `wander`'s pane draw from, so without this the map
            // is up to a second stale. Measured before the fix: four presses
            // inside one tick moved the reading four cells and showed none of
            // it until the tick, while `Escape` — pure surface state — repainted
            // in 35ms. That gap is the bug, and it is invisible to every test
            // that drives `Sim` directly, because the sim was always right.
            //
            // The Bevy build needs no such line and has no such lag: `walk`
            // takes `Tower` by `&mut`, which stamps it, and `refresh_panel`
            // hangs on `resource_changed::<Tower>`. This is that change
            // detection, hand-rolled, in the one place a bare loop can have it.
            //
            // **The whole panel, not just `stacks`.** A walk can pick up a
            // fragment and can solve the maze outright, which moves the
            // cabinet's stock and the archive's rail box with it.
            if owner == Owner::Maze {
                self.panel.refresh(&self.sim);
            }
            // A surface may have opened another — `scribe` from the weave
            // screen cannot happen, but a save can close the editor and hand
            // the prompt back on the same keystroke.
            self.surfaces
                .open(&mut self.sim, &mut self.scroll, self.page);
            return true;
        }

        let Some(key) = self.key(event.code) else {
            return true;
        };
        if let Some(finished) =
            orbs_shell::apply(&key, &mut self.line, &mut self.offered, &self.sim)
        {
            // **Not a tick.** `submit` echoes immediately and queues the command
            // for the next `step`, which is what keeps the echo instant while
            // effects stay tick-aligned (§19).
            self.sim.submit(&finished);
            self.scroll.rewind();
            // `unfurl` and `wander` answer on the tick they are typed, so the
            // surface they ask for must be taken before the next keystroke.
            self.surfaces
                .open(&mut self.sim, &mut self.scroll, self.page);
        }
        self.ghost = self
            .line
            .ghost(self.sim.scene(), !self.sim.choices().is_empty());
        true
    }

    /// Whether this key is auto-repeat left over from a surface that has closed.
    ///
    /// See [`Session::held_over`]. Only the arrows are swallowed: they are the
    /// keys a surface holds down and the only ones whose prompt meaning —
    /// recall history — silently rewrites the command line. A letter arriving
    /// this way is visible and a player can see to delete it.
    fn still_held(&mut self, code: KeyCode) -> bool {
        let arrow = matches!(
            code,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
        );
        let Some(since) = self.held_over else {
            return false;
        };
        let now = Instant::now();
        if arrow && now.duration_since(since) <= HELD_OVER {
            // Still down. Keep the window open behind the repeat, or a long hold
            // would break through the moment it outlasted one interval.
            self.held_over = Some(now);
            return true;
        }
        // A gap, or a key nobody holds down to walk with. They have let go.
        self.held_over = None;
        false
    }

    /// The function keys that are neither the prompt nor a surface.
    ///
    /// Returns whether this key was one of them. **`F4` is why this exists at
    /// all**: `prompt.rs` draws `F4 deep` into the session border on every
    /// frame, and the terminal build shipped without answering it — a key the
    /// screen offered and the build ignored, which is §19's *"the first
    /// affordance the game showed was one that did not work"* a second time.
    ///
    /// The rules behind them are `orbs-shell`'s, so the two builds cannot
    /// disagree about where `Tampered` sits in the register cycle or what the
    /// trace file is called.
    fn shortcut(&mut self, code: KeyCode) -> bool {
        match code {
            // Flip the focus mode. **Visibly this only moves the border's own
            // hint** until multiplexing returns the second pane in Phase 9a —
            // with one pane both tilings are identical, which §19 records as
            // deliberate rather than broken. The Bevy build is equally inert and
            // equally bound.
            KeyCode::F(4) => self.mode = self.mode_flipped(),
            // §14's linear stream — the same pane, described instead of drawn.
            // The terminal build is the one DESIGN.md calls the cheapest route
            // to screen-reader support, so it is the last that should lack it.
            KeyCode::F(5) => self.linear.toggle(),
            // Silent on success, like the Bevy build: it logs, and a log in raw
            // mode would scribble across the screen. The file appearing is the
            // confirmation. A failure must not take the session down — the
            // tester whose run it was recording is still playing.
            KeyCode::F(6) => {
                if let Err(error) = orbs_shell::export_trace(&self.sim) {
                    tracing::warn!("parse trace -> {} failed: {error}", orbs_shell::TRACE_PATH);
                }
            }
            // §3's three tonal registers. **Visibly inert here, and bound
            // anyway** — `Presentation` selects a *face* in the Bevy build's
            // glyph atlas, and the face in a terminal is whatever the user set,
            // which `theme.rs` records as one of this frontend's three accepted
            // degradations. Checked rather than assumed: all three registers
            // capture identically under `tmux capture-pane`.
            //
            // It is still the same key doing the same thing to the same world —
            // the register reaches `Records`, and `F6`'s trace has a `register`
            // column that shows it moving. A key that changed the world in one
            // build and not the other would be the divergence worth avoiding;
            // one that changes it invisibly is a frontend limitation, which is
            // what rule 2 permits.
            KeyCode::F(7) => {
                orbs_shell::cycle_register(&mut self.sim);
            }
            _ => return false,
        }
        true
    }

    /// The focus mode `F4` would move to.
    ///
    /// Through `Screen::flipped` rather than a second `match`, so the two builds
    /// cannot disagree about what the other mode is.
    const fn mode_flipped(&self) -> DisplayMode {
        Screen {
            grid: self.grid,
            mode: self.mode,
            window: (0, 0),
        }
        .flipped()
    }

    /// One crossterm key, as the shared prompt understands it.
    ///
    /// The table itself is `orbs-shell`'s; this is only the mapping, which is
    /// the part that is genuinely backend-shaped.
    fn key(&self, code: KeyCode) -> Option<Key> {
        Some(match code {
            KeyCode::Enter => Key::Enter,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Esc => Key::Escape,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::Tab => Key::Tab,
            KeyCode::Char(glyph) => Key::Text(glyph.to_string()),
            _ => return None,
        })
    }

    /// Ask the shell for a screen and put it on the terminal.
    fn draw(&mut self, out: &mut impl Write) -> std::io::Result<()> {
        // **The card, and nothing else, until the orb is found.** Painted before
        // the layout below rather than over it: `paint_booting` computes its own
        // single-pane layout, and the game's screen has a rail, a prompt and a
        // caret that must not be on this one. §4's `Prompt` stage was removed
        // for exactly that reason — an input line that could not be typed into.
        if !self.boot.is_live() {
            self.frame.reset(self.grid);
            orbs_shell::paint_booting(
                &mut self.frame,
                self.boot.stage(),
                self.boot.progress(),
                &self.engine,
            );
            // No caret: nothing on this screen takes a keystroke, and a caret is
            // the game's one promise about where typing lands.
            self.frame.set_cursor(None);
            return self.screen.draw(&self.frame, out);
        }

        let picture = orbs_render::PICTURE;
        let screen = Screen {
            grid: self.grid,
            ..Screen::for_window(
                (u32::from(picture.0), u32::from(picture.1)),
                Some(self.mode),
            )
        };

        // How far `PageUp` moves is measured from the screen it is about to
        // draw, so a resize changes the page on the same frame it changes the
        // pane. **Records, not rows** — see `orbs_shell::page_step`.
        self.page = orbs_shell::page_step(&screen, &self.sim, self.scroll.back());

        self.frame.reset(self.grid);
        if term::fits(self.grid) {
            orbs_shell::paint(
                &mut self.frame,
                &mut self.linear,
                orbs_shell::View {
                    sim: &self.sim,
                    line: &self.line,
                    screen: &screen,
                    panes: &self.panes,
                    reveal: &self.reveal,
                    offered: &self.offered,
                    ghost: &self.ghost,
                    panel: &self.panel,
                    scroll: &self.scroll,
                    bench: &self.bench,
                    // The editor's viewport follows its caret, and how many
                    // lines fit is a fact only the painter has — which is why
                    // this one field is `&mut`.
                    editing: self.surfaces.editing.as_mut(),
                    weaving: self.surfaces.weaving.as_ref(),
                    walking: self.surfaces.walking,
                },
            );
        } else {
            orbs_shell::paint_too_small(&mut self.frame, &self.sim);
        }
        self.screen.draw(&self.frame, out)
    }

    /// The terminal changed size.
    fn resized(&mut self, cols: u16, rows: u16) {
        self.grid = GridSize::new(cols, rows);
        // **The shadow buffer goes with it.** The diff is addressed by
        // `(col, row)`, so keeping it across a resize would write this frame's
        // cells at last frame's coordinates.
        self.screen.resize(self.grid);
        self.frame.reset(self.grid);
    }
}

/// Play, until the player leaves.
///
/// # Errors
///
/// If the terminal cannot be read from or written to.
pub(crate) fn run(sim: Sim, engine: String) -> std::io::Result<()> {
    let narrow = term::symbols_are_narrow().unwrap_or(true);
    let grid = term::grid()?;
    let mut session = Session::new(sim, grid, narrow, engine);
    let mut out = stdout();

    let start = Instant::now();
    let mut ticked = start;
    let mut painted = start - FRAME;
    session.draw(&mut out)?;

    loop {
        // ── 1. The world, before anything the player did this iteration ──
        //
        // **This order is load-bearing, and the reason is `walk`, not `submit`.**
        //
        // A typed line is queued: `Sim::submit` records it against the current
        // tick and hands the command to the next `step`, so it executes at the
        // start of tick N+1 whatever order this loop runs in. Loop order changes
        // only *which* tick a wall-clock keystroke lands on — latency, not
        // replay.
        //
        // `Sim::walk` does not wait for a clock. It executes immediately, and
        // `Sim::replay`'s contract is that a `Submission::Walked` *"already ran,
        // during its tick, **after** that tick's step"*. Drain an arrow before
        // stepping and the walk runs before tick N's step while being recorded
        // against tick N — and a replay would apply it after. Nothing would
        // fail: the suite stays green, the Bevy build stays green, and the two
        // frontends quietly replay the same seed into different worlds.
        //
        // Step first. The maze arrives in step D and this is already right for it.
        let now = Instant::now();
        // **The tower's clock does not run during boot, and that is a
        // correctness rule rather than a cosmetic one.** `tower::drift` rolls
        // once per tick, so a sim left running through the sequence advances its
        // RNG stream by a wall-clock-dependent number of draws — the same seed
        // would reach a different world depending on how long the animation took
        // and whether anyone skipped it. `Stage::world_runs` says so in as many
        // words. Holding `ticked` at `now` is what keeps the catch-up below from
        // then replaying the whole sequence as a burst of ticks.
        if !session.boot.is_live() {
            ticked = now;
        } else if now.duration_since(ticked) >= TICK {
            ticked += TICK;
            // Catching up rather than skipping, and capped: a suspended laptop
            // must not spend a minute of frames replaying an hour of ticks.
            if now.duration_since(ticked) >= TICK * CATCH_UP {
                ticked = now;
            }
            if !session.tick() {
                // Draw the last frame before going: `quit_begins` was pushed on
                // this tick, and a word whose whole job is to be discoverable
                // should be seen answering.
                session.draw(&mut out)?;
                return Ok(());
            }
        }

        // ── 2. What the player did ──
        //
        // The timeout is *time until the next tick*, capped at a frame, so the
        // loop blocks in `poll` instead of spinning. An idle tower costs nothing.
        let now = Instant::now();
        let until_tick = (ticked + TICK).saturating_duration_since(now);
        let until_frame = (painted + FRAME).saturating_duration_since(now);
        if crossterm::event::poll(until_tick.min(until_frame))? {
            match crossterm::event::read()? {
                // **Nothing is typed during boot, and one thing still is.**
                // Every keyed system in the Bevy build is gated on `booted`, and
                // §19 removed the keypress skip on the grounds that the first
                // thing a player does to the game should not be dismissing it.
                // Both hold here.
                //
                // *Leaving* is the exception, and a terminal is why. Under a
                // window manager there is always a way out of a nine-second
                // animation — close the window — and raw mode takes that away:
                // `Ctrl-C` is ours to answer or nobody's. A full-screen
                // application that cannot be left for ten seconds is a trap, and
                // answering the exit is not a skip: it ends the session rather
                // than jumping to the game.
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if !session.boot.is_live() {
                        if leaving(&key) {
                            return Ok(());
                        }
                    } else if !session.typed(key) {
                        return Ok(());
                    }
                }
                Event::Resize(cols, rows) => session.resized(cols, rows),
                _ => {}
            }
        }

        // ── 3. The clocks the painters read, then the screen ──
        let now = Instant::now();
        if now.duration_since(painted) >= FRAME {
            let elapsed = now.duration_since(painted);
            let delta = elapsed.as_secs_f32();
            let overstep = now.duration_since(ticked).as_secs_f32() / TICK.as_secs_f32();
            // **The sequence's own clock, advanced here and nowhere else.** It
            // walks whole stages per call rather than one per frame, so a stall
            // during boot catches up instead of leaving the animation stuck
            // part-way.
            session.boot.advance(elapsed);
            // `None` leaves whatever `ORBS_FIRE` said: this frontend has no CRT
            // switch to consult, and a missing switch is not a switch set to off.
            session
                .bench
                .advance(delta, overstep.clamp(0.0, 1.0), None, &session.panel);
            // **After the keys, so a keystroke restarts the settle clock before
            // it is advanced rather than after** — otherwise the frame a player
            // types on counts toward the pause they have not taken yet. This is
            // the beat §19 means by *"there is no `save`"*: stop typing and the
            // buffer writes itself out.
            session.surfaces.settle(delta, &mut session.sim);
            painted = now;
            session.draw(&mut out)?;
        }
    }
}

/// How many ticks behind the clock may fall before it stops catching up.
///
/// The Bevy build spends `Time<Virtual>`'s `max_delta` on the same question and
/// picks five seconds: above any frame hitch, below any real absence. An absence
/// belongs to offline progression, which is Phase 9a's.
const CATCH_UP: u32 = 5;

/// The longest gap that still counts as one held arrow rather than two presses.
///
/// Auto-repeat lands every 30-40 ms on every common configuration; the fastest
/// a person double-taps an arrow is well over 150. So this separates *a key that
/// is still down* from *a key pressed again*, which is the distinction a
/// key-release event would give for free and a terminal does not send.
///
/// See [`Session::held_over`].
const HELD_OVER: Duration = Duration::from_millis(120);

/// Whether this key ends the session, asked while the boot card has the screen.
///
/// **The whole of what boot answers**, and it is deliberately not
/// [`Session::typed`]: that routes to surfaces, the prompt and the shortcut
/// table, none of which exist yet. §19 removed the keypress skip — the first
/// thing a player does to the game should not be dismissing it — and this does
/// not put it back, because leaving is not skipping: it closes the session
/// rather than jumping to the tower.
///
/// It exists because a terminal has no window manager behind it. Under Bevy,
/// every keyed system is gated on `booted` and the way out of a nine-second
/// animation is to close the window; raw mode takes that away, so `Ctrl-C` is
/// ours to answer or nobody's.
fn leaving(key: &KeyEvent) -> bool {
    if key.code == KeyCode::F(10) {
        return true;
    }
    key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER)
        && matches!(key.code, KeyCode::Char('c' | 'd'))
}

#[cfg(test)]
mod tests {
    use orbs_sim::Sim;
    use orbs_sim::tower::Way;

    /// Drive a sim the way [`run`] does — **step, then take what the player
    /// did** — and hand back the world it reached.
    fn played(walks: &[Way]) -> Sim {
        let mut sim = Sim::new(3);
        for line in ["attend archive", "research"] {
            sim.submit(line);
            sim.step();
        }
        for way in walks {
            sim.step();
            sim.walk(*way);
        }
        sim
    }

    #[test]
    fn a_walk_replays_into_the_same_world() {
        // **The hazard this loop's statement order exists for**, asserted rather
        // than argued. `Sim::walk` runs immediately, and `Sim::replay`'s contract
        // is that a `Walked` "already ran, during its tick, after that tick's
        // step". So a loop that stepped first replays faithfully...
        let walked = [Way::East, Way::South, Way::West, Way::North];
        let live = played(&walked);
        let recorded: Vec<_> = live.submissions().all().to_vec();

        let mut replayed = Sim::new(3);
        let mut tick = replayed.tick();
        for (at, submission) in recorded {
            while tick < at {
                replayed.step();
                tick = replayed.tick();
            }
            replayed.replay(submission);
        }
        // ...and the proof is the maze itself. How much of it has been opened is
        // the whole of what a walk changes, and it is a count no other verb in
        // this script can move.
        assert_eq!(
            replayed.stacks().map(|maze| maze.explored()),
            live.stacks().map(|maze| maze.explored()),
            "the same walks replayed into a different maze",
        );
        assert_eq!(
            replayed.tick(),
            live.tick(),
            "replay landed on another tick"
        );
    }

    #[test]
    fn draining_a_walk_before_the_step_stamps_it_on_the_wrong_tick() {
        // The other half, and the reason the comment in `run` is as long as it
        // is: get the order wrong and **nothing fails**. The same number of
        // ticks pass and the same squares are walked, so a test comparing either
        // is green — the first draft of this one compared ticks and proved
        // nothing.
        //
        // What actually diverges is the *recording*: a walk drained before the
        // step is stamped with the tick before the one it ran in. Replay stands
        // on the recorded tick, so from then on the log describes a session that
        // did not happen.
        let walked = [Way::East, Way::South, Way::West, Way::North];
        let stepped_first = played(&walked);

        let mut drained_first = Sim::new(3);
        for line in ["attend archive", "research"] {
            drained_first.submit(line);
            drained_first.step();
        }
        for way in walked {
            drained_first.walk(way);
            drained_first.step();
        }

        let right: Vec<_> = stepped_first.submissions().all().to_vec();
        let wrong: Vec<_> = drained_first.submissions().all().to_vec();
        assert_eq!(right.len(), wrong.len(), "a different number of walks");
        assert_ne!(
            right, wrong,
            "the two orders recorded the same thing, so this test proves nothing",
        );
    }
}
