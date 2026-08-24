//! The seal on a far wizard's orb, and the two ways a player breaks it.
//!
//! DESIGN.md §10 gives scrying *deduction*, and this is the shape it took: four
//! sigils drawn from six, **repeats allowed**, 1296 codes. You press figures
//! against the ward and read how it answers. It is Mastermind, and §19 records
//! the version that was Mastermind-shaped without being Mastermind.
//!
//! # The orb keeps no candidate set and deduces nothing
//!
//! **This is the load-bearing rule of the whole domain**, and two designs have
//! broken it. The first had the sim track which codes were still consistent and
//! publish each sigil's standing: "press anything still consistent" solves in
//! ~4.2 presses, so an orb that does the bookkeeping has done the entire puzzle.
//! Mastermind's difficulty *is* the bookkeeping.
//!
//! The second broke it more quietly. It said `settled` per socket — *this
//! position is correct* — and `untried`, which falls to nought on the same fact.
//! No codemaker answers a question about one position, and a player who could
//! ask it never has to deduce.
//!
//! So what is stored here is only what a player could keep on paper: the code,
//! the aperture, the last answer, and which way each half of it moved. Nothing
//! is inferred from any of it.
//!
//! # Two channels onto one ward
//!
//! | | The player | A spell |
//! |---|---|---|
//! | Reads | `aligned` and `astray`, as numbers | `closer`/`level`/`further` and `richer`/`unchanged`/`poorer` |
//! | Method | deduction | one socket at a time |
//! | Presses | ~5.15 | ~11.93 |
//!
//! Both use the same two verbs, and the spell's channel is strictly *less* than
//! the player's — the deltas are derivable from the numbers and not the other way
//! round. That is what lets §8's language automate a puzzle it cannot possibly
//! solve: the language has no variables and no accumulator
//! (`parser::question`), so a spell can only act on what the world has written
//! down, and *which way did it move* is a fact the world can write down without
//! deducing anything.
//!
//! **The sweep terminates, and the proof is why the domain works.** Move one
//! socket and nothing else: `aligned` can then only change because of that
//! socket, so stepping it cyclically through six reaches the code's sigil within
//! five and says so on arrival. A sweep is therefore a *rule*, not a search —
//! exactly what Trémaux is for the archive's maze. See `dev_spells.toml`'s
//! `breaking` for the shape a player writes.

use bevy_ecs::prelude::*;
use rand::Rng as _;

use crate::rng::{RngStream, Rngs};

/// The sigils a ward is drawn from.
///
/// **Six real alchemical substances**, and the names were swept before they were
/// authored. A first set failed: `fuzzy::similarity` scored `crown` against
/// `cron` (a shell synonym for `bind`) at **800**, above the 750 that
/// `tests/naming.rs` documents as the highest the game tolerates, with `iron`,
/// `star` and `sun` close behind. Sigils are `Place` nouns and so nameable from
/// every room, which puts them fully in the parser's way.
pub const SIGILS: [&str; 6] = ["nitre", "alum", "borax", "quartz", "pewter", "ochre"];

/// The four positions of a ward, in order.
///
/// Ordinals rather than invented names: a player reading `aligned 2` needs to
/// know *which* two, and `first`/`second` is the only naming that needs no
/// explanation. They are also what a spell's ladder walks.
pub const SOCKETS: [&str; 4] = ["first", "second", "third", "fourth"];

/// How many sockets a ward has.
pub const WIDTH: usize = SOCKETS.len();

/// Presses a player is expected to need. Beyond it the yield drops (§11.5).
///
/// **Six, and measured rather than chosen.** Consistent play from the opening
/// aperture averages 5.15 with a worst case of 9 over all 1296 codes — so at
/// five, roughly half of *competent* hand play fell into `yield_of`'s reduced
/// tier, which reads as a penalty for playing well. It was five when the space
/// was 360 codes and the average was 4.14.
pub const PAR: u32 = 6;

/// What one press costs the tower, in ticks.
///
/// **Nought: a press is instant and takes no slot.** It was twelve, held through
/// the ordinary production machinery, which is what ROADMAP's *"a read is not a
/// brew"* priced. That scarcity is withdrawn — a press is now `dial`'s equal, free
/// and immediate, and the lens competes with the laboratory for nothing.
///
/// Kept as a named nought rather than deleted, because it is the number the
/// domain's rates were derived against and a future decision to price a press
/// again has to change one line rather than reintroduce a concept.
pub const PRESS_TICKS: u64 = 0;

/// Whether the last press helped, held, or hurt.
///
/// **The spell's whole channel, and it is a tally rather than an inference.**
/// Did `aligned` go up? That is a fact the world has without reasoning about
/// what it implies — which is what keeps [`Ward`] honest while still giving §8's
/// stateless language something to hill-climb on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shift {
    /// `aligned` rose.
    Gained,
    /// `aligned` did not move.
    Held,
    /// `aligned` fell.
    Lost,
}

