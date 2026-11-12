//! Input systems for the white-room prototype.
//!
//! - Step 4:  `input_system` writes `Velocity.x` for player 0 (`A` / `D`).
//!            **Skipped** while `KnockbackState` is active.
//! - Step 5+ `jump_system` writes `Velocity.y` on `Space` as long as the player
//!   still has jumps left (`JumpState`).
//! - Step 6:  `facing_system` mirrors `Facing` from the sign of `Velocity.x`.
//! - Step 10+ `block_system` manages the timed block window and its cooldown.

use bevy::prelude::*;

use crate::components::player::{
    BlockState, Facing, JumpState, KnockbackState, MoveSpeed, OnGround, PlayerTag, Velocity,
};
use crate::resources::Config;

pub fn input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Velocity, &KnockbackState, &PlayerTag, &MoveSpeed)>,
) {
    for (mut vel, knockback, tag, speed) in &mut q {
        if tag.index != 0 {
            continue;
        }

        // No control during knockback — the hit dictates horizontal velocity.
        if knockback.is_active() {
            continue;
        }

        let mut dir = 0.0;
        if keys.pressed(KeyCode::KeyA) {
            dir -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            dir += 1.0;
        }

        vel.x = dir * speed.0;
    }
}

pub fn jump_system(
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<Config>,
    mut q: Query<(&mut Velocity, &mut JumpState, &PlayerTag)>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    for (mut vel, mut jump, tag) in &mut q {
        if tag.index != 0 {
            continue;
        }
        if jump.jumps_remaining == 0 {
            continue;
        }

        jump.jumps_remaining -= 1;
        vel.y = cfg.jump_impulse;
    }
}

pub fn facing_system(mut q: Query<(&Velocity, &mut Facing, &PlayerTag)>) {
    for (vel, mut facing, tag) in &mut q {
        if tag.index != 0 {
            continue;
        }

        if vel.x > 0.1 {
            facing.0 = 1.0;
        } else if vel.x < -0.1 {
            facing.0 = -1.0;
        }
    }
}

