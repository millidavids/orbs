//! What the circle says is enough to hold the beast.

use std::collections::BTreeSet;

use orbs_render::{Circle, CircleGiven, CircleLine, Frame, GridSize, Rect};
use orbs_sim::Sim;
use orbs_sim::content::Prose;
use orbs_sim::tower::circle::Humour;

use super::speech::{speak, summary};

fn given(names: &[&str]) -> Vec<CircleGiven> {
    names
        .iter()
        .map(|name| CircleGiven {
            name: name.trim_start_matches('~').to_owned(),
            turned: name.starts_with('~'),
        })
        .collect()
}

fn board() -> Circle {
    let lit = |sense: usize| (0..8).map(|row| (row >> (2 - sense)) & 1 == 1).collect();
    Circle {
        lines: vec![
            CircleLine {
                glyph: "sunwise".into(),
                humour: "yoke".into(),
                given: given(&["blood", "bone"]),
            },
            CircleLine {
                glyph: "widdershins".into(),
                humour: "heed".into(),
                given: given(&["bone", "~breath"]),
            },
            CircleLine {
                glyph: "keystone".into(),
                humour: "oppose".into(),
                given: given(&["sunwise", "widdershins"]),
            },
        ],
        senses: vec!["blood".into(), "bone".into(), "breath".into()],
        lit: vec![lit(0), lit(1), lit(2)],
        temper: vec![false, true, false, false, false, false, false, false],
        answer: None,
        labels: ["temper".into(), "answer".into()],
        tally: "not yet called in".into(),
    }
}

/// **What a reader hears is a sentence at every count** — one row, many, and
/// none. A hand-edited save whose last answer already agrees loads a waiting
/// beast with nothing balking, and the summary trailed off *"balked at rows "*.
#[test]
fn the_spoken_summary_reads_for_one_row_many_and_none() {
    let prose = Prose::builtin();
    let spoken = |temper: Vec<bool>, answer: Vec<bool>| {
        let circle = Circle {
            temper,
            answer: Some(answer),
            ..board()
        };
        let mut frame = Frame::new(GridSize::new(60, 16));
        let mut painter = frame.painter(Rect::new(0, 0, 60, 16));
        speak(&mut painter, &circle, &prose);
        frame.speech().to_transcript()
    };
    let lit = vec![false, true, false, false, false, false, false, false];

    let one = spoken(lit.clone(), vec![true; 8]);
    assert!(one.contains("lit on row 2 (breath)."), "{one}");
    assert!(one.contains("balked at rows 1 3 4 5 6 7 8"), "{one}");

    let single = spoken(lit.clone(), vec![false; 8]);
    assert!(single.contains("balked at row 2"), "{single}");

    let none = spoken(lit.clone(), lit);
    assert!(none.contains("agreed on every row"), "{none}");
    assert!(!none.contains("balked at"), "{none}");
}

/// **A turned wire is said, and a lit row names its senses** — the two facts the
/// silent painted rows show an eye and nothing else tells an ear.
#[test]
fn a_turned_wire_is_said_and_every_lit_row_names_its_senses() {
    let prose = Prose::builtin();
    let circle = Circle {
        temper: vec![true, false, false, true, false, false, false, true],
        ..board()
    };
    let said = summary(&circle, &prose);
    assert!(said.contains("over bone and turned breath"), "{said}");
    assert!(said.contains("over blood and bone"), "{said}");
    assert!(!said.contains("turned blood"), "{said}");
    assert!(
        said.contains("rows 1 (no sense lit), 4 (bone breath), 8 (blood bone breath)"),
        "{said}",
    );
}

/// What the summary says about the circuit, read back out of the words alone.
struct Heard {
    /// Every sense named anywhere in what a glyph is given.
    senses: Vec<String>,
    /// Each outer glyph, or the lone keystone: its word and what it is given,
    /// `true` where turned.
    wired: Vec<(String, [(String, bool); 2])>,
    /// The sets of senses the temper is lit on.
    lit: BTreeSet<BTreeSet<String>>,
}

/// Read the summary the way a listener would: nothing from the board, nothing
/// from the model — the words `speak` announces and the humours' own rules,
/// which `recall` teaches.
fn hear(said: &str) -> Heard {
    let (glyphs, rest) = said
        .split_once(". the temper is lit on ")
        .unwrap_or_else(|| panic!("no temper in {said:?}"));
    let rows = rest
        .trim_start_matches("rows ")
        .trim_start_matches("row ")
        .split_once(". ")
        .map_or(rest, |(rows, _)| rows);

    let mut wired = Vec::new();
    let mut senses = Vec::new();
    for sentence in glyphs.split(". ") {
        let Some((glyph, over)) = sentence.split_once(" is limned ") else {
            continue;
        };
        let Some((_, inputs)) = over.split_once(", over ") else {
            continue;
        };
        let Some((first, second)) = inputs.split_once(" and ") else {
            panic!("no two inputs in {sentence:?}");
        };
        let input = |text: &str| match text.strip_prefix("turned ") {
            Some(name) => (name.to_owned(), true),
            None => (text.to_owned(), false),
        };
        let pair = [input(first), input(second)];
        for (name, _) in &pair {
            if !senses.contains(name) {
                senses.push(name.clone());
            }
        }
        wired.push((glyph.to_owned(), pair));
    }

    let lit = rows
        .split(", ")
        .map(|item| {
            let inside = item
                .split_once('(')
                .map_or("", |(_, inside)| inside.trim_end_matches(')'));
            inside
                .split(' ')
                .filter(|word| senses.iter().any(|sense| sense == word))
                .map(ToOwned::to_owned)
                .collect()
        })
        .collect();
    Heard { senses, wired, lit }
}

