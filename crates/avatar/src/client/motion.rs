//! Works out how a body moves from where the simulation puts it. Ticks run at 20 Hz but
//! frames run much faster, so smoothing stops characters visibly stepping between ticks.

use super::AvatarSystems;
use super::appearance::Avatar;
use bevy::math::ops;
use bevy::prelude::*;
use movement::{PlayerPosition, Surface};

/// How a body follows its simulated position. Higher smoothing values follow more tightly.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct MotionConfig {
    pub position_smoothing: f32,
    pub speed_smoothing: f32,
    pub turn_smoothing: f32,
    /// A body must move at least this fast to turn toward where it's going.
    pub turn_speed: f32,
    /// Feet this far above the ground count as being in the air.
    pub airborne_height: f32,
}

/// What a body is doing right now, as seen from the outside.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Stride {
    /// Speed across the ground, in world units per second.
    pub speed: f32,
    pub airborne: bool,
}

pub(super) struct MotionPlugin {
    pub config: MotionConfig,
}

impl Plugin for MotionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.add_systems(Update, move_bodies.in_set(AvatarSystems::Move));
    }
}

fn move_bodies(
    time: Res<Time>,
    config: Res<MotionConfig>,
    surface: Res<Surface>,
    mut bodies: Query<(&PlayerPosition, &mut Transform, &mut Stride), With<Avatar>>,
) {
    let delta_secs = time.delta_secs();
    if delta_secs <= 0.0 {
        return;
    }
    for (position, mut transform, mut stride) in &mut bodies {
        let before = transform.translation;
        transform
            .translation
            .smooth_nudge(&position.0, config.position_smoothing, delta_secs);
        let velocity = (transform.translation - before).xz() / delta_secs;

        stride
            .speed
            .smooth_nudge(&velocity.length(), config.speed_smoothing, delta_secs);
        let ground = surface.0.ground_height(position.0.xz());
        stride.airborne = position.0.y - ground > config.airborne_height;

        if let Some(facing) = facing(velocity, config.turn_speed) {
            transform
                .rotation
                .smooth_nudge(&facing, config.turn_smoothing, delta_secs);
        }
    }
}

