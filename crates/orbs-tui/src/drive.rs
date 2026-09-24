//! The loop: a clock, a keyboard, and a repaint.
//!
//! What the Bevy build spends a system graph and four `SystemSet` edges on is
//! straight-line statement order here, so most ordering hazards stop being
//! hazards. One does not — see [`run`]'s first step and the two tests that hold
//! it.

use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use orbs_render::{DisplayMode, Frame, GridSize};
use orbs_shell::{
    Bench, Boot, Key, Line, Linear, Offered, PaneTransition, Panel, Passing, Reveal, Screen,
    Scroll, Threshold, wizard,
};
use orbs_sim::Sim;

use crate::blit;
use crate::surfaces::{Owner, Surfaces, Swap};
use crate::term;

/// One world tick, which DESIGN.md §5.0 makes one real second.
const TICK: Duration = Duration::from_secs(1);

/// What a first launch would show, with no session to ask.
///
/// For the dump, which builds no session — and the only answer available to a
/// capture, since `scripts/dumps.sh` runs with `ORBS_SAVE=off`.
#[must_use]
pub(crate) fn default_settings() -> Vec<orbs_shell::settings::Row> {
    settings(orbs_shell::Driver::default(), false, DisplayMode::Wide)
}

/// What this build can set, and what each is set to now.
///
/// A shorter list than the window's, which is how *unsupported* is said: no
/// tube, no phosphor and no sound, so no rows for them and no pages offering
/// them, because a control that does nothing is the dead affordance §15 weighs
/// heaviest (`orbs-shell`'s `settings::values` carries the argument). What is
/// left is real: `F4`, `F5`, how the orb reads a line, and §4's sticky skip.
pub(crate) fn settings(
    driver: orbs_shell::Driver,
    linear: bool,
    mode: DisplayMode,
) -> Vec<orbs_shell::settings::Row> {
    use orbs_shell::settings::{Category, Row, SKIP, SWITCH, switched};

    vec![
        Row::new(
            orbs_shell::settings::Category::Access,
            orbs_shell::LINEAR_SETTING,
            switched(linear),
            &SWITCH,
        ),
        // `reading` to a player, `driver` in the file — see `Row::key`.
        Row::new(
            Category::Habits,
            "reading",
            driver.word(),
            &orbs_shell::Driver::ALL.map(orbs_shell::Driver::word),
        )
        .keyed(orbs_shell::settings::DRIVER),
        Row::new(
            Category::Habits,
            "focus",
            mode.word(),
            &DisplayMode::ALL.map(DisplayMode::word),
        ),
        Row::new(
            Category::Habits,
            SKIP,
            switched(orbs_shell::settings::skips_boot()),
            &SWITCH,
        ),
    ]
}

/// Nothing reading spells: `plain`, or a build with no weights.
///
/// Abstains on every line, so `Read` is `Held`. The sim's own default, named
/// here rather than branched around, so there is one write path instead of two.
static VERBATIM: orbs_sim::Verbatim = orbs_sim::Verbatim;

/// The spell reader in effect: what this build offers, if the player wants it.
///
/// Two switches answering different questions, the pair `Readers::reader`
/// documents in the Bevy build: `ORBS_SCRIVENER` is a developer's override
/// saying what is on offer, the driver says whether it is consulted.
fn scribing(
    driver: orbs_shell::Driver,
    scribe: Option<&dyn orbs_sim::Scrivener>,
) -> &dyn orbs_sim::Scrivener {
    match driver {
        orbs_shell::Driver::Plain => &VERBATIM,
        orbs_shell::Driver::Augury => scribe.unwrap_or(&VERBATIM),
    }
}

/// How often the screen is redrawn at most.
///
/// Thirty rather than sixty: the animations are meters and a fire, which do not
/// read differently at the higher rate, and every frame is escape sequences down
/// a pipe that may be a network. The diff in [`blit`] does the real work.
const FRAME: Duration = Duration::from_millis(33);