pub fn block_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<Config>,
    mut q: Query<(&mut BlockState, &PlayerTag)>,
) {
    let dt = time.delta_secs();

    for (mut block, tag) in &mut q {
        if tag.index != 0 {
            continue;
        }

        if block.cooldown_remaining > 0.0 {
            block.cooldown_remaining = (block.cooldown_remaining - dt).max(0.0);
        }

        if block.active_remaining > 0.0 {
            block.active_remaining = (block.active_remaining - dt).max(0.0);
            if block.active_remaining <= 0.0 {
                block.active = false;
                block.cooldown_remaining = cfg.block_cooldown;
            }
        }

        if keys.just_pressed(KeyCode::KeyK)
            && !block.active
            && block.cooldown_remaining <= 0.0
        {
            block.active = true;
            block.active_remaining = cfg.block_duration;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::ButtonInput;
    use std::time::Duration;

    const DT: Duration = Duration::from_nanos(16_666_667);

    fn test_world() -> World {
        let mut world = World::new();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.insert_resource(Config::default());
        let mut time = Time::<()>::default();
        time.advance_by(DT);
        world.insert_resource(time);

        for index in 0..2 {
            world.spawn((
                Velocity { x: 0.0, y: 0.0 },
                PlayerTag { index },
                MoveSpeed(8.0),
                OnGround(true),
                Facing(if index == 0 { 1.0 } else { -1.0 }),
                BlockState::default(),
                JumpState::new(2),
                KnockbackState::default(),
            ));
        }

        world
    }

    fn step(world: &mut World) {
        let mut time = Time::<()>::default();
        time.advance_by(DT);
        world.insert_resource(time);
    }

    fn press(world: &mut World, key: KeyCode) {
        world.resource_mut::<ButtonInput<KeyCode>>().press(key);
    }

    fn release(world: &mut World, key: KeyCode) {
        world.resource_mut::<ButtonInput<KeyCode>>().release(key);
    }

    fn velocities(world: &mut World) -> [f32; 2] {
        let mut q = world.query::<(&Velocity, &PlayerTag)>();
        let mut out = [0.0; 2];
        for (vel, tag) in q.iter(world) {
            out[tag.index] = vel.x;
        }
        out
    }

    fn vel_y(world: &mut World, index: usize) -> f32 {
        let mut q = world.query::<(&Velocity, &PlayerTag)>();
        q.iter(world)
            .find(|(_, tag)| tag.index == index)
            .map(|(v, _)| v.y)
            .unwrap()
    }

    fn jumps_remaining(world: &mut World, index: usize) -> u8 {
        let mut q = world.query::<(&JumpState, &PlayerTag)>();
        q.iter(world)
            .find(|(_, tag)| tag.index == index)
            .map(|(j, _)| j.jumps_remaining)
            .unwrap()
    }

    fn facing(world: &mut World, index: usize) -> f32 {
        let mut q = world.query::<(&Facing, &PlayerTag)>();
        q.iter(world)
            .find(|(_, tag)| tag.index == index)
            .map(|(f, _)| f.0)
            .unwrap()
    }

    fn block(world: &mut World) -> BlockState {
        let mut q = world.query::<(&BlockState, &PlayerTag)>();
        q.iter(world)
            .find(|(_, tag)| tag.index == 0)
            .map(|(b, _)| BlockState {
                active: b.active,
                active_remaining: b.active_remaining,
                cooldown_remaining: b.cooldown_remaining,
            })
            .unwrap()
    }

    // --- Step 4: horizontal input ---

    #[test]
    fn no_keys_means_zero_horizontal_velocity() {
        let mut world = test_world();
        world.run_system_once(input_system).unwrap();
        assert_eq!(velocities(&mut world), [0.0, 0.0]);
    }

    #[test]
    fn d_moves_player_0_right() {
        let mut world = test_world();
        press(&mut world, KeyCode::KeyD);
        world.run_system_once(input_system).unwrap();
        assert_eq!(velocities(&mut world), [8.0, 0.0]);
    }

    #[test]
    fn a_moves_player_0_left() {
        let mut world = test_world();
        press(&mut world, KeyCode::KeyA);
        world.run_system_once(input_system).unwrap();
        assert_eq!(velocities(&mut world), [-8.0, 0.0]);
    }

    #[test]
    fn both_keys_cancel_out() {
        let mut world = test_world();
        press(&mut world, KeyCode::KeyA);
        press(&mut world, KeyCode::KeyD);
        world.run_system_once(input_system).unwrap();
        assert_eq!(velocities(&mut world), [0.0, 0.0]);
    }

    #[test]
    fn player_1_ignores_horizontal_input() {
        let mut world = test_world();
        press(&mut world, KeyCode::KeyD);
        world.run_system_once(input_system).unwrap();
        assert_eq!(velocities(&mut world)[1], 0.0);
    }

    #[test]
    fn input_ignored_during_active_knockback() {
        let mut world = test_world();
        // Mark player 0 as knocked back.
        {
            let mut q = world.query::<(&mut KnockbackState, &PlayerTag)>();
            for (mut kb, tag) in q.iter_mut(&mut world) {
                if tag.index == 0 {
                    kb.start(0.2);
                }
            }
        }
        press(&mut world, KeyCode::KeyD);
        world.run_system_once(input_system).unwrap();
        assert_eq!(
            velocities(&mut world)[0],
            0.0,
            "input must be ignored while knockback is active"
        );
    }

    // --- Step 5+: jump / double jump ---

    #[test]
    fn first_jump_consumes_one_charge() {
        let mut world = test_world();
        press(&mut world, KeyCode::Space);
        world.run_system_once(jump_system).unwrap();

        assert_eq!(vel_y(&mut world, 0), 12.0);
        assert_eq!(jumps_remaining(&mut world, 0), 1);
        assert_eq!(jumps_remaining(&mut world, 1), 2);
    }

    #[test]
    fn second_jump_works_in_air() {
        let mut world = test_world();
        press(&mut world, KeyCode::Space);
        world.run_system_once(jump_system).unwrap();

        release(&mut world, KeyCode::Space);
        press(&mut world, KeyCode::Space);
        world.run_system_once(jump_system).unwrap();

        assert_eq!(vel_y(&mut world, 0), 12.0);
        assert_eq!(jumps_remaining(&mut world, 0), 0);
    }

    #[test]
    fn third_jump_is_rejected() {
        let mut world = test_world();

        press(&mut world, KeyCode::Space);
        world.run_system_once(jump_system).unwrap();

        release(&mut world, KeyCode::Space);
        press(&mut world, KeyCode::Space);
        world.run_system_once(jump_system).unwrap();

        release(&mut world, KeyCode::Space);
        press(&mut world, KeyCode::Space);
        world.run_system_once(jump_system).unwrap();

        assert_eq!(vel_y(&mut world, 0), 12.0);
        assert_eq!(jumps_remaining(&mut world, 0), 0);
    }

    #[test]
    fn falling_without_jumping_preserves_both_charges() {
        let mut world = test_world();
        assert_eq!(jumps_remaining(&mut world, 0), 2);
    }

    // --- Step 6: facing ---

    #[test]
    fn facing_flips_with_movement() {
        let mut world = test_world();

        press(&mut world, KeyCode::KeyA);
        world.run_system_once(input_system).unwrap();
        world.run_system_once(facing_system).unwrap();
        assert_eq!(facing(&mut world, 0), -1.0);

        release(&mut world, KeyCode::KeyA);
        press(&mut world, KeyCode::KeyD);
        world.run_system_once(input_system).unwrap();
        world.run_system_once(facing_system).unwrap();
        assert_eq!(facing(&mut world, 0), 1.0);
    }

    #[test]
    fn facing_holds_in_dead_zone() {
        let mut world = test_world();
        world.run_system_once(facing_system).unwrap();
        assert_eq!(facing(&mut world, 0), 1.0);
    }

    // --- Step 10+: timed block ---

    #[test]
    fn k_starts_a_timed_block() {
        let mut world = test_world();
        press(&mut world, KeyCode::KeyK);
        world.run_system_once(block_system).unwrap();

        let b = block(&mut world);
        assert!(b.active);
        assert!((b.active_remaining - 0.5).abs() < 1e-6);
    }

    #[test]
    fn block_expires_and_enters_cooldown() {
        let mut world = test_world();
        press(&mut world, KeyCode::KeyK);
        world.run_system_once(block_system).unwrap();

        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.5));
        world.insert_resource(time);
        world.run_system_once(block_system).unwrap();

        let b = block(&mut world);
        assert!(!b.active);
        assert!((b.cooldown_remaining - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cannot_block_during_cooldown() {
        let mut world = test_world();
        {
            let mut q = world.query::<(&mut BlockState, &PlayerTag)>();
            for (mut b, tag) in q.iter_mut(&mut world) {
                if tag.index == 0 {
                    b.active_remaining = 0.0;
                    b.cooldown_remaining = 1.0;
                }
            }
        }

        press(&mut world, KeyCode::KeyK);
        world.run_system_once(block_system).unwrap();

        let b = block(&mut world);
        assert!(!b.active);
        assert!(b.cooldown_remaining < 1.0);
    }

    #[test]
    fn holding_k_does_not_extend_block() {
        let mut world = test_world();
        press(&mut world, KeyCode::KeyK);
        world.run_system_once(block_system).unwrap();

        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.6));
        world.insert_resource(time);
        world.run_system_once(block_system).unwrap();

        let b = block(&mut world);
        assert!(!b.active);
    }
}