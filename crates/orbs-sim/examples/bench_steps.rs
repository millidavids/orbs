//! How long the sim takes to run, with no window and no frontend.
//!
//! ```text
//! cargo run --release -p orbs-sim --example bench_steps
//! ```
//!
//! 29,000 steps is rule 8's stated worst case — *"offline catch-up is ~29k
//! `step()` calls, which is milliseconds"* — and this is what makes that a
//! measurement rather than a claim. It is also the loop `orbs-balance` will
//! sweep in Phase 2, so a regression here is a regression in every sweep.
//!
//! **Still Phase 2 after the renumber**, and not by accident: the siege moved to
//! Phase 8 but `orbs-balance` moved *to* Phase 2, because five domain phases
//! would otherwise author their durations on top of numbers nothing has swept.
//!
//! It exists because a review proposed caching the `QueryState`s that
//! `tower::burn` and `tower::finish` build per tick. The number says the whole
//! catch-up is ~120 ms at ~4 µs a step, so those four constructions are noise at
//! this entity count and the caching would be complexity bought for nothing.
//! Re-run it before believing otherwise.

fn main() {
    const STEPS: u64 = 29_000;

    let mut sim = orbs_sim::Sim::new(1);
    sim.submit("attend laboratory");
    sim.step();

    let start = std::time::Instant::now();
    sim.step_n(STEPS);
    let elapsed = start.elapsed();

    #[expect(clippy::cast_precision_loss, reason = "a timing report")]
    let per_step = elapsed.as_secs_f64() * 1e6 / STEPS as f64;
    println!("{STEPS} steps in {elapsed:?} ({per_step:.1} us/step)");
    println!("tick {}", sim.tick().get());
}