/// Whether the terminal this process was given has gone away.
///
/// A leaked process that idles costs nothing; one that spins takes the machine.
/// When the pty master closes the slave reports `POLLHUP`, after which
/// `crossterm::event::poll` answers *ready* for ever and `event::read` loops
/// inside itself on the EOF rather than returning — so the process burns a
/// core. 573 of these once took a 32-core machine to a load average of 581, and
/// `term`'s `SIGHUP` handler cannot reach a child reparented away from a dead
/// terminal.
///
/// Asked of the descriptor, because crossterm has no way to say it, and asked
/// before the read, because the read that would be counted never returns. Zero
/// timeout, so an ordinary frame pays one non-blocking `poll`.
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
/// A thread because the loop never gets the chance. [`hung_up`] at the top of
/// each pass is kept, for the tidy exit through `term::leave` while the loop is
/// still turning — but `crossterm::event::read` on a dead pty never returns, so
/// a check there cannot run: measured at 82% CPU with one in place.
///
/// It exits hard, skipping `term::leave`, because everything `leave` does is a
/// write to a terminal just established as gone. The risk is a false positive
/// killing a live session, so `gone` answers only to `HUP`, `ERR` and `NVAL`;
/// both directions are tested.
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
/// `ORBS_LIFETIME`, in seconds, absent by default — a real game runs until
/// somebody stops it and nothing here times a player out mid-brew.
///
/// The play harness sets it, closing the half [`watch_for_hangup`] cannot reach:
/// a terminal perfectly alive while the harness that owned it is not. A detached
/// tmux server is nobody's child, so `SIGKILL`ing `cargo test` leaves its
/// sessions drawing at 30fps until something kills the server. An unreadable
/// value means no limit rather than zero, so a typo cannot exit every game half
/// a second after it starts.
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
/// silently never reports `HUP` is a guard that does nothing, and nothing else
/// notices. A pty slave whose master has closed reports `HUP | ERR` — measured,
/// because a pipe and a pty need not agree and it is the pty that matters.
fn gone<Fd: std::os::fd::AsFd>(fd: &Fd) -> bool {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};

    // No flags requested: a hangup is reported whether or not anything was asked
    // for. This asks *is the far end still there*, which crossterm cannot.
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
    /// Built once with the session, not per keystroke: reading the environment
    /// is not free and the answer cannot change mid-run.
    ///
    /// The trained reader, same as the Bevy build. This was empty while the
    /// reader dragged `wgpu` behind it; it does not — inference is `ndarray` at
    /// 436µs — so the terminal frontend is the whole game. `scripts/play.sh` can
    /// still set `stub` when a scenario wants a fixed answer.
    augury: Option<Box<dyn orbs_sim::Augur>>,
    /// The spell reader, on exactly the same terms.
    ///
    /// Both registers under one setting, because a player who has said *"read
    /// what I mean"* has said it about their whole session — `crate::sim::Readers`
    /// in the Bevy build is the same decision.
    scribe: Option<Box<dyn orbs_sim::Scrivener>>,
    /// Whether the player wants those readers consulted.
    ///
    /// Kept beside the readers rather than replacing them, for the Bevy build's
    /// reason: switching back must not cost a file read on a keystroke.
    driver: orbs_shell::Driver,
    /// The file this tower is kept in, and where it is written back.
    ///
    /// The path travels with the `Sim`, which is what stops loading a second
    /// game destroying the first: a path resolved at the moment of writing is
    /// one resolved *after* a swap. Here they are two fields of one struct
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
    /// Not the boundary being ducked. §14 requires motion be disableable and
    /// this build has no CRT to consult, so it passes `None` for the motion
    /// switch everywhere. An animating crossing would be the first motion here
    /// with no switch a player can reach, which `shell::bench` forbids; the
    /// persisted reduce-motion setting is a later phase's, and this comes back
    /// with it. The boundary is proven regardless — the shapes live in
    /// `orbs-render` and `--example screens` draws them.
    passing: Passing,
    frame: Frame,
    screen: blit::Screen,
    grid: GridSize,
    /// Whichever of the four has the keyboard, if any has.
    surfaces: Surfaces,
    /// How the main window divides among its panes — `F4`.
    ///
    /// Held here rather than rebuilt per frame, because it is a *setting* (§9)
    /// and `Screen` is assembled fresh in [`Session::draw`] — without somewhere
    /// to keep it, `F4` would flip the mode and the next frame flip it back.
    mode: DisplayMode,
    /// How many records the transcript last fitted, for `PageUp`.
    ///
    /// Measured by the painter, because only it knows how many records fit: a
    /// record is not one row, and paging by rows drops the lines in between.
    page: usize,
    /// How far §4's boot sequence has got.
    ///
    /// The clock is `orbs-shell`'s and only the driving is here, the split the
    /// Bevy build has: `Boot::advance` walks the stages, `Stage` owns their
    /// durations, and a frontend supplies a `Time` and a frame. `Boot::default`
    /// reads `ORBS_BOOT=0` itself, so the skip works here for free.
    boot: Boot,
    /// Whether a tower has been chosen yet.
    ///
    /// The same enum the Bevy build gates on, for `Boot`'s reason. At
    /// [`Threshold::Waiting`] the menu is up and cannot be closed,
    /// [`tick`](Session::tick) does not run and `kept` is `None`, so the scratch
    /// world cannot reach a file whatever else happens.
    threshold: Threshold,
    /// When a surface last handed the keyboard back, while arrows keep arriving.
    ///
    /// `HeldOver`, for a terminal with no key-up to hang it on: escape the maze
    /// with a finger still on an arrow and key repeat keeps delivering presses,
    /// and an arrow at the prompt means *recall history*, so `wander` reappears
    /// on the command line. The winit fix drops keys whose *press* went
    /// elsewhere until they are *released*, and a terminal sends no release.
    ///
    /// What it does give is *timing*: auto-repeat arrives every 30-40 ms and a
    /// person cannot press an arrow twice inside [`HELD_OVER`], so an unbroken
    /// run of arrows out of a handover is a held key and the first gap is the
    /// release.
    held_over: Option<Instant>,
    /// What this binary is built out of, for the card's third line.
    ///
    /// §4's card is a diegetic inventory of the machine, so it describes *this*
    /// one — `crossterm 0.29` where the other build says `bevy 0.19.0`. The
    /// frontend's to know and the shell's to place, as the wizard's name is.
    engine: String,
}

/// What a swap carries across, because it is the player's and not the tower's.
///
/// The terminal's answer to `reset_for_swap`'s three exceptions. `run` builds a
/// whole new `Session` per swap, so every field starts at its default and these
/// three defaults read `orbs-settings.toml` — making the file their only memory
/// across a swap, the failure the Bevy build lifted `Linear` out of
/// `shell_resources!` to prevent. With `ORBS_SAVE=off`, a read-only install or a
/// failed write, `F5` on and then choosing a tower turned §14's linear stream
/// off with nothing said; `reading plain` reverted the same way.
///
/// Nothing about a new tower is a reason to stop reading the screen aloud.
#[derive(Debug, Clone, Copy)]
struct Carried {
    driver: orbs_shell::Driver,
    /// Whether §14's linear stream is showing. A bool rather than the `Linear`
    /// itself, which owns a scratch `Frame` worth re-allocating per session.
    linear: bool,
    mode: DisplayMode,
}

