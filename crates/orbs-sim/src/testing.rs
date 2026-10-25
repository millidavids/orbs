//! The helpers the inline test modules share. `tests/common` is the same for
//! the integration tests, which cannot see `cfg(test)` items.

use orbs_render::{FieldName, Value};

use crate::Sim;

/// Type a line and let the tick that runs it pass.
pub(crate) fn run(sim: &mut Sim, line: &str) {
    sim.submit(line);
    sim.step();
}

/// Every message the orb has said.
pub(crate) fn said(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .filter_map(|record| match record.field(FieldName::Message) {
            Some(Value::Text(text)) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

/// Whether anything the orb said contains `needle`.
pub(crate) fn ever_said(sim: &Sim, needle: &str) -> bool {
    said(sim).iter().any(|line| line.contains(needle))
}

/// Every value reads back from its own word, and no two share one.
pub(crate) fn assert_round_trips<T: Copy + PartialEq + std::fmt::Debug>(
    all: &[T],
    word: impl Fn(T) -> &'static str,
    named: impl Fn(&str) -> Option<T>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for &value in all {
        assert_eq!(
            named(word(value)),
            Some(value),
            "{value:?} does not read back"
        );
        assert!(
            seen.insert(word(value)),
            "two share the word {}",
            word(value)
        );
    }
}

/// Every record the orb has drawn, as its rendered line.
pub(crate) fn drawn(sim: &Sim) -> Vec<String> {
    sim.scrollback()
        .records()
        .iter()
        .map(|record| record.to_line())
        .collect()
}
