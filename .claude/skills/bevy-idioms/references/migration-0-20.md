# Bevy 0.19 → 0.20: what bit this project

Each item was met moving O.R.B.S. to 0.20.0 and checked against the 0.20
source, not recalled. `migration-0-19.md` still applies for anything older.

| 0.19 | 0.20 | Symptom if missed |
|---|---|---|
| `#[derive(ExtractComponent)]` alone | add `#[extract_app(RenderApp)]` (`use bevy::render::RenderApp`) | compile error naming `extract_app` |
| `.wgsl` ran through the preprocessor, so `#import bevy_core_pipeline::…` resolved | a `.wgsl` file reaches wgpu as written. Inline what you imported, or rename to `.wesl` and write `import crate::module::item;` before any declaration | **runtime** shader error; the gate stays green |
| `Camera2d` required `Tonemapping::None` | it requires `Tonemapping::Linear` | colours shift silently; compare a screenshot |
| `Core2dSystems` sets were `.chain()`ed | `.chain_weak()`: kept only between systems whose access conflicts. A pass taking `RenderContext` always conflicts (its state is `Deferred`), so pass-to-pass order holds | a system with no conflicting access drifts relative to the passes |
| `World::iter_resources()` yielded `(&ComponentInfo, Ptr)` | `(ComponentId, &ComponentInfo, Ptr)` | compile error |
| `ModifiersChanged` was dropped, so a key-up lost outside the window left a modifier held | `bevy_winit` reconciles held modifiers from it, writing synthetic modifier `KeyboardInput` presses and releases with no `text` | anything reacting to every `KeyboardInput` sees modifier events it never saw before |

New scheduling words: `chain_weak`, `before_weak`, `after_weak` — see
`scheduling.md`.

`bevy` 0.20 needs Rust **1.97.1**; `bevy_ecs` alone needs 1.95.
