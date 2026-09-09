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
    Bench, Boot, Key, Line, Linear, Offered, PaneTransition, Panel, Passing, Reveal, Screen, Scroll,
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

/// Whether the terminal this process was given has gone away.
///
/// # A leaked process that idles costs nothing; one that spins takes the machine
///
/// When the pty master closes — the tmux server dies, an ssh connection drops,
/// the harness that spawned this is `SIGKILL`ed — the slave end reports
/// `POLLHUP`. `crossterm::event::poll` then answers *ready* immediately and for
/// ever, and `crossterm::event::read` responds to the EOF behind it by **looping
/// inside itself** rather than returning or erroring. So `?` never fires, the
/// read never comes back, and the process burns a core until something kills it.
///
/// 573 of these once took a 32-core machine to a load average of 581 with swap
/// exhausted, leaked by test harnesses that were `SIGKILL`ed before their `Drop`
/// could tear the tmux sessions down. The `SIGHUP` handler `term` installs
/// cannot reach them: nothing signals a child already reparented away from a
/// dead terminal.
///
/// # Why it asks the descriptor rather than crossterm
///
/// Because crossterm has no way to say it. A first attempt counted reads that
/// came back with nothing usable, which is the right shape and never runs: the
/// read that would have been counted is the one that does not return. The
/// question has to be put **before** the read, and to the file descriptor.
///
/// Zero timeout, so an ordinary frame pays one non-blocking `poll` and nothing
/// else.
fn hung_up() -> bool {
    gone(&rustix::stdio::stdin())
}

/// How often the watchdog asks whether the terminal is still there.
///
/// Half a second: a leaked process outlives its terminal by that much at most,
/// and a live session pays one non-blocking `poll` twice a second.
const HANGUP_CHECK: Duration = Duration::from_millis(500);

/// Watch for the terminal going away, from a thread the loop cannot block.
///
/// # Why a thread, when the loop could just look
///
/// **Because the loop never gets the chance.** [`hung_up`] at the top of each
/// pass is correct and is kept — it gives the tidy exit, through `term::leave`,
/// whenever the loop is still turning. It does not fire for the case that
/// matters: `crossterm::event::read`, having hit EOF on a dead pty, **loops
/// inside itself and never returns**, so control never reaches the top of the
/// loop again. A check that lives there cannot run, and measuring proved it —
/// the process kept spinning at 82% with the check in place.
///
/// This thread is the answer, and it is why the fix is not simply *look before
/// you read*. It also exits **hard**: `std::process::exit` skips `term::leave`,
/// which is right rather than sloppy, because everything `leave` does is a write
/// to the terminal that has just been established as gone.
///
/// The risk it carries is a false positive killing a live session, and `gone`
/// answers only to `HUP`, `ERR` and `NVAL` — none of which a working terminal
/// reports. Both directions are tested.
fn watch_for_hangup() {
    let deadline = lifetime().map(|span| Instant::now() + span);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(HANGUP_CHECK);
            if hung_up() || deadline.is_some_and(|end| Instant::now() >= end) {
                std::process::exit(0);
            }
        }
    });
}

/// How long this session may live, if something set a limit.
///
/// # A cap the harness sets and a player never has
///
/// `ORBS_LIFETIME`, in seconds, absent by default — so a real game runs until
/// somebody stops it and nothing here can time a player out mid-brew.
///
/// The play harness sets it, and it closes the half [`watch_for_hangup`] cannot
/// reach on its own. That watches for a terminal that has **died**; this is for
/// one that is perfectly alive while the harness that owned it is not. A detached
/// tmux server is nobody's child, so `SIGKILL`ing `cargo test` leaves its
/// sessions running normally — drawing at 30fps, ~2.4% CPU each — until
/// something happens to kill the server.
///
/// An unreadable value is treated as no limit rather than as zero: a typo in an
/// environment variable should not make every game exit half a second after it
/// starts.
fn lifetime() -> Option<Duration> {
    std::env::var("ORBS_LIFETIME")
        .ok()?
        .trim()
        .parse()
        .ok()
        .map(Duration::from_secs)
}

