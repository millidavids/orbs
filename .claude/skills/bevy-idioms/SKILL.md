---
name: bevy-idioms
description: >
  Idiomatic Bevy 0.19 engine code — App/plugin structure, components, required
  components, resources, systems, queries and filters, Commands, scheduling and
  system sets, fixed timestep, states, messages, and observers. Use this skill
  whenever the user is writing, reviewing, debugging, or refactoring Bevy code,
  whenever Cargo.toml depends on `bevy` or `bevy_ecs`, and whenever you see
  `App::new`, `add_systems`, `Query<`, `Commands`, `#[derive(Component)]`, or
  `#[derive(Resource)]` — even if the user does not say "Bevy". Bevy's API breaks
  every minor release, so ALWAYS consult this skill instead of recalling Bevy
  syntax from memory; most Bevy code on the internet targets 0.14-0.16 and will
  not compile on 0.19.
when_to_use: >
  Triggers: "bevy", "ECS system", "spawn an entity", "component", "query",
  "add_systems", "why does my system panic", "B0001", "bundle not found",
  Bevy upgrade/migration questions, or any Rust game code using bevy.
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
metadata:
  targets_bevy: "0.19"
  verified: "All code in this skill compiled against bevy 0.19.0 on 2026-08-02."
  author: davidyurek
---

# Idiomatic Bevy (0.19)

Every snippet here was compiled against **bevy 0.19.0**. Bevy breaks APIs every
minor release, so version accuracy is the whole point of this skill.

## Step 0 — always check the version first

Before writing any Bevy code, read the actual version from `Cargo.toml` or
`Cargo.lock`.

- **0.19** → this skill applies directly.
- **Anything else** → say so explicitly, then verify each API against the docs
  for that version. Do not silently emit 0.19 syntax for a 0.17 project.

Pin the version and upgrade deliberately:

```toml
[dependencies]
bevy = "=0.19.0"
```

## Non-negotiables

1. **Never copy Bevy snippets from memory or the internet without checking the
   version.** Most published Bevy code targets 0.14-0.16 and will not compile.
2. **Bundles do not exist.** They were deprecated in 0.15 and removed in 0.16.
   Use required components. `SpriteBundle`, `Camera2dBundle`, `NodeBundle` are
   all gone.
3. **Drive motion by `time.delta_secs()`**, never by a per-frame constant.
4. **Put simulation in `FixedUpdate`**, rendering/input-feel in `Update`.
5. **Order only what must be ordered.** Bevy parallelises systems whose data
   access does not conflict; needless `.chain()` throws that away.

## Components and required components

Required components replace bundles: declaring `Player` guarantees the rest.

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
#[require(Transform, Velocity, Health(100.0))]
pub struct Player;
```

`commands.spawn(Player)` now yields an entity with `Transform`, `Velocity`, and
`Health(100.0)` attached automatically.

Prefer many small components over one large one — they filter and parallelise
better. Marker components (`struct Player;`) are free and make queries precise.

## Resources

```rust
#[derive(Resource, Default)]
pub struct Score(pub u32);
```

**0.19 change — resources are components.** `#[derive(Resource)]` now implements
`Component`, and resources live as components on dedicated entities. Two
consequences that bite:

- A type can no longer derive **both** `Component` and `Resource`. Split it.
- Very broad queries (`Query<Entity>`, `Query<()>`) now also match resource
  entities. Narrow the query, or filter the resource entities out.

## Systems

Parameters declare data access; that declaration is what lets Bevy parallelise.

```rust
fn movement(time: Res<Time>, mut q: Query<(&mut Transform, &Velocity)>) {
    for (mut tf, vel) in &mut q {
        tf.translation += vel.0.extend(0.0) * time.delta_secs();
    }
}

// Filters: With / Without / Changed / Added
fn only_players(q: Query<&Transform, (With<Player>, Without<Enemy>)>) {}
fn on_health_change(q: Query<&Health, Changed<Health>>) {}

// Commands defer structural change (spawn/despawn/insert/remove)
fn despawn_dead(mut commands: Commands, q: Query<(Entity, &Health)>) {
    for (entity, health) in &q {
        if health.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

Exactly one match — use `Single`, not `single().unwrap()`:

```rust
fn camera_follow(player: Single<&Transform, With<Player>>) {
    let _ = player.translation;
}

