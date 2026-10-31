//! Reads movement keys, sends them to the server, and predicts the result.

use crate::{MoveInput, MovementConfig, PlayerPosition, next_position};
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
}

pub struct MovementClientPlugin {
    pub config: MovementConfig,
    pub bindings: MoveBindings,
}

impl Plugin for MovementClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.insert_resource(self.bindings.clone());
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
    action_state.0 = MoveInput {
        forward: held(&bindings.forward),
        back: held(&bindings.back),
        left: held(&bindings.left),
        right: held(&bindings.right),
    };
}

/// Moves the local player right away instead of waiting for the server.
fn predict_movement(
    time: Res<Time<Fixed>>,
    config: Res<MovementConfig>,
    mut players: Query<(&mut PlayerPosition, &ActionState<MoveInput>), With<Predicted>>,
) {
    let delta_secs = time.timestep().as_secs_f32();
    for (mut position, input) in &mut players {
        let next = next_position(position.0, input, delta_secs, &config);
        if next != position.0 {
            position.0 = next;
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
