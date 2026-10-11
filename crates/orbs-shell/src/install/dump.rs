//! The installation in `ORBS_DUMP`, which builds no `App` and so has no swap.
//!
//! Only on `ORBS_INSTALL=1`: a dump keeps no settings, and every capture
//! expects the open tower it has always drawn.

use orbs_sim::content::Length;
use orbs_sim::{Augur, Sim};

use super::state::{Install, Raise};

/// The practice tower a dump draws, when `ORBS_INSTALL=1` asks for one.
///
/// Baseline, as every other capture is, so a retuned length moves none of them.
#[must_use]
pub fn pose(seed: u64, wizard: Option<&str>) -> (Sim, Option<Install>) {
    let begun = Raise {
        keep: None,
        length: Length::Baseline,
        seed,
        wizard: wizard.map(str::to_owned),
    }
    .begin(true);
    (begun.sim, begun.install)
}

/// One dump segment, through the installation, then a tick.
///
/// A segment that wakes the tower, `skip` or a last `wake`, raises it as the
/// frontends do, and the rest of the script runs there.
pub fn drive(
    sim: &mut Sim,
    installing: &mut Option<Install>,
    line: &str,
    augur: Option<&dyn Augur>,
) {
    let Some(install) = installing.as_mut() else {
        return;
    };
    if let Some(raise) = install.submit(sim, line, augur) {
        *sim = raise.wake().sim;
        for file in crate::prose::load(sim) {
            eprintln!(
                "warn: {} is set but {file} did not load; drawing the built-in text",
                crate::prose::CONTENT_DIR
            );
        }
        *installing = None;
        return;
    }
    sim.step();
    install.observe(sim);
}
