//! A scene's steps: what is typed once it is shown, and the keys it names.

use serde::Deserialize;

/// Seconds between one step landing and the next starting.
pub const PAUSE: f32 = 1.0;

/// Seconds before a scene's first step, which keeps a crossing out of the
/// passage's one-second floor after the scene arrives.
pub const FIRST_PAUSE: f32 = 1.2;

/// One thing done at the keyboard.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// A line typed at whatever has the keyboard, then Enter.
    #[serde(rename = "type", default)]
    pub line: Option<String>,
    /// Keys: `<name>` presses a named key, anything else is typed.
    #[serde(default)]
    pub keys: Option<String>,
    /// Seconds after the step before it landed: [`PAUSE`], or [`FIRST_PAUSE`]
    /// for the first.
    #[serde(default)]
    pub pause: Option<f32>,
    /// What must be up before it starts.
    #[serde(default)]
    pub when: Option<When>,
    /// Seconds after it lands to take a screenshot at.
    #[serde(default)]
    pub shots: Vec<f32>,
}

/// What a step can wait to see.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum When {
    /// The spell editor is open.
    Editor,
    /// The weave screen is up.
    Weave,
    /// The arrows have the archive's maze.
    Maze,
    /// The orb's menu is up.
    Menu,
    /// The manual is open.
    Manual,
    /// The boot card has gone.
    Live,
}

/// One keystroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    /// A character, typed.
    Char(char),
    /// A key with a name.
    Named(Named),
}

/// The keys a step can name: the ones some surface reads, and the window's
/// function keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Named {
    /// `<enter>`.
    Enter,
    /// `<esc>`.
    Escape,
    /// `<tab>`, the completion listing.
    Tab,
    /// `<backspace>`.
    Backspace,
    /// `<up>`.
    Up,
    /// `<down>`.
    Down,
    /// `<left>`.
    Left,
    /// `<right>`.
    Right,
    /// `<home>`.
    Home,
    /// `<end>`.
    End,
    /// `<pgup>`, back through the transcript.
    PageUp,
    /// `<pgdn>`.
    PageDown,
    /// `<f2>`, the phosphor.
    F2,
    /// `<f3>`, the tube.
    F3,
    /// `<f5>`, the linear stream.
    F5,
    /// `<f7>`, the tonal register.
    F7,
    /// `<f8>`, the sight.
    F8,
}

impl Named {
    /// Every key, by the name a step writes between `<` and `>`.
    pub const ALL: [(&'static str, Self); 17] = [
        ("enter", Self::Enter),
        ("esc", Self::Escape),
        ("tab", Self::Tab),
        ("backspace", Self::Backspace),
        ("up", Self::Up),
        ("down", Self::Down),
        ("left", Self::Left),
        ("right", Self::Right),
        ("home", Self::Home),
        ("end", Self::End),
        ("pgup", Self::PageUp),
        ("pgdn", Self::PageDown),
        ("f2", Self::F2),
        ("f3", Self::F3),
        ("f5", Self::F5),
        ("f7", Self::F7),
        ("f8", Self::F8),
    ];

    /// The key called `name`, if there is one.
    #[must_use]
    pub fn called(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, key)| *key)
    }
}

impl Step {
    /// Its keystrokes.
    ///
    /// # Errors
    ///
    /// A step that is both `type` and `keys`, or neither, or names a key there
    /// is none of.
    pub fn strokes(&self) -> Result<Vec<Stroke>, String> {
        match (&self.line, &self.keys) {
            (Some(line), None) => Ok(line
                .chars()
                .map(Stroke::Char)
                .chain([Stroke::Named(Named::Enter)])
                .collect()),
            (None, Some(keys)) => strokes(keys),
            _ => Err("a step is `type` or `keys`, and not both".to_owned()),
        }
    }

    /// Seconds after the step before it to start.
    #[must_use]
    pub fn pause(&self, first: bool) -> f32 {
        self.pause
            .unwrap_or(if first { FIRST_PAUSE } else { PAUSE })
    }
}

/// `keys` as keystrokes: `<name>` a named key, any other character typed.
///
/// A `<` with no `>` after it is typed.
///
/// # Errors
///
/// A `<name>` that names no key.
pub fn strokes(keys: &str) -> Result<Vec<Stroke>, String> {
    let mut strokes = Vec::new();
    let mut rest = keys;
    while let Some(first) = rest.chars().next() {
        if first == '<'
            && let Some(close) = rest.find('>')
        {
            let name = &rest[1..close];
            let key = Named::called(name).ok_or_else(|| format!("no key is called <{name}>"))?;
            strokes.push(Stroke::Named(key));
            rest = &rest[close + 1..];
            continue;
        }
        strokes.push(Stroke::Char(first));
        rest = &rest[first.len_utf8()..];
    }
    Ok(strokes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_typed_except_the_ones_named() {
        use Named::{Enter, Right};
        use Stroke::{Char, Named as Key};
        assert_eq!(
            strokes("<right>a<enter>"),
            Ok(vec![Key(Right), Char('a'), Key(Enter)])
        );
        assert_eq!(strokes("a<b"), Ok(vec![Char('a'), Char('<'), Char('b')]));
        assert!(strokes("<rigth>").is_err());
        let typed = Step {
            line: Some("ok".to_owned()),
            ..Step::default()
        };
        assert_eq!(typed.strokes(), Ok(vec![Char('o'), Char('k'), Key(Enter)]));
        let both = Step {
            line: Some("ok".to_owned()),
            keys: Some("ok".to_owned()),
            ..Step::default()
        };
        assert!(both.strokes().is_err());
    }
}