/// Whether the far end of `fd` has closed.
///
/// Split from [`hung_up`] so it can be tested against a pipe: a `poll` that
/// silently never reports `HUP` is a guard that does nothing, and the whole
/// point of this one is that nothing else notices.
///
/// A pty slave whose master has closed reports `HUP | ERR` — measured, not
/// assumed, because a pipe and a pty need not agree and it is the pty that
/// matters here.
fn gone<Fd: std::os::fd::AsFd>(fd: &Fd) -> bool {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};

    // No flags requested: a hangup is reported whether or not anything was asked
    // for, which is the point — this asks *is the far end still there*, not
    // *is there input*. Crossterm answers the second question and cannot answer
    // the first.
    let mut watched = [PollFd::new(fd, PollFlags::empty())];
    // `rustix` rather than `libc`: `unsafe_code` is denied workspace-wide, and a
    // safe wrapper over one `poll` is exactly what that rule is for.
    if poll(&mut watched, Some(&Timespec::default())).is_err() {
        // A `poll` that will not run is itself a terminal that cannot be read.
        return true;
    }
    watched[0]
        .revents()
        .intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL)
}

/// Everything the loop keeps between frames.
struct Session {
    sim: Sim,
    /// The reader that answers lines the orb cannot read itself (§6), if any.
    ///
    /// **Built once with the session, not per keystroke**, because reading the
    /// environment is not free and the answer cannot change mid-run.
    ///
    /// **The trained reader, same as the Bevy build.** This was empty on
    /// principle while the reader dragged `wgpu` behind it; it does not —
    /// inference is `ndarray` at 436µs — so the terminal frontend is the whole
    /// game rather than a cut-down one. `scripts/play.sh` can still set `stub`
    /// when a scenario is *about* a divined line and wants a fixed answer.
    augury: Option<Box<dyn orbs_sim::Augur>>,
    /// Whether the player wants that reader consulted.
    ///
    /// **Kept beside the reader rather than replacing it**, for the Bevy build's
    /// reason: switching back must not cost a file read on a keystroke.
    driver: orbs_shell::Driver,
    /// The file this tower is kept in, and where it is written back.
    ///
    /// **The path travels with the `Sim`**, which is what stops loading a second
    /// game destroying the first: the save is written every sixty ticks and
    /// again on the way out, and a path resolved at the moment of writing is a
    /// path resolved *after* a swap. Here they are two fields of one struct that
    /// [`run`] replaces together, by building a new `Session`.
    ///
    /// `None` when this session keeps nothing — `ORBS_SAVE=off`.
    kept: Option<std::path::PathBuf>,
    /// Whether a failed write has already been complained about.
    save_failed: bool,
    line: Line,
    offered: Offered,
    ghost: String,
    panel: Panel,
    scroll: Scroll,
    bench: Bench,
    linear: Linear,
    reveal: Reveal,
    panes: PaneTransition,
    /// One screen leaving and the next arriving — settled here, always.
    ///
    /// **Not effort, and not the boundary being ducked.** §14 requires motion be
    /// disableable and this build has no CRT to consult, so it passes `None` for
    /// the motion switch everywhere (see the `bench.advance` call in `play`). An
    /// animating crossing would be the first motion in this build with no switch
    /// a player can reach, which `shell::bench` says a motion effect may not be.
    /// The persisted reduce-motion setting is a later phase's, and this comes
    /// back with it.
    ///
    /// The `orbs-render` boundary is proven regardless: the shapes live there and
    /// `cargo run -p orbs-render --example screens` draws them through the same
    /// public API both frontends use.
    passing: Passing,
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
    fn new(
        sim: Sim,
        kept: Option<std::path::PathBuf>,
        grid: GridSize,
        narrow: bool,
        engine: String,
    ) -> Self {
        let mut panel = Panel::default();
        panel.refresh(&sim);
        Self {
            boot: Boot::default(),
            held_over: None,
            engine,
            sim,
            augury: orbs_shell::augury(),
            driver: orbs_shell::settings::driver(),
            kept,
            save_failed: false,
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
            // Settled from the first frame and never advanced. See the field.
            passing: Passing::default(),
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
        // §8 permits a save at a tick boundary and nowhere else, and this is one
        // — `step` has returned, so `Pending` is drained and `Skip` is spent.
        if self.sim.tick().get().is_multiple_of(AUTOSAVE_TICKS) {
            self.keep();
        }
        // **`quit` lands here, not at `submit`**, and that is not a delay worth
        // engineering away. `submit` echoes and queues; every verb's *effect*
        // runs at the next `step`, which is what keeps effects tick-aligned
        // (§19). So the player sees the echo, then the answer on the tick, then
        // the terminal comes back — the same beat every other word has.
        //
        // **The flag is only set by a confirmed `quit`**: the sim asks first and
        // any other command answers no, so there is nothing to check here beyond
        // the flag itself.
        if self.sim.quitting() {
            return false;
        }
        self.panel.refresh(&self.sim);
        // A verb may have asked for a surface, and an open one may have been
        // closed from under the player by a spell. `menu`'s handshake is taken
        // in here, by `Surfaces::open`.
        self.surfaces
            .open(&mut self.sim, &mut self.scroll, self.page, self.driver);
        self.surfaces.tick(&self.sim);
        // **Taken here rather than in the surface**, for the reason
        // `Surfaces::driving` gives: the reader belongs to the session. Unlike
        // leaving and swapping this does not end the loop — it changes how the
        // next line is read and nothing else.
        if let Some(driver) = self.surfaces.driving.take() {
            self.driver = driver;
        }
        // The menu's own `quit` leaves the loop the same way, and so does a
        // swap; `run` decides which of the three it was.
        if self.surfaces.leaving || self.surfaces.swapping.is_some() {
            return false;
        }
        self.ghost = self
            .line
            .ghost(self.sim.scene(), !self.sim.choices().is_empty());
        true
    }

    /// Take one keystroke. Returns `false` when the player asked to leave.
    fn typed(&mut self, event: KeyEvent) -> bool {
        // **Escape arriving with a letter behind it is two keystrokes, and this
        // build was throwing the first one away.** A terminal sends Escape as
        // one byte, `\x1b`, with nothing to say where it ends; crossterm reads
        // whatever is in the pty buffer in one syscall, and `\x1b` followed by
        // any byte in the same read parses as `Alt+<that byte>`. So closing the
        // spell editor and typing `quit` fast enough lands both in one read, the
        // Escape vanishes, and `quit` goes into the buffer as a line of the
        // spell. That is exactly what it looks like: the key doing nothing.
        //
        // **Splitting it back apart is lossless here, and only here.** Under no
        // keyboard-enhancement flags — which this build never pushes, see the
        // `Repeat` note in `run` — crossterm produces `Alt+Char` from precisely
        // one thing, an `\x1b <byte>` pair. A *real* Alt chord on a special key
        // (`Alt+Left`, `Alt+Home`) arrives CSI-encoded with a modifier
        // parameter instead and is left alone, which matters: those are bound in
        // plenty of terminals and turning one into Escape would drop a player
        // out of the editor for pressing word-left.
        //
        // The game binds no Alt chord at all — the guard below excludes it
        // deliberately, so AltGr can still type `@`, `#` and `\` — so there is
        // no meaning being taken away. On Linux AltGr is a level shift that
        // composes in the terminal and sends the finished character with no
        // modifier, so it never reaches this branch.
        if let Some(rest) = escape_prefixed(&event) {
            if !self.typed(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)) {
                return false;
            }
            // One level deep and no further: `rest` has Alt cleared.
            return self.typed(rest);
        }
        // **`Ctrl-H` is Backspace on a great many terminals, and the chord guard
        // below was eating it.** A terminal configured `stty erase ^H` — which
        // is PuTTY's shipped default — sends `0x08` for the Backspace key, and
        // crossterm turns every `0x01..=0x1A` byte into `Char(letter) +
        // CONTROL`. So the key arrived as a chord, the guard swallowed
        // everything that was not `c` or `d`, and **Backspace did nothing
        // anywhere in the game**: the prompt, the spell editor, the weave
        // screen. §6 makes this a game played entirely by typing, so a typo
        // became uncorrectable short of clearing the whole line.
        //
        // Translated rather than special-cased in the guard, because every
        // surface reads `KeyCode::Backspace` and none of them should have to
        // know this. The cost is that a deliberate `Ctrl-H` chord is
        // unavailable — the game binds none, and it is not distinguishable from
        // Backspace at this layer anyway.
        let event = if event.code == KeyCode::Char('h')
            && event.modifiers.contains(KeyModifiers::CONTROL)
        {
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)
        } else {
            event
        };
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
            if self.still_held(&event) {
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
                .open(&mut self.sim, &mut self.scroll, self.page, self.driver);
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
            // `plain` leaves the reader loaded and simply does not ask it —
            // see `Session::driver`.
            let reader = match self.driver {
                orbs_shell::Driver::Plain => None,
                orbs_shell::Driver::Augury => self.augury.as_deref(),
            };
            match reader {
                Some(augur) => self.sim.submit_reading(&finished, augur),
                None => self.sim.submit(&finished),
            }
            self.scroll.rewind();
            // `unfurl` and `wander` answer on the tick they are typed, so the
            // surface they ask for must be taken before the next keystroke.
            self.surfaces
                .open(&mut self.sim, &mut self.scroll, self.page, self.driver);
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
    fn still_held(&mut self, event: &KeyEvent) -> bool {
        let arrow = matches!(
            event.code,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
        );
        let Some(since) = self.held_over else {
            return false;
        };
        if !arrow {
            // A key nobody holds down to walk with. They have let go.
            self.held_over = None;
            return false;
        }
        // **When the terminal says so, believe it.** With keyboard-enhancement
        // flags active crossterm reports `Repeat` outright, which is the exact
        // answer a key-release event would have given — no timing involved.
        if event.kind == KeyEventKind::Repeat {
            self.held_over = Some(Instant::now());
            return true;
        }
        let now = Instant::now();
        if now.duration_since(since) <= HELD_OVER {
            self.held_over = Some(now);
            return true;
        }
        self.held_over = None;
        false
    }

    /// This terminal as a [`Screen`] — no window behind it, so the picture is
    /// the stand-in and hostability turns entirely on the grid.
    fn screen(&self) -> Screen {
        Screen::windowless(self.grid, Some(self.mode))
    }

    /// Whether this terminal can host the game.
    ///
    /// **`Screen::is_hostable`, not a private copy of half of it.** `term::fits`
    /// asked only the grid question, while the shell's answer is two — the grid
    /// against the authoring floor *and* the scale — so the two frontends routed
    /// to the "too small" card by different rules, and the `--dump` diff in CI
    /// could not see the difference because the dump path already asked the
    /// whole question.
    fn hostable(&self) -> bool {
        self.screen().is_hostable()
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
            // hint** until multiplexing returns the second pane in Phase 11a —
            // with one pane both tilings are identical, which §19 records as
            // deliberate rather than broken. The Bevy build is equally inert and
            // equally bound.
            // Through `DisplayMode::flipped` rather than a second `match`, so
            // the two builds cannot disagree about what the other mode is.
            KeyCode::F(4) => self.mode = self.mode.flipped(),
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
            // §14's accommodation for the menagerie, and **not inert here**: it
            // changes what a strike is worth, which is world state, so a chant
            // sung patiently in a terminal reaches the same troops as one sung
            // patiently under Bevy. Rule 2 is satisfied by more than usual.
            KeyCode::F(9) => {
                orbs_shell::toggle_patient(&mut self.sim);
            }
            _ => return false,
        }
        true
    }

