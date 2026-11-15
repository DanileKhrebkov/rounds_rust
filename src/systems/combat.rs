//! Combat systems: shooting, projectile motion, hit detection, block reflect,
//! knockback, homing, bounce.

use bevy::prelude::*;

use crate::components::cards::StatBlock;
use crate::components::player::{
    Ammo, BlockState, Facing, FireCooldown, Health, KnockbackState, PlayerTag, Velocity,
};
use crate::components::projectile::{
    BouncesRemaining, Damage, Homing, Lifetime, ProjectileOwner, ProjectileTag,
};
use crate::resources::Config;

pub fn shoot_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<Config>,
    mut q: Query<(
        Entity,
        &Transform,
        &Facing,
        &mut Ammo,
        &StatBlock,
        &mut FireCooldown,
        &PlayerTag,
    )>,
) {
    if !keys.just_pressed(KeyCode::KeyJ) {
        return;
    }

    for (entity, tf, facing, mut ammo, stats, mut cooldown, tag) in &mut q {
        if tag.index != 0 {
            continue;
        }
        if ammo.current == 0 {
            continue;
        }
        if !cooldown.is_ready() {
            continue;
        }

        ammo.current -= 1;

        let dir = Vec2::new(facing.0, 0.0);
        let spawn = tf.translation.truncate() + dir * 30.0;
        let actual_damage = cfg.projectile_damage * stats.damage_mult;
        let speed = cfg.projectile_speed * stats.bullet_speed_mult;
        let cooldown_secs = cfg.base_fire_cooldown * stats.fire_cooldown_mult;

        cooldown.start(cooldown_secs);

        let mut bullet = commands.spawn((
            Sprite::from_color(
                Color::srgb(1.0, 0.9, 0.2),
                Vec2::splat(cfg.projectile_radius * 2.0),
            ),
            Transform::from_xyz(spawn.x, spawn.y, 0.0),
            ProjectileTag,
            ProjectileOwner(entity),
            Damage(actual_damage),
            Velocity {
                x: dir.x * speed,
                y: dir.y * speed,
            },
            Lifetime(Timer::from_seconds(
                cfg.projectile_lifetime,
                TimerMode::Once,
            )),
            BouncesRemaining(stats.extra_bounces),
        ));

        if stats.homing {
            bullet.insert(Homing);
        }
    }
}

pub fn projectile_move_system(
    time: Res<Time>,
    cfg: Res<Config>,
    mut bullets: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut BouncesRemaining,
            Option<&Homing>,
            &ProjectileOwner,
        ),
        With<ProjectileTag>,
    >,
    players: Query<(Entity, &Transform), (With<PlayerTag>, Without<ProjectileTag>)>,
) {
    // ... тело без изменений
    let dt = time.delta_secs();

    for (mut tf, mut vel, mut bounces, homing, owner) in &mut bullets {
        // --- Homing: steer toward nearest other player ---
        if homing.is_some() {
            let b_pos = tf.translation.truncate();
            let mut best: Option<(f32, Vec2)> = None;
            for (p_ent, p_tf) in &players {
                if p_ent == owner.0 {
                    continue;
                }
                let p_pos = p_tf.translation.truncate();
                let dist = b_pos.distance_squared(p_pos);
                if best.map_or(true, |(d, _)| dist < d) {
                    best = Some((dist, p_pos));
                }
            }

            if let Some((_, target)) = best {
                let desired = (target - b_pos).normalize_or_zero();
                let current = vel_to_vec2(&vel).normalize_or_zero();
                if current.length_squared() > 0.0 {
                    let speed = vel_to_vec2(&vel).length();
                    let angle_current = current.y.atan2(current.x);
                    let angle_desired = desired.y.atan2(desired.x);
                    let mut delta = angle_desired - angle_current;
                    // Wrap to [-π, π].
                    while delta > std::f32::consts::PI {
                        delta -= 2.0 * std::f32::consts::PI;
                    }
                    while delta < -std::f32::consts::PI {
                        delta += 2.0 * std::f32::consts::PI;
                    }
                    let max_turn = cfg.homing_turn_rate * dt;
                    let turn = delta.clamp(-max_turn, max_turn);
                    let new_angle = angle_current + turn;
                    let new_dir = Vec2::new(new_angle.cos(), new_angle.sin());
                    let new_vel = new_dir * speed;
                    vel.x = new_vel.x;
                    vel.y = new_vel.y;
                }
            }
        }

        // --- Integrate ---
        tf.translation.x += vel.x * dt;
        tf.translation.y += vel.y * dt;

        // --- Bounce off arena bounds ---
        let mut bounced = false;
        if tf.translation.x < cfg.arena_x_min {
            tf.translation.x = cfg.arena_x_min;
            vel.x = vel.x.abs() * cfg.bounce_speed_retention;
            bounced = true;
        } else if tf.translation.x > cfg.arena_x_max {
            tf.translation.x = cfg.arena_x_max;
            vel.x = -vel.x.abs() * cfg.bounce_speed_retention;
            bounced = true;
        }
        if tf.translation.y < cfg.arena_y_min {
            tf.translation.y = cfg.arena_y_min;
            vel.y = vel.y.abs() * cfg.bounce_speed_retention;
            bounced = true;
        } else if tf.translation.y > cfg.arena_y_max {
            tf.translation.y = cfg.arena_y_max;
            vel.y = -vel.y.abs() * cfg.bounce_speed_retention;
            bounced = true;
        }

        if bounced {
            if bounces.0 > 0 {
                bounces.0 -= 1;
            } else {
                // Out of bounces: let it fly straight; lifetime will clean up.
                // Alternatively, despawn here. We keep it simple.
            }
        }
    }
}

