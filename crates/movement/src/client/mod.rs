//! Reads movement keys, sends them to the server, and predicts the result.

use crate::simulation::SimulatedBody;
use crate::{Gait, MoveHeading, MoveInput, MovementConfig, PlayerPosition, Surface};
use bevy::prelude::*;
use lightyear::prelude::client::input::*;
use lightyear::prelude::input::native::*;
use lightyear::prelude::*;

/// Which keys trigger each direction. The game supplies these, so they can
/// later come from a player's settings.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct MoveBindings {
    pub forward: Vec<KeyCode>,
    pub back: Vec<KeyCode>,
    pub left: Vec<KeyCode>,
    pub right: Vec<KeyCode>,
    pub jump: Vec<KeyCode>,
    pub dodge: Vec<KeyCode>,
    /// Pressing one of these switches sneaking on or off.
    pub sneak: Vec<KeyCode>,
}

pub struct MovementClientPlugin {
    pub config: MovementConfig,
    pub bindings: MoveBindings,
    pub surface: Surface,
}

impl Plugin for MovementClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.insert_resource(self.bindings.clone());
        app.insert_resource(self.surface.clone());
        app.init_resource::<MoveHeading>();
        app.init_resource::<SneakToggle>();
        app.add_systems(Update, toggle_sneak);
        app.add_systems(
            FixedPreUpdate,
            write_move_input.in_set(InputSystems::WriteClientInputs),
        );
        app.add_systems(FixedUpdate, predict_movement);
        app.add_observer(mark_controlled_entity);
    }
}

/// Whether the local player has sneaking switched on.
#[derive(Resource, Default)]
struct SneakToggle(bool);

/// Runs every frame, because a single key press can fall between two fixed ticks.
fn toggle_sneak(
    bindings: Res<MoveBindings>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut toggle: ResMut<SneakToggle>,
) {
    let pressed = keys.is_some_and(|keys| keys.any_just_pressed(bindings.sneak.iter().copied()));
    if pressed {
        toggle.0 = !toggle.0;
    }
}

/// Input is optional so that a headless client can run without a keyboard.
fn write_move_input(
    bindings: Res<MoveBindings>,
    heading: Res<MoveHeading>,
    sneak: Res<SneakToggle>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut players: Query<&mut ActionState<MoveInput>, With<InputMarker<MoveInput>>>,
) {
    let Ok(mut action_state) = players.single_mut() else {
        return;
    };
    let held = |bound: &[KeyCode]| {
        keys.as_ref()
            .is_some_and(|keys| keys.any_pressed(bound.iter().copied()))
    };
    let gait = if sneak.0 { Gait::Sneak } else { Gait::Run };
    action_state.0 = MoveInput {
        forward: held(&bindings.forward),
        back: held(&bindings.back),
        left: held(&bindings.left),
        right: held(&bindings.right),
        jump: held(&bindings.jump),
        dodge: held(&bindings.dodge),
        gait,
        yaw: heading.0,
    };
}

/// Moves the local player right away instead of waiting for the server.
fn predict_movement(
    time: Res<Time<Fixed>>,
    config: Res<MovementConfig>,
    surface: Res<Surface>,
    mut players: Query<(SimulatedBody, &ActionState<MoveInput>), With<Predicted>>,
) {
    let delta_secs = time.timestep().as_secs_f32();
    for (mut body, input) in &mut players {
        body.advance(input, delta_secs, &config, surface.0.as_ref());
    }
}

