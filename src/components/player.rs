use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct PlayerTag {
    pub index: usize,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct OnGround(pub bool);

#[derive(Component)]
pub struct MoveSpeed(pub f32);

#[derive(Component)]
pub struct BlockState {
    pub active: bool,
    pub active_remaining: f32,
    pub cooldown_remaining: f32,
}

impl Default for BlockState {
    fn default() -> Self {
        Self {
            active: false,
            active_remaining: 0.0,
            cooldown_remaining: 0.0,
        }
    }
}

#[derive(Component)]
pub struct Ammo {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct Facing(pub f32);

#[derive(Component)]
pub struct JumpState {
    pub jumps_remaining: u8,
}

impl JumpState {
    pub fn new(max_jumps: u8) -> Self {
        Self {
            jumps_remaining: max_jumps,
        }
    }
}

/// Tracks an active knockback window. While `is_active()` is true,
/// `input_system` does **not** override horizontal velocity — the victim
/// flies with whatever velocity the hit imparted.
///
/// A **zero-duration** timer is treated as "not active" regardless of the
/// internal `finished` flag, because `Timer::default()` in Bevy 0.19 is
/// `duration = 0, finished = false` — i.e. it would otherwise report
/// `is_active() == true` on a freshly spawned player and make `input_system`
/// skip them forever.
#[derive(Component, Default)]
pub struct KnockbackState {
    pub timer: Timer,
}

impl KnockbackState {
    pub fn is_active(&self) -> bool {
        self.timer.duration() > Duration::ZERO && !self.timer.is_finished()
    }

    pub fn start(&mut self, duration: f32) {
        self.timer = Timer::from_seconds(duration, TimerMode::Once);
    }
}