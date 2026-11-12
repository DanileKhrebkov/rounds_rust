//! Player movement integration.
//!
//! Gravity pulls players down, the ground plane stops them, `OnGround` is
//! derived from ground contact, `JumpState` is refilled on landing, and
//! `KnockbackState` timers are ticked.

use bevy::prelude::*;

use crate::components::player::{
    JumpState, KnockbackState, OnGround, PlayerTag, Velocity,
};
use crate::resources::Config;

pub fn gravity_system(
    time: Res<Time>,
    cfg: Res<Config>,
    mut q: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut OnGround,
            &mut JumpState,
            &mut KnockbackState,
        ),
        With<PlayerTag>,
    >,
) {
    let dt = time.delta_secs();

    for (mut vel, mut tf, mut on_ground, mut jump, mut kb) in &mut q {
        kb.timer.tick(time.delta());

        vel.y += cfg.gravity * dt;
        tf.translation.y += vel.y * dt;
        tf.translation.x += vel.x * dt;

        if tf.translation.y <= cfg.ground_y {
            tf.translation.y = cfg.ground_y;
            vel.y = 0.0;
            on_ground.0 = true;
            jump.jumps_remaining = cfg.max_jumps;
        } else {
            on_ground.0 = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    const DT: Duration = Duration::from_nanos(16_666_667);
    const DT_SECS: f32 = 16_666_667.0e-9;

    fn test_world(spawn_y: f32) -> World {
        let mut world = World::new();
        world.insert_resource(Config::default());
        world.spawn((
            Transform::from_xyz(0.0, spawn_y, 0.0),
            Velocity { x: 0.0, y: 0.0 },
            OnGround(false),
            JumpState::new(2),
            KnockbackState::default(),
            PlayerTag { index: 0 },
        ));
        world
    }

    fn step(world: &mut World) {
        let mut time = Time::<()>::default();
        time.advance_by(DT);
        world.insert_resource(time);
        world.run_system_once(gravity_system).unwrap();
    }

    fn player(world: &mut World) -> (f32, f32, bool, u8) {
        let mut q = world.query::<(&Transform, &Velocity, &OnGround, &JumpState)>();
        let (tf, vel, og, jump) = q.single(world).unwrap();
        (tf.translation.y, vel.y, og.0, jump.jumps_remaining)
    }

    #[test]
    fn falls_and_rests_exactly_on_ground() {
        let cfg = Config::default();
        let mut world = test_world(0.0);

        let mut landed = false;
        for _ in 0..600 {
            step(&mut world);
            if player(&mut world).2 {
                landed = true;
                break;
            }
        }
        assert!(landed);

        let (y, vel_y, on_ground, _) = player(&mut world);
        assert_eq!(y, cfg.ground_y);
        assert_eq!(vel_y, 0.0);
        assert!(on_ground);
    }

    #[test]
    fn never_sinks_below_ground_plane() {
        let cfg = Config::default();
        let mut world = test_world(cfg.ground_y);

        for _ in 0..120 {
            step(&mut world);
            let (y, _, _, _) = player(&mut world);
            assert!(y >= cfg.ground_y);
        }
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
        assert!((x - 8.0 * DT_SECS).abs() < 1e-6);
    }

    #[test]
    fn landing_restores_jumps() {
        let mut world = test_world(0.0);
        {
            let mut q = world.query::<&mut JumpState>();
            q.single_mut(&mut world).unwrap().jumps_remaining = 0;
        }

        for _ in 0..600 {
            step(&mut world);
            if player(&mut world).2 {
                break;
            }
        }

        let (_, _, on_ground, jumps) = player(&mut world);
        assert!(on_ground);
        assert_eq!(jumps, 2);
    }

    #[test]
    fn knockback_timer_ticks_down_and_expires() {
        let mut world = test_world(0.0);
        {
            let mut q = world.query::<&mut KnockbackState>();
            q.single_mut(&mut world).unwrap().start(0.05);
        }

        // After one 1/60 s tick, still active.
        step(&mut world);
        {
            let mut q = world.query::<&KnockbackState>();
            assert!(q.single(&world).unwrap().is_active());
        }

        // After four more ticks (0.05 s total elapsed), finished.
        for _ in 0..4 {
            step(&mut world);
        }
        let mut q = world.query::<&KnockbackState>();
        assert!(!q.single(&world).unwrap().is_active());
    }
}