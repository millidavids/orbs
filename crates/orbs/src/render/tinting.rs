//! The tint, from the sim's own world through to a resolved colour.
//!
//! **The only test that crosses all three crates**, which is where this can fail
//! with every single-crate test still green: `materials.toml` says sage is green,
//! [`super::tint`] says what green is, and `orbs_shell::panel` is the one thing
//! that connects them. Each half passes on its own while the bar draws in the
//! base hue — a total, invisible failure, because an untinted material draws in
//! the base hue too.
//!
//! # Why it lives here rather than beside the painter
//!
//! It reaches [`super::palette`] and [`super::tint`], which resolve a `Style` to
//! a `bevy::Color` and are this frontend's alone (rule 2). The painter is shared
//! and must not name them.
//!
//! It is a `#[cfg(test)]` module rather than `crates/orbs/tests/` because
//! **`orbs` is a binary crate**: it has `src/main.rs` and no `lib.rs`, so an
//! integration test cannot link it at all. CLAUDE.md records that exact trap for
//! `orbs-balance`, which is *why* its tests had been written driving `Sim` by
//! hand and asserting nothing about the harness.

#[cfg(test)]
mod tests {
    use orbs_render::{Depiction, Frame, GridSize, Intensity, Pos, Rect, Role, Tint, Wash};
    use orbs_sim::Sim;

    use orbs_shell::{Bench, panel};

    use crate::render::{palette, tint};

    #[test]
    fn what_is_in_an_instrument_colours_its_bar() {
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
        let split = panel::split(area, &instruments);
        panel::paint(
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
                    theme.resolve_tinted(cell.style, frame.tint_at(*at), frame.lit_at(*at))
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
}
