//! Seeded randomness with per-subsystem streams.
//!
//! Every subsystem draws from its own independent stream, so adding a roll in one
//! place cannot perturb the sequence anywhere else. Without this, adding a single
//! aberration roll would silently change every parser tie-break in every replay.
//!
//! `ChaCha8Rng` is used rather than `StdRng` because it is reproducible across
//! platforms *and* stable across `rand` releases — `StdRng` explicitly guarantees
//! neither, and replay plus offline/online parity depend on both.

use bevy_ecs::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

/// Independent random streams. Add variants freely — existing streams are
/// unaffected, which is the entire point.
///
/// Deliberately **not** `#[non_exhaustive]`. That attribute exists for downstream
/// compatibility across crate versions; here every consumer is inside this
/// workspace, and exhaustive matching is a feature — adding a stream should force
/// the private `index` mapping to be updated rather than silently compiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RngStream {
    /// Parser tie-breaking when candidate intents score equally.
    Parser,
    /// Nuisance and adversarial aberration selection and timing.
    Aberration,
    /// Procedural threat trait composition.
    Threat,
    /// Soft drift applied to scripts and recipes.
    Drift,
    /// Reagent and fragment yields.
    Yield,
    /// Trace accrual jitter.
    Trace,
    /// The archive's stacks (§10, `tower::maze`).
    Archive,
    /// The lens: a ward's code, and what a broken seal spills (§10, `tower::ward`).
    Lens,
    /// The sanctum: how tall a course is (§10, `tower::erosion::height_for`).
    ///
    /// **The variant keeps the name the room had**, which is deliberate: a
    /// stream's identity is its *index* and a rename here changes nothing, so
    /// the cheap thing is to leave it and the expensive thing is to have it
    /// look like a renumber. The index table below is where that is enforced.
    Battlements,
    /// The menagerie: the figure a chant is drawn to (§10, `tower::chant`).
    Menagerie,
    /// The siege: every die the tower rolls (§10, `tower::dice`).
    ///
    /// **Its own stream, and sharing [`Threat`](Self::Threat) was the tempting
    /// shortcut.** That one is drawn from once a tick by `drift` and again by
    /// `substitution`, so a combat roll taken from it would shift the ambient
    /// sabotage schedule — moving the seeds `scripts/play.sh` chose, and
    /// invalidating every replay that has a siege in it.
    Siege,
}

impl RngStream {
    /// Number of distinct streams. Must equal the variant count.
    ///
    /// **Changing this is a save-format change**, which is not obvious from
    /// here: `save::Save::from_toml` refuses a document whose `[rng].positions`
    /// is not this long, so a world written with eight streams cannot be read by
    /// a build with nine. `save::FORMAT` went to 3 with the ninth so the refusal
    /// reads as *behind* rather than as *malformed*, and to **4** with the tenth
    /// for the same reason, and to **6** with the eleventh when the siege
    /// brought [`Siege`](RngStream::Siege).
    ///
    /// **A new domain almost always brings a stream, and a stream is always a
    /// format change** — three of the last four bumps were exactly this. The
    /// bump belongs in the same commit as the variant rather than being
    /// discovered by the first player whose tower will not open.
    pub const COUNT: usize = 11;

    /// Fixed index into [`Rngs::streams`].
    ///
    /// An explicit match rather than `as usize`: casting depends on declaration
    /// order, so reordering or inserting a variant would silently remap every
    /// stream and invalidate every existing replay, with no compile error.
    const fn index(self) -> usize {
        match self {
            Self::Parser => 0,
            Self::Aberration => 1,
            Self::Threat => 2,
            Self::Drift => 3,
            Self::Yield => 4,
            Self::Trace => 5,
            // **A new highest index, never inserted.** `derive_stream_seed`
            // folds the index in, so renumbering would silently remap every
            // stream and invalidate every existing replay.
            Self::Archive => 6,
            Self::Lens => 7,
            Self::Battlements => 8,
            Self::Menagerie => 9,
            Self::Siege => 10,
        }
    }
}

/// All random streams for a world, derived from one master seed.
#[derive(Resource, Debug, Clone)]
pub struct Rngs {
    master_seed: u64,
    streams: [ChaCha8Rng; RngStream::COUNT],
}

impl Rngs {
    /// Derive every stream from one master seed.
    ///
    /// The same seed always produces the same streams, on any platform and any
    /// build. That property is what replay and offline/online parity rest on.
    #[must_use]
    pub fn from_seed(master_seed: u64) -> Self {
        let streams =
            std::array::from_fn(|i| ChaCha8Rng::seed_from_u64(derive_stream_seed(master_seed, i)));
        Self {
            master_seed,
            streams,
        }
    }

    /// Every stream, wound forward to where it stood when the save was written.
    ///
    /// The master seed alone is **not** enough and never was: a stream is a
    /// `ChaCha8Rng` that advances in place, so reconstructing from the seed
    /// rewinds all eight to tick zero and the next roll is the *first* roll of
    /// the session. `sabotage`'s two systems each draw once per tick
    /// unconditionally, so a rewound stream would re-run the whole schedule of
    /// drifts and swaps from the beginning.
    ///
    /// The positions go beside [`master_seed`](Self::master_seed) in a save.
    /// Order is `RngStream`'s own fixed index order, which is never renumbered.
    #[must_use]
    pub fn positions(&self) -> [u128; RngStream::COUNT] {
        std::array::from_fn(|i| self.streams[i].get_word_pos())
    }

