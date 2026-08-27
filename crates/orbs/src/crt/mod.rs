mod pass;
mod plugin;
mod settings;

pub use plugin::CrtPlugin;
// **A set, so `sight` can order after the tube without seeing inside it.** The
// accommodation pass must run *after* this one — the grille, the aberration and
// the flash all put hue back into a pixel that had none. Exporting `crt_pass`
// itself would have meant exporting `CrtPipeline` and `CrtUniformBuffer` with
// it, which is a lot of the tube's insides for one edge.
pub(crate) use plugin::{CrtPass, Tube, seeded};
pub(crate) use settings::CrtSettings;