fn vel_to_vec2(v: &Velocity) -> Vec2 {
    Vec2::new(v.x, v.y)
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

/// Collision unchanged from the knockback step, but now also respects
/// `BouncesRemaining` — a bullet that has bounces left and hits a blocking
/// player **reflects** (already handled) and does not consume a bounce
/// (bounces are for walls only).
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
                BouncesRemaining(0),
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
        let vel = world.get::<Velocity>(target).unwrap();
        assert!((vel.x - cfg.projectile_knockback).abs() < 1e-6);

        let kb = world.get::<KnockbackState>(target).unwrap();
        assert!(kb.is_active());
    }

    #[test]
    fn blocked_bullet_reflects_and_does_not_apply_knockback() {
        let (mut world, target, bullet) = world_with_bullet(true, 20.0);
        world.run_system_once(collision_system).unwrap();

        assert_eq!(hp(&mut world, target), 100.0);

        let entity = world.get_entity(bullet).expect("bullet survives");
        let owner = entity.get::<ProjectileOwner>().unwrap();
        assert_eq!(owner.0, target);

        let vel = entity.get::<Velocity>().unwrap();
        assert!(vel.x < 0.0);
        assert!((vel.x.abs() - 15.0 * 1.2).abs() < 1e-6);

        let p_vel = world.get::<Velocity>(target).unwrap();
        assert_eq!(p_vel.x, 0.0);
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
            BouncesRemaining(0),
        ));

        world.run_system_once(collision_system).unwrap();
        assert_eq!(hp(&mut world, shooter), 100.0);
    }

    #[test]
    fn bounce_reflects_velocity_at_arena_edge() {
        let mut world = World::new();
        let cfg = Config::default();
        world.insert_resource(cfg.clone());

        let shooter = world
            .spawn((Transform::from_xyz(-300.0, 0.0, 0.0), PlayerTag { index: 0 }))
            .id();

        let bullet = world
            .spawn((
                Transform::from_xyz(cfg.arena_x_max - 1.0, 0.0, 0.0),
                Velocity { x: 100.0, y: 0.0 },
                ProjectileTag,
                ProjectileOwner(shooter),
                BouncesRemaining(2),
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.1));
        world.insert_resource(time);

        world.run_system_once(projectile_move_system).unwrap();

        let vel = world.get::<Velocity>(bullet).unwrap();
        assert!(vel.x < 0.0, "bullet must reflect off the right wall");
        let bounces = world.get::<BouncesRemaining>(bullet).unwrap();
        assert_eq!(bounces.0, 1, "one bounce consumed");
    }

    #[test]
    fn homing_bullet_turns_toward_target() {
        let mut world = World::new();
        let cfg = Config::default();
        world.insert_resource(cfg.clone());

        let shooter = world
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), PlayerTag { index: 0 }))
            .id();

        world.spawn((
            Transform::from_xyz(0.0, 100.0, 0.0),
            PlayerTag { index: 1 },
        ));

        let bullet = world
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity { x: 15.0, y: 0.0 },
                ProjectileTag,
                ProjectileOwner(shooter),
                BouncesRemaining(0),
                Homing,
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.1));
        world.insert_resource(time);

        world.run_system_once(projectile_move_system).unwrap();

        let vel = world.get::<Velocity>(bullet).unwrap();
        assert!(vel.y > 0.0, "homing must steer upward toward target");
    }
}