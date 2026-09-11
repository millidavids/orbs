//! The trials' whole spells are spells the tower can run.
//!
//! A script trial says what a loose spell should become. If that answer does
//! not itself compile, the report would blame the reader for the file's mistake
//! — so the expectation is checked here, against a real room, with no reader in
//! the building at all.

use orbs_sim::Sim;
use orbs_sim::content::Trials;

#[test]
fn every_script_reads_to_a_spell_its_room_compiles() {
    let sim = Sim::new(3);
    for script in Trials::builtin().scripts() {
        assert_eq!(
            script.loose.len(),
            script.reads.len(),
            "{:?} does not say what every line becomes",
            script.name,
        );
        let faults: Vec<String> = sim
            .read_spell(&script.domain, &script.reads)
            .iter()
            .filter_map(|line| {
                line.fault
                    .as_ref()
                    .map(|fault| format!("line {}: {} {:?}", line.line, fault.key, fault.detail))
            })
            .collect();
        assert!(
            faults.is_empty(),
            "{:?} should read to a spell the {} compiles, and would not: {faults:?}",
            script.name,
            script.domain,
        );
    }
}
