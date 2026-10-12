//! The wizard's name — `settings` → `habits` → `name`.
//!
//! The name a *new* tower is raised with. A tower keeps the name it was given:
//! a save outranks the machine (`orbs_sim::Wizard`), so this never renames one
//! already standing. Until a player chooses, the machine's login stands in,
//! which is what the prompt has always shown.

use super::get;

/// The key the name is kept under, which is also the row's word.
pub const NAME: &str = "name";

/// The longest name the prompt carries.
///
/// It shares the line with the location and what is being typed, and a name
/// longer than this is one the 80-column floor has to cut.
pub const LONGEST_NAME: usize = 16;

/// Why a name was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameFault {
    /// Longer than [`LONGEST_NAME`].
    Long,
    /// More than one word — the prompt reads up to the first space.
    Spaced,
    /// A letter the orb's glass cannot draw, counted from 1.
    ///
    /// Its position rather than the letter: a glyph the glass cannot draw in a
    /// name, it cannot draw in the refusal either.
    Undrawable(usize),
}

/// `typed`, if it can be a name.
///
/// Case is kept: it is the player's name, not a command.
///
/// # Errors
///
/// The first thing wrong with it, as a [`NameFault`] the menu turns into a
/// sentence.
pub fn checked_name(typed: &str) -> Result<String, NameFault> {
    let name = typed.trim();
    if name.split_whitespace().nth(1).is_some() {
        return Err(NameFault::Spaced);
    }
    if let Some(at) = name
        .chars()
        .position(|glyph| !orbs_render::cp437::is_renderable(glyph))
    {
        return Err(NameFault::Undrawable(at + 1));
    }
    if name.chars().count() > LONGEST_NAME {
        return Err(NameFault::Long);
    }
    Ok(name.to_owned())
}

/// The name the player chose, as the file says, if they chose one.
///
/// Checked again on the way in: a file is plain text and anyone may edit it,
/// and a name the menu would refuse should not reach a prompt by another door.
#[must_use]
pub fn chosen_name() -> Option<String> {
    get(NAME)
        .filter(|name| !name.trim().is_empty())
        .and_then(|name| checked_name(&name).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_keeps_its_case_and_loses_its_margins() {
        assert_eq!(checked_name("  Morgana "), Ok("Morgana".to_owned()));
    }

    #[test]
    fn a_name_is_one_drawable_word_of_sixteen_letters_or_fewer() {
        assert_eq!(checked_name("Morgan le Fay"), Err(NameFault::Spaced));
        assert_eq!(
            checked_name("Mörgana"),
            Ok("Mörgana".to_owned()),
            "ö is in the codepage"
        );
        assert_eq!(checked_name("Morgana✨"), Err(NameFault::Undrawable(8)));
        assert_eq!(
            checked_name("Mö✨"),
            Err(NameFault::Undrawable(3)),
            "counted in letters, not bytes"
        );
        assert_eq!(
            checked_name(&"a".repeat(LONGEST_NAME)).map(|name| name.len()),
            Ok(16)
        );
        assert_eq!(
            checked_name(&"a".repeat(LONGEST_NAME + 1)),
            Err(NameFault::Long)
        );
    }
}
