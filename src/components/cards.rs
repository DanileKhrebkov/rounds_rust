//! Card data layer: catalog types + per-player inventory/stat components.
//!
//! Pure data, no behaviour. Systems live in `systems::draft` and
//! `systems::combat`.

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

/// Cards owned by one player. Appended by `apply_card_system`, read by
/// `recalculate_stats_system`.
#[derive(Component, Default)]
pub struct CardInventory {
    pub cards: Vec<CardId>,
}

/// Aggregated stat modifiers derived from `CardInventory`. Recomputed every
/// frame by `recalculate_stats_system`; read by `shoot_system`.
#[derive(Component)]
pub struct StatBlock {
    pub damage_mult: f32,
    pub ammo_add: i32,
    pub health_add: f32,
}

impl Default for StatBlock {
    fn default() -> Self {
        Self {
            damage_mult: 1.0,
            ammo_add: 0,
            health_add: 0.0,
        }
    }
}