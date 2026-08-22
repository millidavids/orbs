//! The seal on a far wizard's orb, and the two ways a player breaks it.
//!
//! DESIGN.md §10 gives scrying *deduction*, and this is the shape it took: four
//! sigils drawn from six, no repeats, 360 codes. You press figures against the
//! ward and read how it answers.
//!
//! # The orb keeps no candidate set and deduces nothing
//!
//! **This is the load-bearing rule of the whole domain**, and the design it
//! replaced broke it. A first pass had the sim track which codes were still
//! consistent and publish each sigil's standing; measured over all 360 codes,
//! "press anything still consistent" solves in **4.24 presses** and the best play
//! there is manages **4.08** — so an orb that does the bookkeeping has done the
//! entire puzzle, and there is nothing left for the player to be good at.
//! Mastermind's difficulty *is* the bookkeeping.
//!
//! So what is stored here is only what a player could keep on paper: the code,
//! the aperture, the last answer, a tally of presses per socket and sigil, and
//! whether a socket's last change helped. Nothing is inferred from any of it.
//!
//! # Two channels onto one ward
//!
//! | | The player | A spell |
//! |---|---|---|
//! | Reads | `aligned` and `astray` | `gained`/`held`/`lost`, `marks`, `settled` |
//! | Method | deduction | greedy hill-climbing |
//! | Presses | ~4.1 | ~22.8 |
//!
//! Both use the same two verbs. What differs is what they read, and that is what
//! lets §8's language automate a puzzle it cannot possibly solve: the language
//! has no variables and no accumulator (`parser::question`), so a spell can only
//! act on what the world has written down — and *did this help* is a fact the
//! world can write down without deducing anything.
//!
//! **Greedy terminates, and the proof is why the domain works.** Put the right
//! sigil into a wrong socket and `aligned` rises by at least one, so from any
//! non-solution an improving move exists. Measured over all 360 codes it never
//! failed, worst case 51 presses. A ladder is therefore a *rule*, not a search —
//! exactly what Trémaux is for the archive's maze.

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
/// **Five, and measured rather than chosen.** Consistent play averages 4.14 with
/// a worst case of 6 over all 360 codes, so par is reachable by playing well and
/// not by luck.
pub const PAR: u32 = 5;

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

    /// Every word, for the scene to register unconditionally.
    pub const ALL: [&'static str; 3] = ["closer", "level", "further"];
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
/// Presses that have involved a sigil or a socket.
pub const MARKS: &str = "marks";
/// A socket whose last change gained.
pub const SETTLED: &str = "settled";
/// A socket whose last change did not.
///
/// **Not `open`**, which is `peruse`'s own shell synonym — a reading sharing a
/// verb's word would resolve against `NounKind::Any` from anywhere in the tower
/// and make `purge open` a coin toss.
pub const LOOSE: &str = "loose";
/// Sigils a socket has not been set to since the last gain.
///
/// Counted, so `if the first has 1 or more untried` works with the grammar the
/// rest of the language already has. It is the guard a four-rung ladder needs —
/// see [`Ward::untried`] for what the twenty-four rung version got wrong.
pub const UNTRIED: &str = "untried";

/// Every word a spell may ask a lens noun for.
///
/// Registered unconditionally in `scene_at`, exactly as the maze's are: a
/// spell's `if` resolves **at cast**, when no socket has settled and no sigil has
/// a mark, so a vocabulary that appeared with the state would make half the
/// conditions in a solver unresolvable at the moment they are compiled.
#[must_use]
pub fn readings() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = COUNTS.to_vec();
    out.extend(Shift::ALL);
    out.push(MARKS);
    out.push(SETTLED);
    out.push(LOOSE);
    out.push(UNTRIED);
    out
}

