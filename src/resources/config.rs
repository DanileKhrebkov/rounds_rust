use bevy::prelude::*;

/// Tunables for the whole prototype.
///
/// Bevy 0.19: `Resource` is a subtrait of `Component`, so derive **only**
/// `Resource` — deriving both is a compile error.
#[derive(Resource)]
pub struct Config {
    /// World units / s². Negative = downward.
    pub gravity: f32,
    /// Instant vertical velocity applied on jump (world units / s).
    pub jump_impulse: f32,
    /// Ground plane: players rest with their centre at this y.
    pub ground_y: f32,
    pub player_radius: f32,
    /// Projectile speed in world units / s.
    pub projectile_speed: f32,
    pub projectile_damage: f32,
    pub projectile_radius: f32,
    /// Seconds before a projectile despawns.
    pub projectile_lifetime: f32,
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
        }
    }
}
