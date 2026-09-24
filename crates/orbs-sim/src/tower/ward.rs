//! The seal on a far wizard's orb, and the two ways a player breaks it.
//!
//! DESIGN.md §10 gives scrying *deduction*, and this is the shape it took: four
//! sigils drawn from six, **repeats allowed**, 1296 codes. You press figures
//! against the ward and read how it answers. It is Mastermind, and §19 records
//! the version that was Mastermind-shaped without being Mastermind.
//!
//! The orb keeps no candidate set and deduces nothing — the load-bearing rule of
//! the domain, broken by two designs already (§19). Tracking consistent codes
//! solves in ~4.2 presses, so the orb would have done the puzzle; saying
//! `settled` per socket is the same leak more quietly, since no codemaker answers
//! about one position. So this stores only what a player could keep on paper: the
//! code, the aperture, the last answer, and which way each half moved.
//!
//! Two channels onto one ward — the player reads `aligned`/`astray` as numbers
//! and deduces (~5.15 presses); a spell reads only which way they moved and
//! walks one socket at a time (~11.93). The spell's channel is strictly *less*,
//! which is what lets §8's variable-less language automate a puzzle it cannot
//! solve.
//!
//! The sweep terminates, and that is why the domain works: move one socket and
//! `aligned` can only change because of it, so stepping cyclically through six
//! reaches the code's sigil within five and says so. A rule, not a search —
//! Trémaux's role for the maze. `dev_spells.toml`'s `breaking` is the shape a
//! player writes.

use bevy_ecs::prelude::*;
use rand::Rng as _;

use crate::rng::{RngStream, Rngs};

/// The sigils a ward is drawn from.
///
/// Six real alchemical substances, swept before they were authored: a first set
/// scored `crown` against `cron` (a `bind` synonym) at 800, above the 750
/// `tests/naming.rs` tolerates. Sigils are `Place` nouns, nameable from every
/// room, so they sit fully in the parser's way.
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
/// Measured, not chosen: consistent play averages 5.15 over all 1296 codes,
/// worst case 9, so five put half of competent hand play in `yield_of`'s reduced
/// tier — a penalty for playing well. It was five at 360 codes.
pub const PAR: u32 = 6;

/// What one press costs the tower, in ticks.
///
/// Nought: a press is instant and takes no slot. It was twelve, and that
/// scarcity is withdrawn — a press is `dial`'s equal, so the lens competes with
/// the laboratory for nothing. Kept as a named nought because the domain's rates
/// were derived against it and pricing a press again should be one line.
pub const PRESS_TICKS: u64 = 0;

/// Whether the last press helped, held, or hurt.
///
/// The spell's whole channel, and a tally rather than an inference: did
/// `aligned` go up? The world knows that without reasoning about what it
/// implies, which keeps [`Ward`] honest and still gives §8's stateless language
/// something to hill-climb on.
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
    /// `gained`, `held` and `lost` all leaked: a reading is a `NounKind::Sense`,
    /// which `NounKind::Any` reaches, so `purge grind` fuzzy-matched `gained` at
    /// full confidence. Ordinary English participles sit in the way of half the
    /// words a player might mistype. Comparatives read better anyway.
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
    /// A second set, because one word cannot mean two things: `closer` is about
    /// placement, this is about *membership*. A spell that could not tell them
    /// apart could not tell *swap two of these* from *none of these belong*.
    ///
    /// Swept, not judged. `warmer`/`even`/`cooler` scores 667 and 600 against
    /// `closer` and `level` — two readings colliding inside one domain;
    /// `fuller`/`steady`/`thinner` hits `filter` and `study` at 667 each;
    /// `thicker`/`thinner` score 715 against each other.
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
    /// Two readers, not one: a single `named` over all six would round-trip
    /// `closer` into `drift` and `richer` into `shift`, and the save would read
    /// back as a ward saying something it never said.
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
/// Against the previous press, never a high-water mark. The ratchet used to make
/// the aperture always *be* the best figure; with the revert gone, the only
/// honest baseline is the figure before this one.
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

// `marks`, `settled`, `loose` and `untried` were consts here and are gone with
// the props they served (§19): each was a verdict on a *position*, which is the
// one thing a codemaker may never answer.

