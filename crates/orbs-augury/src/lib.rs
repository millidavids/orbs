//! The trained reader behind §6's deterministic pipeline (DESIGN.md §19).
//!
//! # What this crate is for
//!
//! `orbs-sim` decides *which* lines a reader may see — `parser::is_literal` and
//! `Analysis::reads_outright` between them — and `augur::Augur` is the seam it
//! hands them through. This crate is one implementation of that seam: a small
//! model, trained here, on this game's own content.
//!
//! Nothing depends on it. `orbs-sim` must never gain a dependency here, or the
//! headless boundary CLAUDE.md rule 1 protects would go with it.
//!
//! # Trained here, from scratch, on data this project made
//!
//! No pretrained weights, no pretrained embeddings, no downloaded tokenizer, and
//! no distillation — nothing another model produced reaches the weights. The rule
//! is about the artefact, not about who typed a phrasing: drafting corpus lines
//! in a session is fine, and §19 records the correction of an earlier *"no
//! LLM-generated training data"*, which said far more than was meant. An ethical
//! constraint (§19),
//! and load-bearing twice over: shipped weights are a distributed artefact and
//! the repository is GPL-3.0-or-later, and a model with no general English
//! underneath it has **the corpus as its only source of capability** — which is
//! why it stays small and why `content/phrasings.toml` is the larger half of
//! the work.
//!
//! [`Vocabulary`] is where that rule becomes code: every row comes from a table
//! this repository already contains.

pub mod batch;
pub mod corpus;
pub mod decode;
pub mod model;
pub mod sample;
pub mod scribe;
pub mod spelling;
pub mod trained;
pub mod vocabulary;

pub use batch::Batch;
pub use corpus::{Corpus, Register};
pub use decode::{Decoded, decode};
pub use model::{Reader, ReaderConfig, WIDTH};
pub use sample::{MAX_LEN, MAX_SLOTS, REJECT, Sample, Tag, VERBS};
pub use scribe::{Considered, Copying, Scribe};
pub use spelling::{accounts_for, assemble, command, count_in, kinds, shapes};
pub use trained::{Reading, Trained};
pub use vocabulary::{BUCKETS, Token, Vocabulary};
