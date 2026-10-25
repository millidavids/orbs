//! A two-word setting held as a resource.

/// A [`Pair`](orbs_shell::settings::Pair) resource: a `bool` behind two words,
/// opening as the settings file has it.
macro_rules! pair {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident($field:vis bool) = $key:expr, $words:expr;
    ) => {
        $(#[$meta])*
        #[derive(bevy::prelude::Resource, Debug, Clone, Copy, PartialEq, Eq)]
        $vis struct $name($field bool);

        impl From<bool> for $name {
            fn from(raised: bool) -> Self {
                Self(raised)
            }
        }

        impl orbs_shell::settings::Pair for $name {
            const KEY: &'static str = $key;
            const WORDS: [&'static str; 2] = $words;

            fn raised(self) -> bool {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                <Self as orbs_shell::settings::Pair>::remembered()
            }
        }
    };
}

pub(crate) use pair;
