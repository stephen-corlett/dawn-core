use crate::{Dodge, Gait, MoveInput, MovementConfig, WalkSurface};
use bevy::prelude::*;

/// The parts of a player that movement simulates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub position: Vec3,
    pub vertical_speed: f32,
    pub dodge: Dodge,
    pub sneaking: bool,
}

/// Returns where a player's body ends up after `delta_secs` of input.
pub fn next_body(
    body: Body,
    input: &MoveInput,
    delta_secs: f32,
    config: &MovementConfig,
    surface: &dyn WalkSurface,
) -> Body {
    let dodge = next_dodge(&body, input, delta_secs, config, surface);
    let flat = travel_to(body.position, &dodge, input, delta_secs, config, surface);
    let ground = surface.ground_height(flat);

    let grounded = body.vertical_speed == 0.0 && body.position.y - ground <= config.ground_snap;
    let jumping = grounded && input.jump && !dodge.is_committed();
    let vertical_speed = match (grounded, jumping) {
        (true, true) => config.jump_speed,
        (true, false) => 0.0,
        (false, _) => body.vertical_speed - config.gravity * delta_secs,
    };
    let y = if grounded && !jumping {
        ground
    } else {
        (body.position.y + vertical_speed * delta_secs).max(ground)
    };

    let landed = y <= ground && vertical_speed < 0.0;
    Body {
        position: Vec3::new(flat.x, y, flat.y),
        vertical_speed: if landed { 0.0 } else { vertical_speed },
        dodge,
        sneaking: input.gait == Gait::Sneak,
    }
}

/// Counts down any roll or recovery underway, and starts a new roll if the player can.
/// A roll needs a direction, so a player standing still can't start one.
fn next_dodge(
    body: &Body,
    input: &MoveInput,
    delta_secs: f32,
    config: &MovementConfig,
    surface: &dyn WalkSurface,
) -> Dodge {
    let dodge = body.dodge;
    let direction = steering(input);
    let ground = surface.ground_height(body.position.xz());
    let on_ground = body.vertical_speed == 0.0 && body.position.y - ground <= config.ground_snap;
    if input.dodge && on_ground && !dodge.is_committed() && direction != Vec2::ZERO {
        return Dodge {
            direction,
            remaining_secs: config.dodge_secs,
            recovery_secs: 0.0,
        };
    }
    let remaining_secs = (dodge.remaining_secs - delta_secs).max(0.0);
    let roll_just_ended = dodge.is_rolling() && remaining_secs == 0.0;
    Dodge {
        remaining_secs,
        recovery_secs: if roll_just_ended {
            config.dodge_recovery_secs
        } else {
            (dodge.recovery_secs - delta_secs).max(0.0)
        },
        ..dodge
    }
}

/// Returns where a player's feet end up on the XZ plane.
fn travel_to(
    position: Vec3,
    dodge: &Dodge,
    input: &MoveInput,
    delta_secs: f32,
    config: &MovementConfig,
    surface: &dyn WalkSurface,
) -> Vec2 {
    let velocity = if dodge.is_rolling() {
        dodge.direction * config.dodge_speed
    } else if dodge.is_recovering() {
        steering(input) * gait_speed(input.gait, config).min(config.dodge_recovery_speed)
    } else {
        steering(input) * gait_speed(input.gait, config)
    };
    let moved = position.xz() + velocity * delta_secs;
    let extent = config.world_half_extent;
    surface
        .push_out(moved, config.body_radius)
        .clamp(Vec2::splat(-extent), Vec2::splat(extent))
}

/// The direction a player is steering on the XZ plane, or zero if they aren't.
fn steering(input: &MoveInput) -> Vec2 {
    let held = Vec3::new(
        axis(input.right, input.left),
        0.0,
        axis(input.back, input.forward),
    );
    // Normalizing last also turns a yaw that isn't a number into no movement.
    (Quat::from_rotation_y(input.yaw) * held)
        .normalize_or_zero()
        .xz()
}

