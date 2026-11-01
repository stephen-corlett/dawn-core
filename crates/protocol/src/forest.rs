//! The world both sides walk on. The server needs it as much as the client does, because
//! the ground and the tree trunks decide where players can go.

use movement::MovementConfig;
use std::sync::Arc;
use world::{Forest, Planting, WorldConfig};

use bevy::math::Vec2;

const TREE_REACH: f32 = 80.0;
const UNDERGROWTH_REACH: f32 = 54.0;
const CLEARING_RADIUS: f32 = 16.0;

/// Grows this game's forest. It's the same every time, on both sides.
pub fn forest(movement: &MovementConfig) -> Arc<Forest> {
    Arc::new(Forest::new(WorldConfig {
        seed: 1,
        half_extent: movement.world_half_extent,
        margin: 30.0,
        clearing_radius: CLEARING_RADIUS,
        hill_height: 1.3,
        hill_spacing: 15.0,
        rim_height: 12.0,
        trail_radius: 32.0,
        trail_width: 1.5,
        plantings: plantings(),
    }))
}

fn plantings() -> Vec<Planting> {
    let tree = |model, count, solid_radius, spacing| Planting {
        model,
        count,
        scale: Vec2::new(1.0, 1.6),
        reach: TREE_REACH,
        spacing,
        solid_radius,
        in_clearing: false,
    };
    let growth = |model, count, low: f32, high: f32, in_clearing| Planting {
        model,
        count,
        scale: Vec2::new(low, high),
        reach: UNDERGROWTH_REACH,
        spacing: 0.0,
        solid_radius: 0.0,
        in_clearing,
    };
    let boulder = |model| Planting {
        model,
        count: 12,
        scale: Vec2::new(0.6, 1.3),
        reach: UNDERGROWTH_REACH,
        spacing: 4.0,
        solid_radius: 1.2,
        in_clearing: false,
    };

    vec![
        // Giants first, so that smaller trees fill in around them.
        tree("nature/TwistedTree_1.gltf", 7, 0.9, 12.0),
        tree("nature/TwistedTree_2.gltf", 7, 0.9, 12.0),
        tree("nature/TwistedTree_3.gltf", 7, 0.9, 12.0),
        tree("nature/TwistedTree_4.gltf", 7, 0.9, 12.0),
        tree("nature/TwistedTree_5.gltf", 7, 0.9, 12.0),
        tree("nature/DeadTree_1.gltf", 9, 0.5, 5.0),
        tree("nature/DeadTree_2.gltf", 9, 0.5, 5.0),
        tree("nature/DeadTree_3.gltf", 9, 0.5, 5.0),
        tree("nature/Pine_1.gltf", 34, 0.4, 3.5),
        tree("nature/Pine_2.gltf", 34, 0.4, 3.5),
        tree("nature/Pine_3.gltf", 34, 0.4, 3.5),
        tree("nature/Pine_4.gltf", 34, 0.4, 3.5),
        tree("nature/Pine_5.gltf", 34, 0.4, 3.5),
        tree("nature/CommonTree_1.gltf", 12, 0.4, 4.0),
        tree("nature/CommonTree_2.gltf", 12, 0.4, 4.0),
        tree("nature/CommonTree_3.gltf", 12, 0.4, 4.0),
        boulder("nature/Rock_Medium_1.gltf"),
        boulder("nature/Rock_Medium_2.gltf"),
        boulder("nature/Rock_Medium_3.gltf"),
        growth("nature/Bush_Common.gltf", 70, 0.8, 1.4, false),
        growth("nature/Fern_1.gltf", 320, 0.8, 1.5, false),
        growth("nature/Plant_1.gltf", 60, 0.8, 1.3, false),
        growth("nature/Plant_7.gltf", 60, 0.8, 1.3, false),
        growth("nature/Clover_1.gltf", 260, 0.8, 1.4, true),
        growth("nature/Grass_Common_Short.gltf", 1100, 0.8, 1.4, true),
        growth("nature/Grass_Common_Tall.gltf", 500, 0.8, 1.4, false),
        growth("nature/Grass_Wispy_Short.gltf", 800, 0.8, 1.4, true),
        growth("nature/Grass_Wispy_Tall.gltf", 400, 0.8, 1.4, false),
        growth("nature/Mushroom_Common.gltf", 50, 0.7, 1.2, true),
        growth("nature/Mushroom_Laetiporus.gltf", 20, 0.7, 1.2, false),
        growth("nature/Pebble_Round_1.gltf", 40, 0.6, 1.2, true),
        growth("nature/Pebble_Round_2.gltf", 40, 0.6, 1.2, true),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement_config;
    use bdd::spec;
    use std::path::Path;

    #[spec]
    fn this_games_forest() {
        given("the forest both sides use", || {
            let forest = forest(&movement_config());

            when("its plantings are compared with the asset folder", || {
                let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");

                test("every model exists", || {
                    for planting in &forest.config().plantings {
                        assert!(assets.join(planting.model).exists(), "{}", planting.model);
                    }
                });
            });

            when("it grows", || {
                let props = forest.props();

                test("the place players start is open", || {
                    let start_is_clear = forest
                        .solids()
                        .iter()
                        .all(|solid| solid.center.length() >= CLEARING_RADIUS);
                    assert!(start_is_clear);
                });

                test("it holds a real forest of things", || {
                    assert!(props.len() > 1000, "{} props", props.len());
                });
            });
        });
    }
}
