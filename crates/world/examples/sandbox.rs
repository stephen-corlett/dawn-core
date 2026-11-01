//! Grows a small forest and shows it from above, with no players, server, or network.
//! Change `config` to try different worlds. Run it with `just sandbox world`.

use bevy::prelude::*;
use sandbox_kit::SandboxPlugin;
use std::sync::Arc;
use world::client::{GroundPalette, GroundTexture, Mood, WorldClientPlugin};
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
                sun_color: Color::WHITE,
                sun_illuminance: 130_000.0,
                sun_position: Vec3::new(40.0, 22.0, 20.0),
                exposure_ev100: 14.0,
                sky_light: 1.0,
                fog_color: Color::srgb(0.42, 0.48, 0.5),
                fog_start: 30.0,
                fog_end: 120.0,
                shadow_distance: 90.0,
                saturation: 0.85,
                glow: 0.1,
            },
            ground: GroundPalette {
                moss: Color::srgb(0.62, 0.68, 0.55),
                grass: Color::srgb(0.78, 0.76, 0.55),
                dirt: Color::srgb(0.62, 0.45, 0.33),
            },
            ground_texture: GroundTexture {
                color: "ground/Ground037_Color.jpg".into(),
                normal: "ground/Ground037_NormalGL.jpg".into(),
                roughness: "ground/Ground037_Roughness.jpg".into(),
                tile_size: 5.0,
            },
        })
        .run();
}
