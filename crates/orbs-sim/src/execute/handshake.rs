//! A verb asking a frontend to open a surface the sim does not draw.

use std::marker::PhantomData;

use bevy_ecs::prelude::*;

/// A request a verb raises once and a frontend takes once.
///
/// Taken rather than read, or a polling frontend reopens its surface every
/// frame. The sim records the decision; the frontend owns the screen.
#[derive(Resource, Debug)]
pub struct Handshake<M: Send + Sync + 'static> {
    asked: bool,
    surface: PhantomData<M>,
}

impl<M: Send + Sync + 'static> Default for Handshake<M> {
    fn default() -> Self {
        Self {
            asked: false,
            surface: PhantomData,
        }
    }
}

impl<M: Send + Sync + 'static> Handshake<M> {
    /// Ask for the surface.
    pub const fn ask(&mut self) {
        self.asked = true;
    }

    /// Whether a request is waiting, without taking it.
    ///
    /// Taking needs `&mut`, which stamps the change tick and leaves a
    /// `resource_changed` run condition true for ever.
    #[must_use]
    pub const fn pending(&self) -> bool {
        self.asked
    }

    /// Take the pending request, if there is one.
    pub const fn take(&mut self) -> bool {
        let asked = self.asked;
        self.asked = false;
        asked
    }
}

/// `unfurl`'s surface: the transcript takes the keyboard.
#[derive(Debug)]
pub enum Transcript {}

/// `wander`'s surface: the arrow keys walk the maze.
#[derive(Debug)]
pub enum Arrows {}

/// `weave`'s surface: the ley line's screen.
#[derive(Debug)]
pub enum Weave {}

/// `menu`'s surface: the orb's menu. Its own request rather than a second flag
/// on `Quitting`, since the two are no longer the same question (§19).
#[derive(Debug)]
pub enum Menu {}

/// A request to hand the transcript the keyboard.
pub type Unfurling = Handshake<Transcript>;
/// Whether `wander` has asked for the arrow keys.
pub type Wandering = Handshake<Arrows>;
/// A request to open the weave screen.
pub type Weaving = Handshake<Weave>;
/// A request to open the orb's menu.
pub type Menuing = Handshake<Menu>;
