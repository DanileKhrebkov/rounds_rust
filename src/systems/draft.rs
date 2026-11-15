//! Draft phase systems.

use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::components::cards::{
    BehaviorTag, CardId, CardInventory, StatBlock, StatModifier, StatType,
};
use crate::components::player::{Ammo, Health, PlayerTag};
use crate::resources::{
    CardPool, Config, DraftState, RoundPhase, RoundState,
};

pub fn draft_offer_system(
    round: Res<RoundState>,
    mut draft: ResMut<DraftState>,
    pool: Res<CardPool>,
) {
    if round.phase != RoundPhase::Drafting || !draft.choices.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    let all_ids: Vec<CardId> = pool.defs.iter().map(|c| c.id).collect();

    draft.choices = all_ids.sample(&mut rng, 3).cloned().collect();

    println!("Draft choices: {:?}", draft.choices);
}

pub fn draft_select_system(
    keys: Res<ButtonInput<KeyCode>>,
    round: Res<RoundState>,
    mut draft: ResMut<DraftState>,
) {
    if round.phase != RoundPhase::Drafting || draft.chosen.is_some() {
        return;
    }

    let pick = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else {
        None
    };

    if let Some(idx) = pick {
        if let Some(&card) = draft.choices.get(idx) {
            draft.chosen = Some(card);
            println!("Player picked: {:?}", card);
        }
    }
}

pub fn apply_card_system(
    round: Res<RoundState>,
    mut draft: ResMut<DraftState>,
    pool: Res<CardPool>,
    mut players: Query<(&mut CardInventory, &PlayerTag)>,
) {
    if round.phase != RoundPhase::Drafting {
        return;
    }
    let Some(chosen) = draft.chosen.take() else {
        return;
    };
    let Some(loser_idx) = round.loser else {
        return;
    };

    for (mut inv, tag) in &mut players {
        if tag.index == loser_idx {
            inv.cards.push(chosen);
            if let Some(def) = pool.defs.iter().find(|d| d.id == chosen) {
                println!("Player {} gained: {}", loser_idx, def.name);
            }
        }
    }

    draft.choices.clear();
}

