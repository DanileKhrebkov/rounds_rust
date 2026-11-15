use bevy::prelude::*;

use crate::components::cards::*;

#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug)]
pub enum RoundPhase {
    Combat,
    Drafting,
    Cleanup,
    MatchOver,
}

#[derive(Resource)]
pub struct RoundState {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub loser: Option<usize>,
}

impl Default for RoundState {
    fn default() -> Self {
        Self {
            phase: RoundPhase::Combat,
            round_number: 1,
            loser: None,
        }
    }
}

#[derive(Resource)]
pub struct ScoreBoard {
    pub wins: [u32; 2],
    pub needed: u32,
}

impl Default for ScoreBoard {
    fn default() -> Self {
        Self {
            wins: [0, 0],
            needed: 5,
        }
    }
}

#[derive(Resource, Default)]
pub struct DraftState {
    pub choices: Vec<CardId>,
    pub chosen: Option<CardId>,
}

#[derive(Resource)]
pub struct CardPool {
    pub defs: Vec<CardDef>,
}

impl Default for CardPool {
    fn default() -> Self {
        Self {
            defs: vec![
                CardDef {
                    id: CardId::BigBullet,
                    name: "Big Bullet",
                    modifiers: vec![StatModifier::Mul(StatType::Damage, 1.5)],
                    behavior: None,
                },
                CardDef {
                    id: CardId::MoreAmmo,
                    name: "More Ammo",
                    modifiers: vec![StatModifier::Add(StatType::Ammo, 3.0)],
                    behavior: None,
                },
                CardDef {
                    id: CardId::FastReload,
                    name: "Fast Reload",
                    // -70% reload time → cooldown × 0.3.
                    // FireRate is an additive adjustment to the cooldown
                    // multiplier (baseline 1.0), so the value is -0.7.
                    modifiers: vec![StatModifier::Add(StatType::FireRate, -0.7)],
                    behavior: None,
                },
                CardDef {
                    id: CardId::ExtraHealth,
                    name: "Extra Health",
                    modifiers: vec![StatModifier::Add(StatType::Health, 25.0)],
                    behavior: None,
                },
                CardDef {
                    id: CardId::Homing,
                    name: "Homing",
                    // -25% DMG, -50% ATKSPD → cooldown × 2.0.
                    // -50% ATKSPD means cooldown doubles, so FireRate
                    // adjustment is +1.0 (1.0 + 1.0 = 2.0).
                    modifiers: vec![
                        StatModifier::Mul(StatType::Damage, 0.75),
                        StatModifier::Add(StatType::FireRate, 1.0),
                    ],
                    behavior: Some(BehaviorTag::Homing),
                },
                CardDef {
                    id: CardId::Bounce,
                    name: "Bouncy",
                    // +2 bounces, +25% DMG.
                    modifiers: vec![
                        StatModifier::Add(StatType::BulletBounces, 2.0),
                        StatModifier::Mul(StatType::Damage, 1.25),
                    ],
                    behavior: Some(BehaviorTag::Bounce),
                },
            ],
        }
    }
}