    /// Rebuild from a seed and the positions [`positions`](Self::positions) gave.
    ///
    /// Derives each stream exactly as [`from_seed`](Self::from_seed) does, so a
    /// save carries no key material, then winds each one forward.
    ///
    /// **The cost of deriving rather than storing: a change to
    /// `derive_stream_seed` breaks every existing save silently.** The keys are
    /// recomputed from the current mixing function and wound to the *old* word
    /// positions, so nothing compares and nothing errors — the world simply
    /// diverges from the session that wrote it on the first roll. Changing that
    /// function is therefore a **format change**, and `save::FORMAT` says so.
    #[must_use]
    pub fn restore(master_seed: u64, positions: [u128; RngStream::COUNT]) -> Self {
        let mut rngs = Self::from_seed(master_seed);
        for (stream, position) in rngs.streams.iter_mut().zip(positions) {
            stream.set_word_pos(position);
        }
        rngs
    }

    /// The seed every stream was derived from. Persisted in saves, with
    /// [`positions`](Self::positions) beside it, so a world can be reconstructed
    /// exactly.
    #[must_use]
    pub const fn master_seed(&self) -> u64 {
        self.master_seed
    }

    /// Mutable access to one stream. All randomness in the sim goes through here.
    pub const fn stream(&mut self, stream: RngStream) -> &mut ChaCha8Rng {
        &mut self.streams[stream.index()]
    }
}

/// `SplitMix64` — cheap, well-distributed mixing so adjacent stream indices do not
/// produce correlated sequences.
const fn derive_stream_seed(master: u64, index: usize) -> u64 {
    // `as` rather than `u64::try_from`: this is a const fn, TryFrom is not const,
    // and `index` is bounded by RngStream::COUNT.
    #[allow(clippy::cast_possible_truncation)]
    let index = index as u64;
    let mut z = master.wrapping_add(index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    const ALL: [RngStream; RngStream::COUNT] = [
        RngStream::Parser,
        RngStream::Aberration,
        RngStream::Threat,
        RngStream::Drift,
        RngStream::Yield,
        RngStream::Trace,
        RngStream::Archive,
        RngStream::Lens,
        RngStream::Battlements,
        RngStream::Menagerie,
        RngStream::Siege,
    ];

    #[test]
    fn stream_indices_are_unique_and_in_range() {
        let mut seen = [false; RngStream::COUNT];
        for s in ALL {
            let i = s.index();
            assert!(i < RngStream::COUNT, "{s:?} index out of range");
            assert!(!seen[i], "{s:?} has a duplicate index");
            seen[i] = true;
        }
        assert!(seen.iter().all(|&b| b), "a stream index is unused");
    }

    #[test]
    fn same_seed_reproduces_identical_sequences() {
        let mut a = Rngs::from_seed(0xDEAD_BEEF);
        let mut b = Rngs::from_seed(0xDEAD_BEEF);
        for s in ALL {
            let xs: Vec<u64> = (0..8).map(|_| a.stream(s).random()).collect();
            let ys: Vec<u64> = (0..8).map(|_| b.stream(s).random()).collect();
            assert_eq!(xs, ys, "{s:?} diverged for an identical seed");
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rngs::from_seed(1);
        let mut b = Rngs::from_seed(2);
        let x: u64 = a.stream(RngStream::Threat).random();
        let y: u64 = b.stream(RngStream::Threat).random();
        assert_ne!(x, y);
    }

    #[test]
    fn streams_are_independent() {
        // The property the whole design rests on: draining one stream must not
        // shift any other. If this fails, adding a roll anywhere breaks replay.
        let mut a = Rngs::from_seed(7);
        let baseline: u64 = a.stream(RngStream::Threat).random();

        let mut b = Rngs::from_seed(7);
        for _ in 0..1000 {
            let _: u64 = b.stream(RngStream::Aberration).random();
        }
        let after: u64 = b.stream(RngStream::Threat).random();

        assert_eq!(baseline, after, "streams are correlated");
    }

    #[test]
    fn distinct_streams_are_not_identical() {
        let mut r = Rngs::from_seed(99);
        let p: u64 = r.stream(RngStream::Parser).random();
        let t: u64 = r.stream(RngStream::Threat).random();
        assert_ne!(p, t, "stream seeds are insufficiently mixed");
    }

    #[test]
    fn a_restored_set_of_streams_carries_on_rather_than_starting_over() {
        // The property a save rests on. Draw unevenly across the streams first,
        // because a restore that only worked when every stream stood in the same
        // place would pass a symmetrical test and fail every real session.
        let mut lived = Rngs::from_seed(0x0B5);
        for (n, stream) in ALL.into_iter().enumerate() {
            for _ in 0..=n * 3 {
                let _: u64 = lived.stream(stream).random();
            }
        }

        let mut loaded = Rngs::restore(lived.master_seed(), lived.positions());

        for s in ALL {
            let next: u64 = lived.stream(s).random();
            let same: u64 = loaded.stream(s).random();
            assert_eq!(next, same, "{s:?} did not resume where it stood");
        }
    }

    #[test]
    fn the_seed_alone_would_rewind_every_stream() {
        // Why `positions` exists at all. If this ever stops holding, a
        // `ChaCha8Rng` has become stateless and the save can drop eight fields.
        let mut lived = Rngs::from_seed(7);
        for _ in 0..16 {
            let _: u64 = lived.stream(RngStream::Threat).random();
        }

        let mut rewound = Rngs::from_seed(lived.master_seed());
        let carried_on: u64 = lived.stream(RngStream::Threat).random();
        let from_the_top: u64 = rewound.stream(RngStream::Threat).random();

        assert_ne!(
            carried_on, from_the_top,
            "a stream reconstructed from the seed alone would replay the session",
        );
    }
}
