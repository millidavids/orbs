//! Which tester's word a line is, read before the parser sees it.

/// The word, matched exactly and never advertised.
///
/// Underscored, so no typo of a real word reaches it and it reads as a tool.
pub const SPAWN: &str = "debug_spawn";

/// `debug_learn [name]` — hand the player a recipe the lens would have found.
///
/// Discovery costs six broken wards and a roll. Bare, the next secret in the
/// lens's order; named, that one, refusing a name that is not a secret.
pub const LEARN: &str = "debug_learn";

/// `debug_ward` — make the open ward's answer whatever the aperture holds.
///
/// The next `probe` breaks the seal, and everything downstream runs as it would
/// have. Only the deduction is skipped.
pub const WARD: &str = "debug_ward";

/// `debug_course` — leave the standing course one haul from finished.
///
/// A course is up to 127 hauls. The last haul is real and triggers the real
/// `finish`; only the puzzle is skipped.
pub const COURSE: &str = "debug_course";

/// `debug_circle` — limn the circle so the waiting beast's next call holds it.
///
/// The glyphs are limned to a solution, never the temper rewritten, which
/// could leave one no circle answers. Republished, since a spell reads them;
/// the next `summon` is a real call.
pub const CIRCLE: &str = "debug_circle";

/// `debug_siege` — leave the standing siege one round from won.
///
/// The last round is a real `hold` with real rolls and the real `settle`.
/// Republished, as `debug_course`, since a spell reads the board.
pub const SIEGE: &str = "debug_siege";

/// `debug_swap` — substitute a reagent where the player is standing.
///
/// §8.1's world surface arrives on a 1200-tick roll.
pub const SWAP: &str = "debug_swap";

/// `debug_take <id>` — hold a mastery node without earning it.
///
/// Skips the earning and nothing else: the grant goes through `Taken::hold`.
/// An id nothing grants is refused, a state the game cannot reach. Bare, it
/// lists what there is to take.
pub const TAKE: &str = "debug_take";

/// `debug_reach <id>` — reach a mastery station without doing its deed.
///
/// Every earlier station on the line too, since a line is walked in order. The
/// grant is the real one through `mastery::reach`. Bare, it lists the stations.
pub const REACH: &str = "debug_reach";

/// A tester's door to a standing, for the ranks.
///
/// It *sets* the total, so one word goes both ways — a rank is lost by falling
/// back through it. Anything not a number is refused; see [`Asking`].
pub const RENOWN: &str = "debug_renown";

/// A tester's word, read whole. `debug_spell` is read apart, since it is the one
/// that writes a spell rather than acting on the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugWord {
    /// `debug_spawn`.
    Spawn(Order),
    /// `debug_ward`.
    Ward,
    /// `debug_course`.
    Course,
    /// `debug_siege`.
    Siege,
    /// `debug_circle`.
    Circle,
    /// `debug_swap`.
    Swap,
    /// `debug_learn`, and the secret it names, if any.
    Learn(Option<String>),
    /// `debug_take`, and the node it names, if any.
    Take(Option<String>),
    /// `debug_reach`, and the station it names, if any.
    Reach(Option<String>),
    /// `debug_renown`.
    Renown(Asking),
}

impl DebugWord {
    /// The word `line` is, if it is one at all.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        if let Some(order) = order(line) {
            return Some(Self::Spawn(order));
        }
        for (word, bare) in [
            (WARD, Self::Ward),
            (COURSE, Self::Course),
            (SIEGE, Self::Siege),
            (CIRCLE, Self::Circle),
            (SWAP, Self::Swap),
        ] {
            if argument(line, word) == Some("") {
                return Some(bare);
            }
        }
        if let Some(name) = argument(line, LEARN) {
            return Some(Self::Learn((!name.is_empty()).then(|| name.to_owned())));
        }
        if let Some(id) = argument(line, TAKE) {
            return Some(Self::Take(named(id)));
        }
        if let Some(id) = argument(line, REACH) {
            return Some(Self::Reach(named(id)));
        }
        argument(line, RENOWN).map(|total| {
            Self::Renown(if total.is_empty() {
                Asking::Where
            } else {
                total.parse().map_or(Asking::Unreadable, Asking::Set)
            })
        })
    }
}

/// What follows `word` on `line`, trimmed, if `line` is that word at all:
/// `debug_spawnage` is not `debug_spawn`.
pub(in crate::execute) fn argument<'a>(line: &'a str, word: &str) -> Option<&'a str> {
    let rest = line.trim().strip_prefix(word)?;
    (rest.is_empty() || rest.starts_with(char::is_whitespace)).then(|| rest.trim())
}

