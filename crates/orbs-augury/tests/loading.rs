//! How long the readers take to load — the See-it line for boot warm-up.
//!
//! Both frontends load their readers before a window exists: the Bevy build in
//! `SimPlugin::build`, the terminal build as it constructs its session.
//! Inference was measured the day it shipped — 436µs a line — and the load
//! never was, so whether it needs to hide behind the POST card was a guess.
//!
//! Ignored, because it prints a number for a person rather than asserting one.
//! Whether a pause at boot is acceptable is a judgement, and the answer differs
//! by profile: `cargo run -p orbs` builds this crate unoptimised, where a player
//! gets a release build. Run it both ways:
//!
//! ```text
//! cargo test -p orbs-augury --test loading -- --ignored --nocapture
//! cargo test --release -p orbs-augury --test loading -- --ignored --nocapture
//! ```

use std::time::{Duration, Instant};

/// Time `load` three times: the first read comes off the disk, the others out
/// of the page cache, and a boot is usually the second kind.
fn timed(what: &str, load: impl Fn() -> bool) -> Duration {
    let mut times = Vec::new();
    let mut loaded = false;
    for _ in 0..3 {
        let started = Instant::now();
        loaded = load();
        times.push(started.elapsed());
    }
    let shown: Vec<String> = times.iter().map(|time| format!("{time:>8.1?}")).collect();
    println!(
        "  {what:<22} {}{}",
        shown.join("  "),
        if loaded {
            ""
        } else {
            "   (no weights: nothing loaded)"
        },
    );
    times[1..].iter().copied().min().unwrap_or_default()
}

#[test]
#[ignore = "prints how long each reader takes to load — the See-it line for boot warm-up"]
fn how_long_the_readers_take_to_load() {
    println!("\n  loading, three times each — cold, then warm:\n");
    let prompt = timed("the prompt's reader", || {
        orbs_augury::Reading::cpu().is_ok()
    });
    let spells = timed("the spell reader", || orbs_augury::Copying::cpu().is_ok());
    println!(
        "\n  what a boot pays for both, warm: {:.1?}\n",
        prompt + spells
    );
}
