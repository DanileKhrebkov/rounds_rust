//! Серверный плагин: приём подключений и спавн сущностей игроков.

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::server::*;
use lightyear::server::events::{ConnectEvent, DisconnectEvent}; // <-- Правильный путь
use lightyear_inputs_leafwing::LeafwingInputPlugin; // <-- Правильный путь

use crate::shared::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let tick_duration = core::time::Duration::from_secs_f64(1.0 / 60.0);
        app.add_plugins(ServerPlugins { tick_duration });
        app.add_plugins(LeafwingInputPlugin::<PlayerInput>::default());

        // Регистрация правил интерполяции
        fn lerp_transform(a: &Transform, b: &Transform, t: f32) -> Transform {
            let mut out = *a;
            out.translation = a.translation.lerp(b.translation, t);
            out
        }

        app.component::<Transform>().replicate().predict();
        app.interpolate_with::<Transform>(InterpolationFns::interpolate(lerp_transform)); // <-- Правильный вызов
        app.component::<NetworkedHealth>().replicate().predict();
        app.component::<NetworkedVelocity>().replicate().predict();
        app.component::<PlayerInput>().replicate().predict();

        app.add_systems(Startup, setup_server_camera);
        app.add_systems(Update, handle_connections);
    }
}

fn setup_server_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn handle_connections(
    mut commands: Commands,
    mut connect_events: EventReader<ConnectEvent>,
    mut disconnect_events: EventReader<DisconnectEvent>,
    players: Query<(Entity, &NetworkedPlayer)>,
) {
    for event in connect_events.read() {
        let client_id = event.client_id();
        println!("[Server] Client connected: {client_id:?}");

        commands.spawn((
            Replicate::to_clients(NetworkTarget::All),
            NetworkedPlayer { client_id },
            Transform::from_xyz(0.0, 100.0, 0.0),
            NetworkedHealth { current: 100.0, max: 100.0 },
            NetworkedVelocity { x: 0.0, y: 0.0 },
        ));
    }

    for event in disconnect_events.read() {
        let client_id = event.client_id();
        println!("[Server] Client disconnected: {client_id:?}");
        for (entity, player) in &players {
            if player.client_id == client_id {
                commands.entity(entity).despawn();
            }
        }
    }
}