/// An id a tester typed, in the case the content uses.
fn named(id: &str) -> Option<String> {
    (!id.is_empty()).then(|| id.to_lowercase())
}

/// What a `debug_renown` line asked for.
///
/// Three states, or an unreadable word folds into "nought" and zeroes the
/// renown. Not `Standing`, which is `tower::Standing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asking {
    /// A bare `debug_renown`: say where the tower stands, change nothing.
    Where,
    /// A number: set the total to it.
    Set(u64),
    /// A word that is not a number: refuse, and change nothing.
    Unreadable,
}

/// What a tester asked for, if this line is a spawn order at all.
///
/// `debug_spawn ground-sage 5` is five; a count that is not a number is
/// refused, never defaulted. `debug_spawn fragment 4 lectern` says where — any
/// fixture, from anywhere.
#[must_use]
pub fn order(line: &str) -> Option<Order> {
    let mut words = argument(line, SPAWN)?.split_whitespace();
    let Some(name) = words.next() else {
        // Bare, which is how you ask what there is.
        return Some(Order::List);
    };
    let count = match words.next() {
        None => 1,
        // Zero is a request for nothing, and a `Counted(0)` pile would break
        // `stock::take`'s invariant.
        Some(count) => count.parse().ok().filter(|count| *count > 0)?,
    };
    // A destination is not a number: `debug_spawn sage 2 3` is a mistyped
    // count, not a place called `3`.
    let into = match words.next() {
        None => None,
        Some(word) if word.parse::<u32>().is_ok() => return None,
        Some(word) => Some(word.to_owned()),
    };
    // Every word read, or none of them — the question grammar's rule, and for
    // its reason: `debug_spawn sage 2 lectern spare` naming one destination is a
    // line half-obeyed.
    if words.next().is_some() {
        return None;
    }
    Some(Order::Spawn {
        name: name.to_owned(),
        count,
        into,
    })
}

/// What the word was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    /// Put `count` of `name` on the shelf, or in the fixture named by `into`.
    Spawn {
        /// What to make.
        name: String,
        /// How many.
        count: u32,
        /// Which fixture to put them in. The shelf if unsaid.
        into: Option<String>,
    },
    /// Say what can be made.
    List,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_word_is_matched_exactly() {
        assert_eq!(order("debug_spawn"), Some(Order::List));
        assert_eq!(
            order("debug_spawn sage"),
            Some(Order::Spawn {
                name: "sage".to_owned(),
                count: 1,
                into: None,
            }),
        );
        assert_eq!(
            order("  debug_spawn ground-sage 5  "),
            Some(Order::Spawn {
                name: "ground-sage".to_owned(),
                count: 5,
                into: None,
            }),
        );
        // A destination needs a count before it, which is positional.
        assert_eq!(
            order("debug_spawn fragment 4 lectern"),
            Some(Order::Spawn {
                name: "fragment".to_owned(),
                count: 4,
                into: Some("lectern".to_owned()),
            }),
        );
        assert_eq!(order("debug_spawn fragment lectern"), None);

        // Not this word, and not guessed at: a tester who mistypes it should be
        // told by the ordinary parser rather than have the tower change.
        for other in [
            "debug_spawnage",
            "debug spawn sage",
            "spawn sage",
            "debug_spaw sage",
            "grind sage",
        ] {
            assert_eq!(order(other), None, "{other:?} was read as a spawn");
        }

        // A count that is not a count. Defaulting to one would be the quiet
        // reinterpretation this subsystem exists to refuse.
        assert_eq!(order("debug_spawn sage lots"), None);
        assert_eq!(order("debug_spawn sage 2 3"), None);
    }

    #[test]
    fn a_bare_word_takes_no_argument_and_a_named_one_reads_its_own() {
        assert_eq!(DebugWord::read("  debug_ward "), Some(DebugWord::Ward));
        // Not the word: it falls through to the parser, as it always has.
        assert_eq!(DebugWord::read("debug_ward now"), None);
        assert_eq!(DebugWord::read("debug_wards"), None);
        assert_eq!(
            DebugWord::read("debug_take Satchel_1"),
            Some(DebugWord::Take(Some("satchel_1".to_owned())))
        );
        assert_eq!(DebugWord::read("debug_reach"), Some(DebugWord::Reach(None)));
        assert_eq!(
            DebugWord::read("debug_learn Mending"),
            Some(DebugWord::Learn(Some("Mending".to_owned())))
        );
        assert_eq!(
            DebugWord::read("debug_renown 25"),
            Some(DebugWord::Renown(Asking::Set(25)))
        );
        assert_eq!(
            DebugWord::read("debug_renown -1"),
            Some(DebugWord::Renown(Asking::Unreadable))
        );
        assert_eq!(DebugWord::read("debug_spell"), None);
    }
}
