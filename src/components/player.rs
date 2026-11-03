use bevy::prelude::*;

/// All per-player state components for the white-room prototype.
///
/// Kept as plain data (no behaviour) so Steps 3-6 can add systems that read
/// and write them without touching this file.

#[derive(Component)]
pub struct PlayerTag {
    pub index: usize,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

/// Velocity in **world units per second**.
///
/// Step 3 integrates this as `translation += velocity * dt` with
/// `dt = time.delta_secs()`. The README's `Config` values (`gravity: -30.0`,
/// `jump_impulse: 12.0`, `MoveSpeed(8.0)`) are only sane under a seconds
/// timescale — at a naive per-frame scale, gravity would be -1800 u/s² and the
/// players would fall off-screen in ~0.5 s. See PR description.
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

/// True while the player is resting on the ground plane. Gates jumping in Step 4.
#[derive(Component)]
pub struct OnGround(pub bool);

/// Horizontal movement speed in world units per second.
#[derive(Component)]
pub struct MoveSpeed(pub f32);

/// Blocking stance. Read by the hit-detection system in Step 6.
#[derive(Component)]
pub struct BlockState {
    pub active: bool,
}

#[derive(Component)]
pub struct Ammo {
    pub current: u32,
    pub max: u32,
}

/// `-1.0` = facing left, `1.0` = facing right. Used to orient shots in Step 5.
#[derive(Component)]
pub struct Facing(pub f32);
