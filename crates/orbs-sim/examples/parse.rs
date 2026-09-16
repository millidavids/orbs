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

use std::collections::BTreeMap;

use orbs_sim::content::{Example, Phrasing, Phrasings};
use orbs_sim::parser::{
    Confidence, Mode, NounKind, ParseLog, ParseRecord, Resolution, Scene, analyse, resolve,
};
use orbs_sim::{Augur as _, Grammar};

/// The slice's world: brewing and archive, both thin (DESIGN.md §15).
fn tower() -> Scene {
    Scene::new()
        .with(NounKind::Place, "/tower/laboratory")
        .with(NounKind::Place, "/tower/archive")
        .with(NounKind::Place, "/tower/sanctum")
        .with(NounKind::File, "feed.log")
        .with(NounKind::File, "purge_cycle.log")
        .with(NounKind::Essence, "clarity")
        .with(NounKind::Essence, "warding")
        .with(NounKind::Vessel, "alembic")
        .with(NounKind::Script, "night_watch")
        .with(NounKind::Scroll, "gleaning-scroll")
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

    if std::env::args().any(|arg| arg == "--bench") {
        // **Everything the content names, not the room anyone is standing in.**
        // The slice scene below holds one reagent; the recipes name thirty-three,
        // so expanding a corpus over it measured a thirtieth of the corpus and
        // made the model's parameters-per-example ratio look far worse than it is.
        bench(&orbs_sim::content::corpus_scene());
        return;
    }

    if std::env::args().any(|arg| arg == "--spells") {
        spells(&orbs_sim::content::corpus_scene());
        return;
    }

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

/// What the spell language accepts of `spellings.toml`, before any reader.
///
/// **The baseline, and it is expected to be near nought.** These are lines the
/// language cannot read — that is the whole reason they were written down. A
/// number climbing here means a phrasing was authored that already worked, which
/// would teach a reader to rewrite what needed no rewriting.
fn spells(scene: &Scene) {
    let spellings = orbs_sim::content::Phrasings::spellings();
    let corpus = spellings.corpus(scene);
    let holdout = spellings.holdout(scene);
    let refused = spellings.refused(scene);

    println!("\nO.R.B.S. — how much of a loose spell the language already reads\n");
    println!(
        "  {} templates, expanded to {} corpus and {} holdout lines",
        spellings.entries().len(),
        corpus.len(),
        holdout.len(),
    );
    println!("  {} lines that must come back untouched\n", refused.len());

    // **Parsing is not understanding, and this is where the difference bites.**
    // A loose line usually has no leading spell word and reads as a bare
    // command. But `if the alembic has finished` *parses* — as "holds a thing
    // called finished" — and means something else entirely. That is the class a
    // reader has to fix and the class a re-parse can never catch, which is why
    // the assembler's rule is span coverage rather than "does it parse".
    let already = corpus
        .iter()
        .filter(|example| orbs_sim::tower::spell::reads_cleanly(&example.said))
        .count();
    println!(
        "    parses into *something*      {already} of {}   <- and means the wrong thing",
        corpus.len(),
    );

    // ...and the other side. Every refusal must already be a sound statement: a
    // reader rewriting one would be breaking a line that worked, and a line here
    // that does not parse is an authoring mistake in `spellings.toml` rather
    // than anything about a reader.
    let unsound: Vec<&String> = refused
        .iter()
        .filter(|line| !orbs_sim::tower::spell::reads_cleanly(line))
        .collect();
    println!(
        "    sound already                {} of {}   <- must be all",
        refused.len() - unsound.len(),
        refused.len(),
    );

    if !unsound.is_empty() {
        println!("\n  refusals that are not sound statements — fix the file:\n");
        for line in unsound.iter().take(10) {
            println!("    {line:?}");
        }
    }
    println!();
}

/// Measure the parser against every authored phrasing (§19, *the augury*).
///
/// # Not §15's gate, and the distinction is the whole honesty of it
///
/// ROADMAP: *"the author cannot stand in for [a tester] — someone who knows the
/// canonical vocabulary is measuring their memory. A number that looks like this
/// one but was produced in-house would be worse than no number."* Every line
/// here was written by the person who wrote the synonym table, so this is a
/// **coverage lint**: which authored phrasings the parser misses. That is worth
/// a great deal — each miss is either a one-line synonym fix or a phrasing only
/// a reader will ever catch — and it is not a gate.
///
/// # The holdout is what is reported
///
/// `say` is the corpus a grammar or a model is built from, so measuring on it
/// says only that the building worked. `holdout` is taught to nothing.
fn bench(scene: &Scene) {
    let phrasings = Phrasings::builtin();
    let corpus = phrasings.corpus(scene);
    let holdout = phrasings.holdout(scene);

    println!("\nO.R.B.S. — how much of what people say the orb reads\n");
    println!(
        "  {} templates, expanded to {} corpus and {} holdout examples\n",
        phrasings.entries().len(),
        corpus.len(),
        holdout.len(),
    );

    // **A sentence taught as two commands is a label no reader can get right**,
    // and no single template shows it: `put the {reagent} in the {place}` is
    // `move`, `put the {reagent} in the alembic` is `distil`, and the first
    // becomes the second the moment the alembic is a place. Counted over both
    // populations, because a holdout line that is another command's taught
    // sentence is scored a miss for being read exactly as it was taught.
    let mut meant: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for example in corpus.iter().chain(&holdout) {
        let commands = meant.entry(example.said.as_str()).or_default();
        if !commands.contains(&example.canonical.as_str()) {
            commands.push(example.canonical.as_str());
        }
    }
    let mut twice: BTreeMap<(&str, &str), (usize, &str)> = BTreeMap::new();
    for (said, commands) in &meant {
        if let [first, second, ..] = commands.as_slice() {
            twice
                .entry((head(first), head(second)))
                .or_insert((0, said))
                .0 += 1;
        }
    }
    let taught_twice: usize = twice.values().map(|(count, _)| count).sum();
    println!("  {taught_twice} sentences are taught as two commands   <- must be 0");
    for ((first, second), (count, said)) in &twice {
        println!("    {count:>5}  {first} / {second}   e.g. {said:?}");
    }
    println!();

    let mut misses: Vec<&Example> = Vec::new();
    let mut read = 0usize;
    for example in &holdout {
        if reaches(&example.said, &example.canonical, scene) {
            read += 1;
        } else {
            misses.push(example);
        }
    }

    // The baseline. Compiled from `say` only — a grammar taught the holdout
    // would match it perfectly and measure nothing.
    let grammar = Grammar::builtin();
    let grammar_read = holdout
        .iter()
        .filter(|example| {
            grammar
                .read(&example.said)
                .iter()
                .any(|echo| reaches(echo, &example.canonical, scene))
        })
        .count();
    // **And the reading the game would run**, which the corpus half below
    // already scores by. The line above credits a hit when *any* of four
    // readings reaches the command — the rule the corpus comment calls
    // flattering — and `measure` prints the same pair for the trained reader.
    let grammar_ran = holdout
        .iter()
        .filter(|example| {
            let readings = grammar.read(&example.said);
            orbs_sim::parser::reading_to_run(&readings, scene, Mode::Calm)
                .is_some_and(|echo| reaches(echo, &example.canonical, scene))
        })
        .count();

    // **What shipping the grammar as a reader would actually buy**, which is
    // the only number that decides anything: `Sim::submit_reading` asks the
    // matcher first and a reader only for what it could not read, so the two
    // rates above are halves of this one rather than rivals.
    let together = holdout
        .iter()
        .filter(|example| {
            reaches(&example.said, &example.canonical, scene)
                || grammar
                    .read(&example.said)
                    .iter()
                    .any(|echo| reaches(echo, &example.canonical, scene))
        })
        .count();

    let total = holdout.len().max(1);
    println!("  on the holdout — phrasings nothing has been taught:\n");
    println!("    today's parser      {:>6.1}%", percent(read, total));
    println!(
        "    a grammar from `say`{:>6.1}%   ({} templates) <- any of its readings reaches it",
        percent(grammar_read, total),
        grammar.len(),
    );
    println!(
        "      the reading it runs {:>5.1}%   <- the one `submit_reading` would take",
        percent(grammar_ran, total),
    );
    println!(
        "    the two together    {:>6.1}%   <- what shipping the grammar buys",
        percent(together, total),
    );
    // **Not measured here, and it cannot be**: `orbs-sim` may never depend on
    // `orbs-augury` (CLAUDE.md rule 1), so the trained reader's line is that
    // crate's `measure` example, on these same holdouts.
    println!(
        "    the trained reader     cargo run --release -p orbs-augury --example measure --features train\n"
    );

    // **The corpus rate is the lint half.** A `say` line the parser already
    // reads is one a reader need never see; one it misses is either a
    // synonym-table fix costing one row of `vocabulary.rs`, or a phrasing no
    // table can hold. Both are worth knowing before anything is trained.
    let corpus_read = corpus
        .iter()
        .filter(|example| reaches(&example.said, &example.canonical, scene))
        .count();
    // A grammar reading its *own* templates back is the sanity check on the
    // matcher: well below 100% means `capture` is broken, and the holdout rate
    // above would be measuring that rather than generalisation.
    // **Two failures wear one number, and they mean opposite things.** The
    // grammar may fail to match its own template — that is `capture` broken —
    // or it may match, produce exactly the right command, and have *the parser*
    // refuse it because this thin bench scene holds no such noun. The second is
    // the scene's fault and says nothing about the reader, so it is counted
    // apart rather than left to look like a bug.
    let mut grammar_corpus = 0usize;
    let mut unmatched = 0usize;
    let mut misread: Vec<(Misread, String)> = Vec::new();
    // Outranked misreads by (the command wanted, the command that ran), with a
    // count and the first line that did it.
    let mut outranked: BTreeMap<(String, String), (usize, String)> = BTreeMap::new();
    let mut unresolvable: Vec<&str> = Vec::new();
    for example in &corpus {
        // **The caller's rule, not a looser one.** `Sim::submit_reading` takes
        // the reading `parser::reading_to_run` chooses, so that is what is
        // measured here — scoring against *any* reading would flatter a reader
        // that offers four and means none of them.
        let readings = grammar.read(&example.said);
        let taken = orbs_sim::parser::reading_to_run(&readings, scene, Mode::Calm);
        match (readings.is_empty(), taken) {
            (true, _) => unmatched += 1,
            (false, Some(echo)) if reaches(echo, &example.canonical, scene) => grammar_corpus += 1,
            // It offered something that runs, and it was the wrong command.
            (false, Some(echo)) => {
                let cause = Misread::of(&readings, echo, &example.canonical, scene);
                if cause == Misread::Outranked {
                    outranked
                        .entry((head(&example.canonical).to_owned(), head(echo).to_owned()))
                        .or_insert_with(|| (0, example.said.clone()))
                        .0 += 1;
                }
                misread.push((cause, format!("{:?} -> {echo}", example.said)));
            }
            // It offered readings and this thin scene resolves none of them.
            (false, None) => unresolvable.push(example.canonical.as_str()),
        }
    }
    println!(
        "  on the corpus — what a reader would not have to learn:\n\n    today's parser      {:>6.1}%\n    a grammar from `say`{:>6.1}%   <- its own templates",
        percent(corpus_read, corpus.len().max(1)),
        percent(grammar_corpus, corpus.len().max(1)),
    );
    println!("      {unmatched} it could not match      <- `capture` is broken if this is not 0");
    println!(
        "      {} it read as another command  <- the reader's own mistakes",
        misread.len(),
    );
    println!(
        "      {} this thin scene cannot resolve  <- the bench's world, not the reader",
        unresolvable.len(),
    );
    // **By cause, because the causes need opposite fixes.** One count hid all
    // four, and a fix for one moved the total by less than the other three
    // shifted under it.
    for cause in Misread::ALL {
        let lines: Vec<&String> = misread
            .iter()
            .filter(|(of, _)| *of == cause)
            .map(|(_, line)| line)
            .collect();
        println!("\n      {:>5} {}", lines.len(), cause.says());
        for line in lines.iter().take(3) {
            println!("              {line}");
        }
    }
    // **Which command stood in front of which**, because the largest cause is
    // one number over many different collisions, and each is its own fix.
    let mut pairs: Vec<_> = outranked.into_iter().collect();
    pairs.sort_by(|a, b| b.1.0.cmp(&a.1.0).then_with(|| a.0.cmp(&b.0)));
    println!("\n      outranked, by the command wanted <- the one that ran first:\n");
    for ((wanted, ran), (count, example)) in pairs.iter().take(15) {
        println!("        {count:>5}  {wanted:<9} <- {ran:<9} e.g. {example:?}");
    }
    println!();

    // **The misses are the point, not the percentage.** §15 acts on the
    // *clustering* of failures, and a rate with no examples beside it cannot be
    // acted on at all.
    println!("  what it misses, by command:\n");
    let mut by_command: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for miss in &misses {
        by_command
            .entry(miss.canonical.as_str())
            .or_default()
            .push(miss.said.as_str());
    }
    for (canonical, said) in &by_command {
        println!("    {canonical}");
        for line in said.iter().take(3) {
            println!("      {line}");
        }
        if said.len() > 3 {
            println!("      ...and {} more", said.len() - 3);
        }
    }

    // A template whose own canonical form does not resolve is measuring
    // nothing, and would show up here as a command that misses everything.
    let broken: Vec<&Phrasing> = phrasings
        .entries()
        .iter()
        .filter(|entry| {
            !entry.canonical.contains('{') && !reaches(&entry.canonical, &entry.canonical, scene)
        })
        .collect();
    if !broken.is_empty() {
        println!("\n  ** these canonical forms do not resolve, so their rows mean nothing:");
        for entry in broken {
            println!("     {}", entry.canonical);
        }
    }
    println!();
}

/// A command's first word — the verb, for grouping lines by what they do.
fn head(line: &str) -> &str {
    line.split_whitespace().next().unwrap_or("")
}

/// Why a reader's reading ran as the wrong command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Misread {
    /// The right command was among the readings, and the parser would not run
    /// it here — a refusal, or a thin scene.
    Refused,
    /// The right command was among the readings and would have run, but a wrong
    /// one earlier in the list ran first.
    Outranked,
    /// The command that ran left words over: a reading that did not account for
    /// what it was handed, resolving anyway.
    WordsOver,
    /// A wrong command that accounted for everything — the reader's own choice.
    AnotherVerb,
}