fn gait_speed(gait: Gait, config: &MovementConfig) -> f32 {
    match gait {
        Gait::Sneak => config.sneak_speed,
        Gait::Run => config.run_speed,
        Gait::Sprint => config.sprint_speed,
    }
}

fn axis(positive: bool, negative: bool) -> f32 {
    f32::from(u8::from(positive)) - f32::from(u8::from(negative))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlatSurface;
    use bdd::spec;
    use core::f32::consts::FRAC_PI_2;

    const CONFIG: MovementConfig = MovementConfig {
        sneak_speed: 2.0,
        run_speed: 10.0,
        sprint_speed: 16.0,
        dodge_speed: 12.0,
        dodge_secs: 0.5,
        dodge_recovery_secs: 0.3,
        dodge_recovery_speed: 1.0,
        jump_speed: 8.0,
        gravity: 20.0,
        ground_snap: 0.5,
        body_radius: 0.4,
        world_half_extent: 50.0,
    };
    const DELTA: f32 = 0.1;
    const STEP: f32 = CONFIG.run_speed * DELTA;

    /// Ground that rises one unit for every unit east of the origin.
    struct Ramp;

    impl WalkSurface for Ramp {
        fn ground_height(&self, at: Vec2) -> f32 {
            at.x
        }

        fn push_out(&self, at: Vec2, _radius: f32) -> Vec2 {
            at
        }
    }

    /// A solid wall along the line x = 1.
    struct Wall;

    impl WalkSurface for Wall {
        fn ground_height(&self, _at: Vec2) -> f32 {
            0.0
        }

        fn push_out(&self, at: Vec2, radius: f32) -> Vec2 {
            Vec2::new(at.x.min(1.0 - radius), at.y)
        }
    }

    fn standing_at(position: Vec3) -> Body {
        Body {
            position,
            vertical_speed: 0.0,
            dodge: Dodge::default(),
            sneaking: false,
        }
    }

    fn step(body: Body, input: &MoveInput) -> Body {
        next_body(body, input, DELTA, &CONFIG, &FlatSurface)
    }

    fn step_position(position: Vec3, input: &MoveInput) -> Vec3 {
        step(standing_at(position), input).position
    }

    #[spec]
    fn walking_rules() {
        given("a player with no input", || {
            let start = Vec3::new(1.0, 0.0, 2.0);

            when("one step runs", || {
                let moved = step_position(start, &MoveInput::default());

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
                let moved = step_position(Vec3::ZERO, &input);

                test("the player moves along negative Z", || {
                    assert_eq!(moved, Vec3::new(0.0, 0.0, -STEP));
                });
            });
        });

        given(
            "a player holding forward with a yaw of a quarter turn left",
            || {
                let input = MoveInput {
                    forward: true,
                    yaw: FRAC_PI_2,
                    ..default()
                };

                when("one step runs", || {
                    let moved = step_position(Vec3::ZERO, &input);

                    test("the player moves along negative X", || {
                        assert!(moved.distance(Vec3::new(-STEP, 0.0, 0.0)) < 0.0001);
                    });
                });
            },
        );

        given("a player sending a yaw that isn't a number", || {
            let input = MoveInput {
                forward: true,
                yaw: f32::NAN,
                ..default()
            };

            when("one step runs", || {
                let moved = step_position(Vec3::ZERO, &input);

                test("the player stays where they were", || {
                    assert_eq!(moved, Vec3::ZERO);
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
                let moved = step_position(Vec3::ZERO, &input);

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
                let distance = step_position(Vec3::ZERO, &input).length();

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
                let moved = step_position(edge, &input);

                test("the player stays at the edge", || {
                    assert_eq!(moved, edge);
                });
            });
        });

        given("a player with a run speed of 20 holding right", || {
            let input = MoveInput {
                right: true,
                ..default()
            };
            let fast = MovementConfig {
                run_speed: 20.0,
                ..CONFIG
            };

            when("half a second passes", || {
                let moved = next_body(standing_at(Vec3::ZERO), &input, 0.5, &fast, &FlatSurface);

                test("the player has moved 10 units", || {
                    assert_eq!(moved.position, Vec3::new(10.0, 0.0, 0.0));
                });
            });
        });

        given("a player asking to sneak", || {
            let input = MoveInput {
                right: true,
                gait: Gait::Sneak,
                ..default()
            };

            when("one step runs", || {
                let moved = step(standing_at(Vec3::ZERO), &input);

                test("the player moves at sneaking speed", || {
                    assert_eq!(moved.position.x, CONFIG.sneak_speed * DELTA);
                });

                test("the player counts as sneaking", || {
                    assert!(moved.sneaking);
                });
            });
        });

        given("a player asking to sprint", || {
            let input = MoveInput {
                right: true,
                gait: Gait::Sprint,
                ..default()
            };

            when("one step runs", || {
                let moved = step_position(Vec3::ZERO, &input);

                test("the player moves at sprinting speed", || {
                    assert_eq!(moved.x, CONFIG.sprint_speed * DELTA);
                });
            });
        });

        given("a player running at a wall", || {
            let input = MoveInput {
                right: true,
                forward: true,
                ..default()
            };
            let start = Vec3::new(1.0 - CONFIG.body_radius, 0.0, 0.0);

            when("one step runs", || {
                let moved = next_body(standing_at(start), &input, DELTA, &CONFIG, &Wall);

                test("the wall stops them", || {
                    assert_eq!(moved.position.x, start.x);
                });

                test("they slide along it", || {
                    assert!(moved.position.z < 0.0);
                });
            });
        });
    }

    #[spec]
    fn ground_rules() {
        given("a player walking up a slope", || {
            let input = MoveInput {
                right: true,
                ..default()
            };

            when("one step runs", || {
                let moved = next_body(standing_at(Vec3::ZERO), &input, DELTA, &CONFIG, &Ramp);

                test("their feet stay on the ground", || {
                    assert_eq!(moved.position.y, moved.position.x);
                });
            });
        });

        given(
            "a player going down a slope slowly enough to stay on it",
            || {
                let input = MoveInput {
                    left: true,
                    gait: Gait::Sneak,
                    ..default()
                };

                when("one step runs", || {
                    let moved = next_body(standing_at(Vec3::ZERO), &input, DELTA, &CONFIG, &Ramp);

                    test("they follow the ground instead of falling", || {
                        assert_eq!(moved.vertical_speed, 0.0);
                        assert_eq!(moved.position.y, moved.position.x);
                    });
                });
            },
        );

        given("a player running off a drop too steep to follow", || {
            let input = MoveInput {
                left: true,
                gait: Gait::Sprint,
                ..default()
            };

            when("one step runs", || {
                let moved = next_body(standing_at(Vec3::ZERO), &input, DELTA, &CONFIG, &Ramp);

                test("they fall", || {
                    assert!(moved.vertical_speed < 0.0);
                    assert!(moved.position.y > moved.position.x);
                });
            });
        });

        given("a player standing with the ground far below them", || {
            let start = standing_at(Vec3::new(0.0, 10.0, 0.0));

            when("one step runs", || {
                let moved = step(start, &MoveInput::default());

                test("they start to fall", || {
                    assert!(moved.vertical_speed < 0.0);
                    assert!(moved.position.y < start.position.y);
                });
            });
        });
    }

    #[spec]
    fn dodging_rules() {
        given("a running player pressing dodge", || {
            let input = MoveInput {
                right: true,
                dodge: true,
                ..default()
            };

            when("one step runs", || {
                let moved = step(standing_at(Vec3::ZERO), &input);

                test("they roll the way they were moving", || {
                    assert_eq!(moved.dodge.direction, Vec2::X);
                    assert!(moved.dodge.is_rolling());
                });

                test("they move at rolling speed", || {
                    assert_eq!(moved.position.x, CONFIG.dodge_speed * DELTA);
                });
            });
        });

        given("a player moving relative to a turned camera", || {
            let input = MoveInput {
                forward: true,
                dodge: true,
                yaw: FRAC_PI_2,
                ..default()
            };

            when("they dodge", || {
                let moved = step(standing_at(Vec3::ZERO), &input);

                test("the roll follows the camera too", || {
                    assert!(moved.dodge.direction.distance(Vec2::NEG_X) < 0.0001);
                });
            });
        });

        given("a player standing still pressing dodge", || {
            let input = MoveInput {
                dodge: true,
                ..default()
            };

            when("one step runs", || {
                let moved = step(standing_at(Vec3::ZERO), &input);

                test("nothing happens", || {
                    assert!(!moved.dodge.is_rolling());
                    assert_eq!(moved.position, Vec3::ZERO);
                });
            });
        });

        given("a player in the air pressing dodge", || {
            let input = MoveInput {
                right: true,
                dodge: true,
                ..default()
            };
            let airborne = Body {
                vertical_speed: 1.0,
                ..standing_at(Vec3::new(0.0, 3.0, 0.0))
            };

            when("one step runs", || {
                let moved = step(airborne, &input);

                test("they can't roll", || {
                    assert!(!moved.dodge.is_rolling());
                });
            });
        });

        given("a player partway through a roll", || {
            let rolling = step(
                standing_at(Vec3::ZERO),
                &MoveInput {
                    right: true,
                    dodge: true,
                    ..default()
                },
            );

            when("they steer the other way", || {
                let steered = MoveInput {
                    left: true,
                    ..default()
                };
                let moved = step(rolling, &steered);

                test("the roll carries on", || {
                    assert!(moved.position.x > rolling.position.x);
                });
            });

            when("they stop steering", || {
                let moved = step(rolling, &MoveInput::default());

                test("the roll carries on", || {
                    assert!(moved.position.x > rolling.position.x);
                });

                test("recovery hasn't started yet", || {
                    assert!(!moved.dodge.is_recovering());
                });
            });

            when("they press jump", || {
                let jump = MoveInput {
                    jump: true,
                    ..default()
                };
                let moved = step(rolling, &jump);

                test("they stay on the ground", || {
                    assert_eq!(moved.vertical_speed, 0.0);
                });
            });

            when("they press dodge again", || {
                let again = MoveInput {
                    left: true,
                    dodge: true,
                    ..default()
                };
                let moved = step(rolling, &again);

                test("the roll doesn't restart", || {
                    assert_eq!(moved.dodge.direction, Vec2::X);
                    assert!(moved.dodge.remaining_secs < rolling.dodge.remaining_secs);
                });
            });
        });

        given("a player who has just finished rolling", || {
            let press = MoveInput {
                right: true,
                dodge: true,
                ..default()
            };
            let hold = MoveInput {
                right: true,
                ..default()
            };
            let mut body = step(standing_at(Vec3::ZERO), &press);
            while body.dodge.is_rolling() {
                body = step(body, &hold);
            }

            when("they press dodge straight away", || {
                let moved = step(body, &press);

                test("recovery stops them", || {
                    assert!(!moved.dodge.is_rolling());
                });
            });

            when("they keep running", || {
                let start = body.position.x;
                let moved = step(body, &hold);

                test("they move at recovery speed", || {
                    let moved_by = moved.position.x - start;
                    assert!((moved_by - CONFIG.dodge_recovery_speed * DELTA).abs() < 0.0001);
                });
            });

            when("they try to sprint out of it", || {
                let sprint = MoveInput {
                    right: true,
                    gait: Gait::Sprint,
                    ..default()
                };
                let start = body.position.x;
                let moved = step(body, &sprint);

                test("sprinting doesn't help", || {
                    let moved_by = moved.position.x - start;
                    assert!((moved_by - CONFIG.dodge_recovery_speed * DELTA).abs() < 0.0001);
                });
            });

            when("they try to jump out of it", || {
                let jump = MoveInput {
                    jump: true,
                    ..default()
                };
                let moved = step(body, &jump);

                test("they stay on the ground", || {
                    assert_eq!(moved.vertical_speed, 0.0);
                });
            });

            when("something forgives the roll", || {
                body.dodge.recovery_secs = 0.0;
                let start = body.position.x;
                let moved = step(body, &hold);

                test("they're back to full speed", || {
                    let moved_by = moved.position.x - start;
                    assert!((moved_by - CONFIG.run_speed * DELTA).abs() < 0.0001);
                });
            });

            when("recovery runs out", || {
                for _ in 0..5 {
                    body = step(body, &hold);
                }
                let start = body.position.x;
                let moved = step(body, &hold);

                test("they're back to full speed", || {
                    let moved_by = moved.position.x - start;
                    assert!((moved_by - CONFIG.run_speed * DELTA).abs() < 0.0001);
                });
            });

            when("they wait out recovery", || {
                for _ in 0..5 {
                    body = step(body, &hold);
                }
                let moved = step(body, &press);

                test("they can roll again", || {
                    assert!(moved.dodge.is_rolling());
                });
            });
        });

        given("a roll with a known length", || {
            let press = MoveInput {
                right: true,
                dodge: true,
                ..default()
            };
            let mut body = step(standing_at(Vec3::ZERO), &press);

            when("it finishes", || {
                let mut rolling_steps = 0;
                while body.dodge.is_rolling() {
                    rolling_steps += 1;
                    body = step(body, &MoveInput::default());
                }

                test(
                    "it lasts as long as the config says, to within a tick",
                    || {
                        let lasted = rolling_steps as f32 * DELTA;
                        assert!(
                            (lasted - CONFIG.dodge_secs).abs() <= DELTA * 1.01,
                            "lasted {lasted}"
                        );
                    },
                );
            });
        });
    }

    #[spec]
    fn jumping_rules() {
        given("a grounded player holding jump", || {
            let input = MoveInput {
                jump: true,
                ..default()
            };

            when("one step runs", || {
                let moved = step(standing_at(Vec3::ZERO), &input);

                test("they leave the ground", || {
                    assert_eq!(moved.vertical_speed, CONFIG.jump_speed);
                    assert!(moved.position.y > 0.0);
                });
            });

            when("the jump plays out", || {
                let mut body = step(standing_at(Vec3::ZERO), &input);
                let mut peak = 0.0_f32;
                let released = MoveInput::default();
                for _ in 0..20 {
                    body = step(body, &released);
                    peak = peak.max(body.position.y);
                }

                test("they rise and come back to the ground", || {
                    assert!(peak > 0.5, "peak {peak}");
                    assert_eq!(body.position.y, 0.0);
                    assert_eq!(body.vertical_speed, 0.0);
                });
            });
        });

        given("a player in the air holding jump", || {
            let rising = Body {
                vertical_speed: 2.0,
                ..standing_at(Vec3::new(0.0, 1.0, 0.0))
            };
            let input = MoveInput {
                jump: true,
                ..default()
            };

            when("one step runs", || {
                let moved = step(rising, &input);

                test("they can't jump again", || {
                    assert!(moved.vertical_speed < rising.vertical_speed);
                });
            });
        });

        given(
            "a player rising into ground that climbs faster than they do",
            || {
                let rising = Body {
                    vertical_speed: 1.0,
                    ..standing_at(Vec3::new(5.0, 5.0, 0.0))
                };
                let input = MoveInput {
                    right: true,
                    ..default()
                };

                when("one step runs", || {
                    let moved = next_body(rising, &input, DELTA, &CONFIG, &Ramp);

                    test("the ground lifts them", || {
                        assert_eq!(moved.position.y, moved.position.x);
                    });
                });
            },
        );
    }
}
