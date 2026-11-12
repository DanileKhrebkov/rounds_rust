//! Lifecycle systems: spawn, death detection, round reset, match-over.

use bevy::prelude::*;

use crate::components::cards::{CardInventory, StatBlock};
use crate::components::player::{
    Ammo, BlockState, Facing, Health, JumpState, KnockbackState, MoveSpeed, OnGround, PlayerTag,
    Velocity,
};
use crate::components::projectile::ProjectileTag;
use crate::resources::{Config, DraftState, RoundPhase, RoundState, ScoreBoard};

pub fn setup(mut commands: Commands, cfg: Res<Config>) {
    commands.spawn(Camera2d);

    let colors = [Color::srgb(0.2, 0.4, 1.0), Color::srgb(1.0, 0.2, 0.2)];
    let spawns = [(-200.0_f32, 0_usize), (200.0_f32, 1_usize)];

    for (x, index) in spawns {
        commands.spawn((
            Sprite::from_color(colors[index], Vec2::splat(40.0)),
            Transform::from_xyz(x, 0.0, 0.0),
            PlayerTag { index },
            Health {
                current: cfg.base_health,
                max: cfg.base_health,
            },
            Velocity { x: 0.0, y: 0.0 },
            OnGround(false),
            MoveSpeed(8.0),
            BlockState::default(),
            JumpState::new(cfg.max_jumps),
            KnockbackState::default(),
            Ammo {
                current: cfg.base_ammo,
                max: cfg.base_ammo,
            },
            Facing(if index == 0 { 1.0 } else { -1.0 }),
            CardInventory::default(),
            StatBlock::default(),
        ));
    }
}

pub fn death_system(
    mut round: ResMut<RoundState>,
    mut score: ResMut<ScoreBoard>,
    players: Query<(&Health, &PlayerTag)>,
) {
    if round.phase != RoundPhase::Combat {
        return;
    }

    let mut dead: Vec<usize> = vec![];
    for (hp, tag) in &players {
        if hp.current <= 0.0 {
            dead.push(tag.index);
        }
    }

    if let Some(&loser) = dead.first() {
        round.loser = Some(loser);
        round.phase = RoundPhase::Drafting;
        let winner = 1 - loser;
        score.wins[winner] += 1;
        println!("Player {} lost round {}", loser, round.round_number);
    }
}

pub fn round_reset_system(
    mut commands: Commands,
    mut round: ResMut<RoundState>,
    draft: Res<DraftState>,
    cfg: Res<Config>,
    mut players: Query<
        (
            &mut Health,
            &mut Ammo,
            &mut Transform,
            &mut Velocity,
            &mut BlockState,
            &mut JumpState,
            &mut KnockbackState,
            &PlayerTag,
        ),
        With<PlayerTag>,
    >,
    projectiles: Query<Entity, With<ProjectileTag>>,
) {
    if round.phase != RoundPhase::Drafting {
        return;
    }
    if !draft.choices.is_empty() {
        return;
    }

    for entity in &projectiles {
        commands.entity(entity).despawn();
    }

    for (mut hp, mut ammo, mut tf, mut vel, mut block, mut jump, mut kb, tag) in &mut players {
        hp.current = hp.max;
        ammo.current = ammo.max;
        tf.translation.x = if tag.index == 0 { -200.0 } else { 200.0 };
        tf.translation.y = 0.0;
        vel.x = 0.0;
        vel.y = 0.0;
        *block = BlockState::default();
        jump.jumps_remaining = cfg.max_jumps;
        *kb = KnockbackState::default();
    }

    round.phase = RoundPhase::Combat;
    round.round_number += 1;
    round.loser = None;
    println!("Round {} starting", round.round_number);
}

pub fn match_over_system(score: Res<ScoreBoard>, mut round: ResMut<RoundState>) {
    if round.phase == RoundPhase::MatchOver {
        return;
    }
    if score.wins[0] >= score.needed || score.wins[1] >= score.needed {
        round.phase = RoundPhase::MatchOver;
        let winner = if score.wins[0] >= score.needed { 0 } else { 1 };
        println!("MATCH OVER — Player {} wins!", winner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::PluginGroup;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn startup_spawns_camera_and_two_players() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Config>()
            .add_systems(Startup, setup);

        app.update();
        app.update();

        let world = app.world_mut();

        let cams = world
            .query_filtered::<(), With<Camera2d>>()
            .iter(world)
            .count();
        assert_eq!(cams, 1);

        let mut q = world.query::<(
            &PlayerTag,
            &Health,
            &Velocity,
            &OnGround,
            &MoveSpeed,
            &BlockState,
            &JumpState,
            &KnockbackState,
            &Ammo,
            &Facing,
            &CardInventory,
            &StatBlock,
        )>();
        let players: Vec<_> = q.iter(world).collect();
        assert_eq!(players.len(), 2);

        for (_, hp, _, _, _, block, jump, kb, ammo, facing, inv, stats) in &players {
            assert_eq!(hp.max, 100.0);
            assert_eq!(ammo.max, 6);
            assert!(!block.active);
            assert_eq!(jump.jumps_remaining, 2);
            assert!(!kb.is_active());
            assert!(facing.0.abs() == 1.0);
            assert!(inv.cards.is_empty());
            assert_eq!(stats.damage_mult, 1.0);
        }
    }

    fn world_with_phase(phase: RoundPhase, hp0: f32, hp1: f32) -> World {
        let mut world = World::new();
        world.init_resource::<RoundState>();
        world.init_resource::<ScoreBoard>();
        world.resource_mut::<RoundState>().phase = phase;

        world.spawn((
            Health {
                current: hp0,
                max: 100.0,
            },
            PlayerTag { index: 0 },
        ));
        world.spawn((
            Health {
                current: hp1,
                max: 100.0,
            },
            PlayerTag { index: 1 },
        ));
        world
    }

    #[test]
    fn death_enters_drafting_and_scores_winner() {
        let mut world = world_with_phase(RoundPhase::Combat, 100.0, 0.0);
        world.run_system_once(death_system).unwrap();

        let round = world.resource::<RoundState>();
        assert_eq!(round.phase, RoundPhase::Drafting);
        assert_eq!(round.loser, Some(1));
        assert_eq!(world.resource::<ScoreBoard>().wins, [1, 0]);
    }

    #[test]
    fn death_is_ignored_outside_combat() {
        let mut world = world_with_phase(RoundPhase::Drafting, 100.0, 0.0);
        world.run_system_once(death_system).unwrap();
        assert_eq!(world.resource::<ScoreBoard>().wins, [0, 0]);
    }

    #[test]
    fn no_death_no_transition() {
        let mut world = world_with_phase(RoundPhase::Combat, 100.0, 100.0);
        world.run_system_once(death_system).unwrap();
        assert_eq!(world.resource::<RoundState>().phase, RoundPhase::Combat);
    }

    #[test]
    fn match_ends_at_needed_wins() {
        let mut world = World::new();
        world.init_resource::<RoundState>();
        let mut score = ScoreBoard::default();
        score.wins = [5, 2];
        world.insert_resource(score);

        world.run_system_once(match_over_system).unwrap();
        assert_eq!(world.resource::<RoundState>().phase, RoundPhase::MatchOver);
    }

    #[test]
    fn match_continues_below_needed() {
        let mut world = World::new();
        world.init_resource::<RoundState>();
        let mut score = ScoreBoard::default();
        score.wins = [4, 2];
        world.insert_resource(score);

        world.run_system_once(match_over_system).unwrap();
        assert_eq!(world.resource::<RoundState>().phase, RoundPhase::Combat);
    }
}