//! Intent resolution — **the most important system in the game** (DESIGN.md §6).
//!
//! Deterministic NLU: no LLM, local or remote. Intent is resolved by a
//! hand-authored pipeline over the live world model, which makes it instant,
//! offline, reproducible, unit-testable, and tiny. Its limitation — only
//! anticipated phrasings resolve — is acceptable because the vocabulary is
//! authored and finite, and disambiguation degrades gracefully.
//!
//! ```
//! use orbs_sim::parser::{Mode, NounKind, Scene, resolve};
//!
//! let scene = Scene::new()
//!     .with(NounKind::Place, "/tower/laboratory")
//!     .with(NounKind::Reagent, "sage");
//!
//! // All three registers reach the same canonical command.
//! for input in [
//!     "move sage to laboratory",
//!     "mv sage laboratory",
//!     "transfer the sage to the laboratory",
//! ] {
//!     let resolved = resolve(input, &scene, Mode::Calm);
//!     // A place echoes as its **leaf**: the full path clipped the destination
//!     // off a three-argument `move` at the 80×22 floor, and §7 says players
//!     // say the place rather than the path anyway.
//!     assert_eq!(
//!         resolved.intent().expect("resolves").echo(),
//!         "move sage laboratory",
//!     );
//! }
//! ```

mod arguments;
mod complete;
mod fuzzy;
mod intent;
mod normalise;
mod report;
mod resolve;
mod scene;
mod spellword;
mod trace;
mod verb;
mod vocabulary;

pub use complete::{Completion, Suggestion, complete, is_answer};
pub use fuzzy::{EXACT, MIN_SIMILARITY, distance, is_near, similarity};
pub use intent::{Argument, Candidate, Confidence, Intent, Mode, Resolution, leaf};
pub use normalise::is_filler;
pub use report::report;
pub use resolve::{Analysis, analyse, resolve};
pub use scene::{Noun, NounMatch, Scene};
pub use spellword::{
    Condition, INDENT, SpellWord, State as SpellState, argument as spell_argument, condition,
    indent_around, leading as spell_word, write_condition,
};
pub use trace::{Outcome, ParseLog, ParseRecord};
pub use verb::{NounKind, Slot, Verb};
pub use vocabulary::{Register, SYNONYMS, Synonym};
