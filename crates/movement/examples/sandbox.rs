//! Walks a capsule with the real movement rules, with no server or network.
//! Change `CONFIG` to try different tuning. Run it with `just sandbox movement`.

use bevy::prelude::*;
use lightyear::prelude::input::native::ActionState;
use movement::server::MovementServerPlugin;
use movement::{MoveInput, MovementConfig, PlayerPosition};
use sandbox_kit::SandboxPlugin;

const CONFIG: MovementConfig = MovementConfig {
    speed: 7.0,
    world_half_extent: 10.0,
};
const CAPSULE_CENTER_HEIGHT: f32 = 0.9;

fn main() {
    App::new()
        .add_plugins(SandboxPlugin {
            title: "Movement".into(),
            ground_size: CONFIG.world_half_extent * 2.0,
        })
        .add_plugins(MovementServerPlugin { config: CONFIG })
        .add_systems(Startup, spawn_capsule)
        .add_systems(FixedPreUpdate, read_keyboard)
        .add_systems(Update, move_capsule)
        .run();
}

fn spawn_capsule(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PlayerPosition::default(),
        ActionState(MoveInput::default()),
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.65, 1.0))),
    ));
}

fn read_keyboard(keys: Res<ButtonInput<KeyCode>>, mut inputs: Query<&mut ActionState<MoveInput>>) {
    for mut input in &mut inputs {
        input.0 = MoveInput {
            forward: keys.pressed(KeyCode::KeyW),
            back: keys.pressed(KeyCode::KeyS),
            left: keys.pressed(KeyCode::KeyA),
            right: keys.pressed(KeyCode::KeyD),
        };
    }
}

fn move_capsule(mut capsules: Query<(&PlayerPosition, &mut Transform)>) {
    for (position, mut transform) in &mut capsules {
        transform.translation = position.0 + Vec3::Y * CAPSULE_CENTER_HEIGHT;
    }
}
