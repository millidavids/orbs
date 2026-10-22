//! The loop: a clock, a keyboard, and a repaint.
//!
//! What the Bevy build spends a system graph and four `SystemSet` edges on is
//! straight-line statement order here, so most ordering hazards stop being
//! hazards. One does not — see [`play`]'s first step and the two tests that hold
//! it.

use std::io::stdout;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyEventKind};
use orbs_shell::{Boot, Threshold};
use orbs_sim::Sim;

use crate::carried::Carried;
use crate::hangup::{hung_up, watch_for_hangup};
use crate::keymap::leaving;
use crate::session::Session;
use crate::term;

/// One world tick, which DESIGN.md §5.0 makes one real second.
const TICK: Duration = Duration::from_secs(1);

/// How often the screen is redrawn at most.
///
/// Thirty rather than sixty: the animations are meters and a fire, which do not
/// read differently at the higher rate, and every frame is escape sequences down
/// a pipe that may be a network. The diff in `blit` does the real work.
const FRAME: Duration = Duration::from_millis(33);

/// How many ticks behind the clock may fall before it stops catching up.
///
/// The Bevy build spends `Time<Virtual>`'s `max_delta` on the same question and
/// picks five seconds: above any frame hitch, below any real absence — and an
/// absence belongs to offline progression, which is Phase 11a's.
const CATCH_UP: u32 = 5;

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
    // Once per process, not per session: `play` runs again after every swap,
    // and each pass registered three more signal handlers and left one more
    // watchdog thread polling. Asked for before the first frame, so a process
    // killed during boot puts the terminal back too; a failure to register is
    // not worth refusing to start over.
    let dying = term::dying().unwrap_or_default();
    watch_for_hangup();
    // The one session that plays the boot sequence: the first.
    let remembered = Carried::remembered();
    let mut session = Session::new(
        sim,
        kept,
        Boot::default().held_still(!remembered.motion),
        threshold,
        remembered,
        grid,
        narrow,
        engine,
    );

    loop {
        let result = play(&mut session, &dying);

        // Every way out converges here, which is why the loop is a function of
        // its own. Five exits — the menu's `quit`, `F10`, `Ctrl-C`/`Ctrl-D`, an
        // `io::Error` off the terminal, and a swap — and only the first goes
        // through the `Quitting` flag, so saving at each `return` would have
        // covered one in five and looked complete. (The Bevy build reads
        // `AppExit` in `Last` for the same reason.) Before the swap below, so
        // the tower being left is written to the path it came from while
        // `session` still holds both.
        session.keep();

        let Some(asked) = session.take_swap() else {
            return result;
        };
        // An error off the terminal is not a thing to swap through.
        result?;
        if let Some(next) = session.raise(asked) {
            session = next;
        }
    }
}

/// The loop itself, so [`run`] has somewhere to stand afterwards. `dying` is
/// set by a signal asking the process to stop.
fn play(session: &mut Session, dying: &AtomicBool) -> std::io::Result<()> {
    let mut out = stdout();
    let start = Instant::now();
    let mut ticked = start;
    let mut painted = start - FRAME;
    session.draw(&mut out)?;

    loop {
        // Leave the way `quit` does, so `term::leave` runs on the ordinary path.
        if dying.load(Ordering::Relaxed) {
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
        // Holding `ticked` at `now` while the clock is stopped keeps the
        // catch-up below from replaying the boot card or the menu as a burst.
        // See `Session::clock_runs`.
        if !session.clock_runs() {
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
                // one character, and `HeldOver` was dead code.
                Event::Key(key)
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                {
                    if !session.booted() {
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
        // `Session::answered`.
        if !session.answered() {
            return Ok(());
        }

        // ── 3. The clocks the painters read, then the screen ──
        let now = Instant::now();
        if now.duration_since(painted) >= FRAME {
            let elapsed = now.duration_since(painted);
            let overstep = now.duration_since(ticked).as_secs_f32() / TICK.as_secs_f32();
            painted = now;
            session.frame(elapsed, overstep, &mut out)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use orbs_sim::Sim;
    use orbs_sim::tower::Way;

    /// Drive a sim the way [`super::run`] does — step, then take what the player
    /// did — and hand back the world it reached.
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
        // The other half, and why the comment in `play` is as long as it is: get
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
