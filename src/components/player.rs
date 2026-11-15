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

/// Ticks down between shots. `0` = ready to fire.
#[derive(Component, Default)]
pub struct FireCooldown {
    pub timer: Timer,
}

impl FireCooldown {
    pub fn is_ready(&self) -> bool {
        self.timer.duration() == Duration::ZERO || self.timer.is_finished()
    }

    pub fn start(&mut self, seconds: f32) {
        self.timer = Timer::from_seconds(seconds, TimerMode::Once);
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        if self.timer.duration() > Duration::ZERO {
            self.timer.tick(delta);
        }
    }
}