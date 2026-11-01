//! A third-person camera that orbits the local player. Holding the orbit button turns
//! it, and "forward" for the player always means the way the camera faces.

use super::AvatarSystems;
use super::appearance::Avatar;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use lightyear::prelude::*;
use movement::{MoveHeading, Surface};

/// How the camera handles. The game supplies these, so they can later come from a
/// player's settings.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct CameraConfig {
    /// Held down to turn the camera with the mouse.
    pub orbit_button: MouseButton,
    pub radians_per_pixel: f32,
    /// How far one notch of the scroll wheel zooms.
    pub zoom_per_notch: f32,
    pub start_distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    /// Radians above level. Starting positive looks down on the player.
    pub start_pitch: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    /// The height above a player's feet that the camera looks at.
    pub focus_height: f32,
    /// Higher values follow more tightly.
    pub follow_smoothing: f32,
    pub zoom_smoothing: f32,
    /// The camera stays this far above the ground.
    pub ground_clearance: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct ThirdPersonCamera {
    yaw: f32,
    pitch: f32,
    /// How far away the player has asked the camera to be.
    distance: f32,
    /// How far away it is now, on its way to `distance`.
    current_distance: f32,
    /// The point the camera is looking at. It's `None` until there's a player to find.
    focus: Option<Vec3>,
}

pub(super) struct CameraPlugin {
    pub config: CameraConfig,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.init_resource::<MoveHeading>();
        app.add_systems(PostStartup, adopt_camera);
        app.add_systems(
            Update,
            (grab_cursor, turn_camera, follow_local_player)
                .chain()
                .in_set(AvatarSystems::Present),
        );
    }
}

/// Takes over a camera if something else already made one, such as a sandbox.
fn adopt_camera(
    mut commands: Commands,
    config: Res<CameraConfig>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    let camera = ThirdPersonCamera {
        yaw: 0.0,
        pitch: config.start_pitch,
        distance: config.start_distance,
        current_distance: config.start_distance,
        focus: None,
    };
    match cameras.iter().next() {
        Some(existing) => {
            commands.entity(existing).insert(camera);
        }
        None => {
            commands.spawn((Camera3d::default(), camera));
        }
    }
}