impl Shift {
    /// The word a spell asks for.
    ///
    /// **`gained`, `held` and `lost` were the first names and all three leaked.**
    /// A reading is a `NounKind::Sense`, which `NounKind::Any` reaches — so
    /// `purge` and `verify` resolve against them from every room in the tower,
    /// and `purge grind` fuzzy-matched `gained` at full confidence and reported
    /// *"there is no gained within reach"*. verbs.md warns about exactly this
    /// leak; what makes these three worse than the maze's is that they are
    /// ordinary English participles, so they sit in the way of half the words a
    /// player might mistype.
    ///
    /// Comparatives are safer and read better: a ward is nearer, or it is not.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Gained => "closer",
            Self::Held => "level",
            Self::Lost => "further",
        }
    }

    /// The word for the same movement in `astray`.
    ///
    /// **A second set, because one word cannot mean two things.** `closer` is
    /// about placement and this is about *membership* — whether the figure is
    /// made of more of the right sigils than the last one was, wherever they
    /// sit. A spell that could not tell those apart could not tell *swap two of
    /// these* from *none of these belong*.
    ///
    /// Comparatives again, for the reason [`word`](Self::word) gives, and
    /// **swept rather than judged** — `tests/naming.rs` walks every reading in
    /// the game against every word a player types, which is a sweep this file's
    /// first three readings shipped without. Two earlier sets fail it:
    ///
    /// | | scores | against |
    /// |---|---|---|
    /// | `warmer`/`even`/`cooler` | 667, 600 | `closer` and `level` — two
    ///   readings colliding *inside one domain*, worse than a verb near-miss |
    /// | `fuller`/`steady`/`thinner` | 667, 667 | `filter` (sift) and `study`
    ///   (research) — so `purge filter` resolved to `purge fuller` and answered
    ///   *"there is no fuller within reach"*, which is exactly the leak §19
    ///   records `gained`/`held`/`lost` causing |
    ///
    /// `thicker`/`thinner` fails too, at 715 against each other.
    #[must_use]
    pub const fn drift_word(self) -> &'static str {
        match self {
            Self::Gained => "richer",
            Self::Held => "unchanged",
            Self::Lost => "poorer",
        }
    }

    /// Every word, for the scene to register unconditionally.
    pub const ALL: [&'static str; 6] = [
        "closer",
        "level",
        "further",
        "richer",
        "unchanged",
        "poorer",
    ];

    /// The shift an `aligned` word names, for reading a save back.
    ///
    /// **Two readers, not one, because the two channels do not share a word.**
    /// A single `named` covering all six would round-trip `closer` into the
    /// `drift` field and `richer` into `shift`, which is a save that reads back
    /// as a ward saying something it never said.
    pub(crate) fn named(word: &str) -> Option<Self> {
        Self::reading(word, Self::word)
    }

    /// The same, for the `astray` channel's word.
    pub(crate) fn drift_named(word: &str) -> Option<Self> {
        Self::reading(word, Self::drift_word)
    }

    /// Whichever shift spells itself `word` under `spelling`.
    ///
    /// Derived from the writer rather than written twice: a fourth `Shift` would
    /// otherwise need remembering here, and the failure would be a save that
    /// writes a word nothing reads back — silent, and only on reload.
    fn reading(word: &str, spelling: fn(Self) -> &'static str) -> Option<Self> {
        [Self::Gained, Self::Held, Self::Lost]
            .into_iter()
            .find(|&shift| spelling(shift) == word)
    }
}

/// Which way a count moved between two presses.
///
/// **Against the previous press, never a high-water mark.** The ratchet used to
/// make the aperture always *be* the best figure, so comparing to a running
/// maximum was the same thing; with the revert gone the only honest baseline is
/// the figure before this one.
const fn shift_of(now: u32, before: u32) -> Shift {
    if now > before {
        Shift::Gained
    } else if now < before {
        Shift::Lost
    } else {
        Shift::Held
    }
}

/// The readings the prism publishes, beside [`Shift::ALL`].
///
/// Counted, so the existing comparison grammar works on them:
/// `if the prism has 3 or more aligned`.
pub const COUNTS: [&str; 3] = [ALIGNED, ASTRAY, SPENT];

/// Right sigil, right socket.
pub const ALIGNED: &str = "aligned";
/// Right sigil, wrong socket.
pub const ASTRAY: &str = "astray";
/// Presses used on this reading.
pub const SPENT: &str = "spent";

// **`marks`, `settled`, `loose` and `untried` were consts here** and are gone
// with the props they served (§19). Each was a verdict on a *position* — is this
// socket right, what has been tried in it — which is the one thing a codemaker
// may never answer. What is left is [`Shift::ALL`]: which way each number moved.

/// Every word a spell may ask a lens noun for.
///
/// Registered unconditionally in `scene_at`, exactly as the maze's are: a
/// spell's `if` resolves **at cast**, when no socket has settled and no sigil has
/// a mark, so a vocabulary that appeared with the state would make half the
/// conditions in a solver unresolvable at the moment they are compiled.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    // **Six words, and they are all deltas.** This used to publish `aligned`,
    // `astray` and `spent` as counts, `marks` per socket and per sigil, and
    // `settled`/`loose`/`untried` per socket — nine kinds of answer, several of
    // which no codemaker may give.
    //
    // What is left is the only thing Mastermind tells you: which way each of the
    // two numbers moved. The *counts themselves* are still shown to the player
    // on the sheet and in the transcript — that is the game — but a spell asks
    // about movement, and works out the rest or does not (§19).
    Shift::ALL.to_vec()
}

/// A far orb's seal, and everything the player has learned about it.
#[derive(Component, Debug, Clone)]
pub struct Ward {
    /// The hidden figure, as indices into [`SIGILS`]. Never published.
    code: [usize; WIDTH],
    /// What the next press will send.
    aperture: [usize; WIDTH],
    /// The last press's answer — **both halves of one figure's**.
    ///
    /// These were once one field short of a pair: `astray` was committed
    /// unconditionally while `aligned` only moved on a gain, so a non-improving
    /// press reported the *held* figure's aligned beside the *pressed* figure's
    /// astray, and a pair could sum to more than four on a four-socket lock. The
    /// ratchet that made that possible is gone; the pairing rule remains.
    ///
    /// They are also the baseline the two deltas are measured against.
    aligned: u32,
    astray: u32,
    /// Whether anything has been pressed yet.
    pressed: bool,
    /// How many presses this reading has taken.
    spent: u32,
    /// Which way `aligned` moved on the last press.
    shift: Option<Shift>,
    /// Which way `astray` moved on the last press.
    ///
    /// **The second channel, and Mastermind's other number.** A figure can be
    /// wholly wrong in placement and still be made of the right sigils; without
    /// this a spell cannot tell *swap two of these* from *none of these belong*.
    drift: Option<Shift>,
    /// Every press and its answer, oldest first — the board's whole content.
    ///
    /// **History, and only history.** Each entry is a sentence the transcript
    /// already carried; keeping them lets the board show five presses side by
    /// side, which is what makes deduction possible without a notepad. Nothing
    /// is derived from it — `press` reads the last answer from
    /// [`aligned`](Self::aligned), not from here.
    history: Vec<([usize; WIDTH], u32, u32)>,
}