impl Carried {
    /// What a first launch starts from: the file, read once.
    ///
    /// §9 opens Wide when nothing is remembered — the mode the Bevy build takes
    /// from `Screen::for_window` — and honours the player's choice otherwise.
    /// `focus` was write-only here until `0.16.12`: this build hard-coded `Wide`
    /// and wrote nothing, against `settings::FOCUS`.
    fn remembered() -> Self {
        Self {
            driver: orbs_shell::settings::driver(),
            linear: Linear::default().showing(),
            mode: orbs_shell::settings::get(orbs_shell::settings::FOCUS)
                .and_then(|word| DisplayMode::named(&word))
                .unwrap_or(DisplayMode::Wide),
        }
    }
}

impl Session {
    /// What this session hands to the next one.
    const fn carrying(&self) -> Carried {
        Carried {
            driver: self.driver,
            linear: self.linear.showing(),
            mode: self.mode,
        }
    }

    /// A session on this tower.
    ///
    /// `boot` is a parameter and not `Boot::default()`, which made every
    /// ordinary launch play the boot sequence twice: `run` builds a whole new
    /// `Session` per swap, `Boot::default()` returns the live thirteen-second
    /// sequence unless `ORBS_BOOT=0` or the sticky skip is set, and with the
    /// threshold the swap is the only path to a tower — so a player walked boot,
    /// menu, choose, boot again. Invisible to every instrument: the play suite
    /// and `scripts/tui.sh` both push `ORBS_BOOT=0` and the dump never reaches
    /// `play`. The Bevy build was unaffected — `Boot` is `BootPlugin`'s, not in
    /// `shell_resources!`.
    fn new(
        sim: Sim,
        kept: Option<std::path::PathBuf>,
        boot: Boot,
        threshold: Threshold,
        carried: Carried,
        grid: GridSize,
        narrow: bool,
        engine: String,
    ) -> Self {
        let mut panel = Panel::default();
        panel.refresh(&sim);
        let Carried {
            driver,
            linear: showing,
            mode,
        } = carried;
        let mut linear = Linear::default();
        linear.show(showing);
        Self {
            boot,
            threshold,
            held_over: None,
            engine,
            sim,
            augury: orbs_shell::augury(),
            scribe: orbs_shell::scrivener(),
            driver,
            kept,
            save_failed: false,
            line: Line::default(),
            offered: Offered::default(),
            ghost: String::new(),
            panel,
            scroll: Scroll::default(),
            bench: Bench::default(),
            linear,
            reveal: Reveal::default(),
            // A terminal has one pane and never animates between counts, so this
            // is settled from the first frame. `transition.rs`: *"`orbs-tui` is
            // free to ignore all of it."*
            panes: PaneTransition::settled(1),
            // Settled from the first frame and never advanced. See the field.
            passing: Passing::default(),
            frame: Frame::new(grid),
            screen: blit::Screen::new(grid, narrow),
            grid,
            surfaces: Surfaces {
                // The threshold's menu goes up with the session: no word opens
                // it, there is no prompt to type one at, and it cannot be closed
                // (`Stance::Threshold`) — so this is the only place it goes up.
                menuing: threshold.is_waiting().then(|| {
                    let mut menu = orbs_shell::Menu::at(orbs_shell::Stance::Threshold);
                    menu.show_driver(driver);
                    // The same rows the `menu` verb's opener hands over, so the
                    // page does not differ by which door you came in — and from
                    // the session, not hard-coded: passing `false` and `Wide`
                    // let the Access page report `linear [off]` while the pane
                    // drew the linear stream, §14's only visible control
                    // reporting the opposite of what was in effect.
                    menu.show_settings(settings(driver, showing, mode));
                    menu
                }),
                ..Surfaces::default()
            },
            mode,
            // A sane floor until the first paint measures the real one.
            page: 1,
        }
    }

    /// Advance the world by one tick, and re-read what the painters cache.
    ///
    /// `Panel` is rebuilt here rather than every frame for the reason the Bevy
    /// build guards it on `resource_changed::<Tower>`: it walks every room and
    /// allocates per recipe, for a table that changes at 1 Hz, and a hand-rolled
    /// loop has no change detection. Returns `false` when the session is over.
    fn tick(&mut self) -> bool {
        self.sim.step();
        // §8 permits a save at a tick boundary and nowhere else, and this is one
        // — `step` has returned, so `Pending` is drained and `Skip` is spent.
        if self.sim.tick().get().is_multiple_of(AUTOSAVE_TICKS) {
            self.keep();
        }
        // `quit` lands here, not at `submit`, and the delay is not worth
        // engineering away: every verb's *effect* runs at the next `step` (§19),
        // so quitting gets the same beat as every other word. The flag is only
        // set by a confirmed `quit` — the sim asks first — so nothing else to
        // check.
        if self.sim.quitting() {
            return false;
        }
        self.panel.refresh(&self.sim);
        // A verb may have asked for a surface, and an open one may have been
        // closed from under the player by a spell. `menu`'s handshake is taken
        // in here, by `Surfaces::open`.
        self.open_surfaces();
        self.surfaces.tick(&self.sim);
        if !self.answered() {
            return false;
        }
        self.ghost = self
            .line
            .ghost(self.sim.scene(), !self.sim.choices().is_empty());
        true
    }

