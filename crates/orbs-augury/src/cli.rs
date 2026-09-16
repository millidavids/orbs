//! The command line the examples share.
//!
//! `train`, `measure` and `trials` are driven together by `scripts/seeds.sh`,
//! which needs them to read the same flags the same way: a `--reader` that meant
//! one thing to the measurement and another to the trials would compare two runs
//! through two different readers and report the difference as a result. An
//! example cannot import another example, so the shared half lives here rather
//! than in three copies.

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
/// **Under `--scores` that is a failure**, because an empty score file would
/// average into a run's mean as a reader that read nothing at all.
pub fn missing(scores: bool, what: &str, train: &str) {
    if scores {
        eprintln!("no {what} to measure — `{train}`");
        std::process::exit(1);
    }
    println!("  no {what} yet — `{train}`\n");
}