impl Ward {
    /// Draw a ward from the lens's own stream.
    ///
    /// **Built in one go**, like the maze: a ward that filled in as it was
    /// probed would draw from the stream at a rate depending on how the ticks
    /// were consumed, which is the hazard `heat.rs` records.
    #[must_use]
    pub fn new(rngs: &mut Rngs) -> Self {
        let rng = rngs.stream(RngStream::Lens);
        // **Four independent draws, repeats and all — 1296 codes.** This was a
        // partial Fisher-Yates taking four *distinct* sigils, and the uniqueness
        // was the root of everything the domain had to build around it: with no
        // repeats a socket often cannot take the sigil you want because another
        // holds it, so a dial had to *exchange* the two — and a dial that moves
        // two sockets makes `aligned` rising unattributable, which is why the
        // ward needed a ratchet, a settle-lock and a per-socket tally to be
        // solvable at all. Classic Mastermind allows repeated colours; allowing
        // them here deletes the exchange and all three props with it (§19).
        //
        // **Still exactly four draws**, which is what replay depends on: the
        // count must not vary with the values, which is the hazard `heat.rs`
        // records and the reason the old form used Fisher-Yates over rejection.
        let mut code = [0usize; WIDTH];
        for slot in &mut code {
            *slot = rng.random_range(0..SIGILS.len());
        }

        // **The aperture opens on the first four sigils, not on the code.** A
        // random opening would make one reading in 1296 already solved, and a
        // fixed one makes the first press mean the same thing every time —
        // which is what a player learning the domain needs. It is deliberately
        // four *distinct* sigils even though the code need not be: an opening
        // with a repeat in it would teach the wrong first lesson.
        let aperture = [0, 1, 2, 3];
        Self {
            code,
            aperture,
            aligned: 0,
            astray: 0,
            pressed: false,
            spent: 0,
            shift: None,
            drift: None,
            history: Vec::new(),
        }
    }

    /// Turn `socket` to the next sigil round, and say which.
    ///
    /// **A cursor the ward no longer has to keep.** This used to take the first
    /// sigil the socket had not tried *since the last gain*, which meant storing
    /// a per-socket list of what had been tried — and that list was readable as
    /// `untried`, a per-socket number that fell to zero when a socket was
    /// proven. A count nobody could get from the two answers is the orb doing
    /// the player's bookkeeping (§19).
    ///
    /// Cyclic-next needs no stored cursor at all: the sigil a socket holds *is*
    /// the cursor, and it is on the board where the player can see it. Six dials
    /// return it to where it started, which is what lets a blind sweep be
    /// exhaustive without remembering anything.
    ///
    /// It always moves, so it always returns `true` — a cyclic step from any
    /// sigil lands on a different one.
    pub const fn advance(&mut self, socket: usize) -> bool {
        if socket >= WIDTH {
            return false;
        }
        let next = (self.aperture[socket] + 1) % SIGILS.len();
        self.seat(socket, next)
    }

    // `replenish`, `next_untried` and `best` are gone with the state they served.
    //
    // `replenish` existed because a four-rung ladder guarded on `untried` would
    // fall through to `wait` once every socket was exhausted and then press an
    // unchanged aperture for ever. A cyclic `advance` cannot exhaust, so the
    // termination guarantee is now a property of the dial rather than a system
    // that gives candidates back.
    //
    // `best` was honest only *because* of the ratchet: the aperture always was
    // the best figure, so a high-water mark described what was on the lock. With
    // the revert gone it would describe a figure the player no longer has — the
    // panel's meter would read three of four while the aperture aligns one,
    // which is exactly the "reporting the guess rather than the reading" failure
    // `panel.rs` warns of. The meter reads [`last`](Self::last) instead.

    /// Put `sigil` in `socket`. Returns whether anything moved.
    ///
    /// **One socket, always — which is the whole of what allowing repeats
    /// buys.** This used to *exchange*: with no repeats a socket could not
    /// simply take a sigil another held, so a dial changed one socket or two,
    /// and a press after a two-socket move could not say which of them the
    /// answer had moved for. Everything the ward carried to work around that —
    /// the ratchet, the settle-lock, the per-socket tally of what had been
    /// tried — was scaffolding for an ambiguity the uniqueness rule created.
    ///
    /// With a repeat allowed there is nothing to exchange with. A dial moves one
    /// socket, so `aligned` rising after one is **entailed** to be that socket's
    /// doing, and the player can make that inference themselves — which is the
    /// only inference Mastermind ever offers and the one the orb used to make
    /// for them (§19).
    pub const fn seat(&mut self, socket: usize, sigil: usize) -> bool {
        if socket >= WIDTH || sigil >= SIGILS.len() {
            return false;
        }
        if self.aperture[socket] == sigil {
            return false;
        }
        self.aperture[socket] = sigil;
        true
    }

