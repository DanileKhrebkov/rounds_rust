//! Round flow state + card catalog.
//!
//! Steps 11–13 of the roadmap. `RoundPhase` gates every system; `RoundState`
//! records who lost the current round; `ScoreBoard` tracks wins; `DraftState`
//! holds the pending draft; `CardPool` is the static catalog.

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
                    modifiers: vec![],
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
                    modifiers: vec![],
                    behavior: Some(BehaviorTag::Homing),
                },
                CardDef {
                    id: CardId::Bounce,
                    name: "Bounce",
                    modifiers: vec![],
                    behavior: Some(BehaviorTag::Bounce),
                },
            ],
        }
    }
}