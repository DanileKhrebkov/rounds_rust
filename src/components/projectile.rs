use bevy::prelude::*;

#[derive(Component)]
pub struct ProjectileTag;

#[derive(Component)]
pub struct ProjectileOwner(pub Entity);

#[derive(Component)]
pub struct Damage(pub f32);

#[derive(Component)]
pub struct Lifetime(pub Timer);

/// How many more wall bounces this bullet can perform.
#[derive(Component)]
pub struct BouncesRemaining(pub u32);

/// Marker: this bullet steers toward its target each frame.
#[derive(Component)]
pub struct Homing;