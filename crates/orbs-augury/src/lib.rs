//! The trained reader behind §6's deterministic pipeline (DESIGN.md §19).
//!
//! One implementation of `augur::Augur`; `orbs-sim` decides which lines it sees
//! and must never depend on this crate (rule 1).
//!
//! Trained from scratch on this project's own data: no pretrained weights,
//! embeddings or tokenizer, and no distillation (§19). The corpus is its only
//! capability, so it stays small and `content/phrasings.toml` is most of the
//! work.
//!
//! [`Vocabulary`] is where that rule becomes code: every row comes from a table
//! this repository already contains.

pub mod batch;
pub mod cli;
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
