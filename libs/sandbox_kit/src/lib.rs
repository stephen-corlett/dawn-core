//! A minimal 3D scene for trying a Bevy plugin on its own: a window, a square of
//! ground, lighting, and a camera.

use bevy::prelude::*;

const GROUND_COLOR: Color = Color::srgb(0.28, 0.42, 0.3);

pub struct SandboxPlugin {
    pub title: String,
    pub ground_size: f32,
    /// Where assets load from, relative to the crate being run.
    pub asset_folder: String,
}

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("Sandbox - {}", self.title),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: self.asset_folder.clone(),
                    ..default()
                }),
        );

        let ground_size = self.ground_size;
        app.add_systems(
            Startup,
            move |commands: Commands,
                  meshes: ResMut<Assets<Mesh>>,
                  materials: ResMut<Assets<StandardMaterial>>| {
                spawn_scene(ground_size, commands, meshes, materials);
            },
        );
    }
}

fn spawn_scene(
    size: f32,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
        MeshMaterial3d(materials.add(GROUND_COLOR)),
    ));
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(size / 4.0, size * 0.6, size / 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, size * 0.7, size * 0.8).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