    /// Send the aperture against the ward and record what came back.
    ///
    /// # It no longer ratchets, and that is the point
    ///
    /// A press used to snap the aperture back to the best figure whenever it did
    /// not gain, on the argument that a variable-free ladder cannot remember what
    /// a socket held before a bad press. That is true, and it was still the orb
    /// doing the player's bookkeeping: remembering which of *their* guesses was
    /// best is the whole labour of Mastermind.
    ///
    /// **What you dialled stands.** `aligned` can fall now, which the revert used
    /// to make almost unobservable — and a fallen `aligned` is real information a
    /// player can use, because dialling off a correct socket is the only thing
    /// that causes it.
    ///
    /// Nothing settles here either. The orb makes no claim about any socket; the
    /// player reads the two counts and decides (§19).
    pub fn press(&mut self) {
        let (aligned, astray) = self.answer();
        self.spent = self.spent.saturating_add(1);

        // **Both deltas are against the previous press**, not against a
        // high-water mark. That distinction is the ratchet's ghost: `best` was
        // the baseline because the aperture always *was* the best figure, and
        // with no revert the only honest comparison is to the figure before this
        // one. It is also what makes the deltas usable — a spell dialling one
        // socket learns what that socket did, which is the single inference
        // Mastermind offers.
        //
        // **The first press has nothing to compare against**, so it holds. A
        // player has learned nothing from one press either.
        self.shift = Some(if self.pressed {
            shift_of(aligned, self.aligned)
        } else {
            Shift::Held
        });
        self.drift = Some(if self.pressed {
            shift_of(astray, self.astray)
        } else {
            Shift::Held
        });

        self.aligned = aligned;
        self.astray = astray;
        self.history.push((self.aperture, aligned, astray));
        self.pressed = true;
    }

    /// What the ward says about the aperture, without recording it.
    ///
    /// # Astray is a multiset intersection, and it was a set test
    ///
    /// **The two coincide only while no sigil may repeat**, which is why this
    /// has been correct so far and why nothing caught it. `code.contains(sigil)`
    /// asks *does the code use this sigil at all*, so four of one sigil against
    /// a code holding one of it counted **four** — four pegs on a four-socket
    /// lock, from a figure that shares one.
    ///
    /// Mastermind's second number is the size of the **multiset** intersection,
    /// less the exact matches: a sigil in the aperture may only be answered for
    /// by a sigil in the code that no other has already claimed.
    ///
    /// The existing guard cannot see the difference —
    /// `an_answer_is_always_one_figure_s_and_never_two` asserts
    /// `aligned + astray <= 4`, and the set form satisfies that by construction
    /// while being wrong about 30% of the pairs a repeating code space has.
    #[must_use]
    pub fn answer(&self) -> (u32, u32) {
        let aligned = (0..WIDTH)
            .filter(|slot| self.aperture[*slot] == self.code[*slot])
            .count();

        // Count each sigil on both sides and take the smaller — that *is* the
        // multiset intersection, and it needs no allocation at this width.
        let mut shared = 0usize;
        for sigil in 0..SIGILS.len() {
            let sent = self.aperture.iter().filter(|it| **it == sigil).count();
            let held = self.code.iter().filter(|it| **it == sigil).count();
            shared += sent.min(held);
        }

        let aligned = u32::try_from(aligned).unwrap_or(0);
        let shared = u32::try_from(shared).unwrap_or(0);
        (aligned, shared.saturating_sub(aligned))
    }

    /// Whether the seal is open.
    #[must_use]
    pub const fn broken(&self) -> bool {
        self.pressed && self.aligned as usize == WIDTH
    }

    /// What a solve is worth, before the curve is applied.
    ///
    /// **Presses, not ticks.** The yield is scaled against [`PAR`] and the tick
    /// cost is left to do the rest of the work — see `progression.toml`, which
    /// records why the first design's halving curve was withdrawn.
    #[must_use]
    pub const fn spent(&self) -> u32 {
        self.spent
    }

    /// The last press's answer.
    #[must_use]
    pub const fn last(&self) -> (u32, u32) {
        (self.aligned, self.astray)
    }

    /// Which way `aligned` moved on the last press.
    #[must_use]
    pub const fn shift(&self) -> Option<Shift> {
        self.shift
    }

    /// Which way `astray` moved on the last press — the codemaker's other peg,
    /// and the second half of everything a spell is allowed to know.
    #[must_use]
    pub const fn drift(&self) -> Option<Shift> {
        self.drift
    }

    /// What is in a socket now.
    #[must_use]
    pub fn seated(&self, socket: usize) -> Option<&'static str> {
        SIGILS.get(*self.aperture.get(socket)?).copied()
    }

    /// What a socket is holding, as an index into [`SIGILS`].
    #[must_use]
    pub fn aperture_at(&self, socket: usize) -> usize {
        self.aperture.get(socket).copied().unwrap_or(0)
    }

    // `is_settled` and `sigil_marks` are gone. The first answered *"is this
    // position correct?"* — the one question Mastermind never answers — and the
    // second was a tally a stateless ladder walked, which is the player's own
    // bookkeeping done for them. Neither is derivable from the two counts, which
    // is precisely why neither belonged (§19).

    /// The aperture, for the board to draw.
    #[must_use]
    pub const fn aperture(&self) -> &[usize; WIDTH] {
        &self.aperture
    }

    /// Whether anything has been pressed yet.
    #[must_use]
    pub const fn pressed(&self) -> bool {
        self.pressed
    }

    /// Make the aperture the answer, so the next press breaks the seal.
    ///
    /// **A tester's door, in the builds a tester runs.** Solving a ward honestly
    /// is four or five presses and sixty ticks; a See-it line for what a *broken*
    /// seal does would otherwise open with forty commands of dialling, and the
    /// thing being looked at is the spill rather than the puzzle.
    ///
    /// It sets the code rather than the aperture, so everything downstream — the
    /// answer, the yield, the roll — runs exactly as it would have. The only
    /// thing skipped is the deduction.
    #[cfg(debug_assertions)]
    pub const fn give_away(&mut self) {
        self.code = self.aperture;
    }

    /// The sheet a frontend draws.
    ///
    /// **`orbs-sim` builds it and `orbs-render` owns the type**, which is the
    /// only arrangement available — the sim depends on the render crate and
    /// never the reverse — and is the better one anyway: `Ward` keeps its `code`
    /// private, and what a board may show is decided in exactly one place rather
    /// than in each frontend's painter. `Maze::view` is the precedent.
    ///
    /// **The code is not in it**, and cannot be: every field here is one the
    /// player has already been told, so the board carries nothing the linear
    /// stream lacks (rule 2).
    #[must_use]
    pub fn view(&self) -> orbs_render::Board {
        orbs_render::Board {
            attempts: self
                .history
                .iter()
                .map(|(figure, aligned, astray)| orbs_render::Attempt {
                    figure: *figure,
                    aligned: *aligned,
                    astray: *astray,
                })
                .collect(),
            aperture: self.aperture,
            // **The words, so the sheet can be typed from.** `dial second borax`
            // names a socket and a sigil by word, and a board of bare glyphs made
            // the player count columns and guess which shape was `pewter`. They go
            // through the view for the reason the view exists at all: `orbs-render`
            // may not depend on this crate, and these are content.
            sockets: SOCKETS,
            sigils: SIGILS,
        }
    }
}

