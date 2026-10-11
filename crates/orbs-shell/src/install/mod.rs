//! The installation: a practice tower, and what it teaches before the real one
//! wakes. DESIGN.md §19, *The orb is installed before the tower wakes*.

mod dump;
mod paint;
mod state;
#[cfg(test)]
mod tests;

pub use dump::{drive as drive_dump, pose as pose_for_dump};
pub use paint::{paint, split};
pub use state::{Begun, Install, Raise, posed, wanted};
