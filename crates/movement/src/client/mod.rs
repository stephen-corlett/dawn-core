//! Reads movement keys, sends them to the server, and predicts the result.

use crate::{
    Body, Gait, MoveHeading, MoveInput, MovementConfig, PlayerPosition, Surface, VerticalSpeed,
    next_body,
};
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
    pub walk: Vec<KeyCode>,
    pub sprint: Vec<KeyCode>,
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
        app.add_systems(
            FixedPreUpdate,
            write_move_input.in_set(InputSystems::WriteClientInputs),
        );
        app.add_systems(FixedUpdate, predict_movement);
        app.add_observer(mark_controlled_entity);
    }
}

/// Input is optional so that a headless client can run without a keyboard.
fn write_move_input(
    bindings: Res<MoveBindings>,
    heading: Res<MoveHeading>,
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
    let gait = if held(&bindings.sprint) {
        Gait::Sprint
    } else if held(&bindings.walk) {
        Gait::Walk
    } else {
        Gait::Run
    };
    action_state.0 = MoveInput {
        forward: held(&bindings.forward),
        back: held(&bindings.back),
        left: held(&bindings.left),
        right: held(&bindings.right),
        jump: held(&bindings.jump),
        gait,
        yaw: heading.0,
    };
}

/// Moves the local player right away instead of waiting for the server.
fn predict_movement(
    time: Res<Time<Fixed>>,
    config: Res<MovementConfig>,
    surface: Res<Surface>,
    mut players: Query<
        (
            &mut PlayerPosition,
            &mut VerticalSpeed,
            &ActionState<MoveInput>,
        ),
        With<Predicted>,
    >,
) {
    let delta_secs = time.timestep().as_secs_f32();
    for (mut position, mut vertical_speed, input) in &mut players {
        let body = Body {
            position: position.0,
            vertical_speed: vertical_speed.0,
        };
        let next = next_body(body, input, delta_secs, &config, surface.0.as_ref());
        if next.position != position.0 {
            position.0 = next.position;
        }
        if next.vertical_speed != vertical_speed.0 {
            vertical_speed.0 = next.vertical_speed;
        }
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
        walk_speed: 2.0,
        run_speed: 7.0,
        sprint_speed: 10.0,
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
                walk: vec![KeyCode::ControlLeft],
                sprint: vec![KeyCode::ShiftLeft],
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

            when("the walk key is held", || {
                let input = input_with(&mut app, player, KeyCode::ControlLeft);

                test("they ask to walk", || {
                    assert_eq!(input.gait, Gait::Walk);
                });
            });

            when("the sprint key is held", || {
                let input = input_with(&mut app, player, KeyCode::ShiftLeft);

                test("they ask to sprint", || {
                    assert_eq!(input.gait, Gait::Sprint);
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
