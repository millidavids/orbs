//! Round 3: game-feel snippets (screen shake, hit-stop, easing, squash & stretch).

use bevy::prelude::*;

// ---------- screen shake via decaying trauma ----------
#[derive(Resource, Default)]
pub struct Trauma(pub f32);

#[derive(Component)]
pub struct ShakeCamera {
    pub base: Vec3,
}

pub fn add_trauma(mut trauma: ResMut<Trauma>, amount: f32) {
    trauma.0 = (trauma.0 + amount).min(1.0);
}

pub fn apply_shake(
    time: Res<Time>,
    mut trauma: ResMut<Trauma>,
    mut q: Query<(&mut Transform, &ShakeCamera)>,
) {
    // Trauma decays linearly; offset scales with its square for a punchier curve.
    trauma.0 = (trauma.0 - time.delta_secs() * 1.5).max(0.0);
    let shake = trauma.0 * trauma.0;
    let t = time.elapsed_secs();

    for (mut tf, cam) in &mut q {
        let ox = (t * 37.0).sin() * (t * 11.0).cos();
        let oy = (t * 43.0).cos() * (t * 13.0).sin();
        tf.translation = cam.base + Vec3::new(ox, oy, 0.0) * shake * 12.0;
    }
}

// ---------- hit-stop via virtual time ----------
#[derive(Resource, Default)]
pub struct HitStop(pub Timer);

pub fn start_hit_stop(mut stop: ResMut<HitStop>, mut time: ResMut<Time<Virtual>>) {
    stop.0 = Timer::from_seconds(0.08, TimerMode::Once);
    time.set_relative_speed(0.0);
}

pub fn tick_hit_stop(
    real: Res<Time<Real>>,
    mut stop: ResMut<HitStop>,
    mut time: ResMut<Time<Virtual>>,
) {
    // Tick on the REAL clock — virtual time is what we froze.
    if stop.0.tick(real.delta()).just_finished() {
        time.set_relative_speed(1.0);
    }
}

// ---------- easing ----------
pub fn eased_value(t: f32) -> f32 {
    let curve = EasingCurve::new(0.0f32, 1.0, EaseFunction::CubicOut);
    curve.sample(t).unwrap_or(1.0)
}

#[derive(Component)]
pub struct Tween {
    pub from: Vec3,
    pub to: Vec3,
    pub timer: Timer,
    pub ease: EaseFunction,
}

pub fn drive_tweens(time: Res<Time>, mut q: Query<(&mut Transform, &mut Tween)>) {
    for (mut tf, mut tween) in &mut q {
        tween.timer.tick(time.delta());
        let t = tween.timer.fraction();
        let curve = EasingCurve::new(tween.from, tween.to, tween.ease);
        if let Some(p) = curve.sample(t) {
            tf.translation = p;
        }
    }
}

// ---------- squash & stretch ----------
#[derive(Component)]
pub struct Squash {
    pub timer: Timer,
    pub strength: f32,
}

pub fn apply_squash(time: Res<Time>, mut q: Query<(&mut Transform, &mut Squash)>) {
    for (mut tf, mut squash) in &mut q {
        squash.timer.tick(time.delta());
        // Volume-preserving: squash on Y, stretch on X, decaying to 1.0.
        let k = (1.0 - squash.timer.fraction()) * squash.strength;
        tf.scale = Vec3::new(1.0 + k, 1.0 - k, 1.0);
    }
}

// ---------- knockback ----------
#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

pub fn knockback(mut q: Query<&mut Velocity>, entity: Entity, dir: Vec2, force: f32) {
    if let Ok(mut vel) = q.get_mut(entity) {
        vel.0 += dir.normalize_or_zero() * force;
    }
}

// ---------- damage flash via colour ----------
#[derive(Component)]
pub struct Flash(pub Timer);

pub fn damage_flash(time: Res<Time>, mut q: Query<(&mut Sprite, &mut Flash)>) {
    for (mut sprite, mut flash) in &mut q {
        flash.0.tick(time.delta());
        let k = 1.0 - flash.0.fraction();
        sprite.color = Color::srgb(1.0, 1.0 - k, 1.0 - k);
    }
}