/// Every word a spell may ask a lens noun for.
///
/// Registered unconditionally in `scene_at`, as the maze's are: a spell's `if`
/// resolves at cast, so a vocabulary that appeared with the state would leave
/// half a solver's conditions unresolvable when they compile.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    // Six words, all deltas. The counts are still shown to the *player* on the
    // sheet — that is the game — but a spell only gets movement (§19).
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
    /// They were once one field short of a pair — `astray` committed always,
    /// `aligned` only on a gain — so a non-improving press could report a sum
    /// above four on a four-socket lock. Also the baseline for the two deltas.
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
    /// Mastermind's other number: a figure can be wholly misplaced and still be
    /// made of the right sigils, and without this a spell cannot tell *swap two
    /// of these* from *none of these belong*.
    drift: Option<Shift>,
    /// Every press and its answer, oldest first — the board's whole content.
    ///
    /// History, and only history: showing five presses side by side is what
    /// makes deduction possible without a notepad. Nothing is derived from it —
    /// `press` reads the last answer from [`aligned`](Self::aligned).
    history: Vec<([usize; WIDTH], u32, u32)>,
}

impl Ward {
    /// Draw a ward from the lens's own stream.
    ///
    /// Built in one go, like the maze: a ward that filled in as it was probed
    /// would draw from the stream at a rate depending on tick consumption, which
    /// is the hazard `heat.rs` records.
    #[must_use]
    pub fn new(rngs: &mut Rngs) -> Self {
        let rng = rngs.stream(RngStream::Lens);
        // Four independent draws, repeats and all — 1296 codes. Distinct sigils
        // meant a dial had to *exchange* two sockets, which makes a rising
        // `aligned` unattributable, which is why the ward once needed a ratchet,
        // a settle-lock and a per-socket tally. Repeats delete all three (§19).
        //
        // Still exactly four draws: replay needs the count not to vary with the
        // values, which is why the old form used Fisher-Yates over rejection.
        let mut code = [0usize; WIDTH];
        for slot in &mut code {
            *slot = rng.random_range(0..SIGILS.len());
        }

        // The aperture opens on the first four sigils, not on the code: a random
        // opening would make one reading in 1296 already solved, and a fixed one
        // means the first press teaches the same lesson every time. Distinct
        // even though the code need not be — a repeat here teaches the wrong
        // first lesson.
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
    /// A cursor the ward no longer keeps. Taking the first sigil untried *since
    /// the last gain* meant storing a per-socket list, and that list was
    /// readable as `untried` — the orb doing the player's bookkeeping (§19).
    /// Cyclic-next needs no cursor: the sigil a socket holds is the cursor, and
    /// it is on the board. Six dials return it, which is what makes a blind
    /// sweep exhaustive without remembering anything.
    ///
    /// Always moves, so always `true`.
    pub const fn advance(&mut self, socket: usize) -> bool {
        if socket >= WIDTH {
            return false;
        }
        let next = (self.aperture[socket] + 1) % SIGILS.len();
        self.seat(socket, next)
    }

    // `replenish`, `next_untried` and `best` are gone with the state they served.
    // `replenish` gave candidates back so an `untried`-guarded ladder would not
    // stall; a cyclic `advance` cannot exhaust, so termination is the dial's
    // property now. `best` was honest only because of the ratchet — without it a
    // high-water mark describes a figure the player no longer has, which is
    // `panel.rs`'s "reporting the guess rather than the reading" failure. The
    // meter reads [`last`](Self::last) instead.

    /// Put `sigil` in `socket`. Returns whether anything moved.
    ///
    /// One socket, always, which is the whole of what allowing repeats buys.
    /// This used to *exchange*, and a press after a two-socket move could not
    /// say which socket the answer moved for — the ambiguity all the ward's old
    /// scaffolding worked around. Now `aligned` rising is entailed to be that
    /// socket's doing, and the player makes that inference themselves, which is
    /// the only inference Mastermind offers (§19).
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
    /// It no longer ratchets. Snapping back to the best figure on a non-gaining
    /// press was the orb remembering which of the *player's* guesses was best,
    /// which is the whole labour of Mastermind. What you dialled stands, so
    /// `aligned` can fall — and a fallen `aligned` is real information, because
    /// dialling off a correct socket is the only thing that causes it.
    ///
    /// Nothing settles here either: the orb makes no claim about any socket
    /// (§19).
    pub fn press(&mut self) {
        let (aligned, astray) = self.answer();
        self.spent = self.spent.saturating_add(1);

        // Both deltas are against the previous press, not a high-water mark:
        // with no revert that is the only honest baseline, and it is what makes
        // them usable — a spell dialling one socket learns what that socket did.
        // The first press has nothing to compare against, so it holds; a player
        // has learned nothing from one press either.
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
    /// `astray` is a multiset intersection and was once a set test — the two
    /// coincide only while no sigil may repeat, which is why nothing caught it.
    /// `code.contains(sigil)` counted four of one sigil against a code holding
    /// one of it as four pegs. A sigil may only be answered for by a code sigil
    /// no other has claimed.
    ///
    /// `an_answer_is_always_one_figure_s_and_never_two` cannot see the
    /// difference: the set form satisfies `aligned + astray <= 4` by
    /// construction while being wrong about 30% of a repeating space's pairs.
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
    /// Presses, not ticks: the yield scales against [`PAR`] and the tick cost
    /// does the rest. `progression.toml` records why the first design's halving
    /// curve was withdrawn.
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

    // `is_settled` and `sigil_marks` are gone: the first answered "is this
    // position correct?", the one question Mastermind never answers, and the
    // second was the player's bookkeeping done for them. Neither is derivable
    // from the two counts, which is why neither belonged (§19).

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
    /// A tester's door, in the builds a tester runs: a See-it line for what a
    /// *broken* seal does would otherwise open with forty commands of dialling.
    /// It sets the code rather than the aperture, so the answer, the yield and
    /// the roll all run as they would have — only the deduction is skipped.
    #[cfg(debug_assertions)]
    pub const fn give_away(&mut self) {
        self.code = self.aperture;
    }

    /// The sheet a frontend draws.
    ///
    /// `orbs-sim` builds it and `orbs-render` owns the type — the only
    /// arrangement available, and the better one: `code` stays private and what
    /// a board may show is decided once rather than per painter. `Maze::view` is
    /// the precedent.
    ///
    /// The code is not in it and cannot be: every field is one the player has
    /// already been told, so the board carries nothing the linear stream lacks
    /// (rule 2).
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
            // The words, so the sheet can be typed from: `dial second borax`
            // names both by word, and a board of bare glyphs made the player
            // count columns. Through the view because `orbs-render` may not
            // depend on this crate.
            sockets: SOCKETS,
            sigils: SIGILS,
        }
    }
}