/// A far orb's seal, and everything the player has learned about it.
#[derive(Component, Debug, Clone)]
pub struct Ward {
    /// The hidden figure, as indices into [`SIGILS`]. Never published.
    code: [usize; WIDTH],
    /// What the next press will send.
    aperture: [usize; WIDTH],
    /// The best figure pressed so far — what a press that does not gain reverts to.
    ///
    /// See [`press`](Self::press) for the ratchet this is half of.
    held: [usize; WIDTH],
    /// The last press's answer — **both halves of one figure's**.
    ///
    /// These were one field short of a pair: `astray` was committed
    /// unconditionally while `aligned` only moved on a gain, so a non-improving
    /// press reported the *held* figure's aligned beside the *pressed* figure's
    /// astray. Modelled over all 360 codes, 178 of them could report a pair
    /// summing to more than four — `3 aligned, 2 astray` on a four-socket lock —
    /// and a deducing player pruning on that pair can eliminate the true code.
    /// The ratchet's high-water mark is [`best`](Self::best) and is a different
    /// number.
    aligned: u32,
    astray: u32,
    /// The most sockets any press has aligned — what the ratchet compares to.
    ///
    /// Separate from [`aligned`](Self::aligned) because they answer different
    /// questions: *what did that press say* and *how far have I got*. Conflating
    /// them is what made the pair above impossible.
    best: u32,
    /// Whether anything has been pressed yet.
    pressed: bool,
    /// How many presses this reading has taken.
    spent: u32,
    /// Whether the last press helped.
    shift: Option<Shift>,
    /// Presses each sigil has taken part in.
    sigil_marks: [u32; SIGILS.len()],
    /// Presses each socket has been changed for.
    socket_marks: [u32; WIDTH],
    /// Which sigils each socket has been set to **since the last gain**.
    ///
    /// # The state a stateless language could not keep
    ///
    /// A ladder's only question is *what have I not tried here yet*, and before
    /// this the answer had to be inferred from [`socket_marks`](Self::socket_marks)
    /// — six rungs per socket, each guarded on a different mark count, using the
    /// count as an index into [`SIGILS`]. Twenty-four rungs for four sockets, and
    /// wrong twice over:
    ///
    /// - **A mark is not an index.** `seat` marks a socket for every dial *aimed*
    ///   at it, including one that moved nothing, and a swap marks the socket at
    ///   the far end too. So the count outruns the sigils actually tried and a
    ///   socket exhausts its rungs with candidates left.
    /// - **Nothing reset it.** Once all four counts passed the last rung the
    ///   ladder fired nothing at all, and the loop pressed an unchanged aperture
    ///   for ever — holding the tower's one production slot and earning nothing.
    ///   Measured: seed 1 stopped earning at tick 4800 and never resumed.
    ///
    /// **Cleared by a press that gains**, which is the other half. A gain moves
    /// the baseline the ratchet keeps, so every sigil is worth trying again from
    /// the new figure — the reference ladder in `tests/ward.rs` did exactly this
    /// with `tried.clear()`, and that reset is why it solved where the spell it
    /// was supposed to be modelling did not.
    tried: [[bool; SIGILS.len()]; WIDTH],
    /// Sockets whose last change gained, and which are therefore held.
    ///
    /// **A lock, not a note** — and that is what makes the domain automatable.
    /// §8's language has no variables, so a ladder cannot remember what a socket
    /// held before a bad press and cannot undo one. The world holds the undo
    /// instead: a socket that gained is refused to any later [`seat`](Self::seat),
    /// so a blind walk can only ever move forward and terminates in at most
    /// `WIDTH × (SIGILS - 1) + 1` presses.
    ///
    /// This is the maze's trick in a second shape. Trémaux needs no memory
    /// beyond marks in the passages; a ward needs none beyond a latch on the
    /// sockets it has already got right.
    settled: [bool; WIDTH],
    /// Sockets changed since the last press, so a gain settles the right ones.
    touched: [bool; WIDTH],
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
        // A partial Fisher-Yates over the six sigils, taking four. Sampling with
        // rejection would draw a variable number of times from the stream, which
        // is the same replay hazard in a different shape.
        let mut pool: Vec<usize> = (0..SIGILS.len()).collect();
        let mut code = [0usize; WIDTH];
        for slot in &mut code {
            let pick = rng.random_range(0..pool.len());
            *slot = pool.swap_remove(pick);
        }

