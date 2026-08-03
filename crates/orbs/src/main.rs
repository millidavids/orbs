//! O.R.B.S. — Bevy frontend.
//!
//! A *caller* of the simulation, never its host: it constructs an
//! [`orbs_sim::Sim`] and drives it with `step()`. Bevy's own scheduler never runs
//! sim systems. See CLAUDE.md, architectural rule 3.

fn main() {
    let sim = orbs_sim::Sim::new(0);
    println!("orbs — tick {}, seed {}", sim.tick().get(), sim.seed());
}
