//! How the tower's walls wear down, and what a finished course puts back.
//!
//! DESIGN.md §11.5 lists **Integrity** among the resources — *"Repair, warding |
//! Damaged by sieges, decay, aberrations | Tower health, persistent"* — and until
//! now nothing produced it and nothing spent it. This is both halves.
//!
//! # The first drain in the game
//!
//! Everything the tower has is a faucet: experience only rises, fragments
//! accumulate, a potion made stays made. Integrity is the first number that
//! falls on its own, and it falls whether or not anybody is playing — which is
//! what makes the sanctum a room you come *back* to rather than one you
//! finish.
//!
//! **It is honest but thin, and §19 says so.** A depleting resource is §10.1's
//! first row, and this one depletes without anything else contending for it. It
//! is a drain, not yet a contested pool.
//!
//! # What low integrity does today, and what it will do
//!
//! Today, exactly one thing: [`height_for`] raises a taller course. Seven wards
//! is 127 hauls against three wards' seven, so a tower left alone is expensive to
//! put right and never impossible — §11.5's *"never ruinous, only slower"*.
//!
//! **Deliberately nothing else, and the reason is measurement rather than
//! scope.** The obvious coupling is to feed integrity into `sabotage`'s nuisance
//! rates, which is where DESIGN.md's siege model eventually puts it. Doing it
//! here would move every rate `orbs-balance` has pinned and every seed
//! `scripts/play.sh` chose, for a consequence no siege exists to spend yet. §19
//! records the deferral; Phase 8 is where it lands.
//!
//! # It reads a clock and draws nothing
//!
//! [`erode`] is appended to the tick schedule after `settling`, and like
//! `settling` it takes no value from any `RngStream` — it compares two ticks. A
//! system that drew would shift every existing replay from its position onward,
//! which is the rule `drift` and `substitution` are ordered by. The one draw this
//! domain makes is in [`height_for`], once per `muster`, from
//! `RngStream::Battlements`.

use bevy_ecs::prelude::*;

use super::pylon::{LEAST, MOST};
use crate::tick::Tick;

/// Walls in good repair.
///
/// A hundred because it is the number a player reads as a percentage without
/// being told, and because §11.5's invariant 1 is written as *"a set fraction of
/// its departure value"* — a fraction of a hundred is a number you can say.
pub const STANDING: u32 = 100;

/// Ticks a point of integrity lasts.
///
/// **A placeholder, like every duration in the game.** At thirty, a tower goes
/// from whole to nothing in fifty minutes unattended, which puts one course an
/// hour somewhere near the middle of what a player would want to do anyway.
/// `orbs-balance`'s `warding` policy is the instrument that settles it.
pub const EROSION: u64 = 30;

/// How much of the deficit one ward of a finished course puts back.
///
/// The award scales with the course, because the work does: a seven-ward course
/// is sixteen times the hauling of a three-ward one and would otherwise pay the
/// same. Also a placeholder.
pub const MENDED_PER_WARD: u32 = 8;

/// How the tower's defences stand, from nothing to [`STANDING`].
///
/// **A resource rather than a component on the pylon**, on `Experience`'s
/// precedent and for the same reason: it is one number about the whole tower,
/// the rail reads it while a frame is being painted, and a save carries it as a
/// single value. What the pylon carries is a *reading* of it, republished when
/// it changes — see [`erode`].
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Integrity(u32);

impl Default for Integrity {
    /// A new tower's walls are whole.
    fn default() -> Self {
        Self(STANDING)
    }
}

impl Integrity {
    /// How the walls stand.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// How far from whole they are.
    #[must_use]
    pub const fn deficit(self) -> u32 {
        STANDING.saturating_sub(self.0)
    }