        // **The aperture opens on the first four sigils, not on the code.** A
        // random opening would make one reading in 360 already solved, and a
        // fixed one makes the first press mean the same thing every time —
        // which is what a player learning the domain needs.
        let aperture = [0, 1, 2, 3];
        Self {
            code,
            aperture,
            held: aperture,
            aligned: 0,
            astray: 0,
            best: 0,
            pressed: false,
            spent: 0,
            shift: None,
            sigil_marks: [0; SIGILS.len()],
            socket_marks: [0; WIDTH],
            tried: [[false; SIGILS.len()]; WIDTH],
            settled: [false; WIDTH],
            touched: [false; WIDTH],
            history: Vec::new(),
        }
    }

    /// Sigils this socket has not been set to since the last gain.
    ///
    /// What a ladder's guard asks, and the reading behind [`UNTRIED`]. A settled
    /// socket reports none: it refuses a dial, so there is nothing left to try
    /// there and a rung that fired on it would spin.
    #[must_use]
    pub fn untried(&self, socket: usize) -> u32 {
        if self.settled.get(socket).copied().unwrap_or(false) {
            return 0;
        }
        let Some(tried) = self.tried.get(socket) else {
            return 0;
        };
        u32::try_from(tried.iter().filter(|had| !**had).count()).unwrap_or(0)
    }

    /// Set `socket` to the first sigil it has not tried since the last gain.
    ///
    /// **The whole point of the added state**, and what turns a twenty-four rung
    /// ladder into four: a script can say *try something else here* without being
    /// able to name which, which is the one sentence the variable-free language
    /// could not otherwise form.
    ///
    /// Returns whether anything moved, exactly as [`seat`](Self::seat) does — and
    /// it *is* a `seat`, so the ratchet, the settle-lock and the exchange rule all
    /// apply unchanged. A socket with nothing left refuses.
    pub fn advance(&mut self, socket: usize) -> bool {
        let Some(sigil) = self.next_untried(socket) else {
            return false;
        };
        self.seat(socket, sigil)
    }

    /// Give every unsettled socket its candidates back when none has any left.
    ///
    /// **The termination guarantee, and without it "it can be scripted" is a
    /// measurement rather than a claim.** A four-rung ladder falls through to
    /// `wait` when every socket reports no `untried`, and then presses an
    /// unchanged aperture for ever — holding the tower's one production slot and
    /// earning nothing, which is worse than refusing. Six seeds over 14400 ticks
    /// never reached it; *never observed* is not the same as *cannot happen*.
    ///
    /// It is not a reset of the puzzle. A settled socket keeps its lock, so the
    /// state that makes the walk monotone — `aligned` only rises, settled sockets
    /// only accumulate — is untouched. What comes back is only the ladder's list
    /// of things left to try, which is exactly what a player with squared paper
    /// would do on running out of ideas: start round again from what still moves.
    fn replenish(&mut self) {
        if self.broken() {
            return;
        }
        let stuck = (0..WIDTH).all(|socket| self.untried(socket) == 0);
        if !stuck {
            return;
        }
        for socket in 0..WIDTH {
            if !self.settled[socket] {
                self.tried[socket] = [false; SIGILS.len()];
            }
        }
    }

    /// Which sigil [`advance`](Self::advance) would take, without taking it.
    ///
    /// So the sentence can name what was tried. A settled socket has none, which
    /// is what makes `advance` refuse there rather than spin.
    #[must_use]
    pub fn next_untried(&self, socket: usize) -> Option<usize> {
        if self.settled.get(socket).copied().unwrap_or(false) {
            return None;
        }
        self.tried.get(socket)?.iter().position(|had| !*had)
    }

    /// The most sockets any press has aligned — progress, not the last answer.
    ///
    /// What the panel's meter fills against, and what the ratchet compares to.
    /// [`last`](Self::last) is the other number and they are not
    /// interchangeable.
    #[must_use]
    pub const fn best(&self) -> u32 {
        self.best
    }

    /// Put `sigil` in `socket`, exchanging if it is already in play.
    ///
    /// **Exchanging rather than replacing is what keeps the two channels
    /// honest.** With no repeats a socket cannot simply take a sigil another
    /// holds, so a `seat` changes one socket or two — and that ambiguity is what
    /// stops the `gained`/`lost` channel becoming a per-socket oracle. Made
    /// unambiguous, hill-climbing would tell a player exactly which socket was
    /// right and the deduction channel would be pointless.
    ///
    /// Returns whether anything moved. **A settled socket refuses**, on either
    /// end of the exchange — see [`is_settled`](Self::is_settled) for why that
    /// lock is what makes a blind ladder terminate.
    pub fn seat(&mut self, socket: usize, sigil: usize) -> bool {
        if socket >= WIDTH || sigil >= SIGILS.len() {
            return false;
        }
        // **A settled socket is not marked**, because it is not being tried: the
        // latch means *stop looking here*, and a tally that kept climbing on a
        // socket nobody may touch would tell a ladder it had made progress.
        if self.settled[socket] {
            return false;
        }

        // **A dial aimed at a socket marks it, even when nothing moves**, and
        // this is what makes a stateless ladder able to walk.
        //
        // The tally is *attempts on this socket*, not *changes to it*. A ladder
        // has no variables, so its only way to try a different sigil next lap is
        // a guard that has moved — and `dial first nitre` when the first already
        // holds nitre would otherwise leave every reading exactly as it was, so
        // the same rung fires for ever and the spell spins without pressing.
        // That is what the first `breaking` did: one press, then nothing, for six
        // hundred ticks.
        self.socket_marks[socket] = self.socket_marks[socket].saturating_add(1);
        // **Recorded before the early returns, and for the same reason the mark
        // is.** A dial at a sigil the socket already holds has *tried* it — that
        // is precisely the case where nothing moves, and a ladder that did not
        // record it would aim there again next lap and never advance.
        self.tried[socket][sigil] = true;

        if self.aperture[socket] == sigil {
            return false;
        }
        if let Some(other) = self.aperture.iter().position(|held| *held == sigil) {
            if self.settled[other] {
                return false;
            }
            self.aperture.swap(other, socket);
            self.touched[other] = true;
            self.socket_marks[other] = self.socket_marks[other].saturating_add(1);
        } else {
            self.aperture[socket] = sigil;
        }
        self.touched[socket] = true;
        true
    }

    /// How many dials have been aimed at a socket.
    #[must_use]
    pub fn socket_marks(&self, socket: usize) -> u32 {
        self.socket_marks.get(socket).copied().unwrap_or(0)
    }

    /// Send the aperture against the ward and record what came back.
    ///
    /// # The ward ratchets: only a press that gains is kept
    ///
    /// A press that holds or loses is answered honestly — `aligned` and `astray`
    /// are reported for the figure that was actually sent — and then **the
    /// aperture snaps back** to the best figure so far.
    ///
    /// **This is the undo §8's language cannot express.** A ladder has no
    /// variables, so it cannot remember what a socket held before a bad press;
    /// without a ratchet a blind walk destroys the sockets it has already got
    /// right and never converges. The world holds the undo instead, exactly as
    /// the maze holds the search in its marks.
    ///
    /// **It never blocks correct play**, which is what makes it fair rather than
    /// merely convenient: putting the right sigil into its own socket raises
    /// `aligned` by at least one, so every move a deducing player wants to make
    /// is a move the ward keeps. What it costs is only the ability to hold a
    /// *worse* figure, which nobody wants to do.
    ///
    /// And it makes a wrong press cost ticks and nothing else — §11.5's *"cost
    /// is the resource, never progress"*.
    pub fn press(&mut self) {
        let (aligned, astray) = self.answer();
        self.spent = self.spent.saturating_add(1);

        // **Both halves of the pressed figure's answer, always.** What the ward
        // *said* is a fact about the figure that was sent; how far the player has
        // got is `best`. See the fields for the impossible pair conflating them
        // produced.
        self.aligned = aligned;
        self.astray = astray;

        // **Recorded before the ratchet**, so the sheet shows the figure that
        // was actually sent rather than the one it snapped back to. A board that
        // logged the reverted aperture would say a press answered something the
        // press never asked.
        self.history.push((self.aperture, aligned, astray));
        self.shift = Some(match aligned.cmp(&self.best) {
            std::cmp::Ordering::Greater => Shift::Gained,
            std::cmp::Ordering::Equal => Shift::Held,
            std::cmp::Ordering::Less => Shift::Lost,
        });

        // **Sigils are tallied from the figure that was sent**, before the revert
        // rewrites the aperture. Counting after it credited the *reverted*
        // figure's sigils a second time and left the pressed figure's untouched,
        // which is the same class of mistake as the pair above: a field whose doc
        // says *"presses each sigil has taken part in"* describing a press that
        // never happened.
        for sigil in self.aperture {
            self.sigil_marks[sigil] = self.sigil_marks[sigil].saturating_add(1);
        }

        if self.shift == Some(Shift::Gained) || !self.pressed {
            self.best = aligned;
            self.held = self.aperture;
            // **Only the sockets that moved forget what they have tried.**
            //
            // A gain moves the baseline the ratchet keeps, so a sigil rejected
            // against the old figure says nothing about the new one — that is the
            // argument for clearing, and clearing *everything* was the first
            // version. Measured, it cost about a quarter of the faucet's rate: a
            // ladder that forgets four sockets on every gain spends its next laps
            // re-trying sigils that are still wrong, and the twenty-four rung
            // version it replaced ran 186–440 experience per 14400 ticks against
            // its 216–258.
            //
            // The sockets that did *not* move are unaffected by the new baseline in
            // the only way that matters — their contribution to `aligned` is
            // unchanged — so what they have ruled out stays ruled out.
            for (socket, moved) in self.touched.iter().enumerate() {
                if *moved {
                    self.tried[socket] = [false; SIGILS.len()];
                }
            }
        } else {
            self.aperture = self.held;
        }
        self.pressed = true;
        self.replenish();

        // **A gain settles a socket only when it was the only one that moved**,
        // and both narrower versions of this rule were wrong before it.
        //
        // Settling every socket ever touched locked the whole aperture on the
        // first gain. Settling every socket touched *by this press* is worse in
        // a subtler way: with no repeats, seating a sigil already in play
        // exchanges two sockets, so a gain of one settles both — and the other
        // one can be wrong. Code `[0,1,3,4]` reaches `aligned 3` that way with
        // its fourth socket locked at the wrong sigil, and no ladder can ever
        // finish it.
        //
        // One socket moved and `aligned` rose is **entailed**, not inferred:
        // nothing else changed, so that socket is now right. That is the only
        // claim this may make, and it is what lets a spell safely skip a settled
        // socket — the improving move a ward always has is provably reachable
        // without disturbing one (see `seat`).
        if self.shift == Some(Shift::Gained) && self.touched.iter().filter(|it| **it).count() == 1 {
            for (socket, settled) in self.settled.iter_mut().enumerate() {
                *settled |= self.touched[socket];
            }
        }
        self.touched = [false; WIDTH];
    }

    /// What the ward says about the aperture, without recording it.
    #[must_use]
    pub fn answer(&self) -> (u32, u32) {
        let aligned = (0..WIDTH)
            .filter(|slot| self.aperture[*slot] == self.code[*slot])
            .count();
        let shared = self
            .aperture
            .iter()
            .filter(|sigil| self.code.contains(sigil))
            .count();
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

    /// Whether the last press helped.
    #[must_use]
    pub const fn shift(&self) -> Option<Shift> {
        self.shift
    }

    /// What is in a socket now.
    #[must_use]
    pub fn seated(&self, socket: usize) -> Option<&'static str> {
        SIGILS.get(*self.aperture.get(socket)?).copied()
    }

    /// Whether a socket's last change gained.
    #[must_use]
    pub fn is_settled(&self, socket: usize) -> bool {
        self.settled.get(socket).copied().unwrap_or(false)
    }

    /// How many presses a sigil has taken part in.
    #[must_use]
    pub fn sigil_marks(&self, sigil: usize) -> u32 {
        self.sigil_marks.get(sigil).copied().unwrap_or(0)
    }

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
            settled: self.settled,
            marks: self.sigil_marks,
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
            held: self.held.to_vec(),
            aligned: self.aligned,
            astray: self.astray,
            best: self.best,
            pressed: self.pressed,
            spent: self.spent,
            shift: self.shift.map(|shift| shift.word().to_owned()),
            sigil_marks: self.sigil_marks.to_vec(),
            socket_marks: self.socket_marks.to_vec(),
            tried: self
                .tried
                .iter()
                .map(|socket| socket.iter().map(|&t| if t { 'x' } else { '.' }).collect())
                .collect(),
            settled: self.settled.to_vec(),
            touched: self.touched.to_vec(),
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
        fn sockets<T: Copy + Default>(values: &[T]) -> [T; WIDTH] {
            std::array::from_fn(|i| values.get(i).copied().unwrap_or_default())
        }

        /// A socket's worth of sigil indices, **range-checked**.
        ///
        /// `seat` guards this on the way in, so live play can never seat a sigil
        /// that does not exist — but a hand-edited file can, and `press` indexes
        /// `sigil_marks` directly. Without this, `aperture = [0, 1, 2, 99]` in a
        /// save panics on the next `probe` rather than on the way in, which is
        /// the opposite of what this function's contract promises.
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
            held: seated(&save.held),
            aligned: save.aligned,
            astray: save.astray,
            best: save.best,
            pressed: save.pressed,
            spent: save.spent,
            shift: save.shift.as_deref().and_then(Shift::named),
            sigil_marks: std::array::from_fn(|i| save.sigil_marks.get(i).copied().unwrap_or(0)),
            socket_marks: sockets(&save.socket_marks),
            tried: std::array::from_fn(|socket| {
                let row = save.tried.get(socket).map_or("", String::as_str);
                std::array::from_fn(|sigil| row.as_bytes().get(sigil) == Some(&b'x'))
            }),
            settled: sockets(&save.settled),
            touched: sockets(&save.touched),
            history: save
                .history
                .iter()
                .map(|entry| (sockets(&entry.figure), entry.aligned, entry.astray))
                .collect(),
        }
    }
}