/// The rotation that faces along a velocity on the XZ plane, if it's fast enough to
/// have a clear direction. Facing negative Z is no rotation.
fn facing(velocity: Vec2, turn_speed: f32) -> Option<Quat> {
    (velocity.length() >= turn_speed)
        .then(|| Quat::from_rotation_y(ops::atan2(-velocity.x, -velocity.y)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;
    use core::f32::consts::FRAC_PI_2;
    use core::time::Duration;
    use movement::{FlatSurface, WalkSurface};
    use std::sync::Arc;

    const CONFIG: MotionConfig = MotionConfig {
        position_smoothing: 18.0,
        speed_smoothing: 12.0,
        turn_smoothing: 12.0,
        turn_speed: 0.5,
        airborne_height: 0.2,
    };
    const FRAME: Duration = Duration::from_millis(16);

    /// Level ground at a height of three.
    struct Plateau;

    impl WalkSurface for Plateau {
        fn ground_height(&self, _at: Vec2) -> f32 {
            3.0
        }

        fn push_out(&self, at: Vec2, _radius: f32) -> Vec2 {
            at
        }
    }

    fn motion_app(surface: impl WalkSurface) -> App {
        let mut app = App::new();
        app.add_plugins(MotionPlugin { config: CONFIG });
        app.insert_resource(Surface(Arc::new(surface)));
        app.insert_resource(Time::<()>::default());
        app
    }

    fn spawn_body(app: &mut App, position: Vec3) -> Entity {
        app.world_mut()
            .spawn((
                Avatar,
                PlayerPosition(position),
                Transform::from_translation(position),
                Stride::default(),
            ))
            .id()
    }

    fn run_frames(app: &mut App, frames: u32) {
        for _ in 0..frames {
            app.world_mut().resource_mut::<Time>().advance_by(FRAME);
            app.world_mut().run_schedule(Update);
        }
    }

    /// Walks a body along negative X at a steady speed.
    fn walk(app: &mut App, body: Entity, speed: f32, frames: u32) {
        for _ in 0..frames {
            app.world_mut().get_mut::<PlayerPosition>(body).unwrap().0 +=
                Vec3::NEG_X * speed * FRAME.as_secs_f32();
            run_frames(app, 1);
        }
    }

    #[spec]
    fn facing_rules() {
        given("a body moving along negative Z", || {
            let velocity = Vec2::new(0.0, -5.0);

            when("its facing is worked out", || {
                let rotation = facing(velocity, 0.5).unwrap();

                test("it needs no rotation", || {
                    assert!(rotation.angle_between(Quat::IDENTITY) < 0.001);
                });
            });
        });

        given("a body moving along negative X", || {
            let velocity = Vec2::new(-5.0, 0.0);

            when("its facing is worked out", || {
                let rotation = facing(velocity, 0.5).unwrap();

                test("it faces a quarter turn left", || {
                    let left = Quat::from_rotation_y(FRAC_PI_2);
                    assert!(rotation.angle_between(left) < 0.001);
                });
            });
        });

        given("a body barely moving", || {
            let velocity = Vec2::new(0.1, 0.1);

            when("its facing is worked out", || {
                let rotation = facing(velocity, 0.5);

                test("it keeps whatever facing it had", || {
                    assert_eq!(rotation, None);
                });
            });
        });
    }

    #[spec]
    fn moving_bodies() {
        given("a body with a simulated position far away", || {
            let mut app = motion_app(FlatSurface);
            let body = spawn_body(&mut app, Vec3::ZERO);
            app.world_mut().get_mut::<PlayerPosition>(body).unwrap().0 = Vec3::new(5.0, 0.0, 0.0);

            when("one frame runs", || {
                run_frames(&mut app, 1);
                let moved = app.world().get::<Transform>(body).unwrap().translation.x;

                test("it glides rather than jumps", || {
                    assert!(moved > 0.0 && moved < 5.0, "moved {moved}");
                });
            });
        });

        given("a body walking along", || {
            let mut app = motion_app(FlatSurface);
            let body = spawn_body(&mut app, Vec3::ZERO);

            when("it walks for a while", || {
                walk(&mut app, body, 5.0, 120);

                test("its measured speed settles on its real speed", || {
                    let speed = app.world().get::<Stride>(body).unwrap().speed;
                    assert!((speed - 5.0).abs() < 0.2, "speed {speed}");
                });

                test("it turns to face where it's going", || {
                    let rotation = app.world().get::<Transform>(body).unwrap().rotation;
                    let left = Quat::from_rotation_y(FRAC_PI_2);
                    assert!(rotation.angle_between(left) < 0.05);
                });
            });

            when("it stops", || {
                walk(&mut app, body, 5.0, 120);
                run_frames(&mut app, 60);
                let rotation = app.world().get::<Transform>(body).unwrap().rotation;
                run_frames(&mut app, 120);

                test("its measured speed falls to nothing", || {
                    let speed = app.world().get::<Stride>(body).unwrap().speed;
                    assert!(speed < 0.1, "speed {speed}");
                });

                test("it keeps facing the same way", || {
                    let now = app.world().get::<Transform>(body).unwrap().rotation;
                    assert!(now.angle_between(rotation) < 0.05);
                });
            });
        });

        given("a body standing on raised ground", || {
            let mut app = motion_app(Plateau);
            let body = spawn_body(&mut app, Vec3::new(0.0, 3.0, 0.0));

            when("one frame runs", || {
                run_frames(&mut app, 1);

                test("it counts as grounded", || {
                    assert!(!app.world().get::<Stride>(body).unwrap().airborne);
                });
            });
        });

        given("a body above the ground", || {
            let mut app = motion_app(Plateau);
            let body = spawn_body(&mut app, Vec3::new(0.0, 4.0, 0.0));

            when("one frame runs", || {
                run_frames(&mut app, 1);

                test("it counts as airborne", || {
                    assert!(app.world().get::<Stride>(body).unwrap().airborne);
                });
            });
        });

        given("a paused game", || {
            let mut app = motion_app(FlatSurface);
            let body = spawn_body(&mut app, Vec3::ZERO);
            app.world_mut().get_mut::<PlayerPosition>(body).unwrap().0 = Vec3::X;

            when("a frame runs with no time passing", || {
                app.world_mut().run_schedule(Update);

                test("nothing moves", || {
                    let moved = app.world().get::<Transform>(body).unwrap().translation;
                    assert_eq!(moved, Vec3::ZERO);
                });
            });
        });
    }
}
