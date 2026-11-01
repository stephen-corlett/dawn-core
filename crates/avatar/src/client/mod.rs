//! Shows each player as an animated character, and follows the local one.

mod animation;
mod appearance;
mod camera;
mod motion;

pub use animation::{AnimationConfig, ClipIndices, GaitClip};
pub use appearance::AppearanceConfig;
pub use camera::CameraConfig;
pub use motion::MotionConfig;

use bevy::prelude::*;

pub struct AvatarClientPlugin {
    pub appearance: AppearanceConfig,
    pub motion: MotionConfig,
    pub animation: AnimationConfig,
    pub camera: CameraConfig,
}

/// The order each frame's work happens in.
#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum AvatarSystems {
    Appear,
    Move,
    Present,
}

impl Plugin for AvatarClientPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                AvatarSystems::Appear,
                AvatarSystems::Move,
                AvatarSystems::Present,
            )
                .chain(),
        );
        app.add_plugins((
            animation::AnimationPlugin {
                config: self.animation.clone(),
            },
            appearance::AppearancePlugin {
                config: self.appearance.clone(),
            },
            motion::MotionPlugin {
                config: self.motion,
            },
            camera::CameraPlugin {
                config: self.camera,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;
    use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
    use lightyear::prelude::Predicted;
    use movement::{FlatSurface, MoveHeading, MovementConfig, PlayerPosition, Surface};
    use std::sync::Arc;

    const GAIT_CLIP: GaitClip = GaitClip {
        speed: 1.0,
        slowest_rate: 0.5,
        fastest_rate: 2.0,
    };

    fn plugin() -> AvatarClientPlugin {
        AvatarClientPlugin {
            appearance: AppearanceConfig {
                parts: vec!["body.gltf".into()],
                model_yaw: 0.0,
            },
            motion: MotionConfig {
                position_smoothing: 18.0,
                speed_smoothing: 12.0,
                turn_smoothing: 12.0,
                turn_speed: 0.5,
                airborne_height: 0.2,
            },
            animation: AnimationConfig {
                library: "clips.glb".into(),
                clips: ClipIndices {
                    idle: 0,
                    sneak_idle: 7,
                    sneak: 1,
                    walk: 9,
                    run: 2,
                    roll: 8,
                    jump_start: 4,
                    falling: 5,
                    landing: 6,
                },
                sneak_clip: GAIT_CLIP,
                walk_clip: GAIT_CLIP,
                run_clip: GAIT_CLIP,
                walk_below: 0.5,
                jump_rate: 2.0,
                lunge_rate: 2.5,
                rollout_rate: 1.5,
                fade_secs: 0.1,
                moving_speed: 0.3,
            },
            camera: CameraConfig {
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
            },
        }
    }

    #[spec]
    fn the_whole_plugin() {
        given(
            "a client with the avatar plugin and one local player",
            || {
                let mut app = App::new();
                app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
                app.init_asset::<AnimationClip>();
                app.init_asset::<AnimationGraph>();
                app.init_asset::<WorldAsset>();
                app.insert_resource(Time::<()>::default());
                app.insert_resource(Surface(Arc::new(FlatSurface)));
                app.insert_resource(MovementConfig {
                    sneak_speed: 2.0,
                    run_speed: 6.0,
                    dodge_lunge_speed: 7.0,
                    dodge_lunge_secs: 0.4,
                    dodge_rollout_speed: 2.0,
                    dodge_rollout_secs: 0.3,
                    dodge_recovery_secs: 0.6,
                    dodge_recovery_speed: 1.5,
                    jump_speed: 7.0,
                    gravity: 18.0,
                    ground_snap: 0.4,
                    body_radius: 0.4,
                    world_half_extent: 30.0,
                });
                app.init_resource::<ButtonInput<MouseButton>>();
                app.init_resource::<AccumulatedMouseMotion>();
                app.init_resource::<AccumulatedMouseScroll>();
                app.add_plugins(plugin());
                let player = app
                    .world_mut()
                    .spawn((PlayerPosition(Vec3::new(3.0, 0.0, 1.0)), Predicted))
                    .id();

                when("it runs a frame", || {
                    app.update();

                    test("the player has a body", || {
                        assert!(app.world().get::<Children>(player).is_some());
                    });

                    test("forward is ready for movement to read", || {
                        assert_eq!(app.world().resource::<MoveHeading>().0, 0.0);
                    });
                });
            },
        );
    }
}
