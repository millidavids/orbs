//! A scripted session, driven through the same `Sim` the game drives.
//!
//! ```text
//! cargo run -p orbs-sim --example session
//! ```
//!
//! No window, no GPU, no parser harness — this is the real entry points,
//! `Sim::submit` and `Sim::step`, in the order a frontend calls them. What
//! prints is what the scrollback holds, which is what the prompt draws.

use orbs_sim::Sim;

fn main() {
    let mut sim = Sim::new(0xC0FFEE);
    let script = [
        "look around",
        "meditate 30",
        "status",
        "sift seed orb.log",
        "xyzzy",
        "meditate",
    ];

    for line in script {
        let before = sim.scrollback().records().len();
        sim.submit(line);
        sim.step();

        println!(
            "\n  \x1b[1morbs:~$ {line}\x1b[0m  →  tick {}",
            sim.tick().get()
        );
        for record in sim.scrollback().records().iter().skip(before + 1) {
            let marker = record
                .outcome()
                .map(orbs_render::Outcome::marker)
                .or_else(|| record.kind().marker())
                .unwrap_or(' ');
            println!("    {marker} {}", record.to_speech());
        }
    }

    println!(
        "\n  {} records logged, tick {}",
        sim.scrollback().records().len(),
        sim.tick().get()
    );
}
