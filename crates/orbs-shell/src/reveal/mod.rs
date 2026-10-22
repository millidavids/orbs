//! Output arriving over a wire instead of all at once.
//!
//! A command's answer used to appear whole on the frame the tick produced it.
//! Now it prints, the way a terminal attached to something slow prints.
//!
//! Presentation only: every record is in the
//! [`Scrollback`](orbs_sim::Scrollback) the instant the tick makes it, and this
//! draws fewer of their cells. The sim never learns a reveal happened
//! (architectural rule 2) — `orbs-balance` would owe typewriter delays to stay
//! in step (§13), offline catch-up would owe animation time, and §9's parity
//! rule forbids a display setting becoming a difficulty choice.
//!
//! The cost it is meant to create arrives anyway: it spends the player's
//! attention, and a player who automates stops spending it.
//!
//! The first character lands on the frame the record does — a terminal that
//! takes a second to answer reads as broken (§19), and §6 makes the echo how
//! players learn the vocabulary. Only the rest takes time.

mod burst;
mod pace;
#[cfg(test)]
mod tests;

pub use burst::Reveal;
pub use pace::Pace;