/// Where a name sits in [`SIGILS`].
///
/// Matches the leaf, because a resolved place arrives as its full path:
/// `dial second borax` reaches here as `/tower/lens/second`. Skipping the step
/// made every `dial` refuse with *"/tower/lens/second is not a socket"*.
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
/// Here rather than in `crate::save` because every field of [`Ward`] is private
/// deliberately — `code` is the answer, and ten public getters would put it
/// within reach of any system wanting a shortcut. One pair of methods beside the
/// fields also puts the compiler error in this file when a field is added.
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
    /// Draws no randomness, unlike [`Ward::new`]: the code is read, not rolled,
    /// so a restore does not move `RngStream::Lens`. Lengths are clamped rather
    /// than trusted, so a hand-edited save makes a strange ward rather than a
    /// panic (§15).
    pub(crate) fn from_save(save: &crate::save::WardSave) -> Self {
        /// A socket's worth of sigil indices, range-checked.
        ///
        /// `seat` guards live play, but a hand-edited file can hold
        /// `aperture = [0, 1, 2, 99]`, and both `answer` and the board index
        /// [`SIGILS`] directly — so without this the panic lands on the next
        /// `probe` instead of here.
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
    /// It was 360, because a code could not repeat a sigil. The exchange, the
    /// ratchet and the settle-lock all came out of that one rule.
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
    /// One socket at a time, left to right. Dial and press: if `aligned` fell,
    /// that socket was already right — put it back and press again to re-sync
    /// the baseline. Otherwise keep dialling until `aligned` rises, which can
    /// only be this socket arriving. Reads `closer`/`further` and nothing else.
    /// Returns the presses spent.
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
        // The shape a player can write, over the whole code space rather than a
        // handful of seeds. Only expressible because `dial <socket>` takes no
        // sigil — a variable-free spell cannot name one it has not tried.
        //
        // Termination is a proof, not a measurement: a socket's cyclic walk
        // reaches the code's sigil within five, and only that arrival can raise
        // `aligned`, because only that socket moved.
        let codes = every_code();
        let mut worst = 0;
        let mut total = 0u32;
        for code in &codes {
            let presses = sweep(*code);
            worst = worst.max(presses);
            total += presses;
        }

        // The mean is the margin the domain rests on, so it is pinned. A
        // deducing player averages 5.15 over these same codes and blind guessing
        // averages 648; the spell must sit between, or automating the lens has
        // deleted the puzzle (§19).
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
        // What repeats buy. A dial used to *exchange* when the sigil it wanted
        // was held elsewhere, so a rising `aligned` could not be attributed to
        // either socket — the ambiguity the three old props existed for.
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
        // The test that would have caught the first design: nothing the ward
        // stores may distinguish two codes that answer every press identically.
        //
        // Exhaustive, not a hand list — the old version named the fields it
        // thought of and missed `tried` (§19). Every code is driven through one
        // identical walk and the *whole* published surface is compared, so the
        // failure mode is a new field that reaches the player but not this
        // tuple. `view` makes that hard, since the board holds the most.
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
        // The pair must describe the figure that was sent. `astray` used to be
        // committed always while `aligned` moved only on a gain, so 178 of 360
        // codes could report a pair summing above four — and a player pruning
        // candidates on that pair can eliminate the true code. Two invariants;
        // the second is the one the arithmetic alone cannot make impossible.
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
        // The ratchet is gone, and this is what that means. A reverted press is a
        // codemaker undoing your move; without it, a falling `aligned` is the
        // most informative thing the lens says, because only a socket that *was*
        // right can cause it. The opening aperture is `[0,1,2,3]`, so this code
        // has socket 0 right and socket 3 wrong — one press in, `aligned` is 3.
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
        // A ladder walking (socket, sigil) pairs and restarting on a gain used to
        // break all 360 codes, because the settle-lock and the ratchet meant the
        // walk could not destroy its own progress. Both are gone, so moving
        // without reading a delta loses ground as fast as it gains it — which is
        // why `readings()` publishes the two deltas at all.
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
        // The claim the domain rests on, pinned: consistent play averages ~5.15
        // and the writable spell ~12. If the margin collapses, the two channels
        // have become one and the lens has no puzzle in it.
        //
        // The player is modelled, not played — consistent guessing is the upper
        // bound on how well the orb could do without the deduction it withholds.
        //
        // One ward, re-aimed: a fresh `Ward` per candidate means a fresh `Rngs`
        // per candidate, and this loop asks for millions of answers.
        let codes = every_code();
        let mut trial = with_code([0, 0, 0, 0]);
        let mut player = 0u32;
        for code in &codes {
            let mut live = codes.clone();
            let mut presses = 0;
            // The first press is the aperture the ward opens on, because that is
            // what a player who types `probe` sends. `live[0]` — `[0,0,0,0]` —
            // costs them 0.6 of a press, and the margin should be measured
            // against the play the room affords.
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
        // Mastermind's second number is a multiset intersection. This was a set
        // test — `code.contains(sigil)` — which only coincides while no sigil
        // repeats, so pinned here to stop that regressing quietly.
        //
        // Hand-checked. Each row is (aperture, code) -> (aligned, astray).
        for (sent, code, want) in [
            // The everyday case, unaffected either way.
            ([0, 1, 2, 3], [0, 1, 2, 3], (4, 0)),
            ([0, 1, 2, 3], [3, 2, 1, 0], (0, 4)),
            ([0, 1, 2, 3], [0, 1, 3, 2], (2, 2)),
            // Four of one sigil against a code holding one: the set form
            // answered `(1, 3)`.
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