// Tolerate absence:
fn maybe(player: Option<Single<&Transform, With<Player>>>) {}
```

Fallible systems return `Result`, so `?` works:

```rust
fn fallible(q: Query<&Transform, With<Player>>) -> Result {
    let tf = q.single()?;
    Ok(())
}
```

## Spawning hierarchies

```rust
commands.spawn((
    Player,
    Transform::default(),
    children![
        (Enemy, Transform::default()),
        (Enemy, Transform::from_xyz(1.0, 0.0, 0.0)),
    ],
));
```

Read relationships with `Children` and `ChildOf` (not `Parent` — renamed):

```rust
fn read_hierarchy(q: Query<&Children>, parents: Query<&ChildOf>) {
    for child_of in &parents {
        let _parent = child_of.parent();
    }
}
```

## Messages vs observers — pick deliberately

**Messages** are buffered, read next frame, good for streams of gameplay facts.
Note the 0.17+ rename: buffered events are now `Message`, not `Event`.

```rust
#[derive(Message)]
struct Scored { points: u32 }

fn send(mut writer: MessageWriter<Scored>) { writer.write(Scored { points: 10 }); }
fn receive(mut reader: MessageReader<Scored>) {
    for msg in reader.read() { let _ = msg.points; }
}
```

**Observers** run immediately, targeted at an entity — good for reactions
(damage, death, pickup).

```rust
#[derive(EntityEvent)]
struct Damage { entity: Entity, amount: f32 }

fn on_damage(damage: On<Damage>, mut q: Query<&mut Health>) {
    if let Ok(mut health) = q.get_mut(damage.entity) {
        health.0 -= damage.amount;
    }
}

app.add_observer(on_damage);
```

Use a **message** for "many per frame, order-insensitive"; an **observer** for
"react right now to this specific entity."

## Scheduling

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet { Input, Simulation, Render }

app.configure_sets(Update, (GameSet::Input, GameSet::Simulation, GameSet::Render).chain())
   .add_systems(Update, movement.in_set(GameSet::Simulation))
   .add_systems(FixedUpdate, physics_step)
   .add_systems(Update, gated.run_if(resource_exists::<Score>));
```

Fixed timestep uses its own clock — `Time<Fixed>`:

```rust
fn physics_step(mut q: Query<&mut Transform>, time: Res<Time<Fixed>>) {
    for mut tf in &mut q {
        tf.translation.y -= 9.81 * time.delta_secs();
    }
}
```

See `references/scheduling.md` for ordering strategy, run conditions, and
choosing between `Update` and `FixedUpdate`.

## Organise with plugins

Group every feature into a `Plugin`; keep `main.rs` a list of plugins.

```rust
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (movement, despawn_dead));
    }
}
```

## Debugging the panics you will actually hit

**B0001 — conflicting query access.** Two queries in one system that can alias
the same component mutably. Fix by making them provably disjoint, or use
`ParamSet`:

```rust
// Disjoint via Without — preferred, keeps parallelism
fn ok(
    mut players: Query<&mut Transform, With<Player>>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {}

// Genuinely overlapping — ParamSet serialises access
fn also_ok(mut set: ParamSet<(Query<&mut Transform>, Query<&Transform>)>) {
    for mut tf in &mut set.p0() { tf.translation.y += 1.0; }
    for _tf in &set.p1() {}
}
```

**`SpriteBundle`/`Camera2dBundle` not found** → bundles were removed in 0.16.
Spawn the component directly: `commands.spawn(Camera2d)`.

**`delta_seconds` not found** → renamed to `delta_secs()` in 0.16
(`elapsed_seconds` → `elapsed_secs()`).

**A type derives both `Component` and `Resource`** → illegal as of 0.19.

## 2D quick reference

```rust
fn setup_2d(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::splat(32.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn debug_draw(mut gizmos: Gizmos) {
    gizmos.circle_2d(Vec2::ZERO, 50.0, Color::WHITE);
}
```

## Build ergonomics

For a 2D game, start from `default-features = false` and add what you need —
default features drag in PBR, glTF, and 3D pipelines.

Use `dynamic_linking` in dev only; it cuts incremental builds dramatically.

```toml
[dependencies]
bevy = { version = "=0.19.0", default-features = false, features = [
    "std", "async_executor", "multi_threaded",
    "bevy_winit", "bevy_window", "bevy_render",
    "bevy_core_pipeline", "bevy_sprite", "bevy_asset", "bevy_log", "png", "x11",
] }
```

## Further reference

- `references/scheduling.md` — ordering, run conditions, Update vs FixedUpdate
- `references/migration-0-19.md` — how to recognise and fix stale Bevy snippets