/// Where a name sits in [`SIGILS`].
///
/// **Matches the leaf**, because a resolved place arrives as its full path —
/// `dial second borax` reaches here as `/tower/lens/second`. `research::named`
/// takes the same step for the same reason, and skipping it made every `dial`
/// refuse with *"/tower/lens/second is not a socket"*, which is the parser
/// working correctly and the handler reading it wrong.
#[must_use]
pub fn sigil_of(name: &str) -> Option<usize> {
    let leaf = crate::parser::leaf(name);
    SIGILS.iter().position(|sigil| *sigil == leaf)
}

/// Where a name sits in [`SOCKETS`].
#[must_use]
pub fn socket_of(name: &str) -> Option<usize> {
    let leaf = crate::parser::leaf(name);
    SOCKETS.iter().position(|socket| *socket == leaf)
}

/// Reading a ward out into a save, and back.
///
/// # Why this lives here rather than in `crate::save`
///
/// Because every field of [`Ward`] is private, and deliberately: §19 records
/// that scrying's first design *"deleted its own puzzle"* by letting the orb
/// keep a candidate set, and `code` is the answer. Ten public getters would put
/// the answer within reach of any future system that wanted a shortcut.
///
/// One pair of methods, beside the fields they read, is both narrower and
/// stronger: adding a field to `Ward` puts the compiler error **in this file**,
/// next to the field being added, rather than in a lint that only fires if a
/// test happened to build a world with a ward open in it.
impl Ward {
    /// Everything a save needs to put this ward back.
    pub(crate) fn to_save(&self) -> crate::save::WardSave {
        crate::save::WardSave {
            code: self.code.to_vec(),
            aperture: self.aperture.to_vec(),
            aligned: self.aligned,
            astray: self.astray,
            pressed: self.pressed,
            spent: self.spent,
            shift: self.shift.map(|shift| shift.word().to_owned()),
            drift: self.drift.map(|drift| drift.drift_word().to_owned()),
            history: self
                .history
                .iter()
                .map(|(figure, aligned, astray)| crate::save::HistorySave {
                    figure: figure.to_vec(),
                    aligned: *aligned,
                    astray: *astray,
                })
                .collect(),
        }
    }

