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
    let mut sim = Sim::new(1);
    let script = [
        // Every one of these names something the tower actually holds. Before
        // the domains item, the Essence, Vessel, Fragment and Place slots were
        // unfillable and half the vocabulary could not be exercised.
        "attend laboratory",
        // §10.1's loop, not `decoct`: a rename left two `decoct` lines here
        // that fuzzy-resolved to the manual, so the one example meant to
        // exercise the vocabulary touched none of the pipeline.
        "recall brewing",
        // §10.1's per-instrument verbs: `kindle charcoal` is the `move` and the
        // `wield` in one, and the instrument is named by the verb rather than
        // typed. `move`/`wield` still work — they are the general forms.
        "kindle charcoal",
        "grind sage",
        "meditate 10",
        // No draw-off: `digest ground-sage` reaches into the mortar for what
        // the grind made, which is what retired `siphon` (§19), and `empty`
        // shelves what is left.
        "digest ground-sage",
        "empty mortar_and_pestle",
        "meditate 15",
        "empty balneum_mariae",
        "stop athanor",
        "verify laboratory.log",
        "peruse laboratory.log",
        "purge laboratory.log",
        "verify laboratory.log",
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