    /// Take whatever surface a verb has asked for.
    ///
    /// One call site's worth of arguments in one place, because there are three:
    /// the tick, a keystroke a surface answered, and a line submitted. Three
    /// copies of seven arguments, which the settings rows made eight — and how
    /// one came to hand over a different list.
    fn open_surfaces(&mut self) {
        self.surfaces.open(
            &mut self.sim,
            &mut self.scroll,
            self.page,
            self.driver,
            self.linear.showing(),
            self.mode,
            scribing(self.driver, self.scribe.as_deref()),
        );
    }

    /// Act on what a surface asked for, and say whether the loop goes on.
    ///
    /// Not inside [`tick`](Self::tick) any more: the clock does not run before
    /// a tower is chosen, so neither does `tick`, and the menu could raise a
    /// swap nothing ever looked at — the lengths page stayed on screen for ever.
    /// Found by playing it, because a dump hosts no loop;
    /// `routing::the_threshold_reaches_a_tower` looks at it now. Once per pass
    /// rather than once per tick, so a swap or a `quit` lands on the frame the
    /// key arrived.
    fn answered(&mut self) -> bool {
        // Taken here rather than in the surface, for `Surfaces::driving`'s
        // reason: the reader belongs to the session. Unlike leaving and swapping
        // it does not end the loop — it changes how the next line is read.
        if let Some(driver) = self.surfaces.driving.take() {
            self.driver = driver;
        }
        // The two settings this build actually holds. `skip` is absent for the
        // Bevy build's reason: `Boot::default` reads it at startup and there is
        // nothing to do to a sequence that has already finished.
        if let Some((setting, value)) = self.surfaces.setting.take() {
            if setting == orbs_shell::LINEAR_SETTING {
                self.linear.show(orbs_shell::settings::is_on(&value));
            } else if setting == "focus"
                && let Some(mode) = DisplayMode::named(&value)
            {
                self.mode = mode;
            }
        }
        // The settings page says what is in effect, and stops the moment a
        // function key changes something behind it: the rows are a snapshot
        // taken when the menu opened and `F4`/`F5` are ungated. Without this
        // (`setting::follow_the_keys` in the Bevy build) `F5` left the row
        // saying `off` while the pane drew the linear stream. Only while a
        // settings page is up and only when something moved — the comparison is
        // what keeps it off the per-frame path.
        if self
            .surfaces
            .menuing
            .as_ref()
            .is_some_and(orbs_shell::Menu::showing_settings)
        {
            let fresh = settings(self.driver, self.linear.showing(), self.mode);
            if let Some(menu) = self.surfaces.menuing.as_mut()
                && menu.settings() != fresh.as_slice()
            {
                menu.show_settings(fresh);
            }
        }

        // The net under a menu that cannot be closed. `Stance::Threshold` makes
        // `Outcome::Close` unreachable, so this should never fire — but if
        // anything did close it the player would be stranded at the prompt of a
        // scratch world that never ticks and is never kept. The Bevy build has
        // had `open_at_the_threshold` since the feature shipped.
        if self.threshold.is_waiting() && self.surfaces.menuing.is_none() {
            tracing::error!(
                "the threshold's menu was closed and had to be put back up; \
                 something reached `MenuOutcome::Close` at `Stance::Threshold`",
            );
            let mut menu = orbs_shell::Menu::at(orbs_shell::Stance::Threshold);
            menu.show_driver(self.driver);
            menu.show_settings(settings(self.driver, self.linear.showing(), self.mode));
            self.surfaces.menuing = Some(menu);
        }

        // The menu's own `quit` leaves the loop the same way, and so does a
        // swap; `run` decides which of the three it was.
        !(self.surfaces.leaving || self.surfaces.swapping.is_some())
    }

