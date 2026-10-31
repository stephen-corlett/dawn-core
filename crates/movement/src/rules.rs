use crate::{MoveInput, MovementConfig};
use bevy::prelude::*;

/// Returns where a player ends up after `delta_secs` of input.
pub fn next_position(
    position: Vec3,
    input: &MoveInput,
    delta_secs: f32,
    config: &MovementConfig,
) -> Vec3 {
    let direction = Vec3::new(
        axis(input.right, input.left),
        0.0,
        axis(input.back, input.forward),
    )
    .normalize_or_zero();

    let moved = position + direction * config.speed * delta_secs;
    let extent = config.world_half_extent;
    moved.clamp(
        Vec3::new(-extent, f32::NEG_INFINITY, -extent),
        Vec3::new(extent, f32::INFINITY, extent),
    )
}

fn axis(positive: bool, negative: bool) -> f32 {
    f32::from(u8::from(positive)) - f32::from(u8::from(negative))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const CONFIG: MovementConfig = MovementConfig {
        speed: 10.0,
        world_half_extent: 50.0,
    };
    const DELTA: f32 = 0.1;
    const STEP: f32 = CONFIG.speed * DELTA;

    fn step(position: Vec3, input: &MoveInput) -> Vec3 {
        next_position(position, input, DELTA, &CONFIG)
    }

    #[spec]
    fn movement_rules() {
        given("a player with no input", || {
            let start = Vec3::new(1.0, 0.0, 2.0);

            when("one step runs", || {
                let moved = step(start, &MoveInput::default());

                test("the player stays where they were", || {
                    assert_eq!(moved, start);
                });
            });
        });

        given("a player holding forward", || {
            let input = MoveInput {
                forward: true,
                ..default()
            };

            when("one step runs", || {
                let moved = step(Vec3::ZERO, &input);

                test("the player moves along negative Z", || {
                    assert_eq!(moved, Vec3::new(0.0, 0.0, -STEP));
                });
            });
        });

        given("a player holding left and right", || {
            let input = MoveInput {
                left: true,
                right: true,
                ..default()
            };

            when("one step runs", || {
                let moved = step(Vec3::ZERO, &input);

                test("the player doesn't move", || {
                    assert_eq!(moved, Vec3::ZERO);
                });
            });
        });

        given("a player holding forward and right", || {
            let input = MoveInput {
                forward: true,
                right: true,
                ..default()
            };

            when("one step runs", || {
                let distance = step(Vec3::ZERO, &input).length();

                test("the distance matches one straight step", || {
                    assert!((distance - STEP).abs() < 0.0001);
                });
            });
        });

        given("a player at the world edge, holding outward", || {
            let input = MoveInput {
                right: true,
                ..default()
            };
            let edge = Vec3::new(CONFIG.world_half_extent, 0.0, 0.0);

            when("one step runs", || {
                let moved = step(edge, &input);

                test("the player stays at the edge", || {
                    assert_eq!(moved, edge);
                });
            });
        });

        given("a player with a speed of 20 holding right", || {
            let input = MoveInput {
                right: true,
                ..default()
            };
            let fast = MovementConfig {
                speed: 20.0,
                ..CONFIG
            };

            when("half a second passes", || {
                let moved = next_position(Vec3::ZERO, &input, 0.5, &fast);

                test("the player has moved 10 units", || {
                    assert_eq!(moved, Vec3::new(10.0, 0.0, 0.0));
                });
            });
        });
    }
}
