# Scheduling in Bevy 0.19

Read this when ordering systems, choosing a schedule, or debugging "my system
ran at the wrong time."

## Contents

- Choosing a schedule
- Ordering: the cheapest tool that works
- Run conditions
- Fixed timestep and interpolation
- States

## Choosing a schedule

| Schedule | Runs | Put here |
|---|---|---|
| `Startup` | once, before first update | spawning the world, loading handles |
| `Update` | every rendered frame | input, animation, camera, UI, anything frame-rate-dependent by nature |
| `FixedUpdate` | fixed steps, 0..n times per frame | physics, gameplay simulation, anything that must be deterministic |
| `OnEnter(S)` / `OnExit(S)` | on state transition | setup/teardown per state |

The rule: **if a bug in it would change the outcome of the game, it belongs in
`FixedUpdate`.** If it only affects what the current frame looks like, `Update`.

`FixedUpdate` may run zero times in a fast frame and several times in a slow one.
Never assume it runs once per frame.

## Ordering: the cheapest tool that works

Bevy runs systems in parallel when their declared access does not conflict.
Every ordering constraint you add removes parallelism, so add the least.

1. **No constraint** — default. Correct whenever systems touch disjoint data or
   the order genuinely does not matter.
2. **`.chain()`** — a strict sequence for a small, related group.
   ```rust
   app.add_systems(Update, (despawn_dead, bump_score).chain());
   ```
3. **System sets** — for coarse phases across a whole app. Configure the sets
   once, then assign systems to them.
   ```rust
   #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
   enum GameSet { Input, Simulation, Render }

   app.configure_sets(Update, (GameSet::Input, GameSet::Simulation, GameSet::Render).chain())
      .add_systems(Update, movement.in_set(GameSet::Simulation));
   ```
4. **`.before()` / `.after()`** — targeted pairwise constraints. Use sparingly;
   they get brittle as the app grows. Prefer sets.

Note that `Commands` are deferred: structural changes apply at the next sync
point, not immediately. If system B must observe an entity that system A
spawned, order them **and** be aware the spawn lands at the sync point between
them.

## Run conditions

Gate systems instead of branching inside them — a skipped system costs nothing.

```rust
app.add_systems(Update, gated
    .run_if(resource_exists::<Score>)
    .run_if(any_with_component::<Player>));
```

Common built-ins: `in_state(S)`, `resource_exists::<T>`, `resource_changed::<T>`,
`any_with_component::<T>`, `on_timer(Duration)`. Multiple `.run_if` calls AND
together. Closures work for custom conditions.

## Fixed timestep and interpolation

`FixedUpdate` uses `Time<Fixed>`; `Update` uses `Time` (virtual). Inside
`FixedUpdate`, `Res<Time>` still resolves to the fixed clock, but being explicit
documents intent:

```rust
fn physics_step(mut q: Query<&mut Transform>, time: Res<Time<Fixed>>) {
    for mut tf in &mut q {
        tf.translation.y -= 9.81 * time.delta_secs();
    }
}
```

Change the rate with `Time<Fixed>::from_hz(60.0)` inserted as a resource.

Because `FixedUpdate` and rendering are decoupled, fast-moving objects visibly
stutter unless you interpolate. The standard fix: keep the previous and current
simulated transform, and in `Update` render `lerp(prev, curr, overstep)` using
`Time<Fixed>`'s overstep fraction.

## States

```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState { #[default] Menu, Playing, Paused }

app.init_state::<GameState>()
   .add_systems(OnEnter(GameState::Playing), spawn_level)
   .add_systems(Update, tick.run_if(in_state(GameState::Playing)));
```

Change state by setting `ResMut<NextState<GameState>>`. The transition is applied
at a defined point, not instantly — do not expect the new state to be readable in
the same system that set it.

For entities that should not outlive a state, `DespawnOnExit(State)` handles
teardown automatically. In 0.19 these can fire when entering and exiting the same
state within one update; do not assume the transition is always to a different
state.