impl Shift {
    /// The shift a word names, for reading a save back.
    pub(crate) fn named(word: &str) -> Option<Self> {
        match word {
            "closer" => Some(Self::Gained),
            "level" => Some(Self::Held),
            "further" => Some(Self::Lost),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ward(seed: u64) -> Ward {
        Ward::new(&mut Rngs::from_seed(seed))
    }

    /// Every code a ward can be — 6P4 = 360.
    fn every_code() -> Vec<[usize; WIDTH]> {
        let mut out = Vec::new();
        for a in 0..6 {
            for b in 0..6 {
                for c in 0..6 {
                    for d in 0..6 {
                        let code: [usize; WIDTH] = [a, b, c, d];
                        let mut seen = code.to_vec();
                        seen.sort_unstable();
                        seen.dedup();
                        if seen.len() == WIDTH {
                            out.push(code);
                        }
                    }
                }
            }
        }
        out
    }

    fn with_code(code: [usize; WIDTH]) -> Ward {
        let mut ward = ward(1);
        ward.code = code;
        ward
    }

    #[test]
    fn a_ladder_always_has_a_socket_left_to_turn() {
        // **`replenish`'s claim, and what makes a four-rung ladder safe.** A spell
        // guarded on `untried` falls through to `wait` when every socket reports
        // none, and the loop below it then presses an unchanged aperture for ever —
        // holding the tower's one production slot and earning nothing. That is the
        // failure a faucet has: not a crash, silence.
        //
        // Driven over every one of the 360 codes, because "six seeds never reached
        // it" is not the same claim.
        for code in every_code() {
            let mut ward = with_code(code);
            for _ in 0..60 {
                if ward.broken() {
                    break;
                }
                let left: u32 = (0..WIDTH).map(|socket| ward.untried(socket)).sum();
                assert!(
                    left > 0,
                    "{code:?} left no socket to turn: settled {:?}, aperture {:?}",
                    ward.settled,
                    ward.aperture,
                );
                for socket in 0..WIDTH {
                    ward.advance(socket);
                }
                ward.press();
            }
        }
    }

    #[test]
    fn the_four_rung_ladder_breaks_every_one_of_the_360_codes() {
        // **The shape a player can actually write**, run against the whole code
        // space rather than a handful of seeds: turn whatever socket still has a
        // candidate, press, repeat. That is `dev_spells.toml`'s `breaking` with the
        // `if`/`else` chain flattened, and it is only expressible because
        // `dial <socket>` takes no sigil — a variable-free spell cannot name the
        // one it has not tried.
        let mut worst = 0;
        let mut total = 0u32;
        let codes = every_code();
        for code in &codes {
            let mut ward = with_code(*code);
            let mut presses = 0;
            while !ward.broken() {
                presses += 1;
                assert!(presses <= 200, "{code:?} survived {presses} presses");
                let socket = (0..WIDTH).find(|socket| ward.untried(*socket) > 0);
                if let Some(socket) = socket {
                    ward.advance(socket);
                }
                ward.press();
            }
            worst = worst.max(presses);
            total += presses;
        }

        // **The mean is the margin the whole domain rests on**, so it is pinned
        // rather than left to a comment. A deducing player averages 4.14 over these
        // same 360 codes and the best play there is averages 4.08 — if a blind
        // ladder ever came near that, automating the lens would have deleted the
        // puzzle, which §19 records the first design doing.
        //
        // **And it is not brute force**: trying codes at random until one fits
        // averages 180. The ratchet and the settle-lock make this a monotone climb,
        // so it lands an order of magnitude below that and a *long* way above a
        // person.
        let count = u32::try_from(codes.len()).expect("360 codes fit a u32");
        let mean = f64::from(total) / f64::from(count);
        assert!(
            (8.0..40.0).contains(&mean),
            "a blind ladder averages {mean:.1} presses; a person averages 4.14 and \
             blind guessing averages 180 — this has left the band the domain needs",
        );
        assert!(worst <= 200, "worst case {worst} presses");
    }

    #[test]
    fn a_ward_never_repeats_a_sigil() {
        // The rule two other things rest on: `astray` stays informative, and a
        // `seat` stays ambiguous enough that hill-climbing cannot become a
        // per-socket oracle.
        for seed in 0..200 {
            let ward = ward(seed);
            let mut seen = ward.code.to_vec();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), WIDTH, "seed {seed} drew a repeat");
        }
    }

