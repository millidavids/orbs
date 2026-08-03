//! Headless economy harness.
//!
//! Runs a scripted synthetic player for N simulated hours and dumps the curves in
//! DESIGN.md §11.5 to CSV. Converts balance from vibes into sweeps.

fn main() {
    let mut sim = orbs_sim::Sim::new(0);
    sim.step_n(3600);
    println!("orbs-balance — simulated {} ticks", sim.tick().get());
}
