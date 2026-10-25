//! How the tower's walls wear down, and what a finished course puts back.
//!
//! §11.5's Integrity, both halves. The first drain in the game, falling whether
//! or not anybody plays, which makes the sanctum a room you come back to — a
//! drain, not yet a contested pool (§19).
//!
//! Low integrity only raises a taller course ([`height_for`]): seven wards is
//! 127 hauls against three wards' seven, so a neglected tower is expensive and
//! never impossible. Feeding it into sabotage rates would move every rate
//! `orbs-balance` pins (§19). [`erode`] draws nothing; the domain's one draw is
//! in [`height_for`].

use bevy_ecs::prelude::*;

use super::pylon::{LEAST, MOST};
use crate::tick::Tick;

/// Walls in good repair — a hundred, so it reads as a percentage untold.
pub const STANDING: u32 = 100;

/// Ticks a point of integrity lasts.
///
/// Whole to nothing in fifty minutes unattended, about one course an hour.
/// `orbs-balance`'s `warding` policy settles it.
pub const EROSION: u64 = 30;

/// How much of the deficit one ward of a finished course puts back — per ward,
/// because a seven-ward course is sixteen times the hauling of a three.
pub const MENDED_PER_WARD: u32 = 8;

/// How the tower's defences stand, from nothing to [`STANDING`].
///
/// A resource, `Experience`'s precedent: one number about the whole tower. The
/// pylon carries a reading of it, republished when it changes — see [`erode`].
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

    /// Put a total back, for a save — not [`mend`](Self::mend), since a load is
    /// the world as it already was.
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
/// Exclusive, because the pylon's `integrity` reading is a node, republished on
/// the tick the value moves; leaving it alone had `survey pylon` report a whole
/// defence for ever.
pub fn erode(world: &mut World) {
    // A tower at nothing is a state the game has to be able to sit in.
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
/// What was actually added, which the completion line quotes, so nobody is told
/// they mended forty into a wall missing ten.
pub fn mend(world: &mut World, height: usize) -> u32 {
    // Per ward, plus the Ley Line's `mend` on every finished course.
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
/// [`mend`] in points rather than wards, for a siege, which has none. It
/// republishes through `pylon::fixture`, since its caller is in the bailey.
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
/// The player wearing it, where [`erode`] is ambient — a risk chosen, so it
/// moves no pinned rate until somebody fights. It republishes, which is why it
/// is not two lines at the call site (§19).
pub fn wear_by(world: &mut World, points: u32) -> u32 {
    let mut integrity = world.resource_mut::<Integrity>();
    let before = integrity.get();
    integrity.0 = integrity.0.saturating_sub(points);
    let after = integrity.get();
    // The entity, never `Cwd`: a chant collapsing in the menagerie found no
    // pylon there and returned every time.
    if let Some(pylon) = super::pylon::fixture(world) {
        crate::execute::publish_pylon(world, pylon);
    }
    before.saturating_sub(after)
}

/// How tall a course to raise, given how far the walls have slipped.
///
/// The jitter makes the parity reading load-bearing, or a spell would hard-code
/// its tower's cycle. One draw per `muster`.
pub fn height_for(world: &mut World, deficit: u32) -> usize {
    // 3, 4, 5, 6 across a whole tower's decay, the seventh left to the jitter.
    // Capped at `MOST - 1`, or a wholly worn tower always mustered an odd course.
    let base = LEAST
        .saturating_add((deficit / 25) as usize)
        .min(MOST.saturating_sub(1));
    let roll: u64 = {
        let mut rngs = world.resource_mut::<crate::rng::Rngs>();
        rand::RngExt::random(rngs.stream(crate::rng::RngStream::Battlements))
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
        // A `u32` going under would wrap and read as perfect repair.
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
        // Monotone and capped across the band.
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
        // Every deficit across the band: sampling only nought missed a worn
        // tower that was always odd.
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
        world.resource_mut::<Integrity>().restore(STANDING - 5);
        assert_eq!(mend(world, MOST), 5);
        world.resource_mut::<Integrity>().restore(0);
        assert_eq!(mend(world, 3), MENDED_PER_WARD * 3);
    }
}