/// Hides the cursor and pins it in place while the player turns the camera.
fn grab_cursor(
    config: Res<CameraConfig>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let held = if buttons.just_pressed(config.orbit_button) {
        true
    } else if buttons.just_released(config.orbit_button) {
        false
    } else {
        return;
    };
    for mut cursor in &mut cursors {
        cursor.visible = !held;
        cursor.grab_mode = if held {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
}

fn turn_camera(
    config: Res<CameraConfig>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut heading: ResMut<MoveHeading>,
    mut cameras: Query<&mut ThirdPersonCamera>,
) {
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    if buttons.pressed(config.orbit_button) {
        let turned = motion.delta * config.radians_per_pixel;
        camera.yaw -= turned.x;
        camera.pitch = (camera.pitch + turned.y).clamp(config.min_pitch, config.max_pitch);
    }
    camera.distance = (camera.distance - scroll.delta.y * config.zoom_per_notch)
        .clamp(config.min_distance, config.max_distance);
    heading.0 = camera.yaw;
}

fn follow_local_player(
    time: Res<Time>,
    config: Res<CameraConfig>,
    surface: Res<Surface>,
    player: Query<&Transform, (With<Predicted>, With<Avatar>, Without<ThirdPersonCamera>)>,
    mut cameras: Query<(&mut ThirdPersonCamera, &mut Transform)>,
) {
    let (Ok(player), Ok((mut camera, mut transform))) = (player.single(), cameras.single_mut())
    else {
        return;
    };
    let delta_secs = time.delta_secs();
    let target = player.translation + Vec3::Y * config.focus_height;
    let mut focus = camera.focus.unwrap_or(target);
    focus.smooth_nudge(&target, config.follow_smoothing, delta_secs);
    camera.focus = Some(focus);

    let distance = camera.distance;
    camera
        .current_distance
        .smooth_nudge(&distance, config.zoom_smoothing, delta_secs);

    let mut position = orbit_position(focus, camera.yaw, camera.pitch, camera.current_distance);
    let ground = surface.0.ground_height(position.xz());
    position.y = position.y.max(ground + config.ground_clearance);
    *transform = Transform::from_translation(position).looking_at(focus, Vec3::Y);
}

/// Where a camera sits to look at `focus` from a yaw and pitch. A yaw of zero puts it
/// on the positive Z side, looking toward negative Z.
fn orbit_position(focus: Vec3, yaw: f32, pitch: f32, distance: f32) -> Vec3 {
    focus + Quat::from_euler(EulerRot::YXZ, yaw, -pitch, 0.0) * Vec3::Z * distance
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;
    use core::f32::consts::FRAC_PI_2;
    use core::time::Duration;
    use movement::{FlatSurface, WalkSurface};
    use std::sync::Arc;

    const CONFIG: CameraConfig = CameraConfig {
        orbit_button: MouseButton::Right,
        radians_per_pixel: 0.01,
        zoom_per_notch: 1.0,
        start_distance: 6.0,
        min_distance: 2.0,
        max_distance: 10.0,
        start_pitch: 0.3,
        min_pitch: -0.3,
        max_pitch: 1.0,
        focus_height: 1.5,
        follow_smoothing: 14.0,
        zoom_smoothing: 10.0,
        ground_clearance: 0.5,
    };
    const FRAME: Duration = Duration::from_millis(16);

    /// Ground far above everything else.
    struct HighGround;

    impl WalkSurface for HighGround {
        fn ground_height(&self, _at: Vec2) -> f32 {
            20.0
        }

        fn push_out(&self, at: Vec2, _radius: f32) -> Vec2 {
            at
        }
    }

    fn camera_app(surface: impl WalkSurface) -> App {
        let mut app = App::new();
        app.add_plugins(CameraPlugin { config: CONFIG });
        app.insert_resource(Surface(Arc::new(surface)));
        app.insert_resource(Time::<()>::default());
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<AccumulatedMouseMotion>();
        app.init_resource::<AccumulatedMouseScroll>();
        app.world_mut().run_schedule(PostStartup);
        app.world_mut().flush();
        app
    }

    fn run_frames(app: &mut App, frames: u32) {
        for _ in 0..frames {
            app.world_mut().resource_mut::<Time>().advance_by(FRAME);
            app.world_mut().run_schedule(Update);
        }
    }

    fn state(app: &mut App) -> ThirdPersonCamera {
        *app.world_mut()
            .query::<&ThirdPersonCamera>()
            .single(app.world())
            .expect("there is one camera")
    }

    fn transform(app: &mut App) -> Transform {
        *app.world_mut()
            .query_filtered::<&Transform, With<ThirdPersonCamera>>()
            .single(app.world())
            .expect("there is one camera")
    }

    fn spawn_player(app: &mut App, position: Vec3) -> Entity {
        app.world_mut()
            .spawn((Avatar, Predicted, Transform::from_translation(position)))
            .id()
    }

    fn set_mouse(app: &mut App, delta: Vec2) {
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = delta;
    }

    fn drag_mouse(app: &mut App, delta: Vec2) {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(CONFIG.orbit_button);
        set_mouse(app, delta);
        run_frames(app, 1);
    }

    fn scroll(app: &mut App, notches: f32) {
        app.world_mut()
            .resource_mut::<AccumulatedMouseScroll>()
            .delta
            .y = notches;
        run_frames(app, 1);
    }

    #[spec]
    fn orbiting() {
        given("a focus point", || {
            let focus = Vec3::new(0.0, 2.0, 0.0);

            when("the camera has no yaw or pitch", || {
                let position = orbit_position(focus, 0.0, 0.0, 5.0);

                test("it sits behind the player on positive Z", || {
                    assert!(position.distance(Vec3::new(0.0, 2.0, 5.0)) < 0.001);
                });
            });

            when("the camera has positive pitch", || {
                let position = orbit_position(focus, 0.0, 0.5, 5.0);

                test("it looks down from above", || {
                    assert!(position.y > focus.y);
                });
            });

            when("the camera has a quarter turn of yaw", || {
                let position = orbit_position(focus, FRAC_PI_2, 0.0, 5.0);

                test("it swings around to positive X", || {
                    assert!(position.distance(Vec3::new(5.0, 2.0, 0.0)) < 0.001);
                });
            });

            when("the camera pitches and turns", || {
                let position = orbit_position(focus, 1.0, 0.7, 5.0);

                test("it keeps its distance", || {
                    assert!((position.distance(focus) - 5.0).abs() < 0.001);
                });
            });
        });
    }

    #[spec]
    fn adopting_a_camera() {
        given("a game with no camera", || {
            let mut app = camera_app(FlatSurface);

            when("it starts", || {
                let camera = state(&mut app);

                test("one appears at the configured distance and pitch", || {
                    assert_eq!(camera.distance, CONFIG.start_distance);
                    assert_eq!(camera.pitch, CONFIG.start_pitch);
                });
            });
        });

        given("a sandbox that already made a camera", || {
            let mut app = App::new();
            app.add_plugins(CameraPlugin { config: CONFIG });
            let existing = app.world_mut().spawn(Camera3d::default()).id();

            when("it starts", || {
                app.world_mut().run_schedule(PostStartup);
                app.world_mut().flush();
                let mut cameras = app.world_mut().query::<&Camera3d>();

                test("that camera is taken over", || {
                    assert!(app.world().get::<ThirdPersonCamera>(existing).is_some());
                });

                test("no second camera appears", || {
                    assert_eq!(cameras.iter(app.world()).count(), 1);
                });
            });
        });
    }

    #[spec]
    fn turning_with_the_mouse() {
        given(
            "a player moving the mouse without holding the button",
            || {
                let mut app = camera_app(FlatSurface);
                set_mouse(&mut app, Vec2::new(50.0, 0.0));

                when("a frame runs", || {
                    run_frames(&mut app, 1);

                    test("the camera doesn't turn", || {
                        assert_eq!(state(&mut app).yaw, 0.0);
                    });
                });
            },
        );

        given("a player holding the orbit button", || {
            let mut app = camera_app(FlatSurface);

            when("they drag right", || {
                drag_mouse(&mut app, Vec2::new(100.0, 0.0));

                test("the camera turns right", || {
                    assert_eq!(state(&mut app).yaw, -1.0);
                });

                test("forward now means where the camera faces", || {
                    assert_eq!(app.world().resource::<MoveHeading>().0, -1.0);
                });
            });

            when("they drag down", || {
                drag_mouse(&mut app, Vec2::new(0.0, 10.0));

                test("the camera rises to look further down", || {
                    assert!(state(&mut app).pitch > CONFIG.start_pitch);
                });
            });

            when("they drag too far", || {
                drag_mouse(&mut app, Vec2::new(0.0, 1000.0));

                test("the pitch stops at its limit", || {
                    assert_eq!(state(&mut app).pitch, CONFIG.max_pitch);
                });
            });
        });

        given("a primary window", || {
            let mut app = camera_app(FlatSurface);
            let window = app
                .world_mut()
                .spawn((PrimaryWindow, CursorOptions::default()))
                .id();

            when("the orbit button goes down", || {
                drag_mouse(&mut app, Vec2::ZERO);
                let cursor = app.world().get::<CursorOptions>(window).unwrap();

                test("the cursor hides and stays put", || {
                    assert!(!cursor.visible);
                    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
                });
            });

            when("the orbit button comes back up", || {
                drag_mouse(&mut app, Vec2::ZERO);
                let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
                buttons.clear();
                buttons.release(CONFIG.orbit_button);
                run_frames(&mut app, 1);
                let cursor = app.world().get::<CursorOptions>(window).unwrap();

                test("the cursor comes back", || {
                    assert!(cursor.visible);
                    assert_eq!(cursor.grab_mode, CursorGrabMode::None);
                });
            });
        });
    }

    #[spec]
    fn zooming() {
        given("a camera at its starting distance", || {
            let mut app = camera_app(FlatSurface);

            when("the wheel scrolls up", || {
                scroll(&mut app, 2.0);

                test("it asks to come closer", || {
                    assert_eq!(state(&mut app).distance, CONFIG.start_distance - 2.0);
                });
            });

            when("the wheel scrolls too far out", || {
                scroll(&mut app, -100.0);

                test("it stops at its limit", || {
                    assert_eq!(state(&mut app).distance, CONFIG.max_distance);
                });
            });

            when("the wheel scrolls too far in", || {
                scroll(&mut app, 100.0);

                test("it stops at its limit", || {
                    assert_eq!(state(&mut app).distance, CONFIG.min_distance);
                });
            });
        });
    }

    #[spec]
    fn following() {
        given("a local player", || {
            let mut app = camera_app(FlatSurface);
            let start = Vec3::new(10.0, 0.0, -4.0);
            spawn_player(&mut app, start);

            when("a frame runs", || {
                run_frames(&mut app, 1);
                let camera = transform(&mut app);
                let focus = start + Vec3::Y * CONFIG.focus_height;

                test("the camera looks at the player", || {
                    let toward = (focus - camera.translation).normalize();
                    assert!(camera.forward().as_vec3().distance(toward) < 0.001);
                });

                test("the camera sits at its distance", || {
                    assert!((camera.translation.distance(focus) - 6.0).abs() < 0.001);
                });
            });
        });

        given("a local player who moves suddenly", || {
            let mut app = camera_app(FlatSurface);
            let player = spawn_player(&mut app, Vec3::ZERO);
            run_frames(&mut app, 1);
            let before = transform(&mut app).translation;
            app.world_mut()
                .get_mut::<Transform>(player)
                .unwrap()
                .translation
                .x = 10.0;

            when("one frame runs", || {
                run_frames(&mut app, 1);
                let moved = transform(&mut app).translation.x - before.x;

                test("the camera eases after them", || {
                    assert!(moved > 0.0 && moved < 10.0, "moved {moved}");
                });
            });
        });

        given("ground higher than the camera wants to be", || {
            let mut app = camera_app(HighGround);
            spawn_player(&mut app, Vec3::ZERO);

            when("a frame runs", || {
                run_frames(&mut app, 1);

                test("the camera stays above the ground", || {
                    let height = transform(&mut app).translation.y;
                    assert_eq!(height, 20.0 + CONFIG.ground_clearance);
                });
            });
        });

        given("no local player yet", || {
            let mut app = camera_app(FlatSurface);

            when("a frame runs", || {
                run_frames(&mut app, 1);

                test("the camera has nothing to look at", || {
                    assert_eq!(state(&mut app).focus, None);
                });
            });
        });
    }
}
