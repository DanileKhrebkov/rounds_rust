//! Combat systems: shooting, projectile motion, hit detection, block reflect,
//! knockback.

use bevy::prelude::*;

use crate::components::cards::StatBlock;
use crate::components::player::{
    Ammo, BlockState, Facing, Health, KnockbackState, PlayerTag, Velocity,
};
use crate::components::projectile::{
    Damage, Lifetime, ProjectileOwner, ProjectileTag,
};
use crate::resources::Config;

pub fn shoot_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<Config>,
    mut q: Query<(Entity, &Transform, &Facing, &mut Ammo, &StatBlock, &PlayerTag)>,
) {
    if !keys.just_pressed(KeyCode::KeyJ) {
        return;
    }

    for (entity, tf, facing, mut ammo, stats, tag) in &mut q {
        if tag.index != 0 {
            continue;
        }
        if ammo.current == 0 {
            continue;
        }

        ammo.current -= 1;

        let dir = Vec2::new(facing.0, 0.0);
        let spawn = tf.translation.truncate() + dir * 30.0;
        let actual_damage = cfg.projectile_damage * stats.damage_mult;

        commands.spawn((
            Sprite::from_color(
                Color::srgb(1.0, 0.9, 0.2),
                Vec2::splat(cfg.projectile_radius * 2.0),
            ),
            Transform::from_xyz(spawn.x, spawn.y, 0.0),
            ProjectileTag,
            ProjectileOwner(entity),
            Damage(actual_damage),
            Velocity {
                x: dir.x * cfg.projectile_speed,
                y: 0.0,
            },
            Lifetime(Timer::from_seconds(
                cfg.projectile_lifetime,
                TimerMode::Once,
            )),
        ));
    }
}

pub fn projectile_move_system(
    time: Res<Time>,
    mut q: Query<(&Velocity, &mut Transform), With<ProjectileTag>>,
) {
    let dt = time.delta_secs();
    for (vel, mut tf) in &mut q {
        tf.translation.x += vel.x * dt;
        tf.translation.y += vel.y * dt;
    }
}

