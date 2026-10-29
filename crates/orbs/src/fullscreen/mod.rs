//! A full-screen post-process pass: one uniform, the frame so far, and a
//! fragment shader. The tube and §14's accommodation are both one.
//!
//! Plain systems in the `Core2d` schedule, not a port of `court_wizard`'s
//! `ViewNode`: Bevy has had no render graph since 0.19, and §4 flagged the
//! render-graph Rust as "Bevy's most volatile surface". The shaders ported
//! almost unchanged; the surrounding Rust did not survive at all.

mod history;
mod pass;

pub(crate) use history::History;
pub(crate) use pass::{Extracted, Fullscreen, UniformBuffer, init_pipeline, pass, prepare};
