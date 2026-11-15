//! Клиентский плагин: подключение к серверу и сбор ввода.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::client::*;
use lightyear_inputs_leafwing::prelude::*; // <-- Импортируем prelude

use crate::shared::*;

#[derive(Resource)]
pub struct ClientPlugin {
    pub server_addr: String,
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let tick_duration = core::time::Duration::from_secs_f64(1.0 / 60.0);
        app.add_plugins(ClientPlugins { tick_duration });
        app.add_plugins(LeafwingInputPlugin::<PlayerInput>::default());

        fn lerp_transform(a: &Transform, b: &Transform, t: f32) -> Transform {
            let mut out = *a;
            out.translation = a.translation.lerp(b.translation, t);
            out
        }

        app.component::<Transform>().replicate().predict();
        app.interpolate_with::<Transform>(InterpolationFns::interpolate(lerp_transform));
        app.component::<NetworkedHealth>().replicate().predict();
        app.component::<NetworkedVelocity>().replicate().predict();
        app.component::<PlayerInput>().replicate().predict();

        app.insert_resource(ClientPlugin {
            server_addr: self.server_addr.clone(),
        });

        app.add_systems(Startup, (setup_client_camera, connect_to_server));
        app.add_systems(FixedPreUpdate, capture_input);
        app.add_systems(Update, (log_connected, log_disconnected));
    }
}

fn setup_client_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn connect_to_server(mut commands: Commands, plugin: Res<ClientPlugin>) {
    println!("[Client] Connecting to {}", plugin.server_addr);

    let input_map = InputMap::<PlayerInput>::new([
        (PlayerInput::Jump, KeyCode::Space),
        (PlayerInput::Shoot, KeyCode::KeyJ),
        (PlayerInput::Block, KeyCode::KeyK),
    ]);

    let client = commands
        .spawn((
            Client::default(),
            ActionState::<PlayerInput>::default(),
            input_map,
        ))
        .id();

    commands.trigger(Connect { entity: client });
}

fn capture_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_state: Query<&mut ActionState<PlayerInput>>,
) {
    let Ok(mut state) = action_state.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Space) {
        state.press(&PlayerInput::Jump);
    }
    if keyboard.just_pressed(KeyCode::KeyJ) {
        state.press(&PlayerInput::Shoot);
    }
    if keyboard.pressed(KeyCode::KeyK) {
        state.press(&PlayerInput::Block);
    } else {
        state.release(&PlayerInput::Block);
    }
}

fn log_connected(mut events: EventReader<ConnectEvent>) {
    for event in events.read() {
        println!("[Client] Connected as {:?}", event.client_id());
    }
}

fn log_disconnected(mut events: EventReader<DisconnectEvent>) {
    for _ in events.read() {
        println!("[Client] Disconnected");
    }
}