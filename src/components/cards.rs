use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CardId {
    BigBullet,
    MoreAmmo,
    FastReload,
    ExtraHealth,
    Homing,
    Bounce,
}

#[derive(Clone, Copy, Debug)]
pub enum StatType {
    Damage,
    Ammo,
    Health,
    /// Positive = faster fire rate (shorter cooldown), negative = slower.
    /// Value is a **multiplier delta**: `0.3` means `+30%` fire rate.
    FireRate,
    /// Adds whole bounces to bullets. `2.0` = `+2 bounces`.
    BulletBounces,
}

#[derive(Clone, Copy, Debug)]
pub enum StatModifier {
    Add(StatType, f32),
    Mul(StatType, f32),
}

#[derive(Clone, Debug)]
pub struct CardDef {
    pub id: CardId,
    pub name: &'static str,
    pub modifiers: Vec<StatModifier>,
    pub behavior: Option<BehaviorTag>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorTag {
    Homing,
    Bounce,
}

#[derive(Component, Default)]
pub struct CardInventory {
    pub cards: Vec<CardId>,
}

#[derive(Component)]
pub struct StatBlock {
    pub damage_mult: f32,
    pub ammo_add: i32,
    pub health_add: f32,
    /// Multiplier on `base_fire_cooldown`. `<1.0` = faster shooting.
    pub fire_cooldown_mult: f32,
    /// Extra bounces granted to every bullet.
    pub extra_bounces: u32,
    /// Whether bullets steer toward the opponent.
    pub homing: bool,
    /// Multiplier on bullet speed.
    pub bullet_speed_mult: f32,
}

impl Default for StatBlock {
    fn default() -> Self {
        Self {
            damage_mult: 1.0,
            ammo_add: 0,
            health_add: 0.0,
            fire_cooldown_mult: 1.0,
            extra_bounces: 0,
            homing: false,
            bullet_speed_mult: 1.0,
        }
    }
}