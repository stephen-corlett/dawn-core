//! Runs one character with the real movement rules, with no server or network. Move
//! with WASD, sprint with Shift, roll with Control, hold C to sneak, jump with Space, and
//! hold the right mouse button to look around. Run it with `just sandbox avatar`.

use avatar::client::{
    AnimationConfig, AppearanceConfig, AvatarClientPlugin, CameraConfig, ClipIndices, GaitClip,
    MotionConfig,
};
use bevy::prelude::*;
use core::f32::consts::PI;
use lightyear::prelude::Predicted;
use movement::{
    Body, Dodge, FlatSurface, Gait, MoveHeading, MoveInput, MovementConfig, PlayerPosition,
    Sneaking, Surface, VerticalSpeed, next_body,
};
use sandbox_kit::SandboxPlugin;
use std::sync::Arc;

const MOVEMENT: MovementConfig = MovementConfig {
    sneak_speed: 1.5,
    run_speed: 5.4,
    sprint_speed: 8.2,
    dodge_lunge_speed: 9.0,
    dodge_lunge_secs: 0.4,
    dodge_rollout_speed: 1.7,
    dodge_rollout_secs: 0.8,
    dodge_recovery_secs: 0.1,
    dodge_recovery_speed: 1.5,
    jump_speed: 7.0,
    gravity: 18.0,
    ground_snap: 0.4,
    body_radius: 0.4,
    world_half_extent: 30.0,
};

fn main() {
    App::new()
        .add_plugins(SandboxPlugin {
            title: "Avatar".into(),
            ground_size: MOVEMENT.world_half_extent * 2.0,
            asset_folder: "../../assets".into(),
        })
        .insert_resource(MOVEMENT)
        .insert_resource(Surface(Arc::new(FlatSurface)))
        .add_plugins(AvatarClientPlugin {
            appearance: AppearanceConfig {
                parts: vec![
                    "characters/Male_Ranger.gltf".into(),
                    "characters/Male_Head.gltf".into(),
                ],
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
                library: "animations/UAL1_Standard.glb".into(),
                clips: ClipIndices {
                    idle: 9,
                    sneak_idle: 2,
                    sneak: 1,
                    walk: 42,
                    run: 13,
                    sprint: 36,
                    roll: 27,
                    jump_start: 16,
                    falling: 15,
                    landing: 14,
                },
                sneak_clip: GaitClip {
                    speed: 0.75,
                    slowest_rate: 0.5,
                    fastest_rate: 2.0,
                },
                walk_clip: GaitClip {
                    speed: 0.97,
                    slowest_rate: 0.5,
                    fastest_rate: 2.0,
                },
                run_clip: GaitClip {
                    speed: 5.36,
                    slowest_rate: 0.5,
                    fastest_rate: 1.2,
                },
                sprint_clip: GaitClip {
                    speed: 8.25,
                    slowest_rate: 0.8,
                    fastest_rate: 1.25,
                },
                walk_below: 0.5,
                sprint_above: 1.1,
                jump_rate: 2.0,
                lunge_rate: 1.62,
                rollout_rate: 1.03,
                fade_secs: 0.15,
                moving_speed: 0.3,
            },
            camera: CameraConfig {
                orbit_button: MouseButton::Right,
                radians_per_pixel: 0.004,
                zoom_per_notch: 0.8,
                start_distance: 6.0,
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
        })
        .add_systems(Startup, spawn_player)
        .add_systems(FixedUpdate, move_player)
        .run();
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((PlayerPosition::default(), Predicted));
}

fn move_player(
    time: Res<Time<Fixed>>,
    keys: Res<ButtonInput<KeyCode>>,
    heading: Res<MoveHeading>,
    surface: Res<Surface>,
    mut players: Query<(
        &mut PlayerPosition,
        &mut VerticalSpeed,
        &mut Dodge,
        &mut Sneaking,
    )>,
) {
    let input = MoveInput {
        forward: keys.pressed(KeyCode::KeyW),
        back: keys.pressed(KeyCode::KeyS),
        left: keys.pressed(KeyCode::KeyA),
        right: keys.pressed(KeyCode::KeyD),
        jump: keys.pressed(KeyCode::Space),
        dodge: keys.pressed(KeyCode::ControlLeft),
        gait: if keys.pressed(KeyCode::ShiftLeft) {
            Gait::Sprint
        } else if keys.pressed(KeyCode::KeyC) {
            Gait::Sneak
        } else {
            Gait::Run
        },
        yaw: heading.0,
    };
    for (mut position, mut vertical_speed, mut dodge, mut sneaking) in &mut players {
        let body = Body {
            position: position.0,
            vertical_speed: vertical_speed.0,
            dodge: *dodge,
            sneaking: sneaking.0,
        };
        let next = next_body(
            body,
            &input,
            time.timestep().as_secs_f32(),
            &MOVEMENT,
            surface.0.as_ref(),
        );
        position.0 = next.position;
        vertical_speed.0 = next.vertical_speed;
        *dodge = next.dodge;
        sneaking.0 = next.sneaking;
    }
}
