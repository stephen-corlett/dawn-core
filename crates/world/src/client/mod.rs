//! Draws the world: the ground, everything growing on it, and its mood.

mod ground;
mod mood;
mod props;

pub use ground::GroundPalette;
pub use mood::Mood;

use crate::Forest;
use bevy::prelude::*;
use std::sync::Arc;

pub struct WorldClientPlugin {
    pub forest: Arc<Forest>,
    pub mood: Mood,
    pub ground: GroundPalette,
}

impl Plugin for WorldClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ground::GroundPlugin {
                forest: self.forest.clone(),
                palette: self.ground,
            },
            props::PropsPlugin {
                forest: self.forest.clone(),
            },
            mood::MoodPlugin { mood: self.mood },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Planting, WorldConfig};
    use bdd::spec;

    fn plugin() -> WorldClientPlugin {
        WorldClientPlugin {
            forest: Arc::new(Forest::new(WorldConfig {
                seed: 7,
                half_extent: 10.0,
                margin: 2.0,
                clearing_radius: 3.0,
                hill_height: 1.0,
                hill_spacing: 8.0,
                rim_height: 3.0,
                trail_radius: 6.0,
                trail_width: 1.0,
                plantings: vec![Planting {
                    model: "tree.gltf",
                    count: 3,
                    scale: Vec2::ONE,
                    reach: 10.0,
                    spacing: 2.0,
                    solid_radius: 0.5,
                    in_clearing: false,
                }],
            })),
            mood: Mood {
                sky: Color::BLACK,
                sun_color: Color::WHITE,
                sun_illuminance: 9000.0,
                sun_position: Vec3::new(30.0, 20.0, 10.0),
                ambient_color: Color::WHITE,
                ambient_brightness: 200.0,
                fog_color: Color::BLACK,
                fog_start: 10.0,
                fog_end: 80.0,
                shadow_distance: 60.0,
                saturation: 0.8,
                glow: 0.1,
            },
            ground: GroundPalette {
                moss: Color::BLACK,
                grass: Color::WHITE,
                dirt: Color::BLACK,
            },
        }
    }

    #[spec]
    fn the_whole_plugin() {
        given("a client with the world plugin", || {
            let mut app = App::new();
            app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
            app.init_asset::<Mesh>();
            app.init_asset::<StandardMaterial>();
            app.init_asset::<WorldAsset>();
            app.add_plugins(plugin());

            when("it starts", || {
                app.update();
                let mut ground = app.world_mut().query::<&Mesh3d>();
                let mut props = app.world_mut().query::<&WorldAssetRoot>();
                let mut suns = app.world_mut().query::<&DirectionalLight>();

                test("the ground, props, and sun all appear", || {
                    assert_eq!(ground.iter(app.world()).count(), 1);
                    assert!(props.iter(app.world()).count() > 0);
                    assert_eq!(suns.iter(app.world()).count(), 1);
                });
            });
        });
    }
}
