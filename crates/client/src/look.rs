//! How this game looks and handles: its mood, its one kind of character, and its
//! camera. The game modules know how to draw these, and this is what they're told to draw.

use avatar::client::{
    AnimationConfig, AppearanceConfig, AvatarClientPlugin, CameraConfig, ClipIndices, MotionConfig,
};
use bevy::prelude::*;
use core::f32::consts::PI;
use std::sync::Arc;
use world::Forest;
use world::client::{GroundPalette, Mood, WorldClientPlugin};

const CHARACTER_PARTS: [&str; 3] = [
    "characters/Male_Ranger.gltf",
    "characters/Male_Head.gltf",
    "characters/Hair_Beard.gltf",
];
const ANIMATION_LIBRARY: &str = "animations/UAL1_Standard.glb";

/// The world's mood: late, low sun through a cold, hazy forest.
pub fn world_plugin(forest: Arc<Forest>) -> WorldClientPlugin {
    WorldClientPlugin {
        forest,
        mood: Mood {
            sky: Color::srgb(0.42, 0.5, 0.53),
            sun_color: Color::srgb(1.0, 0.8, 0.58),
            sun_illuminance: 6000.0,
            sun_position: Vec3::new(45.0, 24.0, 18.0),
            ambient_color: Color::srgb(0.55, 0.66, 0.74),
            ambient_brightness: 1200.0,
            fog_color: Color::srgb(0.42, 0.5, 0.53),
            fog_start: 10.0,
            fog_end: 85.0,
            shadow_distance: 70.0,
            saturation: 0.78,
            glow: 0.08,
        },
        ground: GroundPalette {
            moss: Color::srgb(0.13, 0.2, 0.1),
            grass: Color::srgb(0.27, 0.32, 0.13),
            dirt: Color::srgb(0.3, 0.23, 0.16),
        },
    }
}

/// How players look and move, and how the camera frames them.
pub fn avatar_plugin() -> AvatarClientPlugin {
    AvatarClientPlugin {
        appearance: AppearanceConfig {
            parts: CHARACTER_PARTS.map(String::from).to_vec(),
            model_yaw: PI,
        },
        motion: MotionConfig {
            position_smoothing: 18.0,
            speed_smoothing: 12.0,
            turn_smoothing: 12.0,
            turn_speed: 0.5,
            airborne_height: 0.15,
        },
        animation: AnimationConfig {
            library: ANIMATION_LIBRARY.into(),
            clips: ClipIndices {
                idle: 9,
                sneak_idle: 2,
                sneak: 1,
                run: 13,
                sprint: 36,
                roll: 27,
                jump_start: 16,
                falling: 15,
                landing: 14,
            },
            sneak_clip_speed: 0.75,
            run_clip_speed: 5.36,
            sprint_clip_speed: 8.25,
            jump_rate: 2.0,
            roll_rate: 2.67,
            fade_secs: 0.15,
            moving_speed: 0.3,
        },
        camera: CameraConfig {
            orbit_button: MouseButton::Right,
            radians_per_pixel: 0.004,
            zoom_per_notch: 0.8,
            start_distance: 6.5,
            min_distance: 2.5,
            max_distance: 14.0,
            start_pitch: 0.3,
            min_pitch: -0.3,
            max_pitch: 1.3,
            focus_height: 1.6,
            follow_smoothing: 14.0,
            zoom_smoothing: 10.0,
            ground_clearance: 0.3,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;
    use protocol::{forest, movement_config};
    use std::path::Path;

    #[spec]
    fn the_games_look() {
        given("this game's plugins", || {
            let avatar = avatar_plugin();
            let world = world_plugin(forest(&movement_config()));
            let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");

            when("their files are compared with the asset folder", || {
                let exists = |file: &str| assets.join(file).exists();

                test("every part of the character exists", || {
                    assert!(avatar.appearance.parts.iter().all(|part| exists(part)));
                });

                test("the animation library exists", || {
                    assert!(exists(&avatar.animation.library));
                });
            });

            when("their moods are inspected", || {
                let fog = world.mood.fog_color;

                test("the distance fades into the sky", || {
                    assert_eq!(fog, world.mood.sky);
                });
            });

            when("the camera's limits are inspected", || {
                let camera = avatar.camera;
                let distance = camera.min_distance..=camera.max_distance;

                test("it starts within them", || {
                    assert!(distance.contains(&camera.start_distance));
                    assert!((camera.min_pitch..=camera.max_pitch).contains(&camera.start_pitch));
                });
            });
        });
    }
}
