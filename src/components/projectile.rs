//! Projectile components.
//!
//! Step 7 of the roadmap: shooting spawns an entity carrying this full set.
//! Behaviours (movement, lifetime, hit detection) live in `systems::combat`.

use bevy::prelude::*;

/// Marker: entity is a live bullet.
#[derive(Component)]
pub struct ProjectileTag;

/// The entity that fired this bullet. Used to skip self-hits in Step 9.
#[derive(Component)]
pub struct ProjectileOwner(pub Entity);

/// Damage dealt on hit, before any `StatBlock` multipliers from Step 15.
#[derive(Component)]
pub struct Damage(pub f32);

/// Despawns the bullet when the timer finishes.
#[derive(Component)]
pub struct Lifetime(pub Timer);