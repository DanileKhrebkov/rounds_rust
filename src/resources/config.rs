use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct Config {
    pub gravity: f32,
    pub jump_impulse: f32,
    pub ground_y: f32,
    pub player_radius: f32,
    pub projectile_speed: f32,
    pub projectile_damage: f32,
    pub projectile_radius: f32,
    pub projectile_lifetime: f32,

    pub base_ammo: u32,
    pub base_health: f32,

    pub block_duration: f32,
    pub block_cooldown: f32,
    pub block_reflect_speed_mult: f32,

    pub max_jumps: u8,

    pub projectile_knockback: f32,
    pub knockback_duration: f32,

    /// Delay between shots, in seconds, before card modifiers.
    /// ROUNDS has a noticeable attack cooldown; this is what `FastReload`
    /// and `Homing`'s `-50% ATKSPD` actually modify.
    pub base_fire_cooldown: f32,

    /// Arena bounds for bullet bounces. Bullets reflect off these planes.
    /// Keep `y_min` at or slightly below `ground_y` so the ground is a wall.
    pub arena_x_min: f32,
    pub arena_x_max: f32,
    pub arena_y_min: f32,
    pub arena_y_max: f32,

    /// How strongly homing bullets steer toward their target, in radians
    /// per second. `0.0` disables steering even if `Homing` is present.
    pub homing_turn_rate: f32,

    /// Bounce speed retention. `1.0` keeps full speed; `0.5` halves it.
    pub bounce_speed_retention: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gravity: -30.0,
            jump_impulse: 12.0,
            ground_y: -200.0,
            player_radius: 20.0,
            projectile_speed: 15.0,
            projectile_damage: 10.0,
            projectile_radius: 5.0,
            projectile_lifetime: 2.0,
            base_ammo: 6,
            base_health: 100.0,
            block_duration: 0.5,
            block_cooldown: 1.0,
            block_reflect_speed_mult: 1.2,
            max_jumps: 2,
            projectile_knockback: 40.0,
            knockback_duration: 0.2,
            base_fire_cooldown: 0.4,
            arena_x_min: -400.0,
            arena_x_max: 400.0,
            arena_y_min: -220.0,
            arena_y_max: 300.0,
            homing_turn_rate: 4.0, // rad/s
            bounce_speed_retention: 0.8,
        }
    }
}