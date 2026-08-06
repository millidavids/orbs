//! Driving the boot sequence, and the condition everything else hangs off.
//!
//! Separate from `plugin.rs`, which does registration and nothing else
//! (CLAUDE.md). These are system bodies and a run condition.

use bevy::prelude::*;

use super::stage::Boot;

/// Let the sequence run.
///
/// Guarded on [`booting`], not unguarded. Removing the keypress skip left this
/// registered as a bare `add_systems(Update, advance)` — the only such system in
/// the workspace, against CLAUDE.md's *"every `Update` system has a `run_if()`
/// guard; never run unconditionally"* — so it kept adding to a finished
/// `Duration` every frame for the whole session.
pub(super) fn advance(time: Res<Time>, mut boot: ResMut<Boot>) {
    boot.advance(time.delta());
}

/// Run condition: the sequence is still playing.
pub(crate) fn booting(boot: Option<Res<Boot>>) -> bool {
    boot.is_some_and(|boot| !boot.stage().world_runs())
}

/// Run condition: the game is up.
///
/// Everything the player can press hangs off this, and so does the sim driver.
/// Without it `F6` would write a trace of a session that has not happened, a
/// keystroke would land in the input line before there is one, and the world
/// would tick through the animation — which is the one that is a correctness bug
/// rather than an annoyance. See
/// [`Stage::world_runs`](super::Stage::world_runs).
///
/// `Option<Res<_>>`, so an app without [`BootPlugin`](super::BootPlugin) behaves
/// as it did before the sequence existed rather than panicking on a missing
/// resource. The headless harnesses in `shell::plugin` and `sim::clock` build
/// exactly such an app — they test typing and the clock, not the splash — and
/// requiring every one of them to install a plugin they do not exercise would be
/// a tax paid forever for a resource that is absent in precisely one situation.
pub(crate) fn booted(boot: Option<Res<Boot>>) -> bool {
    boot.is_none_or(|boot| boot.stage().world_runs())
}

/// Log what the player is about to sit through.
///
/// The sequence is the one thing between launching and playing, so a run that
/// hangs part-way through it looks identical to a hang at startup. Naming its
/// budget once makes the difference visible in a log nobody has to opt into.
pub(super) fn announce(boot: Res<Boot>) {
    if boot.is_live() {
        info!("boot: skipped");
        return;
    }
    let total: core::time::Duration = super::Stage::SEQUENCE
        .iter()
        .map(|stage| stage.duration())
        .sum();
    info!("boot: {:.1}s", total.as_secs_f32());
}