pub fn projectile_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Lifetime), With<ProjectileTag>>,
) {
    for (entity, mut life) in &mut q {
        life.0.tick(time.delta());
        if life.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Circle-circle hit detection with **block reflection** and **knockback**.
///
/// Non-blocked hit:
/// - `Health.current -= Damage.0`.
/// - `KnockbackState::start(cfg.knockback_duration)` — `input_system` will
///   skip `Velocity.x` overrides while the timer runs.
/// - `Velocity.x/y` is set to `bullet_direction * cfg.projectile_knockback`.
/// - Bullet despawns.
///
/// Blocked hit:
/// - Owner reassigned to the blocker; velocity inverted and scaled.
/// - Bullet nudged out of the blocker's hitbox.
/// - **No knockback** — ROUNDS blocks absorb energy.
///
/// Self-hits are always skipped via `p_ent == owner.0`.
pub fn collision_system(
    mut commands: Commands,
    cfg: Res<Config>,
    mut bullets: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut ProjectileOwner,
            &Damage,
        ),
        With<ProjectileTag>,
    >,
    mut players: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            &BlockState,
            &mut Velocity,
            &mut KnockbackState,
        ),
        (With<PlayerTag>, Without<ProjectileTag>),
    >,
) {
    let hit_radius = cfg.player_radius + cfg.projectile_radius;

    for (_bullet_entity, mut b_tf, mut b_vel, mut owner, dmg) in &mut bullets {
        let b_pos = b_tf.translation.truncate();
        let bullet_dir = Vec2::new(b_vel.x, b_vel.y).normalize_or_zero();

        for (p_ent, p_tf, mut hp, block, mut p_vel, mut kb) in &mut players {
            if p_ent == owner.0 {
                continue;
            }

            let p_pos = p_tf.translation.truncate();
            if b_pos.distance(p_pos) < hit_radius {
                if block.active {
                    owner.0 = p_ent;
                    b_vel.x = -b_vel.x * cfg.block_reflect_speed_mult;
                    b_vel.y = -b_vel.y * cfg.block_reflect_speed_mult;

                    let away = (b_pos - p_pos).normalize_or_zero();
                    let nudge = (cfg.player_radius + cfg.projectile_radius) * 0.5;
                    b_tf.translation.x += away.x * nudge;
                    b_tf.translation.y += away.y * nudge;
                } else {
                    hp.current -= dmg.0;

                    p_vel.x = bullet_dir.x * cfg.projectile_knockback;
                    p_vel.y = bullet_dir.y * cfg.projectile_knockback;
                    kb.start(cfg.knockback_duration);

                    commands.entity(_bullet_entity).despawn();
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn world_with_bullet(block_active: bool, hit_offset: f32) -> (World, Entity, Entity) {
        let mut world = World::new();
        world.insert_resource(Config::default());

        let shooter = world
            .spawn((
                Transform::from_xyz(-200.0, 0.0, 0.0),
                PlayerTag { index: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                BlockState::default(),
                Velocity { x: 0.0, y: 0.0 },
                KnockbackState::default(),
            ))
            .id();

        let target = world
            .spawn((
                Transform::from_xyz(200.0, 0.0, 0.0),
                PlayerTag { index: 1 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                BlockState {
                    active: block_active,
                    active_remaining: if block_active { 0.5 } else { 0.0 },
                    cooldown_remaining: 0.0,
                },
                Velocity { x: 0.0, y: 0.0 },
                KnockbackState::default(),
            ))
            .id();

        let bullet = world
            .spawn((
                Transform::from_xyz(200.0 - hit_offset, 0.0, 0.0),
                ProjectileTag,
                ProjectileOwner(shooter),
                Damage(10.0),
                Velocity { x: 15.0, y: 0.0 },
            ))
            .id();

        (world, target, bullet)
    }

    fn hp(world: &mut World, e: Entity) -> f32 {
        world.get::<Health>(e).unwrap().current
    }

    #[test]
    fn bullet_hits_and_deals_damage() {
        let (mut world, target, _bullet) = world_with_bullet(false, 20.0);
        world.run_system_once(collision_system).unwrap();
        assert_eq!(hp(&mut world, target), 90.0);
    }

    #[test]
    fn non_blocked_hit_applies_knockback() {
        let (mut world, target, _bullet) = world_with_bullet(false, 20.0);
        world.run_system_once(collision_system).unwrap();

        let cfg = Config::default();
        // Bullet was travelling right at 15 u/s → bullet_dir = (1, 0).
        let vel = world.get::<Velocity>(target).unwrap();
        assert!(
            (vel.x - cfg.projectile_knockback).abs() < 1e-6,
            "victim must be pushed right by knockback (got {})",
            vel.x
        );

        let kb = world.get::<KnockbackState>(target).unwrap();
        assert!(kb.is_active(), "knockback timer must be running");
    }

    #[test]
    fn blocked_bullet_reflects_and_does_not_apply_knockback() {
        let (mut world, target, bullet) = world_with_bullet(true, 20.0);
        world.run_system_once(collision_system).unwrap();

        assert_eq!(hp(&mut world, target), 100.0);

        let entity = world
            .get_entity(bullet)
            .expect("bullet must survive a block");
        let owner = entity.get::<ProjectileOwner>().unwrap();
        assert_eq!(owner.0, target);

        let vel = entity.get::<Velocity>().unwrap();
        assert!(vel.x < 0.0);
        assert!((vel.x.abs() - 15.0 * 1.2).abs() < 1e-6);

        // Blocker took no knockback.
        let p_vel = world.get::<Velocity>(target).unwrap();
        assert_eq!(p_vel.x, 0.0, "blocked bullet must not push the blocker");
        let kb = world.get::<KnockbackState>(target).unwrap();
        assert!(!kb.is_active());
    }

    #[test]
    fn reflected_bullet_can_damage_the_original_shooter() {
        let (mut world, _target, bullet) = world_with_bullet(true, 20.0);
        world.run_system_once(collision_system).unwrap();

        {
            let mut tf = world.get_mut::<Transform>(bullet).unwrap();
            tf.translation.x = -200.0;
        }
        world.run_system_once(collision_system).unwrap();

        let shooter_hp = {
            let mut q = world.query::<(&PlayerTag, &Health)>();
            q.iter(&world)
                .find(|(tag, _)| tag.index == 0)
                .map(|(_, hp)| hp.current)
                .unwrap()
        };
        assert_eq!(shooter_hp, 90.0);
    }

    #[test]
    fn shooter_does_not_hit_themselves() {
        let mut world = World::new();
        world.insert_resource(Config::default());

        let shooter = world
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                PlayerTag { index: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                BlockState::default(),
                Velocity { x: 0.0, y: 0.0 },
                KnockbackState::default(),
            ))
            .id();

        world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            ProjectileTag,
            ProjectileOwner(shooter),
            Damage(10.0),
            Velocity { x: 0.0, y: 0.0 },
        ));

        world.run_system_once(collision_system).unwrap();
        assert_eq!(hp(&mut world, shooter), 100.0);
    }
}