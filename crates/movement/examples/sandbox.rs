//! Walks a capsule with the real movement rules, with no server or network.
//! Run it with `just sandbox movement`, then edit `assets/sandbox_tuning.ron` to try
//! different tuning while it runs.

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use lightyear::prelude::input::native::ActionState;
use movement::server::MovementServerPlugin;
use movement::{FlatSurface, Gait, MoveInput, MovementConfig, PlayerPosition, Surface};
use sandbox_kit::SandboxPlugin;
use serde::Deserialize;
use std::sync::Arc;

const TUNING_ASSET: &str = "sandbox_tuning.ron";
// Assets load in the background, but the plugins need their values now.
const STARTING_TUNING: &str = include_str!("../assets/sandbox_tuning.ron");
const CAPSULE_CENTER_HEIGHT: f32 = 0.9;

/// The tuning file, as an asset so Bevy reloads it when it changes.
#[derive(Asset, TypePath, Deserialize)]
#[serde(transparent)]
struct MovementTuning(MovementConfig);

#[derive(Default, TypePath)]
struct TuningLoader;

impl AssetLoader for TuningLoader {
    type Asset = MovementTuning;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<MovementTuning> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Keeps the tuning loaded. Bevy unloads an asset once nothing holds its handle.
#[derive(Resource)]
struct TuningHandle(#[expect(dead_code)] Handle<MovementTuning>);

fn main() {
    let tuning: MovementTuning =
        ron::from_str(STARTING_TUNING).expect("sandbox_tuning.ron should be valid");
    let config = tuning.0;

    App::new()
        .add_plugins(SandboxPlugin {
            title: "Movement".into(),
            ground_size: config.world_half_extent * 2.0,
            asset_folder: "assets".into(),
        })
        .add_plugins(MovementServerPlugin {
            config,
            surface: Surface(Arc::new(FlatSurface)),
        })
        .init_asset::<MovementTuning>()
        .init_asset_loader::<TuningLoader>()
        .add_systems(Startup, (load_tuning, spawn_capsule))
        .add_systems(FixedPreUpdate, read_keyboard)
        .add_systems(Update, (apply_tuning, move_capsule))
        .run();
}

fn load_tuning(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(TuningHandle(assets.load(TUNING_ASSET)));
}

fn apply_tuning(
    mut events: MessageReader<AssetEvent<MovementTuning>>,
    tunings: Res<Assets<MovementTuning>>,
    mut config: ResMut<MovementConfig>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } | AssetEvent::Modified { id } = event
            && let Some(tuning) = tunings.get(*id)
        {
            *config = tuning.0;
            info!("Applied movement tuning: {:?}", *config);
        }
    }
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
            jump: keys.pressed(KeyCode::Space),
            dodge: keys.pressed(KeyCode::ControlLeft),
            gait: if keys.pressed(KeyCode::ShiftLeft) {
                Gait::Sprint
            } else {
                Gait::Run
            },
            yaw: 0.0,
        };
    }
}

fn move_capsule(mut capsules: Query<(&PlayerPosition, &mut Transform)>) {
    for (position, mut transform) in &mut capsules {
        transform.translation = position.0 + Vec3::Y * CAPSULE_CENTER_HEIGHT;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const SLOW: MovementConfig = MovementConfig {
        sneak_speed: 0.5,
        run_speed: 1.0,
        sprint_speed: 2.0,
        dodge_speed: 3.0,
        dodge_secs: 0.7,
        dodge_cooldown_secs: 0.4,
        jump_speed: 7.0,
        gravity: 18.0,
        ground_snap: 0.4,
        body_radius: 0.4,
        world_half_extent: 10.0,
    };
    const FAST: MovementConfig = MovementConfig {
        run_speed: 20.0,
        ..SLOW
    };

    fn tuning_app() -> App {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
        app.init_asset::<MovementTuning>();
        app.insert_resource(SLOW);
        app.add_systems(Update, apply_tuning);
        app
    }

    fn add_tuning(app: &mut App, config: MovementConfig) -> Handle<MovementTuning> {
        let tuning = app
            .world_mut()
            .resource_mut::<Assets<MovementTuning>>()
            .add(MovementTuning(config));
        app.update();
        app.update();
        tuning
    }

    #[spec]
    fn tuning() {
        given("the committed tuning file", || {
            when("it is parsed", || {
                let tuning: Result<MovementTuning, _> = ron::from_str(STARTING_TUNING);

                test("it is valid", || {
                    assert!(tuning.is_ok());
                });
            });
        });

        given("a sandbox running slowly", || {
            let mut app = tuning_app();

            when("the tuning asset loads", || {
                add_tuning(&mut app, FAST);

                test("the movement config takes its values", || {
                    assert_eq!(*app.world().resource::<MovementConfig>(), FAST);
                });
            });
        });

        given("a sandbox with loaded tuning", || {
            let mut app = tuning_app();
            let tuning = add_tuning(&mut app, SLOW);

            when("the tuning changes", || {
                app.world_mut()
                    .resource_mut::<Assets<MovementTuning>>()
                    .get_mut(&tuning)
                    .unwrap()
                    .0 = FAST;
                app.update();
                app.update();

                test("the movement config takes the new values", || {
                    assert_eq!(*app.world().resource::<MovementConfig>(), FAST);
                });
            });
        });
    }
}
