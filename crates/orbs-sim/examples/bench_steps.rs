//! How long the sim takes to run, with no window and no frontend.
//!
//! ```text
//! cargo run --release -p orbs-sim --example bench_steps
//! ```
//!
//! Rule 8's 29,000-step catch-up, measured: ~120 ms. Re-run it before caching
//! a `QueryState`.

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