    /// One crossterm key, as the shared prompt understands it.
    ///
    /// The table itself is `orbs-shell`'s; this is only the mapping, which is
    /// the part that is genuinely backend-shaped.
    fn key(&self, code: KeyCode) -> Option<Key> {
        as_key(code)
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
            // **The floor applies to the card too, and it did not.** This
            // returned before the hostable check below, so a player launching
            // into a small terminal watched the logo run off the right edge and
            // print through the pane border for the whole thirteen seconds —
            // and only then got the card explaining what was wrong. `§19` calls
            // a shrunk terminal a normal runtime state; it is normal during boot
            // as well.
            if !self.hostable() {
                orbs_shell::paint_too_small(&mut self.frame, &self.sim);
                self.frame.set_cursor(None);
                return self.screen.draw(&self.frame, out);
            }
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

        let screen = self.screen();

        // How far `PageUp` moves is measured from the screen it is about to
        // draw, so a resize changes the page on the same frame it changes the
        // pane. **Records, not rows** — see `orbs_shell::page_step`.
        self.page = orbs_shell::page_step(&screen, &self.sim, self.scroll.back());

        self.frame.reset(self.grid);
        if screen.is_hostable() {
            orbs_shell::paint(
                &mut self.frame,
                &mut self.linear,
                // Settled, always — see the field. It still travels through
                // here because `paint` records each screen's regions in it, and
                // a build that skipped that would be one where the shared
                // painter took a different path for this frontend.
                &mut self.passing,
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
                    menuing: self.surfaces.menuing.as_ref(),
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

/// How often the tower writes itself out, in world ticks.
///
/// The Bevy build's `sim::persist` carries the reasoning and picks the same
/// number: §8 asks for *"every N ticks and on significant events"*, and a minute
/// is the most a crash may cost in a game whose slowest single action is 94
/// ticks. Two builds, one cadence — a player who moved between them and found
/// the terminal lost four times as much would be right to call that a bug.
const AUTOSAVE_TICKS: u64 = 60;

impl Session {
    /// Write the tower out, and complain **once** if it will not go.
    ///
    /// Once, for the reason the other build gives: there is no `save` verb, so a
    /// silent failure is a whole session lost with nothing said — and a save is
    /// attempted every sixty ticks, so a line per attempt is sixty an hour.
    fn keep(&mut self) {
        // **This tower's own path**, never `save_path()` — see `Session::kept`.
        let Some(path) = self.kept.clone() else {
            self.save_failed = false;
            return;
        };
        let Err(error) = orbs_shell::write_save_to(&path, &self.sim.snapshot()) else {
            self.save_failed = false;
            return;
        };
        if !self.save_failed {
            self.save_failed = true;
            // The terminal is in raw mode and the screen is ours, so this cannot
            // go to stdout. `tracing` is where the other build's goes too.
            tracing::error!("the tower could not be written out: {error}");
            // And in voice: the terminal is in raw mode, so `tracing` reaches
            // nobody until the session is over.
            self.sim.say_save_failed();
        }
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
    let mut session = Session::new(sim, orbs_shell::save_path(), grid, narrow, engine.clone());

    loop {
        let result = play(&mut session);

        // **Every way out converges here**, which is why the loop is a function
        // of its own. There are five exits now — the menu's `quit`, `F10`,
        // `Ctrl-C`/`Ctrl-D`, an `io::Error` off the terminal, and a swap — and
        // only the first goes through the `Quitting` flag. Saving at each
        // `return` in turn would have covered one in five and looked complete;
        // the Bevy build reads `AppExit` in `Last` for exactly the same reason.
        //
        // **And it happens before the swap below**, which is that feature's
        // whole exit criterion: the tower being left is written to the path it
        // came from while `session` still holds both.
        session.keep();

        let Some(asked) = session.surfaces.swapping.take() else {
            return result;
        };
        // An error off the terminal is not a thing to swap through.
        result?;

        // **Read before the old session is dropped**, so a save that will not
        // open leaves the player where they were rather than in a half-built
        // world. `read_save_from` has already set an unreadable one aside.
        let raised = match asked.length {
            Some(length) => orbs_sim::Sim::begun(orbs_shell::seed(), length),
            None => match orbs_shell::read_save_from(&asked.path) {
                orbs_shell::Opened::Restored(save) => {
                    let mut resumed = orbs_sim::Sim::restored(&save);
                    resumed.say_resumed(orbs_shell::away_for(&save));
                    resumed
                }
                orbs_shell::Opened::Unreadable => {
                    tracing::error!("the tower in {} could not be read", asked.path.display());
                    session.surfaces.menuing = Some({
                        // The options page marks what is *in effect*, which the
                        // session holds — see `Menu::show_driver`.
                        let mut menu = orbs_shell::Menu::default();
                        menu.show_driver(session.driver);
                        menu
                    });
                    continue;
                }
                orbs_shell::Opened::New => {
                    orbs_sim::Sim::begun(orbs_shell::seed(), orbs_sim::content::Length::Medium)
                }
            },
        };

        // **A whole new `Session`**, which is the terminal's answer to the other
        // build's eighteen-resource reset: every derived field goes back to its
        // default because it is a new struct, and none of them can be forgotten.
        // The grid is re-read because the terminal may have been resized while
        // the menu was up.
        session = Session::new(
            raised,
            Some(asked.path),
            term::grid().unwrap_or(grid),
            narrow,
            engine.clone(),
        );
    }
}

/// The loop itself, so [`run`] has somewhere to stand afterwards.
fn play(session: &mut Session) -> std::io::Result<()> {
    let mut out = stdout();

    // **Asked for before the first frame**, so a process killed during boot puts
    // the terminal back too. A failure to register is not worth refusing to
    // start over — the game runs, it just cannot tidy up after a signal.
    let dying = term::dying().unwrap_or_default();

    watch_for_hangup();

    let start = Instant::now();
    let mut ticked = start;
    let mut painted = start - FRAME;
    session.draw(&mut out)?;

    loop {
        // Leave the way `quit` does, so `term::leave` runs on the ordinary path.
        if dying.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
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
                // **No last frame, and that was wishful.** This drew one on the
                // argument that `quit_begins` "should be seen answering" — but
                // the paint lands on the alternate screen that `term::leave`
                // tears down microseconds later, so nothing of it ever reached a
                // person, and `F10`/`Ctrl-C` returned here without drawing at
                // all. Two exits, two last frames, neither visible.
                //
                // The record is the point and it is kept: `execute::quit` puts
                // it in the scrollback, which is the log a player can `peruse`
                // next session — so a recording ends with someone choosing to
                // stop rather than simply stopping.
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
        // **Before the read, and that placement is the whole fix.** A dead pty
        // makes `poll` answer ready for ever and `read` never return at all, so
        // anything that inspects the *result* of a read is code that never runs.
        // See [`hung_up`] for what 573 of these cost once.
        if hung_up() {
            return Ok(());
        }
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
                // **`Repeat` as well as `Press`, or auto-repeat is lost.**
                // crossterm reports a third kind whenever the terminal has
                // keyboard-enhancement flags pushed — which this build never
                // does, but the flags live on a *terminal* stack, so a crashed
                // editor leaves them set for everything launched afterwards. In
                // that state holding Backspace deleted exactly one character and
                // holding an arrow in the maze moved exactly one square, because
                // the first press arrives as `Press` and every repeat after it
                // was dropped here.
                //
                // It also made [`Session::held_over`] dead code on precisely
                // those terminals: a guard against key repeat, on a loop that
                // filtered key repeat out.
                Event::Key(key)
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                {
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
/// belongs to offline progression, which is Phase 11a's.
const CATCH_UP: u32 = 5;

/// One crossterm key, as every shared table understands it.
///
/// **A free function, because all four surfaces need it**, not only the prompt.
/// Mapping a backend's events onto [`Key`] is the genuinely backend-shaped half;
/// what each surface then *does* with one is `orbs-shell`'s, and was written
/// twice until it was not.
pub(crate) fn as_key(code: KeyCode) -> Option<Key> {
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

/// The longest gap that still counts as one held arrow rather than two presses.
///
/// **The fallback, and only the fallback.** Where the terminal reports
/// `KeyEventKind::Repeat` the answer is exact and this is never consulted; this
/// is for the ordinary case, where auto-repeat and a deliberate press are the
/// same event and only their spacing tells them apart.
///
/// **Sixty, and it was a hundred and twenty.** Auto-repeat lands every 30-40 ms
/// on a default configuration, so sixty catches it with room to spare — while
/// 120 was long enough to swallow *scripted* input: `scripts/tui.sh key` spaces
/// its presses 100 ms apart, so `key Escape Up Up` after any surface lost both
/// arrows and the project's own driving tool could never reach history recall.
/// Being off by twenty milliseconds against our own script is the kind of
/// constant that is only ever wrong in one direction.
///
/// It is a heuristic and it has a floor: a repeat rate slower than this (`xset r
/// rate 660 2`, some accessibility settings) breaks through on a terminal with
/// no enhancement flags. That case is the one a key-release event would fix and
/// nothing here can.
///
/// See [`Session::held_over`].
const HELD_OVER: Duration = Duration::from_millis(60);

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
/// The keystroke hiding behind an `Alt+<letter>`, if that is what this is.
///
/// See the note at the top of [`Session::typed`]. `Some(rest)` means the
/// terminal sent `\x1b` and a byte in one read and the pair should be replayed
/// as Escape, then `rest`. Restricted to [`KeyCode::Char`] because that is the
/// only shape crossterm builds from an escape-prefixed byte; every other Alt
/// chord is CSI-encoded and is a chord the player really pressed.
fn escape_prefixed(key: &KeyEvent) -> Option<KeyEvent> {
    if !key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    if !matches!(key.code, KeyCode::Char(_)) {
        return None;
    }
    let mut rest = *key;
    rest.modifiers.remove(KeyModifiers::ALT);
    Some(rest)
}

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

#[cfg(test)]
mod hangup {
    use super::gone;

    /// A pipe whose writer is still open has not hung up.
    #[test]
    fn a_live_descriptor_is_not_gone() {
        let (reader, writer) = rustix::pipe::pipe().expect("a pipe");
        assert!(
            !gone(&reader),
            "a pipe with its writer open read as hung up"
        );
        drop(writer);
    }

    /// ...and one whose writer has closed has.
    ///
    /// **The half that silently does nothing if `poll` is asked wrongly.** The
    /// request flags are empty on purpose — `HUP` is reported whether or not
    /// anything was asked for — and getting that wrong gives a guard that never
    /// fires, which is exactly the state this shipped in.
    #[test]
    fn a_descriptor_whose_far_end_closed_is_gone() {
        let (reader, writer) = rustix::pipe::pipe().expect("a pipe");
        drop(writer);
        assert!(gone(&reader), "a pipe with its writer closed read as live");
    }
}

/// The two keystrokes a terminal cannot tell apart from one.
///
/// See [`escape_prefixed`]. Kept apart from [`tests`] because these need no
/// world at all — the split is a fact about VT input, not about the tower.
#[cfg(test)]
mod escaping {
    use super::escape_prefixed;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Escape and a letter in one read are two keystrokes, not one chord.
    ///
    /// The case that shipped broken: closing the spell editor and typing `quit`
    /// inside a frame put `quit` in the buffer as a line of the spell, because
    /// `\x1bq` reached crossterm in one syscall and came back `Alt+q`.
    #[test]
    fn an_escape_prefixed_letter_splits_back_into_two_keys() {
        let alt_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT);
        let rest = escape_prefixed(&alt_q).expect("alt+letter is an escape pair");
        assert_eq!(rest.code, KeyCode::Char('q'));
        assert!(
            !rest.modifiers.contains(KeyModifiers::ALT),
            "the replayed key kept the modifier and would split again"
        );
    }

    /// ...and a chord the player really pressed is left alone.
    ///
    /// `Alt+Left` is word-left in a great many terminals and arrives CSI-encoded
    /// rather than escape-prefixed. Turning it into Escape would drop a player
    /// out of the editor for pressing it — which is the failure this whole
    /// split exists to stop, arriving from the other side.
    #[test]
    fn a_real_alt_chord_is_not_an_escape_pair() {
        for code in [KeyCode::Left, KeyCode::Home, KeyCode::Backspace] {
            let chord = KeyEvent::new(code, KeyModifiers::ALT);
            assert!(
                escape_prefixed(&chord).is_none(),
                "{code:?} with Alt was read as an escape pair"
            );
        }
        let plain = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(escape_prefixed(&plain).is_none(), "a bare letter split");
    }
}
