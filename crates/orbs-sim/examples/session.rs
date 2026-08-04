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
        // Every one of these names something the tower actually holds — which is
        // the point of the domains item. Before it, the Essence, Vessel,
        // Fragment and Place slots were unfillable and half the sixteen-verb
        // vocabulary could not be exercised at all.
        "attend alembic",
        "make a potion of clarity",
        "meditate 25",
        "look around",
        "purge residue-9",
        "purge alembic",
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
            let marker = record.marker().unwrap_or(' ');
            println!("    {marker} {}", record.to_speech());
        }
    }

    println!(
        "\n  {} records logged, tick {}",
        sim.scrollback().records().len(),
        sim.tick().get()
    );
}
