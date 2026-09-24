//! Where the menu is standing: over a tower, or in front of none.
//!
//! One menu, two stances, not two menus. The orb's menu was reachable only from
//! inside a game; the threshold needs the same screen in front of no game, and
//! the difference is small enough to be a field — which words the top page
//! offers, and whether there is anything to go back to. A second front-of-house
//! screen would have duplicated `words.rs`'s prefix invariant, the complaint
//! sentences, the floor, the swap and every keystroke path to change four words
//! (§19).
//!
//! The stance gates the way out, which is the load-bearing half.
//! [`Menu::escape`](super::Menu::escape) answers the top page with
//! `Outcome::Close` and both frontends take the menu down. Over a tower that is
//! right; at the threshold there is nothing behind it but a scratch world that
//! will never tick and never be saved, so closing strands the player at a prompt
//! with no way back and no way out.
//!
//! So at [`Stance::Threshold`] the menu cannot close: `resume` is not a word it
//! answers to, and Escape on the top page does nothing.

/// Where the menu is standing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Stance {
    /// Over a game. `resume` goes back to it and Escape is `resume`.
    ///
    /// The default, because `Menu::default()` is what four frontend tests and
    /// the dump's `menued` build, and every one means the menu a player opened
    /// with the `menu` verb. The threshold is the stance something asks for.
    #[default]
    InTower,
    /// In front of no game at all, after the boot card.
    ///
    /// Nothing is behind it, so nothing closes it.
    Threshold,
}

impl Stance {
    /// Whether there is a tower behind the menu to go back to.
    ///
    /// The one question the rest of the menu asks: it decides whether `resume`
    /// is a word, and whether Escape on the top page is an answer or a no-op.
    #[must_use]
    pub const fn may_close(self) -> bool {
        matches!(self, Self::InTower)
    }

    /// Whether the player has yet to choose a tower.
    #[must_use]
    pub const fn is_threshold(self) -> bool {
        matches!(self, Self::Threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_the_menu_over_a_tower() {
        // `Menu::default()` is built by the frontend tests and by `dump::menued`,
        // and all of them mean the menu the `menu` verb opens. A default of
        // `Threshold` would silently change what every one of them tests.
        assert_eq!(Stance::default(), Stance::InTower);
        assert!(Stance::default().may_close());
    }

    #[test]
    fn only_the_tower_stance_has_somewhere_to_go_back_to() {
        assert!(Stance::InTower.may_close());
        assert!(!Stance::Threshold.may_close());
        assert!(Stance::Threshold.is_threshold());
        assert!(!Stance::InTower.is_threshold());
    }
}
