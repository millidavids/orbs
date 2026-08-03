//! Round 2: the risky / advanced snippets.

use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

// ---------- conflicting queries: ParamSet ----------
// Two &mut Transform queries in one system panic at startup (B0001) unless
// they are provably disjoint or wrapped in a ParamSet.
pub fn disjoint_by_filter(
    mut players: Query<&mut Transform, With<Player>>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {
    for mut tf in &mut players {
        tf.translation.x += 1.0;
    }
    for mut tf in &mut enemies {
        tf.translation.x -= 1.0;
    }
}

pub fn overlapping_via_paramset(mut set: ParamSet<(Query<&mut Transform>, Query<&Transform>)>) {
    for mut tf in &mut set.p0() {
        tf.translation.y += 1.0;
    }
    for _tf in &set.p1() {}
}

// ---------- Single / Option<Single> ----------
pub fn single_player(player: Single<&Transform, With<Player>>) {
    let _ = player.translation;
}

pub fn maybe_player(player: Option<Single<&Transform, With<Player>>>) {
    if let Some(p) = player {
        let _ = p.translation;
    }
}

// ---------- Local state ----------
pub fn with_local(mut counter: Local<u32>) {
    *counter += 1;
}

// ---------- exclusive world access ----------
pub fn exclusive(world: &mut World) {
    let count = world.query::<&Health>().iter(world).count();
    let _ = count;
}

// ---------- run conditions ----------
pub fn gated() {}

pub fn register_conditions(app: &mut App) {
    app.add_systems(
        Update,
        gated
            .run_if(resource_exists::<Score>)
            .run_if(any_with_component::<Player>),
    );
}

// ---------- 2D setup ----------
pub fn setup_2d(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::splat(32.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

// ---------- gizmos (debug draw) ----------
pub fn debug_draw(mut gizmos: Gizmos) {
    gizmos.circle_2d(Vec2::ZERO, 50.0, Color::WHITE);
    gizmos.line_2d(Vec2::ZERO, Vec2::new(100.0, 100.0), Color::WHITE);
}

// ---------- query lookup by entity ----------
pub fn lookup(q: Query<&Health>, entities: Query<Entity, With<Enemy>>) {
    for e in &entities {
        if let Ok(h) = q.get(e) {
            let _ = h.0;
        }
    }
}

// ---------- commands: insert / remove components ----------
pub fn toggle_components(mut commands: Commands, q: Query<Entity, With<Enemy>>) {
    for e in &q {
        commands.entity(e).insert(Health(10.0)).remove::<Velocity>();
    }
}

// ---------- resource init vs insert ----------
pub fn register_resources(app: &mut App) {
    app.init_resource::<Score>()
        .insert_resource(Config { gravity: -20.0 });
}
