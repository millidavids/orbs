//! Every scene there is, and which a term selects.

use serde::Deserialize;

use super::scene::Scene;

/// The catalogue the binary carries.
const BUILTIN: &str = include_str!("scenes.toml");

/// A catalogue to read instead of the built-in one, so a timeline can be tuned
/// without relinking the game.
pub const FILE: &str = "ORBS_TOUR_FILE";

/// Every scene there is, in the order a tour plays them.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalogue {
    #[serde(rename = "scene")]
    scenes: Vec<Scene>,
}

impl Catalogue {
    /// The catalogue the binary carries.
    ///
    /// # Panics
    ///
    /// If `scenes.toml` does not read, which a test holds can never ship.
    #[must_use]
    pub fn builtin() -> Self {
        match Self::read(BUILTIN) {
            Ok(catalogue) => catalogue,
            Err(why) => panic!("the built-in tour does not read: {why}"),
        }
    }

    /// The catalogue in `text`.
    ///
    /// # Errors
    ///
    /// Why it does not read, or which scene could not be played.
    pub fn read(text: &str) -> Result<Self, String> {
        let catalogue: Self = toml::from_str(text).map_err(|error| error.to_string())?;
        for scene in &catalogue.scenes {
            scene.playable()?;
        }
        Ok(catalogue)
    }

    /// [`FILE`]'s catalogue, or the built-in one.
    ///
    /// # Errors
    ///
    /// When [`FILE`] names a file that cannot be read as one.
    pub fn chosen() -> Result<Self, String> {
        let Some(path) = crate::environment::switched(FILE) else {
            return Ok(Self::builtin());
        };
        let text = std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
        Self::read(&text).map_err(|why| format!("{path}: {why}"))
    }

    /// Every scene, in order.
    #[must_use]
    pub fn scenes(&self) -> &[Scene] {
        &self.scenes
    }

    /// The scenes `terms` selects — comma-separated names and groups, or
    /// `all` — in catalogue order within each term, each once.
    ///
    /// # Errors
    ///
    /// A term that selects nothing, with the scenes nearest it, or the groups
    /// there are when none is near.
    pub fn select(&self, terms: &str) -> Result<Vec<&Scene>, String> {
        let mut chosen: Vec<&Scene> = Vec::new();
        for term in terms
            .split(',')
            .map(str::trim)
            .filter(|term| !term.is_empty())
        {
            let found: Vec<&Scene> = self
                .scenes
                .iter()
                .filter(|scene| term == "all" || scene.is(term))
                .collect();
            if found.is_empty() {
                return Err(self.unknown(term));
            }
            for scene in found {
                if !chosen.iter().any(|kept| kept.name == scene.name) {
                    chosen.push(scene);
                }
            }
        }
        if chosen.is_empty() {
            return Err("name a scene, a group, or `all`".to_owned());
        }
        Ok(chosen)
    }

    /// Why `term` selects nothing: the scenes nearest it — a name containing
    /// it, or a part of a name the parser's own fuzzy match reads as it — or
    /// the groups when none is near.
    fn unknown(&self, term: &str) -> String {
        let near: Vec<&str> = self
            .scenes
            .iter()
            .map(|scene| scene.name.as_str())
            .filter(|name| {
                name.contains(term)
                    || name
                        .split('.')
                        .any(|part| orbs_sim::parser::is_near(term, part))
            })
            .take(6)
            .collect();
        if near.is_empty() {
            format!(
                "no scene or group is called {term:?}; the groups are {}",
                self.groups().join(", ")
            )
        } else {
            format!(
                "no scene or group is called {term:?}; nearest: {}",
                near.join(", ")
            )
        }
    }

    /// The groups, in order.
    fn groups(&self) -> Vec<&str> {
        let mut groups: Vec<&str> = Vec::new();
        for scene in &self.scenes {
            let group = scene.name.split('.').next().unwrap_or_default();
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
        groups
    }

    /// One line a scene: its name, how long it holds, and what it shows.
    #[must_use]
    pub fn listing(&self) -> String {
        let width = self
            .scenes
            .iter()
            .map(|scene| scene.name.chars().count())
            .max()
            .unwrap_or_default();
        self.scenes
            .iter()
            .map(|scene| {
                format!(
                    "{:<width$}  {:>4.1}s  {}\n",
                    scene.name, scene.hold, scene.about
                )
            })
            .collect()
    }
}