    /// Put one back.
    ///
    /// **Draws no randomness**, unlike [`Ward::new`] — the code is read rather
    /// than rolled. That is what lets a restore happen without moving
    /// `RngStream::Lens`, which every later roll in the session depends on.
    ///
    /// A hand-edited save is the one caller that can pass nonsense here, and §15
    /// accepts that: *"hand-editing a TOML file only affects the person doing
    /// it."* Lengths are clamped rather than trusted, so the worst a bad file
    /// does is make a strange ward rather than panic on the way in.
    pub(crate) fn from_save(save: &crate::save::WardSave) -> Self {
        /// A socket's worth of sigil indices, **range-checked**.
        ///
        /// `seat` guards this on the way in, so live play can never seat a sigil
        /// that does not exist — but a hand-edited file can, and both `answer`
        /// and the board index [`SIGILS`] directly. Without this, `aperture =
        /// [0, 1, 2, 99]` in a save panics on the next `probe` rather than on
        /// the way in, which is the opposite of what this function's contract
        /// promises.
        fn seated(values: &[usize]) -> [usize; WIDTH] {
            std::array::from_fn(|i| {
                values
                    .get(i)
                    .copied()
                    .filter(|sigil| *sigil < SIGILS.len())
                    .unwrap_or(i % SIGILS.len())
            })
        }

        Self {
            code: seated(&save.code),
            aperture: seated(&save.aperture),
            aligned: save.aligned,
            astray: save.astray,
            pressed: save.pressed,
            spent: save.spent,
            shift: save.shift.as_deref().and_then(Shift::named),
            drift: save.drift.as_deref().and_then(Shift::drift_named),
            history: save
                .history
                .iter()
                .map(|entry| (seated(&entry.figure), entry.aligned, entry.astray))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ward(seed: u64) -> Ward {
        Ward::new(&mut Rngs::from_seed(seed))
    }

    /// Every code a ward can be — 6⁴ = 1296.
    ///
    /// **It was 360**, because a code could not repeat a sigil. Everything the
    /// domain used to need — the exchange, the ratchet, the settle-lock — came
    /// out of that one rule, so widening it here is what those deletions rest on.
    fn every_code() -> Vec<[usize; WIDTH]> {
        let mut out = Vec::new();
        for a in 0..6 {
            for b in 0..6 {
                for c in 0..6 {
                    for d in 0..6 {
                        out.push([a, b, c, d]);
                    }
                }
            }
        }
        out
    }

    /// The shape `dev_spells.toml`'s `breaking` writes, as a function.
    ///
    /// **One socket at a time, left to right.** Dial it once and press: if
    /// `aligned` fell, that socket was already right — put it back and press
    /// again to re-sync the baseline. Otherwise keep dialling until `aligned`
    /// rises, which can only be this socket arriving.
    ///
    /// The whole of what it reads is `closer` / `further`, which is the whole of
    /// what a codemaker says. Returns the presses spent.
    fn sweep(code: [usize; WIDTH]) -> u32 {
        let mut ward = with_code(code);
        ward.press();
        for socket in 0..WIDTH {
            if ward.broken() {
                break;
            }
            // `if the prism is working` — the guard that stops a later rung
            // dialling at a ward the earlier ones already broke.
            let opening = ward.aperture_at(socket);
            ward.advance(socket);
            ward.press();
            if ward.shift() == Some(Shift::Lost) {
                ward.seat(socket, opening);
                ward.press();
                continue;
            }
            while ward.shift() != Some(Shift::Gained) {
                assert!(ward.spent() < 200, "{code:?} never came round");
                ward.advance(socket);
                ward.press();
            }
        }
        assert!(ward.broken(), "{code:?} survived the sweep");
        ward.spent()
    }

    fn with_code(code: [usize; WIDTH]) -> Ward {
        let mut ward = ward(1);
        ward.code = code;
        ward
    }

    #[test]
    fn the_writable_spell_breaks_every_one_of_the_1296_codes() {
        // **The shape a player can actually write**, run against the whole code
        // space rather than a handful of seeds. It is `dev_spells.toml`'s
        // `breaking` with the four socket rungs collapsed into a loop, and it is
        // only expressible because `dial <socket>` takes no sigil — a
        // variable-free spell cannot name the one it has not tried.
        //
        // **Termination is a proof, not a measurement.** A socket's walk leaves
        // a sigil that is not the code's and steps cyclically through six, so it
        // reaches the code's within five — and only that arrival can raise
        // `aligned`, because only that socket moved.
        let codes = every_code();
        let mut worst = 0;
        let mut total = 0u32;
        for code in &codes {
            let presses = sweep(*code);
            worst = worst.max(presses);
            total += presses;
        }

        // **The mean is the margin the whole domain rests on**, so it is pinned
        // rather than left to a comment. A deducing player averages 5.15 over
        // these same 1296 codes — if the spell ever came near that, automating
        // the lens would have deleted the puzzle, which §19 records the first
        // design doing.
        //
        // **And it is not brute force**: guessing codes at random until one fits
        // averages 648. This sits between, which is the band the domain needs.
        let count = u32::try_from(codes.len()).expect("1296 codes fit a u32");
        let mean = f64::from(total) / f64::from(count);
        assert!(
            (9.0..18.0).contains(&mean),
            "the writable spell averages {mean:.2} presses; a person averages \
             5.15 and blind guessing averages 648 — this has left the band",
        );
        assert!(worst <= 30, "worst case {worst} presses");
    }

    #[test]
    fn a_ward_may_repeat_a_sigil() {
        // **The rule that was inverted**, and the one everything else turned on.
        // 4 draws of 6 repeat something 44.9% of the time, so 200 seeds finding
        // none would mean the draw had quietly gone back to being a shuffle.
        let repeats = (0..200u64)
            .filter(|seed| {
                let ward = ward(*seed);
                let mut seen = ward.code.to_vec();
                seen.sort_unstable();
                seen.dedup();
                seen.len() < WIDTH
            })
            .count();
        assert!(repeats > 40, "only {repeats} of 200 wards repeated a sigil");
    }

    #[test]
    fn a_dial_moves_one_socket_and_only_one() {
        // **What repeats buy, and the reason the props could go.** A dial used to
        // *exchange* when the sigil it wanted was held elsewhere, so `aligned`
        // rising could not be attributed to either socket — which is what the
        // ratchet, the settle-lock and the per-socket tally were all propping up.
        for socket in 0..WIDTH {
            for sigil in 0..SIGILS.len() {
                let mut ward = with_code([0, 1, 2, 3]);
                let before = ward.aperture;
                if !ward.seat(socket, sigil) {
                    assert_eq!(ward.aperture, before, "a refused dial still moved");
                    continue;
                }
                let moved = (0..WIDTH).filter(|at| ward.aperture[*at] != before[*at]);
                assert_eq!(moved.count(), 1, "dialling {socket} moved more than it");
                assert_eq!(ward.aperture_at(socket), sigil, "the dial missed");
            }
        }
    }

    #[test]
    fn a_bare_dial_walks_all_six_and_comes_back() {
        // `advance` is the whole of what a variable-free spell can aim, so the
        // walk has to be a cycle: six turns from anywhere returns what it started
        // with, and every sigil is offered exactly once on the way.
        let mut ward = with_code([0, 1, 2, 3]);
        let mut seen = Vec::new();
        for _ in 0..SIGILS.len() {
            seen.push(ward.aperture_at(2));
            assert!(ward.advance(2), "the walk stalled");
        }
        assert_eq!(ward.aperture_at(2), 2, "six turns did not come round");
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), SIGILS.len(), "the walk skipped a sigil");
    }