    /// Take one keystroke. Returns `false` when the player asked to leave.
    fn typed(&mut self, event: KeyEvent) -> bool {
        // Escape with a letter behind it is two keystrokes and this build threw
        // the first away: crossterm reads the pty buffer in one syscall, and
        // `\x1b` plus any byte in the same read parses as `Alt+<that byte>` —
        // so leaving the spell editor and typing `quit` fast enough put `quit`
        // in the buffer as a line of the spell.
        //
        // Splitting it apart is lossless only here: with no keyboard-enhancement
        // flags (this build pushes none, see the `Repeat` note in `run`)
        // crossterm builds `Alt+Char` from an `\x1b <byte>` pair and nothing
        // else. A *real* Alt chord on a special key arrives CSI-encoded and is
        // left alone, which matters — `Alt+Left` is word-left in plenty of
        // terminals. The game binds none anyway, and the guard below excludes
        // Alt so AltGr can type `@`, `#` and `\`; on Linux AltGr composes in the
        // terminal and never reaches this branch.
        if let Some(rest) = escape_prefixed(&event) {
            if !self.typed(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)) {
                return false;
            }
            // One level deep and no further: `rest` has Alt cleared.
            return self.typed(rest);
        }
        // `Ctrl-H` is Backspace on a great many terminals and the chord guard
        // below was eating it: `stty erase ^H` (PuTTY's default) sends `0x08`,
        // crossterm turns every `0x01..=0x1A` byte into `Char(letter) +
        // CONTROL`, and §6 makes this a game played entirely by typing — so a
        // typo became uncorrectable short of clearing the line. Translated here
        // rather than special-cased in the guard, because every surface reads
        // `KeyCode::Backspace` and none should have to know this; the cost is a
        // deliberate `Ctrl-H`, which the game does not bind.
        let event = if event.code == KeyCode::Char('h')
            && event.modifiers.contains(KeyModifiers::CONTROL)
        {
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)
        } else {
            event
        };
        // F10 leaves, from anywhere — the Bevy build's binding, ungated there
        // too. Not Escape: with a text field on screen, Escape gets a player out
        // of something *smaller*, and all four surfaces use it that way. A
        // player who learns one build's exit must not be trapped in the other's.
        if event.code == KeyCode::F(10) {
            return false;
        }
        // The two that write about a session are gated on there being one: `F6`
        // exports a parse trace and `F7` mutates the sim, and at the threshold
        // both act on a scratch world the first swap throws away — *"pressed at
        // the menu it writes `orbs-parse.tsv` for a session that has not
        // happened"*. `F4`, `F5` and `F10` stay ungated for that build's reason:
        // they are about the screen, the menu is on it, and a way out that only
        // works once you are in is not one.
        if self.threshold.is_waiting() && matches!(event.code, KeyCode::F(6) | KeyCode::F(7)) {
            return true;
        }
        // The rest of the function keys, ungated by surface, exactly as the Bevy
        // build has them: `input_just_pressed` in `Update` with no surface
        // condition, so `F5` works from inside the editor and must here too.
        if self.shortcut(event.code) {
            return true;
        }
        // The chord guard, and it is one field read. Under winit this needs
        // `HeldOver`, `Quiet` and `chord_is_stale` — a press/release model, key
        // repeat, and a ghost modifier left behind when a key-up went to the
        // screenshot overlay — but a terminal carries the modifiers on the
        // event itself. Alt is deliberately absent, as on the other side: AltGr
        // is how European layouts type `@`, `#` and `\`.
        let chord = event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER);
        if chord {
            // Raw mode means the terminal does not turn this into SIGINT, so it
            // is ours to answer — and a full-screen application that cannot be
            // left by the one chord everybody tries is a trap.
            //
            // One accepted divergence: the Bevy build lets `Cmd+←/→` mean
            // Home/End, *"which a Mac keyboard has no other key for"*. Every
            // macOS emulator sends real Home and End on `Fn+←/→`, and a second
            // chord path would be a second place to forget a surface.
            return !matches!(event.code, KeyCode::Char('c' | 'd'));
        }

        // Exactly one surface takes a keystroke: two consuming a key is
        // invisible until a player types `:wq` and finds it in their history.
        //
        // The transcript scrolls without `unfurl` and from behind a surface —
        // the word exists because the *key* could not be discovered, not
        // because the key needs permission. Answered before the surface
        // dispatch, which is where this was wrong: below it the editor, the
        // weave screen and the maze swallowed both keys through their `_ => {}`,
        // while the Bevy build gates `scroll_back`/`scroll_forward` on `booted`
        // alone.
        //
        // Except the manual, the one surface that answers these keys itself;
        // without the guard PgDn pages the chapter *and* the transcript under
        // it. The Bevy build's is `manualling::not_reading_the_manual`.
        let total = self.sim.scrollback().records().drawn_len();
        match event.code {
            KeyCode::PageUp | KeyCode::PageDown if self.surfaces.reading.is_some() => {}
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
            self.surfaces.typed(
                owner,
                event.code,
                &mut self.sim,
                &mut self.scroll,
                scribing(self.driver, self.scribe.as_deref()),
            );
            // The one keystroke that reaches the world without a tick.
            // `Sim::walk` is the third entry point (§19): an arrow moves the
            // reading *now* and spends no world time, so it lands between the
            // ticks `Panel` is rebuilt on — and `panel.stacks` is what the
            // inline map and `wander`'s pane draw from, so without this the map
            // runs up to a second stale, invisibly to every test that drives
            // `Sim` directly. The Bevy build needs no such line: `walk` takes
            // `Tower` by `&mut` and `refresh_panel` hangs on
            // `resource_changed::<Tower>`. This is that, by hand.
            //
            // The whole panel, not just `stacks`: a walk can pick up a fragment
            // and can solve the maze outright, which moves the cabinet's stock
            // and the archive's rail box with it.
            if owner == Owner::Maze {
                self.panel.refresh(&self.sim);
            }
            // A surface may have opened another — `scribe` from the weave
            // screen cannot happen, but a save can close the editor and hand
            // the prompt back on the same keystroke.
            self.open_surfaces();
            return true;
        }

        let Some(key) = self.key(event.code) else {
            return true;
        };
        if let Some(finished) =
            orbs_shell::apply(&key, &mut self.line, &mut self.offered, &self.sim)
        {
            // Not a tick: `submit` echoes now and queues for the next `step`,
            // which keeps the echo instant while effects stay tick-aligned
            // (§19). `plain` leaves the reader loaded and does not ask it — see
            // `Session::driver`.
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
            self.open_surfaces();
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
    /// recall history — silently rewrites the command line, where a letter
    /// arriving this way is visible and can be deleted.
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
        // When the terminal says so, believe it: with keyboard-enhancement flags
        // active crossterm reports `Repeat` outright, which is what a key-release
        // event would have given — no timing involved.
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
    /// `Screen::is_hostable`, not a private copy of half of it: `term::fits`
    /// asked only the grid question where the shell asks two — the grid against
    /// the authoring floor *and* the scale — so the frontends routed to the "too
    /// small" card by different rules, invisibly to CI's `--dump` diff.
    fn hostable(&self) -> bool {
        self.screen().is_hostable()
    }

    /// The function keys that are neither the prompt nor a surface.
    ///
    /// Returns whether this key was one of them. `F4` is why it exists at all:
    /// `prompt.rs` draws `F4 deep` into the session border every frame and the
    /// terminal build shipped without answering it — §19's *"the first
    /// affordance the game showed was one that did not work"* a second time. The
    /// rules behind them are `orbs-shell`'s, so the two builds cannot disagree
    /// about the register cycle or what the trace file is called.
    fn shortcut(&mut self, code: KeyCode) -> bool {
        match code {
            // Flip the focus mode. Visibly this only moves the border's own hint
            // until multiplexing returns the second pane in Phase 11a — with one
            // pane both tilings are identical, which §19 records as deliberate,
            // and the Bevy build is equally inert and equally bound. Through
            // `DisplayMode::flipped`, so the two cannot disagree about what the
            // other mode is.
            KeyCode::F(4) => {
                self.mode = self.mode.flipped();
                // Written down, which this build did not do. `F4` and the `focus`
                // row are one setting seen twice, and the key that changed it
                // wrote nothing — so this page and the window's opening mode
                // both disagreed with what the player had just pressed.
                orbs_shell::settings::set(
                    orbs_shell::settings::FOCUS,
                    orbs_render::DisplayMode::word(self.mode),
                );
            }
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
            // §3's three tonal registers. Visibly inert here and bound anyway:
            // `Presentation` picks a *face* in the Bevy build's glyph atlas, and
            // a terminal's face is whatever the user set — one of this
            // frontend's accepted degradations (`theme.rs`). It still does the
            // same thing to the same world, and `F6`'s trace shows the register
            // move. A key that changed the world in one build and not the other
            // is the divergence worth avoiding; one that changes it invisibly is
            // the frontend limitation rule 2 permits.
            KeyCode::F(7) => {
                orbs_shell::cycle_register(&mut self.sim);
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
        // The card and nothing else until the orb is found. Painted before the
        // layout below rather than over it: `paint_booting` computes its own
        // single-pane layout, and the game's screen has a rail, a prompt and a
        // caret that must not be on this one — §4's `Prompt` stage was removed
        // for that reason.
        if !self.boot.is_live() {
            self.frame.reset(self.grid);
            // The floor applies to the card too, and it did not: this returned
            // before the hostable check below, so a small terminal watched the
            // logo run off the right edge for thirteen seconds before the card
            // explained why. `§19` calls a shrunk terminal a normal runtime
            // state, and it is normal during boot as well.
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
                // Settled, always — see the field. It still travels through here
                // because `paint` records each screen's regions in it, and
                // skipping that would make the shared painter take a different
                // path for this frontend.
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
                    // `&mut` for the editor's reason, one surface over: a
                    // chapter is wrapped to the pane and scrolled within it, and
                    // only the painter knows how wide and how tall it is.
                    reading_manual: self.surfaces.reading.as_mut(),
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
        // The shadow buffer goes with it: the diff is addressed by `(col, row)`,
        // so keeping it across a resize would write this frame's cells at last
        // frame's coordinates.
        self.screen.resize(self.grid);
        self.frame.reset(self.grid);
    }
}

/// How often the tower writes itself out, in world ticks.
///
/// The Bevy build's `sim::persist` picks the same number: §8 asks for *"every N
/// ticks and on significant events"*, and a minute is the most a crash may cost
/// in a game whose slowest single action is 94 ticks. One cadence for both, or
/// a player moving between them loses four times as much in one.
const AUTOSAVE_TICKS: u64 = 60;

impl Session {
    /// Write the tower out, and complain once if it will not go.
    ///
    /// Once, for the reason the other build gives: there is no `save` verb, so a
    /// silent failure is a whole session lost with nothing said — but a save is
    /// attempted every sixty ticks, so a line per attempt is sixty an hour.
    fn keep(&mut self) {
        // This tower's own path, never `save_path()` — see `Session::kept`.
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
pub(crate) fn run(sim: Sim, threshold: Threshold, engine: String) -> std::io::Result<()> {
    let narrow = term::symbols_are_narrow().unwrap_or(true);
    let grid = term::grid()?;
    // No path at the threshold, which is the structural guarantee:
    // `Session::keep` returns early without one, so a scratch world nobody asked
    // for cannot be written by the autosave or the way out. The swap below
    // installs the real path with the tower it belongs to.
    let kept = threshold.is_playing().then(orbs_shell::save_path).flatten();
    // The one session that plays the boot sequence: the first.
    let mut session = Session::new(
        sim,
        kept,
        Boot::default(),
        threshold,
        Carried::remembered(),
        grid,
        narrow,
        engine.clone(),
    );

    loop {
        let result = play(&mut session);

        // Every way out converges here, which is why the loop is a function of
        // its own. Five exits — the menu's `quit`, `F10`, `Ctrl-C`/`Ctrl-D`, an
        // `io::Error` off the terminal, and a swap — and only the first goes
        // through the `Quitting` flag, so saving at each `return` would have
        // covered one in five and looked complete. (The Bevy build reads
        // `AppExit` in `Last` for the same reason.) Before the swap below, so
        // the tower being left is written to the path it came from while
        // `session` still holds both.
        session.keep();

        let Some(asked) = session.surfaces.swapping.take() else {
            return result;
        };
        // An error off the terminal is not a thing to swap through.
        result?;

        // Read before the old session is dropped, so a save that will not open
        // leaves the player where they were rather than in a half-built world.
        // A new game gets a seed of its own (`orbs_shell::new_game_seed`), both
        // when asked for and when a slot turns out empty.
        let (raised, kept) = match asked {
            // The wizard's name, which this build used to drop: `Sim::begun`
            // takes no name where `Tower::begun` does, so a new game came up as
            // `orbs $ `. A corner while launching always restored a save; the
            // threshold makes every new game take this path.
            Swap::Begin { keep, length } => {
                let mut fresh = orbs_sim::Sim::begun(orbs_shell::new_game_seed(), length);
                if let Some(name) = wizard() {
                    fresh.rename(&name);
                }
                (fresh, keep)
            }
            Swap::Load(path) => match orbs_shell::read_save_from(&path) {
                orbs_shell::Opened::Restored(save) => {
                    let mut resumed = orbs_sim::Sim::restored(&save);
                    resumed.say_resumed(orbs_shell::away_for(&save));
                    (resumed, Some(path))
                }
                orbs_shell::Opened::Unreadable => {
                    tracing::error!("the tower in {} could not be read", path.display());
                    session.surfaces.menuing = Some({
                        // The options page marks what is *in effect*, which the
                        // session holds (`Menu::show_driver`), and the stance it
                        // had, because a refusal must leave the player where
                        // they were: here, a menu with no way out but a tower.
                        let mut menu = orbs_shell::Menu::at(if session.threshold.is_waiting() {
                            orbs_shell::Stance::Threshold
                        } else {
                            orbs_shell::Stance::InTower
                        });
                        menu.show_driver(session.driver);
                        // And the rows, which this rebuild was missing: without
                        // them every settings category answered *"unknown"*, and
                        // at the threshold there is nothing to leave to.
                        // `Surfaces::open` normally supplies these; this path
                        // builds its own menu and has to too.
                        menu.show_settings(settings(
                            session.driver,
                            session.linear.showing(),
                            session.mode,
                        ));
                        menu
                    });
                    continue;
                }
                orbs_shell::Opened::New => {
                    let mut fresh = orbs_sim::Sim::begun(
                        orbs_shell::new_game_seed(),
                        orbs_sim::content::Length::Medium,
                    );
                    if let Some(name) = wizard() {
                        fresh.rename(&name);
                    }
                    (fresh, Some(path))
                }
            },
        };

        // `ORBS_CONTENT` again, on the tower that just arrived. It was applied
        // once, to the scratch sim built before this loop — and the swap is the
        // *only* path to a tower, so a writer with the switch set silently got
        // the `include_str!` text back for every line and manual chapter, from
        // their fast no-GPU target. That is CLAUDE.md rule 6's whole workflow.
        let mut raised = raised;
        for file in orbs_shell::load(&mut raised) {
            tracing::warn!("content: {file} did not load; running on the built-in text");
        }

        // A whole new `Session`, the terminal's answer to the other build's
        // eighteen-resource reset: every derived field goes back to its default
        // because it is a new struct, so none can be forgotten. The grid is
        // re-read in case the terminal was resized while the menu was up, and
        // the threshold is crossed here because this is the one place a tower
        // the player asked for arrives.
        session = Session::new(
            raised,
            kept,
            // Already booted: the orb woke once and the player has been typing
            // at its menu since. See `Session::new`.
            Boot::finished(),
            Threshold::Playing,
            // Carried from the session being replaced, not re-read from the file
            // — see `Carried`. The file is a best-effort write, so re-reading
            // turned §14's linear stream off on the way into a tower with
            // nothing said.
            session.carrying(),
            term::grid().unwrap_or(grid),
            narrow,
            engine.clone(),
        );
    }
}

/// The loop itself, so [`run`] has somewhere to stand afterwards.
fn play(session: &mut Session) -> std::io::Result<()> {
    let mut out = stdout();

    // Asked for before the first frame, so a process killed during boot puts the
    // terminal back too. A failure to register is not worth refusing to start
    // over — the game runs, it just cannot tidy up after a signal.
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
        // This order is load-bearing, and the reason is `walk`, not `submit`. A
        // typed line is queued to the next `step` whatever order this loop runs
        // in, so its placement is latency, not replay.
        //
        // `Sim::walk` executes immediately, and `Sim::replay`'s contract is that
        // a `Submission::Walked` *"already ran, during its tick, after that
        // tick's step"*. Drain an arrow before stepping and it runs before tick
        // N's step while recorded against tick N, so a replay applies it after —
        // nothing fails, and the two frontends replay one seed into two worlds.
        //
        // Step first. The maze arrives in step D and this is already right for it.
        let now = Instant::now();
        // The tower's clock does not run during boot, and that is correctness
        // rather than cosmetics: `tower::drift` rolls once per tick, so a sim
        // left running through the sequence advances its RNG stream by a
        // wall-clock-dependent number of draws and the same seed reaches a
        // different world (`Stage::world_runs`). Holding `ticked` at `now` keeps
        // the catch-up below from replaying the sequence as a burst. Nor at the
        // threshold, one step out: how long somebody reads a menu for is
        // wall-clock time too, and `Threshold` is where the Bevy build spends it.
        if !session.boot.is_live() || session.threshold.is_waiting() {
            ticked = now;
        } else if now.duration_since(ticked) >= TICK {
            ticked += TICK;
            // Catching up rather than skipping, and capped: a suspended laptop
            // must not spend a minute of frames replaying an hour of ticks.
            if now.duration_since(ticked) >= TICK * CATCH_UP {
                ticked = now;
            }
            if !session.tick() {
                // No last frame, and drawing one was wishful: the paint lands on
                // the alternate screen `term::leave` tears down microseconds
                // later, and `F10`/`Ctrl-C` returned here without drawing at
                // all anyway. The record is the point and it is kept —
                // `execute::quit` puts it in the scrollback a player can
                // `peruse` next session, so a recording ends with someone
                // choosing to stop rather than simply stopping.
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
        // Before the read, which is the whole fix: a dead pty makes `poll`
        // answer ready for ever and `read` never return, so anything inspecting
        // the *result* of a read never runs. See [`hung_up`] for what 573 of
        // these cost once.
        if hung_up() {
            return Ok(());
        }
        if crossterm::event::poll(until_tick.min(until_frame))? {
            match crossterm::event::read()? {
                // Nothing is typed during boot — §19 removed the keypress skip
                // because the first thing a player does to the game should not
                // be dismissing it. *Leaving* is the exception, because raw mode
                // takes away the window manager's way out: `Ctrl-C` is ours to
                // answer or nobody's, and it ends the session rather than
                // jumping to the game, so it is not a skip.
                //
                // `Repeat` as well as `Press`, or auto-repeat is lost. crossterm
                // reports a third kind when keyboard-enhancement flags are
                // pushed — which this build never does, but the flags live on a
                // *terminal* stack, so a crashed editor leaves them set for
                // everything after it. In that state holding Backspace deleted
                // one character, and [`Session::held_over`] was dead code.
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

        // What a surface asked for, every pass and not every tick. At the
        // threshold the menu is the only surface there is and the clock does not
        // run, so a check inside `tick` was one nothing ever reached. See
        // [`Session::answered`].
        if !session.answered() {
            return Ok(());
        }

        // ── 3. The clocks the painters read, then the screen ──
        let now = Instant::now();
        if now.duration_since(painted) >= FRAME {
            let elapsed = now.duration_since(painted);
            let delta = elapsed.as_secs_f32();
            let overstep = now.duration_since(ticked).as_secs_f32() / TICK.as_secs_f32();
            // The sequence's own clock, advanced here and nowhere else. It walks
            // whole stages per call rather than one per frame, so a stall during
            // boot catches up instead of leaving the animation stuck part-way.
            session.boot.advance(elapsed);
            // `None` leaves whatever `ORBS_FIRE` said: this frontend has no CRT
            // switch to consult, and a missing switch is not a switch set to off.
            session
                .bench
                .advance(delta, overstep.clamp(0.0, 1.0), None, &session.panel);
            // After the keys, so a keystroke restarts the settle clock before it
            // is advanced — otherwise the frame a player types on counts toward
            // a pause they have not taken yet. §19's *"there is no `save`"*:
            // stop typing and the buffer writes itself out.
            session.surfaces.settle(
                delta,
                &mut session.sim,
                scribing(session.driver, session.scribe.as_deref()),
            );
            painted = now;
            session.draw(&mut out)?;
        }
    }
}

/// How many ticks behind the clock may fall before it stops catching up.
///
/// The Bevy build spends `Time<Virtual>`'s `max_delta` on the same question and
/// picks five seconds: above any frame hitch, below any real absence — and an
/// absence belongs to offline progression, which is Phase 11a's.
const CATCH_UP: u32 = 5;

/// One crossterm key, as every shared table understands it.
///
/// A free function, because all four surfaces need it and not only the prompt.
/// Mapping a backend's events onto [`Key`] is the genuinely backend-shaped half;
/// what each surface *does* with one is `orbs-shell`'s, and was written twice.
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
        // See `orbs/src/shell/input.rs`: both mappers were missing these for a
        // whole version while the manual drew `pgdn for more` and PageDown did
        // nothing. `_ => return None` swallows a new variant without a murmur.
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Char(glyph) => Key::Text(glyph.to_string()),
        _ => return None,
    })
}

/// The longest gap that still counts as one held arrow rather than two presses.
///
/// The fallback only, for where auto-repeat and a deliberate press are the same
/// event and only spacing tells them apart; where the terminal reports
/// `KeyEventKind::Repeat` the answer is exact.
///
/// Sixty, and it was a hundred and twenty. Auto-repeat lands every 30-40 ms, so
/// sixty catches it with room to spare — while 120 swallowed *scripted* input:
/// `scripts/tui.sh key` spaces presses 100 ms apart, so `key Escape Up Up` lost
/// both arrows. A repeat rate slower than this (`xset r rate 660 2`) breaks
/// through, which only a key-release event could fix.
///
/// See [`Session::held_over`].
const HELD_OVER: Duration = Duration::from_millis(60);

/// The keystroke hiding behind an `Alt+<letter>`, if that is what this is.
///
/// See the note at the top of [`Session::typed`]. `Some(rest)` means the
/// terminal sent `\x1b` and a byte in one read and the pair should be replayed
/// as Escape, then `rest`. Restricted to [`KeyCode::Char`] because that is the
/// only shape crossterm builds from an escape-prefixed byte; every other Alt
/// chord is CSI-encoded and is one the player really pressed.
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

/// Whether this key ends the session, asked while the boot card has the screen.
///
/// Not [`Session::typed`], which routes to surfaces, the prompt and the
/// shortcut table, none of which exist yet. §19's removed keypress skip is not
/// being put back — leaving closes the session rather than jumping to the tower
/// — and this exists because raw mode takes away the window manager's way out.
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

    /// Drive a sim the way [`run`] does — step, then take what the player did —
    /// and hand back the world it reached.
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
        // The hazard this loop's statement order exists for, asserted rather
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
        // ...and the proof is the maze itself: how much of it has been opened is
        // the whole of what a walk changes, and no other verb in this script can
        // move that count.
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
        // The other half, and why the comment in `run` is as long as it is: get
        // the order wrong and nothing fails. The same ticks pass and the same
        // squares are walked, so a test comparing either is green — the first
        // draft of this one compared ticks and proved nothing. What diverges is
        // the *recording*: a walk drained before the step is stamped with the
        // tick before the one it ran in, and replay stands on that.
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
    /// The half that silently does nothing if `poll` is asked wrongly: the
    /// request flags are empty on purpose, and getting that wrong gives a guard
    /// that never fires, which is the state this shipped in.
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
    /// rather than escape-prefixed; turning it into Escape would drop a player
    /// out of the editor — this split's own failure, from the other side.
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
