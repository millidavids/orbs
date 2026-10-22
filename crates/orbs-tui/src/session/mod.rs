//! Everything the loop keeps between frames, and what it does with a keystroke,
//! a tick, a frame and a swap.

mod drawing;
mod motion;
mod state;
mod swap;
#[cfg(test)]
mod tests;
mod ticking;
mod typing;

pub(crate) use state::Session;
