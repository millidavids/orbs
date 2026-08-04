//! Registration for the boot sequence.

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::stage::Boot;

/// The orb waking up (DESIGN.md §4).
pub struct BootPlugin;

impl Plugin for BootPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boot>()
            .add_systems(Startup, announce)
            .add_systems(
                Update,
                (advance, skip.run_if(on_message::<KeyboardInput>)).chain(),
            );
    }
}

impl Default for Boot {
    fn default() -> Self {
        // `ORBS_BOOT=0` lands straight in the game. Boot happens once per launch,
        // so without this every "see it" pass on anything else costs a four-second
        // wait — and CLAUDE.md's warning about unmaintained debug affordances is
        // about inventing surfaces nobody uses, not about the one that makes the
        // gate cheap to run.
        if std::env::var("ORBS_BOOT").is_ok_and(|value| value == "0") {
            return Self::finished();
        }
        Self::new()
    }
}

fn advance(time: Res<Time>, mut boot: ResMut<Boot>) {
    boot.advance(time.delta());
}

/// Any key goes straight to the game.
///
/// Deliberately *any*, including the ones bound to something else: `F10` quits,
/// and a player reaching for it during boot means to skip, not to leave. Every
/// other keyed system is gated on [`booted`] for the same reason, so this is the
/// only thing a keystroke can do until the game is up.
fn skip(mut boot: ResMut<Boot>) {
    boot.skip();
}

/// Run condition: the game is up.
///
/// Everything the player can press hangs off this, and so does the sim driver.
/// Without it `F10` during boot would skip *and* quit, `F6` would write a trace
/// of a session that has not happened, a keystroke meant to skip would also land
/// in the input line, and the world would tick through the animation — which is
/// the one that is a correctness bug rather than an annoyance. See
/// [`Stage::world_runs`](super::Stage::world_runs).
/// `Option<Res<_>>`, so an app without [`BootPlugin`] behaves as it did before
/// the sequence existed rather than panicking on a missing resource. The
/// headless harnesses in `shell::plugin` and `sim::clock` build exactly such an
/// app — they test typing and the clock, not the splash — and requiring every
/// one of them to install a plugin they do not exercise would be a tax paid
/// forever for a resource that is absent in precisely one situation.
pub(crate) fn booted(boot: Option<Res<Boot>>) -> bool {
    boot.is_none_or(|boot| boot.stage().world_runs())
}

/// Log what the player is about to sit through.
///
/// The sequence is the one thing between launching and playing, so a run that
/// hangs part-way through it looks identical to a hang at startup. Naming its
/// budget once makes the difference visible in a log nobody has to opt into.
fn announce(boot: Res<Boot>) {
    if boot.is_live() {
        info!("boot: skipped");
        return;
    }
    let total: core::time::Duration = super::Stage::SEQUENCE
        .iter()
        .map(|stage| stage.duration())
        .sum();
    info!("boot: {:.1}s, any key skips", total.as_secs_f32());
}
