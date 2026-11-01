//! Spawns everything that grows on the ground. Things of one kind share a loaded
//! scene, so the renderer can draw them together.

use crate::{Forest, PlacedProp};
use bevy::prelude::*;
use std::sync::Arc;

pub(super) struct PropsPlugin {
    pub forest: Arc<Forest>,
}

impl Plugin for PropsPlugin {
    fn build(&self, app: &mut App) {
        let forest = self.forest.clone();
        app.add_systems(
            Startup,
            move |mut commands: Commands, assets: Res<AssetServer>| {
                let scenes: Vec<Handle<WorldAsset>> = forest
                    .config()
                    .plantings
                    .iter()
                    .map(|planting| {
                        assets.load(GltfAssetLabel::Scene(0).from_asset(planting.model))
                    })
                    .collect();
                for prop in forest.props() {
                    commands.spawn((
                        WorldAssetRoot(scenes[prop.planting].clone()),
                        prop_transform(prop),
                    ));
                }
            },
        );
    }
}

fn prop_transform(prop: &PlacedProp) -> Transform {
    Transform::from_translation(prop.position)
        .with_rotation(Quat::from_rotation_y(prop.yaw))
        .with_scale(Vec3::splat(prop.scale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Planting, WorldConfig};
    use bdd::spec;

    fn forest() -> Forest {
        Forest::new(WorldConfig {
            seed: 7,
            half_extent: 20.0,
            margin: 5.0,
            clearing_radius: 5.0,
            hill_height: 1.5,
            hill_spacing: 8.0,
            rim_height: 3.0,
            trail_radius: 10.0,
            trail_width: 1.0,
            plantings: vec![Planting {
                model: "tree.gltf",
                count: 12,
                scale: Vec2::new(1.0, 2.0),
                reach: 20.0,
                spacing: 2.0,
                solid_radius: 0.5,
                in_clearing: false,
            }],
        })
    }

    #[spec]
    fn prop_placement() {
        given("a forest", || {
            let forest = Arc::new(forest());

            when("the plugin starts", || {
                let mut app = App::new();
                app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
                app.init_asset::<WorldAsset>();
                app.add_plugins(PropsPlugin {
                    forest: forest.clone(),
                });
                app.update();
                let mut scenes = app.world_mut().query::<(&WorldAssetRoot, &Transform)>();
                let spawned: Vec<_> = scenes.iter(app.world()).collect();

                test("every prop gets a scene", || {
                    assert_eq!(spawned.len(), forest.props().len());
                });

                test("props of one kind share one scene", || {
                    assert!(spawned.iter().all(|(scene, _)| scene.0 == spawned[0].0.0));
                });
            });
        });

        given("a placed prop", || {
            let prop = PlacedProp {
                planting: 0,
                position: Vec3::new(1.0, 2.0, 3.0),
                yaw: 0.5,
                scale: 1.5,
            };

            when("its transform is built", || {
                let transform = prop_transform(&prop);

                test("it matches where the prop was placed", || {
                    assert_eq!(transform.translation, prop.position);
                    assert_eq!(transform.scale, Vec3::splat(1.5));
                    assert_eq!(transform.rotation, Quat::from_rotation_y(0.5));
                });
            });
        });
    }
}
