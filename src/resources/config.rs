use bevy::prelude::*;

#[derive(Resource)]
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

    /// Horizontal velocity imparted to the victim on a non-blocked hit.
    /// `0.0` disables knockback.
    pub projectile_knockback: f32,
    /// How long the victim loses horizontal control after a hit, in seconds.
    pub knockback_duration: f32,
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
        }
    }
}