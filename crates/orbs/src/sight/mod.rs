mod pass;
mod plugin;
mod settings;

pub use plugin::SightPlugin;
// The accommodation as a *setting*: what it is called, what it can be, and the
// resource that holds it. `shell::setting` builds the settings row from these,
// for the reason that module's header gives — a page shows what is in effect,
// asked of the thing that holds it, never of the file.
pub(crate) use settings::{Sight, Vision};