    #[test]
    fn the_orb_never_deduces() {
        // **The test that would have caught the first design.** Nothing the ward
        // stores may distinguish two codes that answer every press identically —
        // if it could, the orb would be holding knowledge the player has not
        // earned, which is what deleted the puzzle last time.
        //
        // **Exhaustive, not a hand list.** The old version named the fields it
        // could think of and missed `tried` — the leak §19 records — so this
        // compares the *whole* published surface instead: every code is driven
        // through one arbitrary but identical sequence of dials, and any two that
        // answered every press alike must be indistinguishable afterwards.
        //
        // Everything a ward can be asked for goes in the tuple: the counts, both
        // deltas, whether it is broken, and the whole board a player is shown.
        // A field added to `Ward` that reaches the player and *not* this tuple is
        // the failure mode; `view` is what makes that hard, since the board is
        // the surface with the most in it.
        type Surface = (
            u32,
            u32,
            Option<Shift>,
            Option<Shift>,
            bool,
            orbs_render::Board,
        );
        let told = |code: [usize; WIDTH]| -> (Vec<(u32, u32)>, Surface) {
            let mut ward = with_code(code);
            ward.press();
            let mut answers = vec![ward.last()];
            // An arbitrary walk that touches every socket and several sigils —
            // the point is that it is the *same* walk for every code, so anything
            // that differs afterwards differs because of the code.
            for (socket, sigil) in [(0, 4), (1, 5), (2, 0), (3, 1), (0, 2), (2, 3)] {
                ward.seat(socket, sigil);
                ward.press();
                answers.push(ward.last());
            }
            let surface = (
                ward.last().0,
                ward.last().1,
                ward.shift(),
                ward.drift(),
                ward.broken(),
                ward.view(),
            );
            (answers, surface)
        };

        let mut by_answers: std::collections::HashMap<Vec<(u32, u32)>, ([usize; WIDTH], Surface)> =
            std::collections::HashMap::new();
        let mut pairs = 0u32;
        for code in every_code() {
            let (answers, surface) = told(code);
            if let Some((first, known)) = by_answers.get(&answers) {
                pairs += 1;
                assert_eq!(
                    *known, surface,
                    "{code:?} and {first:?} answer alike and the orb can still \
                     tell them apart",
                );
            } else {
                by_answers.insert(answers, (code, surface));
            }
        }
        // The test is only worth anything if such pairs exist at all — a walk
        // that happened to separate all 1296 codes would assert nothing.
        assert!(pairs > 100, "only {pairs} codes answered alike");
    }

    #[test]
    fn an_answer_is_always_one_figure_s_and_never_two() {
        // **The pair must describe the figure that was sent.** `astray` used to
        // be committed unconditionally while `aligned` only moved on a gain, so
        // a non-improving press reported the *held* figure's aligned beside the
        // *pressed* figure's astray. Over all 360 codes, 178 could report a pair
        // summing to more than four — `3 aligned, 2 astray` on a four-socket
        // lock — and a player pruning candidates on that pair can eliminate the
        // true code.
        //
        // Two invariants, and the second is what the arithmetic makes impossible.
        let mut check = with_code([0, 0, 0, 0]);
        for code in every_code() {
            let mut ward = with_code(code);
            ward.press();
            for socket in 0..WIDTH {
                for sigil in 0..SIGILS.len() {
                    let mut trial = ward.clone();
                    if !trial.seat(socket, sigil) {
                        continue;
                    }
                    let sent = trial.aperture;
                    trial.press();

                    let (aligned, astray) = trial.last();
                    let width = u32::try_from(WIDTH).expect("four sockets fit a u32");
                    assert!(
                        aligned + astray <= width,
                        "{code:?} answered {aligned} aligned and {astray} astray",
                    );

                    // ...and it is the *sent* figure's answer, recomputed here
                    // from the figure the history recorded rather than trusted.
                    check.code = code;
                    check.aperture = sent;
                    assert_eq!(
                        check.answer(),
                        (aligned, astray),
                        "{code:?} reported an answer for a figure it did not send",
                    );
                }
            }
        }
    }

    #[test]
    fn a_press_that_does_not_help_stands() {
        // **The ratchet is gone, and this is what that means.** A press used to
        // be silently reverted unless it improved, so the aperture always held
        // the best figure ever sent and `aligned` could not be observed to fall.
        // That is a codemaker undoing your move for you — and with it gone, the
        // fall is the single most informative thing the lens says: only a socket
        // that *was* right can make `aligned` drop when it alone moved.
        // The opening aperture is `[0,1,2,3]`, so this code has socket 0 already
        // right and socket 3 already wrong — one press in, `aligned` is 3.
        let mut ward = with_code([0, 1, 2, 4]);
        ward.press();
        let before = ward.last().0;
        assert_eq!(before, 3, "the fixture no longer opens at three");
        assert!(ward.seat(0, 5));
        ward.press();
        assert_eq!(ward.aperture_at(0), 5, "the press was reverted");
        assert!(ward.last().0 < before, "aligned did not fall");
        assert_eq!(ward.shift(), Some(Shift::Lost), "the fall was not reported");
    }

    #[test]
    fn greedy_always_has_an_improving_move() {
        // The termination proof, as an assertion. Put the right sigil in a wrong
        // socket and `aligned` rises — so from any non-solution a `seat` exists
        // that helps, and a blind ladder cannot stall.
        for code in every_code() {
            let mut ward = with_code(code);
            ward.press();
            if ward.broken() {
                continue;
            }
            let here = ward.last().0;
            let improves = (0..WIDTH).any(|socket| {
                (0..SIGILS.len()).any(|sigil| {
                    let mut trial = ward.clone();
                    trial.seat(socket, sigil) && trial.answer().0 > here
                })
            });
            assert!(improves, "{code:?} has no improving move from the opening");
        }
    }