/// Lightyear adds `Controlled` to the entity this client owns.
fn mark_controlled_entity(
    trigger: On<Add<Controlled>>,
    entities: Query<(), (With<PlayerPosition>, Without<InputMarker<MoveInput>>)>,
    mut commands: Commands,
) {
    if entities.contains(trigger.entity) {
        commands
            .entity(trigger.entity)
            .insert(InputMarker::<MoveInput>::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlatSurface;
    use bdd::spec;
    use std::sync::Arc;

    const CONFIG: MovementConfig = MovementConfig {
        sneak_speed: 2.0,
        run_speed: 7.0,
        dodge_lunge_speed: 12.0,
        dodge_lunge_secs: 0.3,
        dodge_rollout_speed: 3.0,
        dodge_rollout_secs: 0.2,
        dodge_recovery_secs: 0.3,
        dodge_recovery_speed: 1.0,
        jump_speed: 8.0,
        gravity: 20.0,
        ground_snap: 0.5,
        body_radius: 0.4,
        world_half_extent: 50.0,
    };

    fn input_app() -> (App, Entity) {
        let mut app = App::new();
        app.insert_resource(Time::<Fixed>::from_hz(20.0));
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_plugins(MovementClientPlugin {
            config: CONFIG,
            bindings: MoveBindings {
                forward: vec![KeyCode::KeyW],
                back: vec![KeyCode::KeyS],
                left: vec![KeyCode::KeyA],
                right: vec![KeyCode::KeyD],
                jump: vec![KeyCode::Space],
                dodge: vec![KeyCode::ControlLeft],
                sneak: vec![KeyCode::KeyC],
            },
            surface: Surface(Arc::new(FlatSurface)),
        });
        let player = app
            .world_mut()
            .spawn((
                ActionState(MoveInput::default()),
                InputMarker::<MoveInput>::default(),
            ))
            .id();
        (app, player)
    }

    fn input_with(app: &mut App, player: Entity, key: KeyCode) -> MoveInput {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.world_mut().run_schedule(FixedPreUpdate);
        app.world()
            .get::<ActionState<MoveInput>>(player)
            .unwrap()
            .0
            .clone()
    }

    /// Presses and lets go of the sneak key, as one frame would see it.
    fn tap_sneak(app: &mut App, player: Entity) -> MoveInput {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::KeyC);
        keys.clear();
        keys.press(KeyCode::KeyC);
        app.world_mut().run_schedule(Update);
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        app.world_mut().run_schedule(FixedPreUpdate);
        app.world()
            .get::<ActionState<MoveInput>>(player)
            .unwrap()
            .0
            .clone()
    }

    #[spec]
    fn reading_input() {
        given("a local player", || {
            let (mut app, player) = input_app();

            when("only a direction is held", || {
                let input = input_with(&mut app, player, KeyCode::KeyW);

                test("they ask to run that way", || {
                    assert!(input.forward);
                    assert_eq!(input.gait, Gait::Run);
                });
            });

            when("the dodge key is held", || {
                let input = input_with(&mut app, player, KeyCode::ControlLeft);

                test("they ask to dodge", || {
                    assert!(input.dodge);
                });
            });

            when("the sneak key is tapped", || {
                let input = tap_sneak(&mut app, player);

                test("they ask to sneak", || {
                    assert_eq!(input.gait, Gait::Sneak);
                });
            });

            when("the sneak key is tapped twice", || {
                tap_sneak(&mut app, player);
                let input = tap_sneak(&mut app, player);

                test("they go back to running", || {
                    assert_eq!(input.gait, Gait::Run);
                });
            });

            when("the sneak key stays down for several frames", || {
                let first = tap_sneak(&mut app, player);
                app.world_mut().run_schedule(Update);

                test("it only counts once", || {
                    assert_eq!(first.gait, Gait::Sneak);
                    assert!(app.world().resource::<SneakToggle>().0);
                });
            });

            when("the jump key is held", || {
                let input = input_with(&mut app, player, KeyCode::Space);

                test("they ask to jump", || {
                    assert!(input.jump);
                });
            });

            when("the camera has turned", || {
                app.world_mut().resource_mut::<MoveHeading>().0 = 1.5;
                let input = input_with(&mut app, player, KeyCode::KeyW);

                test("forward follows it", || {
                    assert_eq!(input.yaw, 1.5);
                });
            });
        });
    }
}
