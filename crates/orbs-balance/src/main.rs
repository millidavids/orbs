//! The CLI over the harness — see the crate docs in `lib.rs` for what it drives.
//!
//! ```text
//! cargo run -p orbs-balance -- list
//! cargo run -p orbs-balance -- run clarity --ticks 400
//! cargo run -p orbs-balance -- sweep --hours 1 --csv curves.csv
//! ```

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use orbs_balance::policy::Policy;
use orbs_balance::{drive, report};

/// Ticks in an hour — one tick is one real second with the window open (§11.5).
const HOUR: u64 = 3600;

/// The default sampling interval, in ticks.
///
/// A minute. Fine enough to see the shape of a curve over hours, coarse enough
/// that a 25-hour sweep is 1,500 rows rather than 90,000.
const EVERY: u64 = 60;

#[derive(Parser)]
#[command(name = "orbs-balance", about = "Headless economy harness for O.R.B.S.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List the policies a sweep can run.
    List,
    /// Run one policy and print its curve.
    Run {
        /// Which policy — see `list`.
        policy: String,
        #[command(flatten)]
        span: Span,
    },
    /// Run every policy and compare them.
    Sweep {
        #[command(flatten)]
        span: Span,
    },
}

/// How long to run, and where to put the curve.
#[derive(clap::Args)]
struct Span {
    /// World seed. Everything generated is one seed's worth of evidence, so a
    /// maze policy wants several.
    #[arg(long, default_value_t = 0)]
    seed: u64,
    /// Simulated hours. Ignored if `--ticks` is given.
    #[arg(long, default_value_t = 1)]
    hours: u64,
    /// Simulated ticks, for a short run where an hour is too coarse.
    #[arg(long)]
    ticks: Option<u64>,
    /// Sampling interval, in ticks.
    #[arg(long, default_value_t = EVERY)]
    every: u64,
    /// Write the full curve here as CSV. Without it, only the table is printed.
    #[arg(long)]
    csv: Option<std::path::PathBuf>,
    /// Print why the tower turned commands down. Read this whenever `refused` is
    /// not nought — a rate measured through refusals is not the loop you wrote.
    #[arg(long)]
    why: bool,
    /// How long a game to measure: `baseline`, `short`, `medium` or `long`.
    ///
    /// **Baseline unless asked**, which is what every pinned rate in
    /// `report::EXPECTED` was measured against. The rates themselves do not move
    /// with length — thresholds are not in the numerator or the denominator — but
    /// **`concentration` and `renown` do**, and `bound` moves most: it earns its
    /// first slot by hand before it can bind anything, so a longer game is a
    /// longer hand-played prefix inside the same tick budget.
    ///
    /// Without this flag the shipped game would run a curve the instrument never
    /// measures, which is the divergence §13 exists to prevent.
    #[arg(long, default_value = "baseline")]
    length: String,
}

impl Span {
    const fn ticks(&self) -> u64 {
        match self.ticks {
            Some(ticks) => ticks,
            None => self.hours * HOUR,
        }
    }

    /// The length to measure, or `None` if the word is not one.
    ///
    /// **Refused rather than defaulted.** `ORBS_LENGTH` falls through on a typo
    /// because a player mid-boot has nowhere to be told; a harness has a stderr
    /// and a person reading it, and a sweep that silently measured the wrong
    /// curve is the one output nobody would question.
    fn length(&self) -> Option<orbs_sim::content::Length> {
        orbs_sim::content::Length::named(&self.length)
    }
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::List => {
            for policy in Policy::ALL {
                println!("{:<10} {}", policy.name, policy.gloss);
            }
            ExitCode::SUCCESS
        }
        Command::Run { policy, span } => {
            let Some(policy) = Policy::named(&policy) else {
                // Named rather than silently swept: a typo that ran everything
                // would look like a working command with a surprising answer.
                eprintln!("no policy named `{policy}` — try `orbs-balance list`");
                return ExitCode::FAILURE;
            };
            sweep(&[policy], &span)
        }
        Command::Sweep { span } => sweep(&Policy::ALL, &span),
    }
}

fn sweep(policies: &[Policy], span: &Span) -> ExitCode {
    let Some(length) = span.length() else {
        eprintln!(
            "no length called `{}` — try baseline, short, medium or long",
            span.length,
        );
        return ExitCode::FAILURE;
    };
    let runs: Vec<_> = policies
        .iter()
        .map(|policy| drive::run(*policy, span.seed, span.ticks(), span.every, length))
        .collect();

    print!("{}", report::table(&runs));

    if span.why {
        print!("{}", report::why(&runs));
    }

    if let Some(path) = &span.csv {
        if let Err(error) = std::fs::write(path, report::csv(&runs)) {
            eprintln!("could not write {}: {error}", path.display());
            return ExitCode::FAILURE;
        }
        println!("\ncurves written to {}", path.display());
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cli_parses() {
        // `clap`'s own assertion that the command tree is well-formed. It
        // catches a duplicate flag or a bad default at test time rather than on
        // the first run.
        use clap::CommandFactory as _;
        Cli::command().debug_assert();
    }

    #[test]
    fn hours_and_ticks_agree_about_which_wins() {
        let span = Span {
            seed: 0,
            hours: 2,
            ticks: None,
            every: EVERY,
            why: false,
            csv: None,
            length: "baseline".to_owned(),
        };
        assert_eq!(span.ticks(), 2 * HOUR);

        let span = Span {
            ticks: Some(94),
            ..span
        };
        assert_eq!(span.ticks(), 94, "--ticks must win over --hours");
    }
}
