use bevy::prelude::*;

mod components;
mod resources;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<resources::Config>()
        .init_resource::<resources::RoundState>()
        .init_resource::<resources::ScoreBoard>()
        .init_resource::<resources::DraftState>()
        .init_resource::<resources::CardPool>()
        .add_systems(Startup, systems::lifecycle::setup)
        .add_systems(
            Update,
            (
                // Combat-loop chain
                systems::input::input_system,
                systems::input::jump_system,
                systems::input::facing_system,
                systems::input::block_system,
                systems::movement::gravity_system,
                systems::combat::shoot_system,
                systems::combat::projectile_move_system,
                systems::combat::collision_system,
                systems::combat::projectile_lifetime_system,
                systems::lifecycle::death_system,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                // Draft-loop chain
                systems::draft::draft_offer_system,
                systems::draft::draft_select_system,
                systems::draft::apply_card_system,
                systems::draft::recalculate_stats_system,
                systems::lifecycle::round_reset_system,
                systems::lifecycle::match_over_system,
            )
                .chain(),
        )
        .run();
}