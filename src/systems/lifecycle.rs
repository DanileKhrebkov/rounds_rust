use bevy::prelude::*;

use crate::components::player::{
    Ammo, BlockState, Facing, Health, MoveSpeed, OnGround, PlayerTag, Velocity,
};

/// Step 2 acceptance: camera + two players carrying the full stat component set.
pub fn setup(mut commands: Commands) {
    // Bevy 0.19 idiom: `Camera2dBundle` is gone, the unit struct is enough.
    commands.spawn(Camera2d);

    let colors = [Color::srgb(0.2, 0.4, 1.0), Color::srgb(1.0, 0.2, 0.2)];
    let spawns = [(-200.0_f32, 0_usize), (200.0_f32, 1_usize)];

    for (x, index) in spawns {
        commands.spawn((
            Sprite::from_color(colors[index], Vec2::splat(40.0)),
            Transform::from_xyz(x, 0.0, 0.0),
            PlayerTag { index },
            Health { current: 100.0, max: 100.0 },
            Velocity { x: 0.0, y: 0.0 },
            OnGround(false),
            MoveSpeed(8.0),
            BlockState { active: false },
            Ammo { current: 6, max: 6 },
            Facing(if index == 0 { 1.0 } else { -1.0 }),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::Config;
    use bevy::app::PluginGroup;

    /// Headless smoke test for Steps 1-2.
    ///
    /// `MinimalPlugins` + no render/audio windowing means this runs in CI with no
    /// display server. We still get `Time`, so we can drive two update ticks and
    /// assert that `Startup` actually produced a camera and two fully-populated
    /// players — which is exactly what "cargo run opens a window" is verifying.
    #[test]
    fn startup_spawns_camera_and_two_players() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Config>()
            .add_systems(Startup, setup);

        app.update(); // runs Startup
        app.update(); // one Update tick

        let world = app.world_mut();

        let cams = world
            .query_filtered::<(), With<Camera2d>>()
            .iter(world)
            .count();
        assert_eq!(cams, 1, "expected exactly one Camera2d");

        let mut q = world.query::<(
            &Transform,
            &PlayerTag,
            &Health,
            &Velocity,
            &OnGround,
            &MoveSpeed,
            &BlockState,
            &Ammo,
            &Facing,
        )>();
        let players: Vec<_> = q.iter(world).collect();

        assert_eq!(players.len(), 2, "expected two players");

        // Both carry the complete stat component set with the documented defaults.
        for (_, tag, hp, vel, og, speed, block, ammo, facing) in &players {
            assert_eq!(hp.max, 100.0);
            assert_eq!(hp.current, 100.0);
            assert_eq!(vel.x, 0.0);
            assert_eq!(og.0, false);
            assert_eq!(speed.0, 8.0);
            assert!(!block.active);
            assert_eq!(ammo.current, 6);
            assert_eq!(ammo.max, 6);
            assert!(facing.0.abs() == 1.0);
            assert!(tag.index <= 1);
        }

        // Distinct indices, mirrored spawn positions, opposite facings.
        let idx: Vec<usize> = players.iter().map(|p| p.1.index).collect();
        assert_ne!(idx[0], idx[1]);
        let xs: Vec<f32> = players.iter().map(|p| p.0.translation.x).collect();
        assert!((xs[0] + xs[1]).abs() < 1e-6, "players should mirror at +/-200");
        let faces: Vec<f32> = players.iter().map(|p| p.8.0).collect();
        assert_ne!(faces[0], faces[1], "players must face each other");
    }
}