    /// Wear one point away. Never below nothing.
    const fn wear(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    /// Put some back. Never above whole.
    const fn mend(&mut self, by: u32) {
        self.0 = self.0.saturating_add(by);
        if self.0 > STANDING {
            self.0 = STANDING;
        }
    }

    /// Put a total back, for a save.
    ///
    /// **Not [`mend`](Self::mend)**, on `Experience::restore`'s precedent: a load
    /// is the world being what it already was, and clamping a restored value up
    /// to whole would quietly repair a tower the player left broken.
    pub(crate) const fn restore(&mut self, standing: u32) {
        self.0 = if standing > STANDING {
            STANDING
        } else {
            standing
        };
    }
}

/// Wear the walls down, one point every [`EROSION`] ticks.
///
/// Reads the clock and nothing else, so it can sit anywhere after the threat
/// rolls without touching a replay.
///
/// # Exclusive, because the number has to reach the world
///
/// It would be a two-argument system if all it did was decrement a resource. It
/// is not: the pylon carries an `integrity` **reading**, and a reading is a
/// node. The first version wore the resource down and left the node where it
/// was, so `survey pylon` and a spell's `if the pylon has fewer than 20
/// integrity` both went on reporting a whole defence for ever while the rail —
/// which reads the resource directly — counted down beside them.
///
/// Republished only on the tick the value actually moves, which is one in
/// [`EROSION`] rather than every one.
pub fn erode(world: &mut World) {
    // Nought is the floor and the system stops doing anything there, rather than
    // wrapping or going negative. A tower at nothing is at its worst, which is a
    // state the game has to be able to sit in.
    if world.resource::<Integrity>().get() == 0 {
        return;
    }
    if !world.resource::<Tick>().get().is_multiple_of(EROSION) {
        return;
    }
    world.resource_mut::<Integrity>().wear();
    if let Some(pylon) = super::pylon::fixture(world) {
        crate::execute::publish_pylon(world, pylon);
    }
}

/// Put back what a finished course is worth, and say how much.
///
/// Returns what was actually added, which is less than asked for when the walls
/// were nearly whole — the number the completion line quotes, so a player is
/// never told they mended forty points into a wall that had ten missing.
pub fn mend(world: &mut World, height: usize) -> u32 {
    // Per ward, plus the Ley Line's `mend` on top of every finished course.
    let asked = MENDED_PER_WARD
        .saturating_mul(u32::try_from(height).unwrap_or(u32::MAX))
        .saturating_add(super::grant::mend_bonus(world));
    let mut integrity = world.resource_mut::<Integrity>();
    let before = integrity.get();
    integrity.mend(asked);
    let after = integrity.get();
    after.saturating_sub(before)
}

/// Put a number of points back onto the barrier, and say how many landed.
///
/// [`mend`]'s point-denominated twin, and [`wear_by`]'s mirror. It exists
/// because `mend` is denominated in **wards** — the sanctum's unit — and a siege
/// has no wards in it: expressing a held wall as "one and a half wards" would
/// leak one domain's currency into another's for no reason but the shape of an
/// existing signature.
///
/// **It republishes**, for the reason `wear_by` documents at length: a writer
/// that moves the resource and leaves the reading alone is §19's defect, and it
/// has been made three times. `pylon::fixture`, never `Cwd` — the caller here is
/// a siege ending in the *bailey*.
pub fn mend_by(world: &mut World, points: u32) -> u32 {
    let mut integrity = world.resource_mut::<Integrity>();
    let before = integrity.get();
    integrity.mend(points);
    let after = integrity.get();
    if let Some(pylon) = super::pylon::fixture(world) {
        crate::execute::publish_pylon(world, pylon);
    }
    after.saturating_sub(before)
}

/// Take a number of points off the barrier, and say how many were actually
/// taken.
///
/// [`mend`]'s mirror, and it returns for the same reason: a chant that collapses
/// against a barrier already down to two must not be told it cost five. The
/// caller quotes the number.
///
/// # This is the second thing that writes integrity, and the first was ambient
///
/// [`erode`] is the tower wearing on its own; this is the player wearing it,
/// which is why it is admissible where §19 deferred the *nuisance* coupling to
/// Phase 8. That entry's objection was that raising `drift`'s odds from integrity
/// would move every rate `orbs-balance` has pinned; a cost the player chooses to
/// risk moves nothing until somebody chants.
///
/// **It republishes**, and that is the whole of why it is not two lines at the
/// call site. §19 records `erode` having to become an exclusive system for
/// exactly this — it wore the resource down and left the *reading* alone, so
/// `survey pylon` and every `if the pylon has fewer than n integrity` reported a
/// whole barrier while the rail counted down.
pub fn wear_by(world: &mut World, points: u32) -> u32 {
    let mut integrity = world.resource_mut::<Integrity>();
    let before = integrity.get();
    integrity.0 = integrity.0.saturating_sub(points);
    let after = integrity.get();
    // **`pylon::fixture`, never `refresh_pylon`.** The latter reads `Cwd`, and
    // the caller that matters here is a chant collapsing in the *menagerie* — so
    // it found no pylon and returned every single time, leaving `survey pylon`
    // and every `if the pylon has fewer than n integrity` reporting a whole
    // barrier while the rail counted down. That is the §19 defect this function's
    // own doc claims to prevent, made three lines below the claim.
    //
    // `erode` above takes the same route for the same reason, and its comment
    // at `pylon::fixture` says **not `Cwd`** in bold. A publisher that runs on
    // anything but the player's own command must take the entity.
    if let Some(pylon) = super::pylon::fixture(world) {
        crate::execute::publish_pylon(world, pylon);
    }
    before.saturating_sub(after)
}

/// How tall a course to raise, given how far the walls have slipped.
///
/// **The jitter is what makes the parity reading load-bearing.** Without it a
/// player learns their tower's height and hard-codes the cycle, and the one
/// thing this puzzle asks anybody to *read* stops being read. One draw, from the
/// domain's own stream, taken once per `muster`.
pub fn height_for(world: &mut World, deficit: u32) -> usize {
    // A quarter of the range per extra ward, which walks 3, 4, 5, 6 across a
    // whole tower's decay and leaves the seventh to the jitter.
    //
    // **Capped at `MOST - 1`, and it shipped without that.** At a deficit of 100
    // the division alone reaches 7, and `+ roll % 2` then clamped straight back
    // to 7 — so a wholly worn tower mustered exactly seven wards on every seed
    // and the course was *always odd*. That is the one state where a course is
    // 127 hauls long, and it was the one state where the parity was a constant a
    // player or a spell could hard-code: the reading this jitter exists to
    // protect stopped being worth asking for exactly where it mattered most.
    let base = LEAST
        .saturating_add((deficit / 25) as usize)
        .min(MOST.saturating_sub(1));
    let roll: u64 = {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        rand::Rng::random(rngs.stream(crate::rng::RngStream::Battlements))
    };
    base.saturating_add((roll % 2) as usize).clamp(LEAST, MOST)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sim;

    #[test]
    fn a_new_tower_stands_whole_and_wears_down_on_its_own() {
        let mut sim = Sim::new(1);
        assert_eq!(sim.integrity(), STANDING);
        sim.step_n(EROSION * 10);
        assert!(
            sim.integrity() < STANDING,
            "an hour of nothing left the walls untouched",
        );
    }

    #[test]
    fn integrity_never_falls_below_nothing_and_never_rises_above_whole() {
        // Both clamps, and the floor is the one that matters: a `u32` going
        // under would wrap to four billion and read as a tower in perfect
        // repair, which is the worst possible way for this to fail.
        let mut integrity = Integrity::default();
        for _ in 0..(STANDING + 50) {
            integrity.wear();
        }
        assert_eq!(integrity.get(), 0);
        integrity.mend(u32::MAX);
        assert_eq!(integrity.get(), STANDING);
    }

    #[test]
    fn a_worn_tower_musters_a_taller_course() {
        // The whole of what erosion does today. Checked across the band rather
        // than at one point, because the interesting property is that it is
        // monotone and capped, not what any single deficit gives.
        let mut sim = Sim::new(3);
        let world = sim.world_mut();
        let mut tallest = 0;
        for deficit in [0, 25, 50, 75, 100] {
            let height = height_for(world, deficit);
            assert!(
                (LEAST..=MOST).contains(&height),
                "a deficit of {deficit} asked for {height} wards",
            );
            assert!(
                height + 1 >= tallest,
                "a worse tower mustered a shorter course: {height} after {tallest}",
            );
            tallest = tallest.max(height);
        }
        assert!(
            tallest > LEAST,
            "a wholly worn tower musters no more than a whole one does",
        );
    }

    #[test]
    fn the_height_is_jittered_so_the_parity_cannot_be_assumed() {
        // The reading `odd` exists because this is true. If one deficit always
        // gave one height, a spell could hard-code its cycle and never ask.
        //
        // **Every deficit across the band, and it only sampled nought.** The
        // shipped curve reached `MOST` by division at a deficit of 100 and the
        // jitter clamped straight back to it, so a wholly worn tower always
        // mustered seven wards — always odd — and this test could not see it
        // from the one end where the bug is absent.
        let mut sim = Sim::new(7);
        let world = sim.world_mut();
        for deficit in [0, 25, 50, 75, 100, STANDING] {
            let mut seen = std::collections::BTreeSet::new();
            for _ in 0..40 {
                seen.insert(height_for(world, deficit));
            }
            assert!(
                seen.len() > 1,
                "forty musters at a deficit of {deficit} gave one height: {seen:?}",
            );
            // ...and both parities, which is the property `odd` is published for.
            assert!(
                seen.iter().any(|height| height.is_multiple_of(2))
                    && seen.iter().any(|height| !height.is_multiple_of(2)),
                "a deficit of {deficit} only ever gives one parity: {seen:?}",
            );
        }
    }

    #[test]
    fn mending_reports_what_it_actually_put_back() {
        let mut sim = Sim::new(1);
        let world = sim.world_mut();
        // Nearly whole: a big course cannot add more than is missing.
        world.resource_mut::<Integrity>().restore(STANDING - 5);
        assert_eq!(mend(world, MOST), 5);
        // ...and a worn one takes the whole award.
        world.resource_mut::<Integrity>().restore(0);
        assert_eq!(mend(world, 3), MENDED_PER_WARD * 3);
    }
}
