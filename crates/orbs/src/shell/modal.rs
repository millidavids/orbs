//! A surface that is either up or not.

use bevy::prelude::*;

/// A modal surface, if it is up.
///
/// A resource rather than a component: there is one of each, it is modal, and
/// an entity would invite a second.
#[derive(Resource, Debug)]
pub(crate) struct Modal<T: Send + Sync + 'static>(Option<T>);

impl<T: Send + Sync + 'static> Default for Modal<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Send + Sync + 'static> Modal<T> {
    /// One already up.
    pub(crate) const fn up(surface: T) -> Self {
        Self(Some(surface))
    }

    /// The surface, if it is up.
    pub(crate) const fn get(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// The surface, to type into — or to paint, for the editor and the manual:
    /// only the painter knows how many lines fit.
    pub(crate) const fn get_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut()
    }

    /// Whether it is up, and so has the keyboard.
    #[must_use]
    pub(crate) const fn is_open(&self) -> bool {
        self.0.is_some()
    }

    /// Put `surface` up, replacing whatever was there.
    ///
    /// The one way in, so tests use the same door the game does rather than a
    /// second one that could drift.
    pub(crate) fn open(&mut self, surface: T) {
        self.0 = Some(surface);
    }

    /// Take it down.
    pub(crate) fn close(&mut self) {
        self.0 = None;
    }
}

/// Run condition: the surface `T` is up.
pub(crate) fn is_open<T: Send + Sync + 'static>(modal: Res<Modal<T>>) -> bool {
    modal.is_open()
}
