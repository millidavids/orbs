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
//!     .with(NounKind::Place, "/tower/alembic")
//!     .with(NounKind::Essence, "clarity");
//!
//! // All three registers reach the same canonical command.
//! for input in ["decoct clarity", "brew clarity", "make a potion of clarity"] {
//!     let resolved = resolve(input, &scene, Mode::Calm);
//!     assert_eq!(resolved.intent().expect("resolves").echo(), "decoct clarity");
//! }
//! ```

mod arguments;
mod fuzzy;
mod intent;
mod normalise;
mod report;
mod resolve;
mod scene;
mod trace;
mod verb;
mod vocabulary;

pub use fuzzy::{EXACT, MIN_SIMILARITY, distance, is_near, similarity};
pub use intent::{Argument, Candidate, Confidence, Intent, Mode, Resolution};
pub use report::{CANDIDATE, FORCED, INCOMPLETE, RESOLVED, SUGGESTION, UNRESOLVED, report};
pub use resolve::{Analysis, analyse, resolve};
pub use scene::{Noun, NounMatch, Scene};
pub use trace::{Outcome, ParseLog, ParseRecord};
pub use verb::{NounKind, Slot, Verb};
pub use vocabulary::{Register, SYNONYMS, Synonym};
