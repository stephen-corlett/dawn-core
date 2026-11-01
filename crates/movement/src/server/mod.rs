//! Authoritative movement. Nothing here touches networking, so these rules
//! can be tested by placing inputs directly on entities.

use crate::simulation::SimulatedBody;
use crate::{MoveInput, MovementConfig, Surface};
use bevy::prelude::*;
use lightyear::prelude::input::native::ActionState;

pub struct MovementServerPlugin {
    pub config: MovementConfig,
    pub surface: Surface,
}

impl Plugin for MovementServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.insert_resource(self.surface.clone());
        app.add_systems(FixedUpdate, apply_player_movement);
    }
}

fn apply_player_movement(
    time: Res<Time<Fixed>>,
    config: Res<MovementConfig>,
    surface: Res<Surface>,
    mut players: Query<(SimulatedBody, &ActionState<MoveInput>)>,
) {
    let delta_secs = time.timestep().as_secs_f32();
    for (mut body, input) in &mut players {
        body.advance(input, delta_secs, &config, surface.0.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dodge, FlatSurface, Gait, PlayerPosition, Sneaking, VerticalSpeed};
    use bdd::spec;
    use std::sync::Arc;

    const TICK_RATE_HZ: f64 = 20.0;
    const ONE_SECOND_OF_TICKS: u32 = 20;
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

    fn test_app(config: MovementConfig) -> App {
        let mut app = App::new();
        app.insert_resource(Time::<Fixed>::from_hz(TICK_RATE_HZ));
        app.add_plugins(MovementServerPlugin {
            config,
            surface: Surface(Arc::new(FlatSurface)),
        });
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
                    assert!((moved - CONFIG.run_speed).abs() < 0.001, "moved {moved}");
                });
            });
        });

        given("a plugin configured with double the speed", || {
            let mut app = test_app(MovementConfig {
                run_speed: 14.0,
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

        given("a moving player holding dodge", || {
            let mut app = test_app(CONFIG);
            let roll = MoveInput {
                forward: true,
                dodge: true,
                ..default()
            };
            let player = spawn_player(&mut app, roll);

            when("one tick runs", || {
                run_ticks(&mut app, 1);

                test("the server starts the roll", || {
                    assert!(app.world().get::<Dodge>(player).unwrap().is_rolling());
                });
            });
        });

        given("a player asking to sneak", || {
            let mut app = test_app(CONFIG);
            let sneak = MoveInput {
                gait: Gait::Sneak,
                ..default()
            };
            let player = spawn_player(&mut app, sneak);

            when("one tick runs", || {
                run_ticks(&mut app, 1);

                test("the server marks them as sneaking", || {
                    assert!(app.world().get::<Sneaking>(player).unwrap().0);
                });
            });
        });

        given("a player holding jump", || {
            let mut app = test_app(CONFIG);
            let jump = MoveInput {
                jump: true,
                ..default()
            };
            let player = spawn_player(&mut app, jump);

            when("one tick runs", || {
                run_ticks(&mut app, 1);

                test("the player is rising", || {
                    let speed = app.world().get::<VerticalSpeed>(player).unwrap().0;
                    assert_eq!(speed, CONFIG.jump_speed);
                });

                test("the player has left the ground", || {
                    assert!(position_of(&app, player).y > 0.0);
                });
            });
        });
    }
}