    #[test]
    fn a_ladder_that_never_reads_the_answer_cannot_break_the_seal() {
        // **The prop that did not fall out of the repeats change**, recorded here
        // rather than in a comment. A ladder that walks (socket, sigil) pairs and
        // only restarts on a gain used to break all 360 codes — because the
        // settle-lock froze a socket that gained and the ratchet undid anything
        // that did not, so the walk could not destroy its own progress. Both are
        // gone, and moving without reading a per-press delta now loses ground as
        // fast as it makes it.
        //
        // This is why `readings()` still publishes the two deltas at all: without
        // them there is no writable spell, and the domain would be hand-play only.
        const BUDGET: u32 = 80;
        let codes = every_code();
        let broke = codes
            .iter()
            .filter(|code| {
                let mut ward = with_code(**code);
                ward.press();
                let mut tried: Vec<(usize, usize)> = Vec::new();
                while !ward.broken() && ward.spent() < BUDGET {
                    let next = (0..WIDTH)
                        .flat_map(|socket| (0..SIGILS.len()).map(move |sigil| (socket, sigil)))
                        .find(|pair| !tried.contains(pair));
                    let Some((socket, sigil)) = next else { break };
                    tried.push((socket, sigil));
                    if !ward.seat(socket, sigil) {
                        continue;
                    }
                    ward.press();
                    if ward.shift() == Some(Shift::Gained) {
                        tried.clear();
                    }
                }
                ward.broken()
            })
            .count();
        assert!(
            broke * 4 < codes.len(),
            "a delta-blind ladder broke {broke} of {} codes — the props it needed \
             have come back",
            codes.len(),
        );
    }

    #[test]
    fn deduction_beats_the_ladder() {
        // **The claim the whole domain rests on**, pinned. A player who presses
        // only figures consistent with every answer so far averages ~5.15 over
        // the 1296 codes; the writable spell above averages ~12. If this margin
        // ever collapses, the lens has no puzzle in it and the two channels have
        // become one.
        //
        // The player is modelled rather than played: consistent guessing is what
        // a careful person does, and it is the *upper* bound on how well the orb
        // could ever do without the deduction it deliberately does not hold.
        //
        // **One ward, re-aimed.** Building a fresh `Ward` per candidate means a
        // fresh `Rngs` per candidate, and this loop asks for millions of answers;
        // the code and the aperture are the only state `answer` reads.
        let codes = every_code();
        let mut trial = with_code([0, 0, 0, 0]);
        let mut player = 0u32;
        for code in &codes {
            let mut live = codes.clone();
            let mut presses = 0;
            // **The first press is the aperture the ward opens on**, because
            // that is what a player who types `probe` sends. Opening on
            // `live[0]` instead — `[0,0,0,0]` — costs them 0.6 of a press, and
            // the margin should be measured against the play the room actually
            // affords.
            let mut guess = [0, 1, 2, 3];
            loop {
                presses += 1;
                trial.code = *code;
                trial.aperture = guess;
                let (aligned, astray) = trial.answer();
                if aligned as usize == WIDTH {
                    break;
                }
                live.retain(|candidate| {
                    trial.code = *candidate;
                    trial.answer() == (aligned, astray)
                });
                guess = live[0];
            }
            player += presses;
        }
        let mean = f64::from(player) / f64::from(u32::try_from(codes.len()).unwrap_or(u32::MAX));
        // 5.15 measured; the band leaves room for the opening aperture changing
        // and none for the margin closing. Knuth's minimax play is 4.48 on the
        // standard board, so a person cannot get far under this either.
        assert!(
            mean < 6.0,
            "a deducing player now needs {mean:.2} presses, so the spell is competitive",
        );
    }

    #[test]
    fn astray_counts_a_sigil_once_per_copy_the_code_actually_holds() {
        // **Mastermind's second number is a multiset intersection**, and this was
        // a set test — `code.contains(sigil)` — which is the same thing only
        // while no sigil repeats. It is therefore correct today and wrong the
        // moment the code space allows a repeat, which is a change under
        // consideration; pinned now so that change cannot land quietly.
        //
        // Hand-checked. Each row is (aperture, code) -> (aligned, astray).
        for (sent, code, want) in [
            // The everyday case, unaffected either way.
            ([0, 1, 2, 3], [0, 1, 2, 3], (4, 0)),
            ([0, 1, 2, 3], [3, 2, 1, 0], (0, 4)),
            ([0, 1, 2, 3], [0, 1, 3, 2], (2, 2)),
            // **Four of one sigil against a code holding exactly one.** The set
            // form answered `(1, 3)` — three pegs claiming a sigil the code has
            // one of.
            ([0, 0, 0, 0], [0, 1, 2, 3], (1, 0)),
            // Two sent, one held: one is answered for, the other is not.
            ([0, 0, 1, 2], [0, 3, 4, 5], (1, 0)),
            // Two sent and two held, neither in place.
            ([0, 0, 4, 5], [1, 2, 0, 0], (0, 2)),
            // ...and the same pair with one of them landing, which is the row
            // that caught a wrong hand-count writing this table.
            ([0, 0, 4, 5], [1, 0, 0, 2], (1, 1)),
            // One sent against two held — the aperture's count is the cap too.
            ([0, 4, 5, 1], [0, 0, 2, 3], (1, 0)),
        ] {
            let mut ward = with_code(code);
            ward.aperture = sent;
            assert_eq!(
                ward.answer(),
                want,
                "aperture {sent:?} against code {code:?}",
            );
        }
    }

    #[test]
    fn seating_a_sigil_already_in_play_duplicates_it() {
        // The inverse of what this asserted: it used to *exchange* the two
        // sockets, because a figure could not repeat a sigil. It can now, and the
        // far socket is left alone (§19).
        let mut ward = with_code([0, 1, 2, 3]);
        assert_eq!(ward.seated(0), Some("nitre"));
        assert!(ward.seat(0, 1), "nothing moved");
        assert_eq!(ward.seated(0), Some("alum"));
        assert_eq!(ward.seated(1), Some("alum"), "the far socket was dragged");
    }

    #[test]
    fn seating_what_is_already_there_moves_nothing() {
        let mut ward = with_code([0, 1, 2, 3]);
        assert!(!ward.seat(0, 0), "a no-op counted as a change");
    }
}