    #[test]
    fn the_orb_never_deduces() {
        // **The test that would have caught the first design.** Nothing the ward
        // stores may distinguish two codes that answer every press identically —
        // if it could, the orb would be holding knowledge the player has not
        // earned, which is what deleted the puzzle last time.
        //
        // Two codes chosen to answer the opening press *identically* —
        // `[1,0,2,3]` and `[0,1,3,2]` both read `aligned 2, astray 2` against
        // the opening `[0,1,2,3]`. Every field the ward publishes must therefore
        // match: if one of them differed, the orb would be holding knowledge the
        // player has not earned.
        let mut a = with_code([1, 0, 2, 3]);
        let mut b = with_code([0, 1, 3, 2]);
        a.press();
        b.press();
        assert_eq!(a.last(), (2, 2), "the fixture no longer answers as chosen");
        assert_eq!(a.last(), b.last(), "the two codes answered differently");
        assert_eq!(a.shift(), b.shift());
        assert_eq!(a.settled, b.settled, "a settled socket leaked the code");
        assert_eq!(a.sigil_marks, b.sigil_marks);
        assert_eq!(a.socket_marks, b.socket_marks);
        assert_eq!(a.broken(), b.broken());
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
                    let mut check = with_code(code);
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
    fn a_sigil_is_tallied_for_the_press_it_was_actually_in() {
        // Counting after the ratchet credited the *reverted* figure's sigils a
        // second time and left the pressed figure's untouched.
        let mut ward = with_code([0, 1, 2, 3]);
        ward.press();
        assert_eq!(ward.sigil_marks(4), 0, "pewter was never pressed");

        // Seat pewter, press, lose, snap back — pewter took part all the same.
        assert!(ward.seat(0, 4));
        ward.press();
        assert_eq!(
            ward.shift(),
            Some(Shift::Lost),
            "the fixture stopped losing"
        );
        assert_eq!(
            ward.sigil_marks(4),
            1,
            "a reverted press did not credit the sigil it sent",
        );
    }

    #[test]
    fn the_meter_only_rises() {
        // What `best` is for. A gauge fed from the last answer would fall back
        // whenever a guess did not help, reporting the guess and not the reading.
        let mut ward = with_code([0, 1, 2, 3]);
        ward.press();
        let high = ward.best();
        assert!(ward.seat(0, 4));
        ward.press();
        assert!(ward.last().0 < high, "the fixture stopped losing ground");
        assert_eq!(ward.best(), high, "progress went backwards");
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
    fn a_blind_ladder_breaks_every_ward_within_its_budget() {
        // **What `debug_spell breaking` will do, and it cannot revert.** §8's
        // language has no variables, so the ladder has no way to remember what a
        // socket held before a bad press — it can only move forward. That is
        // exactly what the settle-lock makes safe: a socket that gains is held,
        // every other seat is still available, and the walk cannot cycle.
        //
        // **The budget is measured, not derived**, and the arithmetic that looks
        // like a bound is not one: four sockets times five sigils plus the
        // opening is 21, and a real ladder needs more, because a `repeat until`
        // re-walks its rungs after every gain and a socket that was already
        // right at the opening costs five presses to prove it should not move.
        //
        // 80 against a measured worst of 51 in the Python model this design was
        // settled on — headroom, because a budget fitted to the measurement is a
        // budget that fails on the first content change.
        const BUDGET: u32 = 80;
        let mut worst = 0;
        for code in every_code() {
            let mut ward = with_code(code);
            ward.press();
            let mut tried: Vec<(usize, usize)> = Vec::new();

            while !ward.broken() && ward.spent() < BUDGET {
                let next = (0..WIDTH)
                    .filter(|socket| !ward.is_settled(*socket))
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
            assert!(
                ward.broken(),
                "{code:?} survived {BUDGET} presses at {} aligned",
                ward.last().0,
            );
            worst = worst.max(ward.spent());
        }
        // Pinned well under the budget, so the headroom stays headroom: a worst
        // case creeping toward 80 is the signal that the ladder has stopped
        // being a rule and started being a search.
        assert!(
            worst <= 60,
            "the blind ladder's worst case is now {worst} presses",
        );
    }

    #[test]
    fn deduction_beats_the_ladder() {
        // **The claim the whole domain rests on**, pinned. A player who presses
        // only figures consistent with every answer so far averages ~4.1; the
        // blind ladder above averages ~22.8. If this margin ever collapses, the
        // lens has no puzzle in it and the two channels have become one.
        //
        // The player is modelled rather than played: consistent guessing is what
        // a careful person does, and it is the *upper* bound on how well the orb
        // could ever do without the deduction it deliberately does not hold.
        let codes = every_code();
        let mut player = 0u32;
        for code in &codes {
            let mut live = codes.clone();
            let mut presses = 0;
            loop {
                let guess = live[0];
                presses += 1;
                let mut trial = with_code(*code);
                trial.aperture = guess;
                let (aligned, astray) = trial.answer();
                if aligned as usize == WIDTH {
                    break;
                }
                live.retain(|candidate| {
                    let mut probe = with_code(*candidate);
                    probe.aperture = guess;
                    probe.answer() == (aligned, astray)
                });
            }
            player += presses;
        }
        let mean = f64::from(player) / f64::from(u32::try_from(codes.len()).unwrap_or(u32::MAX));
        assert!(
            mean < 6.0,
            "a deducing player now needs {mean:.2} presses, so the ladder is competitive",
        );
    }

    #[test]
    fn seating_a_sigil_already_in_play_exchanges_rather_than_duplicates() {
        let mut ward = with_code([0, 1, 2, 3]);
        assert_eq!(ward.seated(0), Some("nitre"));
        assert!(ward.seat(0, 1), "nothing moved");
        assert_eq!(ward.seated(0), Some("alum"));
        assert_eq!(ward.seated(1), Some("nitre"), "the sigil was duplicated");
    }

    #[test]
    fn seating_what_is_already_there_moves_nothing() {
        let mut ward = with_code([0, 1, 2, 3]);
        assert!(!ward.seat(0, 0), "a no-op counted as a change");
    }
}
