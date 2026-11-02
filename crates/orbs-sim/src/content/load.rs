//! Reading an authored file, once, for all three of them.
//!
//! [`Prose`](super::Prose), [`Recipes`](super::Recipes) and [`Fuels`](super::Fuels)
//! each had their own `BUILTIN` include, their own `builtin()`, their own
//! `parse()`, and their own single-variant error enum wrapping the same
//! `toml::de::Error`. The code was duplicated; more to the point the *policy*
//! was — the `expect` message on a malformed built-in, and the "keep the last
//! good text rather than fall back to silence" contract that a hot reload
//! depends on. A fourth content file meant copying thirty lines and inventing a
//! fourth `XError`.

use serde::de::DeserializeOwned;

/// Why an authored file could not be used.
///
/// Carries **which file**, which the three separate error types could only say
/// in their message text — so a caller could not tell them apart without
/// matching on a string.
///
/// Two kinds, because a file can be well-formed TOML and still wrong.
/// `materials.toml` selecting a colour that does not exist parses perfectly and
/// is unusable, and the writer needs to be told which name and what the real
/// ones are — a `toml::de::Error` has no room to say either.
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    /// The text is not valid TOML of the expected shape.
    #[error("{file} is not valid content: {source}")]
    Malformed {
        /// The file's name, as a writer would look for it.
        file: &'static str,
        /// What the parser objected to.
        #[source]
        source: toml::de::Error,
    },
    /// The text parsed, and says something the game cannot act on.
    #[error("{file}: {message}")]
    Invalid {
        /// The file's name, as a writer would look for it.
        file: &'static str,
        /// What is wrong, in terms a writer can fix.
        message: String,
    },
}

impl ContentError {
    /// A well-formed file that says something impossible.
    pub(super) const fn new(file: &'static str, message: String) -> Self {
        Self::Invalid { file, message }
    }
}

/// Parse an authored file.
///
/// # Errors
///
/// [`ContentError`] if the text is not valid TOML of the expected shape. A
/// frontend hot-reloading a file the writer just broke should keep the content
/// it already has and report this, **never** fall back to silence: a writer
/// mid-edit saves broken TOML constantly, and going mute is the worst possible
/// answer at the exact moment someone is looking at the screen.
pub(super) fn parse<T: DeserializeOwned>(
    file: &'static str,
    text: &str,
) -> Result<T, ContentError> {
    toml::from_str(text).map_err(|source| ContentError::Malformed { file, source })
}

/// Parse a file that ships with the crate.
///
/// # Panics
///
/// If the built-in file is malformed. That is a build-time authoring error, not
/// a runtime condition — each content module has a `the_builtin_file_parses`
/// test that fails first.
pub(super) fn builtin<T: DeserializeOwned>(file: &'static str, text: &str) -> T {
    match parse(file, text) {
        Ok(parsed) => parsed,
        Err(error) => panic!("the built-in {file} is authored with the crate: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        Charms, Fuels, Manual, Materials, Phrasings, Prose, Recipes, Spells, Trials,
    };
    use super::{ContentError, parse};

    /// A key nothing reads. A string, so its value cannot be what is refused.
    const STRAY: &str = "mistyped = \"x\"";

    /// `text` with the stray key as the first thing under `head`.
    fn under(text: &str, head: &str) -> String {
        text.replacen(head, &format!("{head}\n{STRAY}"), 1)
    }

    /// `text` with the stray key above its first table.
    fn above(text: &str) -> String {
        format!("{STRAY}\n{text}")
    }

    #[track_caller]
    fn refused<T>(parsed: Result<T, ContentError>) {
        let Err(error) = parsed else {
            panic!("a key nothing reads was taken");
        };
        let said = error.to_string();
        assert!(said.contains("unknown field `mistyped`"), "{said}");
    }

    #[test]
    fn a_mistyped_key_in_an_authored_file_is_refused() {
        // A defaulted field mistyped would otherwise take its default in silence.
        let fuel = include_str!("../../content/fuel.toml");
        refused(Fuels::parse(&under(fuel, "[charcoal]")));

        let forge = include_str!("../../content/forge.toml");
        refused(Charms::parse(&under(forge, "[surcharge]")));
        refused(Charms::parse(&under(forge, "[shielded]")));

        let materials = include_str!("../../content/materials.toml");
        refused(Materials::parse(&under(materials, "[sage]")));

        let manual = include_str!("../../content/manual.toml");
        refused(Manual::parse(&above(manual)));
        refused(Manual::parse(&under(manual, "[[chapters]]")));

        let prose = include_str!("../../content/prose.toml");
        refused(Prose::parse(&above(prose)));

        let spells = include_str!("../../content/spells.toml");
        refused(Spells::parse(&under(spells, "[first_light]")));

        let recipes = include_str!("../../content/recipes.toml");
        refused(Recipes::parse(&under(recipes, "[[mortar_and_pestle]]")));

        let phrasings = include_str!("../../content/phrasings.toml");
        refused(Phrasings::parse(&above(phrasings)));
        refused(Phrasings::parse(&under(phrasings, "[[entry]]")));
        refused(Phrasings::parse(&under(phrasings, "[[refusal]]")));

        let trials = include_str!("../../content/spell_trials.toml");
        let read = |text: &str| parse::<Trials>("spell_trials.toml", text);
        refused(read(&above(trials)));
        refused(read(&under(trials, "[[script]]")));
        refused(read(&trials.replacen(
            "trial = [",
            "trial = [\n  { said = \"x\", mistyped = \"x\" },",
            1,
        )));
    }
}
