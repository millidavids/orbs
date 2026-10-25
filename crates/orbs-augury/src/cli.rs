//! The command line the examples share.
//!
//! One reading of the flags for `scripts/seeds.sh`'s three examples, which
//! cannot import each other.

/// The word after `flag` on the command line.
///
/// [`Register::asked`](crate::Register::asked) is the same question for a flag
/// that carries no value.
#[must_use]
pub fn argument(flag: &str) -> Option<String> {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next();
        }
    }
    None
}

/// Where `flag` says to load weights from — a path without its `.bin` — or the
/// shipped ones.
#[must_use]
pub fn weights(flag: &str, shipped: &str) -> String {
    argument(flag).unwrap_or_else(|| shipped.to_owned())
}

/// Say there is nothing to measure.
///
/// Under `--scores` that is a failure, because an empty score file would average
/// into a run's mean as a reader that read nothing at all.
pub fn missing(scores: bool, what: &str, train: &str) {
    if scores {
        eprintln!("no {what} to measure — `{train}`");
        std::process::exit(1);
    }
    println!("  no {what} yet — `{train}`\n");
}

/// The scrivener `--scribe` and `--reader` name, or the shipped one — or
/// `None`, having said there is none to measure.
#[must_use]
pub fn scribe(scores: bool) -> Option<crate::Scribe<burn::backend::NdArray<f32>>> {
    let loaded = crate::Scribe::load_from(
        &weights("--scribe", crate::trained::SCRIBE_WEIGHTS),
        &weights("--reader", crate::trained::WEIGHTS),
        Default::default(),
    );
    if loaded.is_err() {
        missing(
            scores,
            "spell weights",
            "cargo run --release -p orbs-augury --example train --features train -- --spells",
        );
    }
    loaded.ok()
}

/// `part` of `whole` as a percentage, and nought of nothing.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "counts of lines and epochs, not 2^24"
)]
pub fn percent(part: usize, whole: usize) -> f32 {
    if whole == 0 {
        return 0.0;
    }
    part as f32 / whole as f32 * 100.0
}