impl Misread {
    const ALL: [Self; 4] = [
        Self::Refused,
        Self::Outranked,
        Self::WordsOver,
        Self::AnotherVerb,
    ];

    /// Which of the four `taken` was, of the `readings` offered for `meant`.
    fn of(readings: &[String], taken: &str, meant: &str, scene: &Scene) -> Self {
        if readings.iter().any(|reading| reading == meant) {
            return if reaches(meant, meant, scene) {
                Self::Outranked
            } else {
                Self::Refused
            };
        }
        let left_over = analyse(taken, scene, Mode::Calm)
            .candidates
            .first()
            .is_some_and(|best| best.leftover > 0);
        if left_over {
            Self::WordsOver
        } else {
            Self::AnotherVerb
        }
    }

    /// What a line of the report says about it.
    const fn says(self) -> &'static str {
        match self {
            Self::Refused => "the right one was offered and would not run here",
            Self::Outranked => "the right one was offered and a wrong one ran first",
            Self::WordsOver => "the one that ran left words it could not use",
            Self::AnotherVerb => "the one that ran used every word, and is wrong",
        }
    }
}

/// Whether `said` resolves to the command `meant` names.
fn reaches(said: &str, meant: &str, scene: &Scene) -> bool {
    matches!(
        resolve(said, scene, Mode::Calm),
        Resolution::Resolved { ref intent, .. } if intent.echo() == meant
    )
}

/// A percentage, to one decimal, without pulling in a formatter.
fn percent(part: usize, whole: usize) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a corpus is thousands of lines, not 2^53"
    )]
    let (part, whole) = (part as f64, whole as f64);
    part / whole * 100.0
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
        Resolution::TakesNothing { verb, extra, .. } => {
            println!(
                "{indent}{} takes nothing — '{extra}' is not something it can use",
                verb.canonical()
            );
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
            Resolution::TakesNothing { verb, .. } => {
                println!("  {label}  takes nothing: {}", verb.canonical());
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
