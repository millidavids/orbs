//! Echo verbosity (§6, §14) — whether an echo that only repeats the line is
//! drawn.
//!
//! The stream decides which echoes those are (`Records::drawn`); this only
//! keeps the tower's stream told what the player chose, swap or not.

use bevy::prelude::*;
use orbs_shell::{Reveal, Scroll};

use crate::sim::Tower;

super::setting::pair! {
    /// Whether echoes that only repeat the line are left out — `settings` →
    /// `access` → `echo`. Anything but `changed` shows every echo.
    pub(crate) struct Echoing(pub(crate) bool) =
        orbs_shell::settings::ECHO, orbs_shell::settings::ECHOES;
}

/// Tell the tower's stream, whenever it does not already know.
///
/// Every frame, since a swap brings a fresh stream; asked through `sim()` first
/// so nothing is marked changed. A change of row resets the reveal and scroll,
/// which count drawn records.
pub(crate) fn keep_echo(
    echoing: Res<Echoing>,
    mut tower: ResMut<Tower>,
    mut reveal: ResMut<Reveal>,
    mut scroll: ResMut<Scroll>,
) {
    if tower.sim().scrollback().records().terse_echo() == echoing.0 {
        return;
    }
    tower.set_terse_echo(echoing.0);
    if echoing.is_changed() && !echoing.is_added() {
        reveal.settle_at(tower.sim().scrollback().records().drawn_len());
        scroll.rewind();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echoes_put_back_by_the_row_do_not_type_themselves() {
        let mut app = App::new();
        app.insert_resource(Tower::new(1))
            .insert_resource(Echoing(true))
            .init_resource::<Reveal>()
            .init_resource::<Scroll>()
            .add_systems(Update, keep_echo);
        app.update();
        app.world_mut().resource_mut::<Tower>().submit("survey");
        let drawn = |app: &App| {
            app.world()
                .resource::<Tower>()
                .sim()
                .scrollback()
                .records()
                .drawn_len()
        };
        let before = drawn(&app);
        app.world_mut().resource_mut::<Reveal>().settle_at(before);

        *app.world_mut().resource_mut::<Echoing>() = Echoing(false);
        app.update();
        let after = drawn(&app);
        assert!(after > before, "no echo was hidden to put back");
        // The next frame's look, with no time for anything to type.
        let mut reveal = *app.world().resource::<Reveal>();
        let tower = app.world().resource::<Tower>();
        reveal.drive(
            tower.sim().scrollback().records(),
            0.0,
            Some(true),
            orbs_shell::Pace::default(),
        );
        assert!(reveal.is_settled(), "the echoes put back began a burst");
    }
}
