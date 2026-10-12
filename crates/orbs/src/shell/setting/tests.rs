use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use orbs_shell::settings::{self, Row};

use super::{Naming, Settable, Skipping, THEME, apply, defaults};
use crate::crt::CrtSettings;
use crate::render::{Theme, palette};
use crate::sight::Vision;

/// A world holding everything a row is read from, with a tube on a camera.
fn settable_world() -> World {
    let mut world = World::new();
    world.init_resource::<Theme>();
    world.init_resource::<Vision>();
    world.init_resource::<orbs_shell::Linear>();
    world.init_resource::<orbs_shell::Screen>();
    world.init_resource::<crate::sound::Levels>();
    world.init_resource::<Skipping>();
    world.init_resource::<crate::shell::Installs>();
    world.init_resource::<crate::shell::Stillness>();
    world.init_resource::<crate::crt::Picture>();
    world.init_resource::<crate::render::Contrast>();
    world.init_resource::<crate::render::Weight>();
    world.init_resource::<crate::shell::revealing::Paced>();
    world.init_resource::<crate::shell::echoing::Echoing>();
    world.init_resource::<crate::sound::Kinds>();
    world.init_resource::<crate::shell::Away>();
    world.init_resource::<Naming>();
    world.spawn(CrtSettings::DEFAULT);
    world
}

/// The rows, as `Settable` reads them out of `world`.
fn read(world: &mut World) -> Vec<Row> {
    world
        .run_system_once(|settable: Settable| settable.rows())
        .expect("the rows could not be read")
}

#[test]
fn every_row_is_honoured() {
    // §19's dead row: a setting the menu writes down and nothing applies.
    // Step each row and read the page back from what holds it. `reading` is
    // exempt — the menu answers it with `Drive`, never with `Set`.
    let mut world = settable_world();
    for row in read(&mut world) {
        if row.key == settings::DRIVER {
            continue;
        }
        // A row with no list takes what it is given — the name.
        let next = row.next().unwrap_or("Morgana");
        apply(&mut world, &row.key, next);
        let shown = read(&mut world)
            .into_iter()
            .find(|again| again.key == row.key)
            .expect("the row went away");
        // The name row shows what a new tower would take, and an exported
        // `ORBS_WIZARD` outranks the setting there.
        let named = (row.key == settings::NAME).then(|| orbs_shell::new_wizard(Some(next)));
        assert_eq!(
            shown.value,
            named.as_deref().unwrap_or(next),
            "`{}` was set and its row still says `{}`",
            row.word,
            shown.value,
        );
    }
}

#[test]
fn every_row_answers_to_its_first_letter() {
    // `Menu::set` refuses a letter two rows on one page share, and `back`
    // is read before any row, so a row starting with `b` loses its letter.
    for page in orbs_shell::settings::Category::ALL {
        let words: Vec<String> = defaults()
            .into_iter()
            .filter(|row| row.page == page)
            .map(|row| row.word)
            .collect();
        for word in &words {
            let first = &word[..1];
            assert_ne!(first, "b", "`{word}` starts with `back`'s letter");
            assert_eq!(
                words
                    .iter()
                    .filter(|other| other.starts_with(first))
                    .count(),
                1,
                "`{first}` names more than one row on the `{}` page",
                page.word(),
            );
        }
    }
}

#[test]
fn a_phosphor_number_sets_a_theme_no_list_offers() {
    let mut world = settable_world();
    let theme = |world: &mut World| {
        read(world)
            .into_iter()
            .find(|row| row.word == THEME)
            .expect("a theme row")
    };
    let secret = theme(&mut world)
        .secret("p7")
        .cloned()
        .expect("p7 is answered");
    apply(&mut world, THEME, &secret.value);
    assert_eq!(world.resource::<Theme>().0.name, "p7");
    let shown = theme(&mut world);
    assert_eq!(shown.value, "p7", "the row does not say what is in effect");
    assert!(
        !shown.values.contains(&"p7".to_owned()),
        "p7 was offered on the page",
    );
    for (number, phosphor) in palette::NUMBERED {
        assert!(
            palette::named(phosphor).is_some(),
            "{number} names a theme there is not",
        );
    }
}

#[test]
fn a_motion_word_this_build_does_not_know_is_still() {
    // §14: the quieter reading is the safe one.
    let mut world = settable_world();
    apply(&mut world, settings::MOTION, "gently");
    assert!(world.resource::<crate::shell::Stillness>().still());
}

#[test]
fn every_value_on_every_row_is_reachable_by_a_prefix() {
    // The lint that has to live here, because the values come from this
    // frontend's palette and `orbs-shell` cannot see them. `palette::ALL` is
    // `[amber, green, muted violet, monochrome]`, and while `value_named`
    // took the first prefix match, `theme m` meant muted violet and
    // monochrome — §14's accessibility phosphor — was unreachable by its
    // first letter. Now `m` is refused, and this holds that some prefix
    // still reaches every value.
    for row in defaults() {
        for value in &row.values {
            let reached = (1..=value.len())
                .map(|len| &value[..len])
                .any(|prefix| row.value_named(prefix) == Some(value.as_str()));
            assert!(
                reached,
                "`{}` on the `{}` row cannot be reached by any prefix of itself",
                value, row.word,
            );
        }
    }
}

#[test]
fn no_settings_page_is_taller_than_the_menu_can_draw() {
    // The check `orbs-shell` cannot make: its own version compared two
    // constants in the same file and could not fail. These rows are this
    // build's, assembled from what it has, and this crate is the only one
    // that can count them.
    //
    // `menu/paint.rs` refuses to draw below `MIN_ROWS`, sized for the
    // tallest page — so a page with more rows than fit is one whose last
    // setting a player at the 80×22 floor simply cannot see, which is §15's
    // dead end with a scrollbar-shaped hole where the fix would go.
    for page in orbs_shell::settings::Category::ALL {
        let rows = defaults().iter().filter(|row| row.page == page).count();
        assert!(
            rows <= orbs_shell::SETTINGS_ROWS as usize,
            "the `{}` page has {rows} rows and the menu draws {}",
            page.word(),
            orbs_shell::SETTINGS_ROWS,
        );
    }
}

#[test]
fn no_row_offers_a_value_it_is_not_set_to_one_of() {
    // A page shows `[value]` from the row, and cycling starts from it — so a
    // current value outside its own list makes `Row::next` wrap from the
    // start rather than step, which reads as the first keystroke doing
    // nothing. A row with no list takes free text and has nothing to cycle.
    for row in defaults().into_iter().filter(|row| !row.values.is_empty()) {
        assert!(
            row.values.contains(&row.value),
            "the `{}` row reads `{}`, which is not one of {:?}",
            row.word,
            row.value,
            row.values,
        );
    }
}
