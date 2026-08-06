//! The parser, driven the way a playtester will drive it.
//!
//! ```text
//! cargo run -p orbs-sim --example parse        # scripted walkthrough
//! cargo run -p orbs-sim --example parse -- -i  # type at it yourself
//! cargo run -p orbs-sim --example parse -- --tsv > session.tsv
//! ```
//!
//! The `-i` mode is the Phase 0 gate in miniature: type what you would type, and
//! see whether the orb understood. The TSV it exports is the artefact §15's
//! numeric gate is computed from.

use std::io::{BufRead as _, IsTerminal as _, Write as _};

use orbs_sim::parser::{
    Confidence, Mode, NounKind, ParseLog, ParseRecord, Resolution, Scene, analyse, resolve,
};

/// The slice's world: brewing and archive, both thin (DESIGN.md §15).
fn tower() -> Scene {
    Scene::new()
        .with(NounKind::Place, "/tower/laboratory")
        .with(NounKind::Place, "/tower/archive")
        .with(NounKind::Place, "/tower/battlements")
        .with(NounKind::File, "feed.log")
        .with(NounKind::File, "purge_cycle.log")
        .with(NounKind::Essence, "clarity")
        .with(NounKind::Essence, "warding")
        .with(NounKind::Vessel, "alembic")
        .with(NounKind::Script, "night_watch")
        .with(NounKind::Fragment, "sigil-iv")
        .with(NounKind::Topic, "brewing")
        .with(NounKind::Any, "sludge")
}

/// One walkthrough line: what a player types, and why it is interesting.
const WALKTHROUGH: &[(&str, &str)] = &[
    ("survey", "canonical arcane — what an expert types"),
    ("ls", "shell muscle memory reaches the same command"),
    ("what's here", "and so does plain English"),
    (
        "go to the laboratory",
        "a multi-word phrase, filler, and a leaf name",
    ),
    ("make a potion of clarity", "DESIGN.md §6's own example"),
    (
        "grep march feed.log",
        "§6's other example — pattern is free text",
    ),
    ("brew clarty", "a typo still lands"),
    ("grim brewing", "an abbreviation still lands"),
    (
        "please rm sludge",
        "leading filler, shell verb, arcane echo",
    ),
    ("brew", "a missing argument becomes a numbered prompt"),
    ("purge", "an Any slot enumerates every surface (§8.1)"),
    (
        "sift march nowhere.log",
        "the pattern survives a missing file",
    ),
    (
        "meditate",
        "a free-text slot cannot be listed, so it is named",
    ),
    ("xyzzy plugh", "nonsense is never a bare error"),
];

fn main() {
    let interactive = std::env::args().any(|arg| arg == "-i" || arg == "--interactive");
    let want_tsv = std::env::args().any(|arg| arg == "--tsv");

    let scene = tower();
    let mut log = ParseLog::new();
    let mut tick = 0u64;

    if !want_tsv {
        println!("\nO.R.B.S. — intent resolution (DESIGN.md §6)\n");
        println!("The scene: {} nameable things.\n", scene.nouns().len());
    }

    for (input, why) in WALKTHROUGH {
        tick += 1;
        let analysis = analyse(input, &scene, Mode::Calm);
        if !want_tsv {
            report(input, why, &analysis.resolution);
        }
        log.push(ParseRecord::new(tick, input, Mode::Calm, &analysis));
    }

    if !want_tsv {
        siege_contrast(&scene);
        summary(&log);
    }

    if interactive {
        tick = repl(&scene, &mut log, tick);
        let _ = tick;
    }

    if want_tsv {
        print!("{}", log.to_tsv());
    }
}

/// A walkthrough line: the prompt, the outcome, and why it is interesting.
fn report(input: &str, why: &str, resolution: &Resolution) {
    println!("  orbs:~$ {input}");
    show(resolution, "    ");
    println!("    · {why}\n");
}