/// `StatType::FireRate` is an **additive adjustment to cooldown multiplier**,
/// baseline 1.0:
/// - `-0.7` → cooldown × 0.3 (FastReload: -70% reload time)
/// - `+1.0` → cooldown × 2.0 (Homing: -50% ATKSPD = halved attack speed)
pub fn recalculate_stats_system(
    pool: Res<CardPool>,
    cfg: Res<Config>,
    mut players: Query<(&CardInventory, &mut StatBlock, &mut Ammo, &mut Health), With<PlayerTag>>,
) {
    for (inv, mut stats, mut ammo, mut health) in &mut players {
        *stats = StatBlock::default();

        let mut cooldown_add = 0.0;
        let mut damage_mult = 1.0;
        let mut ammo_add = 0i32;
        let mut health_add = 0.0;
        let mut extra_bounces = 0u32;
        let mut homing = false;
        let bullet_speed_mult = 1.0;

        for card_id in &inv.cards {
            if let Some(def) = pool.defs.iter().find(|d| d.id == *card_id) {
                if let Some(BehaviorTag::Homing) = def.behavior {
                    homing = true;
                }
                for modifier in &def.modifiers {
                    match modifier {
                        StatModifier::Mul(StatType::Damage, v) => damage_mult *= *v,
                        StatModifier::Add(StatType::Ammo, v) => ammo_add += *v as i32,
                        StatModifier::Add(StatType::Health, v) => health_add += v,
                        StatModifier::Add(StatType::FireRate, v) => cooldown_add += *v,
                        StatModifier::Add(StatType::BulletBounces, v) => {
                            extra_bounces += *v as u32
                        }
                        _ => {}
                    }
                }
            }
        }

        stats.damage_mult = damage_mult;
        stats.ammo_add = ammo_add;
        stats.health_add = health_add;
        stats.fire_cooldown_mult = (1.0 + cooldown_add).max(0.1);
        stats.extra_bounces = extra_bounces;
        stats.homing = homing;
        stats.bullet_speed_mult = bullet_speed_mult;

        ammo.max = (cfg.base_ammo as i32 + stats.ammo_add).max(0) as u32;
        health.max = cfg.base_health + stats.health_add;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn player_world(cards: Vec<CardId>) -> World {
        let mut world = World::new();
        world.insert_resource(Config::default());
        world.insert_resource(CardPool::default());

        world.spawn((
            PlayerTag { index: 0 },
            CardInventory { cards },
            StatBlock::default(),
            Ammo { current: 6, max: 6 },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ));
        world
    }

    #[test]
    fn big_bullet_multiplies_damage() {
        let mut world = player_world(vec![CardId::BigBullet]);
        world.run_system_once(recalculate_stats_system).unwrap();
        let mut q = world.query::<&StatBlock>();
        let stats = q.single(&world).unwrap();
        assert!((stats.damage_mult - 1.5).abs() < 1e-6);
    }

    #[test]
    fn homing_sets_flag_and_penalties() {
        let mut world = player_world(vec![CardId::Homing]);
        world.run_system_once(recalculate_stats_system).unwrap();
        let mut q = world.query::<&StatBlock>();
        let stats = q.single(&world).unwrap();
        assert!(stats.homing);
        assert!((stats.damage_mult - 0.75).abs() < 1e-6);
        // -50% ATKSPD → cooldown × 2.0
        assert!((stats.fire_cooldown_mult - 2.0).abs() < 1e-6);
    }

    #[test]
    fn bouncy_adds_bounces_and_damage() {
        let mut world = player_world(vec![CardId::Bounce]);
        world.run_system_once(recalculate_stats_system).unwrap();
        let mut q = world.query::<&StatBlock>();
        let stats = q.single(&world).unwrap();
        assert_eq!(stats.extra_bounces, 2);
        assert!((stats.damage_mult - 1.25).abs() < 1e-6);
    }

    #[test]
    fn fast_reload_reduces_cooldown() {
        let mut world = player_world(vec![CardId::FastReload]);
        world.run_system_once(recalculate_stats_system).unwrap();
        let mut q = world.query::<&StatBlock>();
        let stats = q.single(&world).unwrap();
        // -70% reload time → cooldown × 0.3
        assert!((stats.fire_cooldown_mult - 0.3).abs() < 1e-6);
    }

    #[test]
    fn more_ammo_raises_magazine() {
        let mut world = player_world(vec![CardId::MoreAmmo, CardId::MoreAmmo]);
        world.run_system_once(recalculate_stats_system).unwrap();
        let mut q = world.query::<&Ammo>();
        let ammo = q.single(&world).unwrap();
        assert_eq!(ammo.max, 12);
    }

    #[test]
    fn extra_health_raises_max_hp() {
        let mut world = player_world(vec![CardId::ExtraHealth]);
        world.run_system_once(recalculate_stats_system).unwrap();
        let mut q = world.query::<&Health>();
        let hp = q.single(&world).unwrap();
        assert!((hp.max - 125.0).abs() < 1e-6);
    }

    #[test]
    fn empty_inventory_yields_defaults() {
        let mut world = player_world(vec![]);
        world.run_system_once(recalculate_stats_system).unwrap();
        let mut q = world.query::<&StatBlock>();
        let stats = q.single(&world).unwrap();
        assert_eq!(stats.damage_mult, 1.0);
        assert!(!stats.homing);
        assert_eq!(stats.extra_bounces, 0);
    }

    #[test]
    fn apply_card_gives_to_loser_and_clears_draft() {
        let mut world = World::new();
        world.insert_resource(Config::default());
        world.insert_resource(CardPool::default());

        let mut round = RoundState::default();
        round.phase = RoundPhase::Drafting;
        round.loser = Some(1);
        world.insert_resource(round);

        let mut draft = DraftState::default();
        draft.choices = vec![CardId::BigBullet];
        draft.chosen = Some(CardId::BigBullet);
        world.insert_resource(draft);

        for idx in 0..2 {
            world.spawn((PlayerTag { index: idx }, CardInventory::default()));
        }

        world.run_system_once(apply_card_system).unwrap();

        let mut q = world.query::<(&CardInventory, &PlayerTag)>();
        for (inv, tag) in q.iter(&world) {
            if tag.index == 1 {
                assert_eq!(inv.cards, vec![CardId::BigBullet]);
            } else {
                assert!(inv.cards.is_empty());
            }
        }

        let draft = world.resource::<DraftState>();
        assert!(draft.chosen.is_none());
        assert!(draft.choices.is_empty());
    }
}