//! Authoritative movement. Nothing here touches networking, so these rules
//! can be tested by placing inputs directly on entities.

use crate::{MoveInput, MovementConfig, PlayerPosition, next_position};
use bevy::prelude::*;
use lightyear::prelude::input::native::ActionState;

pub struct MovementServerPlugin {
    pub config: MovementConfig,
}

impl Plugin for MovementServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.add_systems(FixedUpdate, apply_player_movement);
    }
}

fn apply_player_movement(
    time: Res<Time<Fixed>>,
    config: Res<MovementConfig>,
    mut players: Query<(&mut PlayerPosition, &ActionState<MoveInput>)>,
) {
    let delta_secs = time.timestep().as_secs_f32();
    for (mut position, input) in &mut players {
        let next = next_position(position.0, input, delta_secs, &config);
        // Writing only on change keeps idle players out of network updates.
        if next != position.0 {
            position.0 = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const TICK_RATE_HZ: f64 = 20.0;
    const ONE_SECOND_OF_TICKS: u32 = 20;
    const CONFIG: MovementConfig = MovementConfig {
        speed: 7.0,
        world_half_extent: 50.0,
    };

    fn test_app(config: MovementConfig) -> App {
        let mut app = App::new();
        app.insert_resource(Time::<Fixed>::from_hz(TICK_RATE_HZ));
        app.add_plugins(MovementServerPlugin { config });
        app
    }

    fn spawn_player(app: &mut App, input: MoveInput) -> Entity {
        app.world_mut()
            .spawn((PlayerPosition::default(), ActionState(input)))
            .id()
    }

    fn run_ticks(app: &mut App, ticks: u32) {
        for _ in 0..ticks {
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn position_of(app: &App, player: Entity) -> Vec3 {
        app.world().get::<PlayerPosition>(player).unwrap().0
    }

    #[spec]
    fn server_plugin() {
        given("a player holding forward", || {
            let mut app = test_app(CONFIG);
            let forward = MoveInput {
                forward: true,
                ..default()
            };
            let player = spawn_player(&mut app, forward);

            when("one second of ticks runs", || {
                run_ticks(&mut app, ONE_SECOND_OF_TICKS);

                test("the player has moved the configured speed", || {
                    let moved = position_of(&app, player).length();
                    assert!((moved - CONFIG.speed).abs() < 0.001, "moved {moved}");
                });
            });
        });

        given("a plugin configured with double the speed", || {
            let mut app = test_app(MovementConfig {
                speed: 14.0,
                ..CONFIG
            });
            let forward = MoveInput {
                forward: true,
                ..default()
            };
            let player = spawn_player(&mut app, forward);

            when("one second of ticks runs", || {
                run_ticks(&mut app, ONE_SECOND_OF_TICKS);

                test("the player has moved twice as far", || {
                    let moved = position_of(&app, player).length();
                    assert!((moved - 14.0).abs() < 0.001, "moved {moved}");
                });
            });
        });

        given("one player holding right and one idle player", || {
            let mut app = test_app(CONFIG);
            let right = MoveInput {
                right: true,
                ..default()
            };
            let walking = spawn_player(&mut app, right);
            let idle = spawn_player(&mut app, MoveInput::default());

            when("five ticks run", || {
                run_ticks(&mut app, 5);

                test("the walking player moves", || {
                    assert!(position_of(&app, walking).x > 0.0);
                });

                test("the idle player stays still", || {
                    assert_eq!(position_of(&app, idle), Vec3::ZERO);
                });
            });
        });
    }
}