/// Show one resolution the way the game will.
fn show(resolution: &Resolution, indent: &str) {
    match resolution {
        Resolution::Resolved { intent, confidence } => {
            println!("{indent}-> {}", intent.echo());
            if *confidence == Confidence::Forced {
                println!("{indent}(taken under pressure — say `undo` if that was wrong)");
            }
        }
        Resolution::Ambiguous { candidates } => {
            println!("{indent}ambiguous. did you mean:");
            for (index, candidate) in candidates.iter().enumerate() {
                println!("{indent}  {}) {}", index + 1, candidate.intent.echo());
            }
        }
        Resolution::Incomplete {
            verb,
            missing,
            filled,
            ..
        } => {
            let so_far: Vec<_> = filled.iter().map(|a| a.value.as_str()).collect();
            println!("{indent}-> {} {}", verb.canonical(), so_far.join(" "));
            println!("{indent}{} what? ({missing:?})", verb.canonical());
        }
        Resolution::Elsewhere { verb } => {
            println!(
                "{indent}there is nothing here to {} with (§7)",
                verb.canonical()
            );
        }
        Resolution::InSpell { word } => {
            println!("{indent}`{}` is a word for spells (§8)", word.canonical());
        }
        Resolution::Unresolved { suggestions } => {
            let names: Vec<_> = suggestions.iter().map(|verb| verb.canonical()).collect();
            println!(
                "{indent}the orb does not know that word. perhaps: {}",
                names.join(", ")
            );
        }
    }
}

/// The same ambiguous input, calm and under siege (§6).
fn siege_contrast(scene: &Scene) {
    println!("\nThe same input, calm and under siege — §6: disambiguation never blocks\n");
    for mode in [Mode::Calm, Mode::Siege] {
        let resolution = resolve("brew", scene, mode);
        let label = if mode == Mode::Calm { "calm " } else { "siege" };
        match resolution {
            Resolution::Ambiguous { candidates } => {
                let offered: Vec<_> = candidates.iter().map(|c| c.intent.echo()).collect();
                println!("  {label}  asks: {}", offered.join(" | "));
            }
            Resolution::Resolved { intent, confidence } => {
                println!("  {label}  runs: {} ({confidence:?})", intent.echo());
            }
            Resolution::Incomplete { verb, missing, .. } => {
                println!("  {label}  needs a {missing:?} for {}", verb.canonical());
            }
            Resolution::Elsewhere { verb } => {
                println!("  {label}  not here: {}", verb.canonical());
            }
            Resolution::InSpell { word } => {
                println!("  {label}  for spells: {}", word.canonical());
            }
            Resolution::Unresolved { .. } => println!("  {label}  nothing"),
        }
    }
}

/// The numbers §15's gate is computed from.
fn summary(log: &ParseLog) {
    let total = log.records().len();
    println!("\nInstrumentation — {total} inputs\n");
    println!("  resolved    {}", log.resolved());
    println!("  ambiguous   {}", log.ambiguous());
    println!("  incomplete  {}", log.incomplete());
    println!("  unresolved  {}", log.unresolved());
    println!("\n  Export with --tsv. One row per candidate, so failures can be");
    println!("  clustered by cause rather than only counted.\n");
}

/// Type at the orb.
fn repl(scene: &Scene, log: &mut ParseLog, mut tick: u64) -> u64 {
    if std::io::stdin().is_terminal() {
        println!("\nType a command. Blank line or Ctrl-D to leave.\n");
    }

    let stdin = std::io::stdin();
    loop {
        print!("orbs:~$ ");
        let _ = std::io::stdout().flush();

        // The lock is taken and released *before* the match rather than in its
        // scrutinee, where the temporary would live to the end of the match and
        // hold stdin across the arms.
        let mut line = String::new();
        let read = stdin.lock().read_line(&mut line);
        match read {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                eprintln!("input error: {error}");
                break;
            }
        }

        let input = line.trim();
        if input.is_empty() {
            break;
        }

        tick += 1;
        let analysis = analyse(input, scene, Mode::Calm);
        show(&analysis.resolution, "  ");
        println!();
        log.push(ParseRecord::new(tick, input, Mode::Calm, &analysis));
    }

    println!("\nthe orb cools.\n");
    tick
}
