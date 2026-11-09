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
        .add_systems(Update, systems::movement::gravity_system)
        .run();
}