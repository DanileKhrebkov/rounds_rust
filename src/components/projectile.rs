use bevy::prelude::*;

/// Placeholder for Step 5 (shooting). Declared now so the module layout from
/// Step 1 is complete and later steps only add behaviour, never new files.
#[derive(Component)]
pub struct Projectile {
    pub owner: usize,
}
