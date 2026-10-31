//! Draws the world: a ground plane, a capsule per player, and a follow camera.

use bevy::prelude::*;
use foundation::player::PlayerId;
use lightyear::prelude::*;
use movement::{MovementConfig, PlayerPosition};

/// Half the capsule's height, which lifts its base onto the ground.
const PLAYER_CENTER_HEIGHT: f32 = 0.9;
const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 12.0, 14.0);

/// Higher values follow more tightly. Ticks run at 20 Hz but frames run much
/// faster, so smoothing stops characters visibly stepping between ticks.
const POSITION_SMOOTHING: f32 = 18.0;
const CAMERA_SMOOTHING: f32 = 6.0;

const LOCAL_PLAYER_COLOR: Color = Color::srgb(0.25, 0.65, 1.0);
const REMOTE_PLAYER_COLOR: Color = Color::srgb(1.0, 0.45, 0.25);

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scene);
        app.add_systems(
            Update,
            (add_player_visuals, smooth_players, follow_local_player).chain(),
        );
    }
}

#[derive(Component)]
struct FollowCamera;

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    movement: Res<MovementConfig>,
) {
    let world_size = movement.world_half_extent * 2.0;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(world_size, world_size))),
        MeshMaterial3d(materials.add(Color::srgb(0.28, 0.42, 0.3))),
    ));
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 40.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        FollowCamera,
        Camera3d::default(),
        Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Gives each displayed player a capsule. Filtering on `Predicted` and
/// `Interpolated` skips the replicated data before it's ready to display.
fn add_player_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<
        (Entity, &PlayerPosition, Has<Predicted>),
        (
            With<PlayerId>,
            Or<(With<Predicted>, With<Interpolated>)>,
            Without<Mesh3d>,
        ),
    >,
) {
    for (entity, position, is_local) in &players {
        let color = if is_local {
            LOCAL_PLAYER_COLOR
        } else {
            REMOTE_PLAYER_COLOR
        };
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_translation(visual_position(position)),
        ));
    }
}

fn smooth_players(
    time: Res<Time>,
    mut players: Query<(&PlayerPosition, &mut Transform), With<Mesh3d>>,
) {
    for (position, mut transform) in &mut players {
        transform.translation.smooth_nudge(
            &visual_position(position),
            POSITION_SMOOTHING,
            time.delta_secs(),
        );
    }
}

fn follow_local_player(
    time: Res<Time>,
    player: Query<&Transform, (With<Predicted>, With<Mesh3d>, Without<FollowCamera>)>,
    mut camera: Query<&mut Transform, With<FollowCamera>>,
) {
    let (Ok(player), Ok(mut camera)) = (player.single(), camera.single_mut()) else {
        return;
    };
    camera.translation.smooth_nudge(
        &(player.translation + CAMERA_OFFSET),
        CAMERA_SMOOTHING,
        time.delta_secs(),
    );
}

fn visual_position(position: &PlayerPosition) -> Vec3 {
    position.0 + Vec3::Y * PLAYER_CENTER_HEIGHT
}
