//! O.R.B.S. — terminal frontend.
//!
//! A full-screen raw-mode application. It does not shell out, touch the real
//! filesystem, or interoperate with the host shell — the terminal is a
//! framebuffer and a keyboard. Same sim, same commands, same simulated tower.

fn main() {
    let sim = orbs_sim::Sim::new(0);
    println!("orbs-tui — tick {}, seed {}", sim.tick().get(), sim.seed());
}
