//! What a room is waiting on — one answer for the rail, `status rates` and the
//! balance sweep (§19, *the tower speaks up*).
//!
//! The rail used to report a room by its first busy instrument or else its
//! first, so a laboratory with a fouled alembic and a cold athanor read `idle`:
//! most silent exactly when it was stalled (ROADMAP, *the tower as one
//! machine*).

mod room;
mod said;
mod short;
mod slot;

pub use room::{Wait, reported, wait_in, wait_of};
pub use said::sentence;
pub use short::short;
pub use slot::{Slot, slot};
