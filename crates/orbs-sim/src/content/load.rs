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