/// A limning that answers what was heard, found by trying all of them.
fn solve(heard: &Heard) -> Option<Vec<(String, Humour)>> {
    let combos: Vec<BTreeSet<String>> = (0..1u32 << heard.senses.len())
        .map(|bits| {
            heard
                .senses
                .iter()
                .enumerate()
                .filter(|(index, _)| bits >> index & 1 == 1)
                .map(|(_, name)| name.clone())
                .collect()
        })
        .collect();
    let given = |pair: &[(String, bool); 2], combo: &BTreeSet<String>| {
        let value = |(name, turned): &(String, bool)| combo.contains(name) != *turned;
        (value(&pair[0]), value(&pair[1]))
    };
    let answers = |humours: &[Humour]| {
        combos.iter().all(|combo| {
            let lit = match (heard.wired.as_slice(), humours) {
                ([(_, pair)], [keystone]) => {
                    let (a, b) = given(pair, combo);
                    keystone.answer(a, b)
                }
                ([(_, sunwise), (_, widdershins)], [keystone, outer, inner]) => {
                    let (a, b) = given(sunwise, combo);
                    let (c, d) = given(widdershins, combo);
                    keystone.answer(outer.answer(a, b), inner.answer(c, d))
                }
                _ => return false,
            };
            lit == heard.lit.contains(combo)
        })
    };
    match heard.wired.as_slice() {
        [(glyph, _)] => Humour::ALL
            .into_iter()
            .find(|keystone| answers(&[*keystone]))
            .map(|keystone| vec![(glyph.clone(), keystone)]),
        [(sunwise, _), (widdershins, _)] => Humour::ALL.into_iter().find_map(|keystone| {
            Humour::ALL.into_iter().find_map(|outer| {
                Humour::ALL.into_iter().find_map(|inner| {
                    answers(&[keystone, outer, inner]).then(|| {
                        vec![
                            ("keystone".to_owned(), keystone),
                            (sunwise.clone(), outer),
                            (widdershins.clone(), inner),
                        ]
                    })
                })
            })
        }),
        _ => None,
    }
}

/// **§14's claim, made executable: a reader who hears the board can hold the
/// beast in one call.** For beasts generated across two hundred seeds — whole
/// circles with every kind of turned wire, and a sealed tower's lesser ones —
/// the summary alone is parsed, a limning found that answers it, typed, and the
/// beast called in once. The solver never sees the board, the model or the row
/// numbering; if the words left anything out, some beast would balk.
#[test]
fn a_reader_who_hears_the_summary_holds_every_beast_in_one_call() {
    let prose = Prose::builtin();
    let mut turned = 0;
    for seed in 0..200 {
        // `debug_reach` is how a test opens the menagerie in a sealed tower, and
        // it exists only in a debug build; a release run holds the whole circles.
        let sealed = cfg!(debug_assertions) && seed % 10 == 9;
        let mut sim = if sealed {
            Sim::sealed(seed)
        } else {
            Sim::new(seed)
        };
        if sealed {
            sim.submit("debug_reach lens_1");
            sim.step();
        }
        for line in ["attend menagerie", "summon"] {
            sim.submit(line);
            sim.step();
        }
        let Some(circle) = sim.circle() else {
            panic!("seed {seed} drew no beast");
        };
        let said = summary(&circle, &prose);
        let heard = hear(&said);
        turned += heard
            .wired
            .iter()
            .flat_map(|(_, pair)| pair)
            .filter(|(_, turned)| *turned)
            .count();
        let Some(limning) = solve(&heard) else {
            panic!("seed {seed}: nothing answers what was heard: {said:?}");
        };
        for (glyph, humour) in limning {
            sim.submit(&format!("limn {glyph} {}", humour.word()));
            sim.step();
        }
        sim.submit("summon");
        sim.step();
        assert!(
            sim.circle().is_none() && sim.tally("event:figure") == 1,
            "seed {seed}: a limning heard from the summary did not hold: {said:?}",
        );
    }
    assert!(turned > 100, "only {turned} turned wires were heard");
}
