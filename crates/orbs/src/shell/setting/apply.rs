//! Honouring what the menu writes down, and keeping an open page true.

use bevy::ecs::component::Mutable;
use bevy::prelude::*;

use orbs_shell::settings::{self, Pair, is_on};

use super::held::{CRT, Naming, Skipping, THEME, VOICE};
use super::rows::Settable;
use crate::crt::CrtSettings;
use crate::render::{Contrast, Theme, Weight, palette};
use crate::shell::echoing::Echoing;
use crate::sight::Vision;

/// Honour a setting the menu has just written down.
///
/// Now, not at the next launch: a player who turns the tube off and cannot see
/// it go off reasonably thinks nothing happened, which is the dead end §15
/// weighs heaviest arriving through a settings screen.
pub(crate) fn apply(world: &mut World, setting: &str, value: &str) {
    match setting {
        CRT => {
            if let Some(state) = crate::crt::state_named(value) {
                let mut tubes = world.query::<&mut CrtSettings>();
                for mut settings in tubes.iter_mut(world) {
                    *settings = state;
                }
            }
        }
        // Over every theme, offered or not — `palette::named` is the one lookup.
        THEME => {
            if let Some(phosphor) = palette::named(value)
                && let Some(mut theme) = world.get_resource_mut::<Theme>()
            {
                theme.0 = phosphor;
            }
        }
        settings::SIGHT => {
            if let Some(sight) = crate::sight::Sight::parse(value)
                && let Some(mut vision) = world.get_resource_mut::<Vision>()
            {
                vision.0 = sight;
            }
        }
        // `orbs-shell`'s word, because both builds bind `F5` and `F4` and both
        // must write the same key.
        orbs_shell::LINEAR_SETTING => {
            if let Some(mut linear) = world.get_resource_mut::<orbs_shell::Linear>() {
                linear.show(is_on(value));
            }
        }
        settings::FOCUS => {
            if let Some(mode) = orbs_render::DisplayMode::named(value)
                && let Some(mut screen) = world.get_resource_mut::<orbs_shell::Screen>()
            {
                screen.mode = mode;
            }
        }
        // Nothing takes effect this session — `Boot::default` read it at
        // startup — but the *row* has to become true or `follow_the_keys`
        // rebuilds it from a resource that never heard.
        settings::SKIP => put(world, Skipping(is_on(value))),
        // Asked at the next `new`, so this is all there is to honour.
        settings::INSTALL => put(world, crate::shell::Installs(is_on(value))),
        // A word this build does not know is still, for `settings::moves`'
        // reason.
        settings::MOTION => put(world, crate::shell::Stillness::asked(!is_on(value))),
        // `Screen` carries it, so the camera and the tube read one scale.
        settings::SCALE => {
            if let Some(mut screen) = world.get_resource_mut::<orbs_shell::Screen>() {
                screen.fit = orbs_shell::Fit::named(value).unwrap_or_default();
            }
        }
        crate::shell::Away::KEY => put(world, crate::shell::Away::named(value)),
        // Asked at the next `new`; a tower already standing keeps its name.
        settings::NAME => put(
            world,
            Naming(
                settings::checked_name(value)
                    .ok()
                    .filter(|name| !name.is_empty()),
            ),
        ),
        // `keep_echo` hands it to the tower's stream.
        Echoing::KEY => put(world, Echoing::named(value)),
        Contrast::KEY => put(world, Contrast::named(value)),
        Weight::KEY => put(world, Weight::named(value)),
        orbs_shell::settings::PACE => put(
            world,
            crate::shell::revealing::Paced(orbs_shell::Pace::named(value).unwrap_or_default()),
        ),
        VOICE => {
            if let Some(mut levels) = world.get_resource_mut::<crate::sound::Levels>() {
                // `set` decides whether the word is one this build knows, so a
                // level from a later build's settings file leaves the current
                // one alone rather than muting the game.
                levels.set(value);
            }
        }
        // The four kinds of sound and the picture's seven, by name. `reading`
        // never arrives here: the menu answers it with `MenuOutcome::Drive`,
        // which `menuing` acts on, because *stop consulting a reader now* is not
        // something a generic setting can say.
        _ => {
            if let Some(kind) = crate::sound::Kind::named(setting) {
                if let Some(mut kinds) = world.get_resource_mut::<crate::sound::Kinds>() {
                    kinds.set(kind, is_on(value));
                }
            } else if let Some(effect) = crate::crt::Effect::named(setting)
                && let Some(mut picture) = world.get_resource_mut::<crate::crt::Picture>()
            {
                picture.set(effect, crate::crt::Strength::read(value));
            }
        }
    }
}

/// Replace the resource `held` is, if this world has one.
fn put<T: Resource<Mutability = Mutable>>(world: &mut World, held: T) {
    if let Some(mut resource) = world.get_resource_mut::<T>() {
        *resource = held;
    }
}

/// Keep an open settings page telling the truth.
///
/// The rows are a snapshot, and `F3` works over an open page, so refresh them
/// rather than cycle from a stale value. Only when something moved, so
/// `Standing` stays unchanged.
pub(crate) fn follow_the_keys(settable: Settable, mut standing: ResMut<crate::shell::Standing>) {
    let fresh = settable.rows();
    if standing
        .bypass_change_detection()
        .get_mut()
        .is_some_and(|menu| menu.follow(fresh))
    {
        standing.set_changed();
    }
}
