//! Grows a small forest and shows it from above, with no players, server, or network.
//! Change `config` to try different worlds. Run it with `just sandbox world`.

use bevy::prelude::*;
use sandbox_kit::SandboxPlugin;
use std::sync::Arc;
use world::client::{GroundPalette, Mood, WorldClientPlugin};
use world::{Forest, Planting, WorldConfig};

fn config() -> WorldConfig {
    WorldConfig {
        seed: 7,
        half_extent: 25.0,
        margin: 15.0,
        clearing_radius: 8.0,
        hill_height: 1.2,
        hill_spacing: 14.0,
        rim_height: 8.0,
        trail_radius: 16.0,
        trail_width: 1.6,
        plantings: vec![
            Planting {
                model: "nature/Pine_1.gltf",
                count: 40,
                scale: Vec2::new(0.9, 1.5),
                reach: 40.0,
                spacing: 4.0,
                solid_radius: 0.5,
                in_clearing: false,
            },
            Planting {
                model: "nature/Fern_1.gltf",
                count: 120,
                scale: Vec2::new(0.8, 1.3),
                reach: 25.0,
                spacing: 0.0,
                solid_radius: 0.0,
                in_clearing: true,
            },
        ],
    }
}

fn main() {
    let forest = Arc::new(Forest::new(config()));
    App::new()
        .add_plugins(SandboxPlugin {
            title: "World".into(),
            // The world brings its own ground, so the sandbox's only needs to exist.
            ground_size: 0.01,
            asset_folder: "../../assets".into(),
        })
        .add_plugins(WorldClientPlugin {
            forest,
            mood: Mood {
                sky: Color::srgb(0.42, 0.48, 0.5),
                sun_color: Color::srgb(1.0, 0.82, 0.62),
                sun_illuminance: 9000.0,
                sun_position: Vec3::new(40.0, 22.0, 20.0),
                ambient_color: Color::srgb(0.5, 0.6, 0.7),
                ambient_brightness: 250.0,
                fog_color: Color::srgb(0.42, 0.48, 0.5),
                fog_start: 30.0,
                fog_end: 120.0,
                shadow_distance: 90.0,
                saturation: 0.85,
                glow: 0.1,
            },
            ground: GroundPalette {
                moss: Color::srgb(0.13, 0.2, 0.11),
                grass: Color::srgb(0.2, 0.26, 0.12),
                dirt: Color::srgb(0.25, 0.19, 0.13),
            },
        })
        .run();
}
