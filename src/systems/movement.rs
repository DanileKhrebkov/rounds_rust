//! Player movement integration.
//!
//! Step 3 of the white-room plan: gravity pulls players down, the ground plane
//! stops them, and `OnGround` is derived from ground contact so Step 5 can
//! gate jumps on it.

use bevy::prelude::*;
use std::time::Duration;

use crate::components::player::{OnGround, PlayerTag, Velocity};
use crate::resources::Config;

/// Integrates gravity into [`Velocity`] and the transform, clamps players to
/// the ground plane, and derives [`OnGround`].
///
/// Runs after the input systems (see `main.rs` for ordering): horizontal
/// velocity written by `input_system` in Step 4 is applied to `translation.x`
/// here.
///
/// # Preconditions
/// - `cfg.gravity` is the downward acceleration in world units / s² (negative).
/// - `cfg.ground_y` is the ground-plane height; players rest with their centre
///   at this y.
/// - `time.delta_secs()` is the frame delta in seconds (Bevy 0.19: renamed
///   from `delta_seconds()`).
///
/// # Postconditions
/// - `transform.translation.y >= cfg.ground_y` — players never sink below ground.
/// - Resting on the plane: `velocity.y == 0` and `on_ground.0 == true`.
/// - Airborne: `on_ground.0 == false` and `velocity.y` was integrated by `dt`.
pub fn gravity_system(
    time: Res<Time>,
    cfg: Res<Config>,
    mut q: Query<(&mut Velocity, &mut Transform, &mut OnGround), With<PlayerTag>>,
) {
    let dt = time.delta_secs(); // 0.19: delta_seconds() was renamed

    for (mut vel, mut tf, mut on_ground) in &mut q {
        vel.y += cfg.gravity * dt;
        tf.translation.y += vel.y * dt;
        tf.translation.x += vel.x * dt;

        if tf.translation.y <= cfg.ground_y {
            tf.translation.y = cfg.ground_y;
            vel.y = 0.0;
            on_ground.0 = true;
        } else {
            on_ground.0 = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    /// ~1/60 s, fixed. Real-clock `Time` from `MinimalPlugins` would make the
    /// 200-unit fall take billions of ticks, so tests drive the system manually.
    const DT: Duration = Duration::from_nanos(16_666_667);
    const DT_SECS: f32 = 16_666_667.0e-9;

    /// One player at the given height, no plugins.
    fn test_world(spawn_y: f32) -> World {
        let mut world = World::new();
        world.insert_resource(Config::default());
        world.spawn((
            Transform::from_xyz(0.0, spawn_y, 0.0),
            Velocity { x: 0.0, y: 0.0 },
            OnGround(false),
            PlayerTag { index: 0 },
        ));
        world
    }

    fn step(world: &mut World) {
        let mut time = Time::default();
        time.advance_by(DT);
        world.insert_resource(time);
        world.run_system_once(gravity_system).unwrap();
    }

    fn player(world: &mut World) -> (f32, f32, bool) {
        let mut q = world.query::<(&Transform, &Velocity, &OnGround)>();
        let (tf, vel, og) = q.single(world).unwrap();
        (tf.translation.y, vel.y, og.0)
    }

    #[test]
    fn falls_and_rests_exactly_on_ground() {
        let cfg = Config::default();
        let mut world = test_world(0.0); // 200 units above the plane

        let mut landed = false;
        for _ in 0..600 {
            step(&mut world);
            if player(&mut world).2 {
                landed = true;
                break;
            }
        }
        assert!(
            landed,
            "player must land within 600 ticks (10 s of simulated time)"
        );

        let (y, vel_y, on_ground) = player(&mut world);
        assert_eq!(y, cfg.ground_y);
        assert_eq!(vel_y, 0.0);
        assert!(on_ground);
    }

    #[test]
    fn never_sinks_below_ground_plane() {
        let cfg = Config::default();
        let mut world = test_world(cfg.ground_y); // spawned resting on the plane

        for _ in 0..120 {
            step(&mut world);
            let (y, _, _) = player(&mut world);
            assert!(y >= cfg.ground_y, "sank below the ground plane: {y}");
        }

        // A resting player must not drift or accumulate velocity.
        let (y, vel_y, on_ground) = player(&mut world);
        assert_eq!(y, cfg.ground_y);
        assert_eq!(vel_y, 0.0);
        assert!(on_ground);
    }

    #[test]
    fn horizontal_velocity_is_integrated_into_position() {
        let mut world = test_world(0.0);
        {
            let mut q = world.query::<&mut Velocity>();
            q.single_mut(&mut world).unwrap().x = 8.0;
        }

        step(&mut world);

        let mut q = world.query::<&Transform>();
        let x = q.single(&world).unwrap().translation.x;
        assert!((x - 8.0 * DT_SECS).abs() < 1e-6, "x = {x}");
    }
}