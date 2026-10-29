//! The `picture` page: each of the tube's effects on its own.
//!
//! A ceiling on whatever state the tube is in, never a second tube: `crt` and
//! `F3` still choose the state, and these say how much of each effect it may
//! use. [`effective`](super::effective) is the one place the two meet.

use bevy::prelude::*;

/// How much of one effect the player allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Strength {
    Off,
    Soft,
    Full,
}

impl Strength {
    /// All three, in the order a row cycles them.
    pub(crate) const ALL: [Self; 3] = [Self::Off, Self::Soft, Self::Full];

    /// The word for it, on the page and in the file.
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Soft => "soft",
            Self::Full => "full",
        }
    }

    /// The strength `word` names. One this build does not know is off: the
    /// quieter reading, as for `motion`.
    pub(crate) fn read(word: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|strength| strength.word() == word)
            .unwrap_or(Self::Off)
    }

    /// How much of the effect survives.
    pub(crate) const fn share(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Soft => 0.5,
            Self::Full => 1.0,
        }
    }
}

/// The effects a player can temper, one row each.
///
/// No two share a first letter and none starts with `b`, which `back` takes
/// first. The rounded bezel is not here: it is the shape of the glass rather
/// than an effect on it, and it loses no cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Effect {
    /// The barrel.
    Curve,
    /// The dark line between scan rows.
    Lines,
    /// Phosphor bloom, and the afterglow: the phosphor in space and in time.
    Glow,
    /// The slow hum band — the one effect that moves.
    Flicker,
    /// The aperture grille.
    Mask,
    /// Convergence error, the coloured fringe.
    Split,
    /// The vignette.
    Edge,
}

impl Effect {
    /// Every effect, in the order the page lists them.
    pub(crate) const ALL: [Self; 7] = [
        Self::Curve,
        Self::Lines,
        Self::Glow,
        Self::Flicker,
        Self::Mask,
        Self::Split,
        Self::Edge,
    ];

    /// The word a player types, which is also its key in the settings file.
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::Curve => "curve",
            Self::Lines => "lines",
            Self::Glow => "glow",
            Self::Flicker => "flicker",
            Self::Mask => "mask",
            Self::Split => "split",
            Self::Edge => "edge",
        }
    }

    /// The effect `word` names, whole.
    pub(crate) fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|effect| effect.word() == word)
    }

    const fn at(self) -> usize {
        self as usize
    }
}

/// What the player allows of each effect.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Picture([Strength; Effect::ALL.len()]);

impl Picture {
    /// Every effect at full: the tube exactly as its state says.
    pub(crate) const FULL: Self = Self([Strength::Full; Effect::ALL.len()]);

    /// How much of `effect` is allowed.
    pub(crate) const fn of(self, effect: Effect) -> Strength {
        self.0[effect.at()]
    }

    /// Allow `strength` of `effect`.
    pub(crate) const fn set(&mut self, effect: Effect, strength: Strength) {
        self.0[effect.at()] = strength;
    }
}

impl Default for Picture {
    /// What the player chose, read once. Nothing chosen is full.
    fn default() -> Self {
        // One read of the file, not one per effect.
        let kept = orbs_shell::settings::load();
        let mut picture = Self::FULL;
        for effect in Effect::ALL {
            if let Some(word) = kept.get(effect.word()) {
                picture.set(effect, Strength::read(word));
            }
        }
        picture
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_effect_is_reached_by_its_first_letter() {
        // `Menu::set` refuses a letter two rows share, and `back` takes `b`.
        for effect in Effect::ALL {
            let first = &effect.word()[..1];
            assert_ne!(
                first,
                "b",
                "`{}` starts with `back`'s letter",
                effect.word()
            );
            let sharing = Effect::ALL
                .iter()
                .filter(|other| other.word().starts_with(first))
                .count();
            assert_eq!(sharing, 1, "`{first}` names more than one effect");
        }
    }

    #[test]
    fn a_strength_this_build_does_not_know_is_off() {
        assert_eq!(Strength::read("medium"), Strength::Off);
        for strength in Strength::ALL {
            assert_eq!(Strength::read(strength.word()), strength);
        }
    }
}
