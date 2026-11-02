use crate::{Planting, Solid, Solids, WorldConfig, terrain};
use bevy::prelude::*;
use movement::WalkSurface;

/// How many spots to try for each thing before giving up on it.
const PLACEMENT_ATTEMPTS: u32 = 12;
/// Keeps the random number streams of different kinds of thing apart.
const STREAM_STRIDE: u32 = 4096;

/// One thing placed in the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacedProp {
    /// An index into [`WorldConfig::plantings`].
    pub planting: usize,
    pub position: Vec3,
    pub yaw: f32,
    pub scale: f32,
}

/// A grown world: the ground and everything placed on it. The same config always
/// grows the same forest.
#[derive(Clone, Debug, PartialEq)]
pub struct Forest {
    config: WorldConfig,
    props: Vec<PlacedProp>,
    solids: Solids,
}

impl Forest {
    pub fn new(config: WorldConfig) -> Self {
        let mut forest = Self {
            props: Vec::new(),
            solids: Solids::default(),
            config,
        };
        let mut spaced = Vec::new();
        let mut solids = Vec::new();
        for planting in 0..forest.config.plantings.len() {
            forest.plant(planting, &mut spaced, &mut solids);
        }
        forest.solids = Solids::new(solids);
        forest
    }

    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    pub fn props(&self) -> &[PlacedProp] {
        &self.props
    }

    pub fn solids(&self) -> &[Solid] {
        self.solids.all()
    }

    /// How far a point is from the middle of the nearest trail.
    pub fn trail_distance(&self, at: Vec2) -> f32 {
        terrain::trail_distance(&self.config, at)
    }

    /// A smooth pattern between 0 and 1 for breaking up flat areas of ground.
    pub fn patchiness(&self, at: Vec2) -> f32 {
        terrain::noise(self.config.seed + 2, at)
    }

    fn plant(&mut self, planting: usize, spaced: &mut Vec<(Vec2, f32)>, solids: &mut Vec<Solid>) {
        let kind = self.config.plantings[planting].clone();
        for item in 0..kind.count {
            let stream = planting as u32 * STREAM_STRIDE + item;
            let spot = (0..PLACEMENT_ATTEMPTS)
                .map(|attempt| self.candidate(&kind, stream, attempt))
                .find(|&at| self.has_room(&kind, at, spaced));
            let Some(at) = spot else {
                continue;
            };

            let seed = self.config.seed;
            let scale =
                kind.scale.x + (kind.scale.y - kind.scale.x) * terrain::random(seed, stream, 4);
            self.props.push(PlacedProp {
                planting,
                position: at.extend(self.ground_height(at)).xzy(),
                yaw: terrain::random_angle(seed, stream, 5),
                scale,
            });
            if kind.spacing > 0.0 {
                spaced.push((at, kind.spacing));
            }
            if kind.solid_radius > 0.0 {
                solids.push(Solid {
                    center: at,
                    radius: kind.solid_radius * scale,
                });
            }
        }
    }

    fn candidate(&self, kind: &Planting, stream: u32, attempt: u32) -> Vec2 {
        let seed = self.config.seed.wrapping_add(attempt * 2);
        let reach = kind.reach.min(self.config.half_extent + self.config.margin);
        Vec2::new(
            terrain::random(seed, stream, 0),
            terrain::random(seed, stream, 1),
        ) * reach
            * 2.0
            - Vec2::splat(reach)
    }

    fn has_room(&self, kind: &Planting, at: Vec2, spaced: &[(Vec2, f32)]) -> bool {
        let in_clearing = at.length() < self.config.clearing_radius;
        let on_trail = self.trail_distance(at) < self.config.trail_width;
        let crowded = kind.spacing > 0.0
            && spaced
                .iter()
                .any(|&(other, spacing)| at.distance(other) < spacing.max(kind.spacing));
        !(on_trail || crowded || (in_clearing && !kind.in_clearing))
    }
}

impl WalkSurface for Forest {
    fn ground_height(&self, at: Vec2) -> f32 {
        terrain::height(&self.config, at)
    }

    fn push_out(&self, at: Vec2, radius: f32) -> Vec2 {
        self.solids.push_out(at, radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const TREE: Planting = Planting {
        model: "tree.gltf",
        count: 60,
        scale: Vec2::new(1.0, 2.0),
        reach: 70.0,
        spacing: 4.0,
        solid_radius: 0.5,
        in_clearing: false,
    };
    const GRASS: Planting = Planting {
        model: "grass.gltf",
        count: 200,
        scale: Vec2::ONE,
        reach: 40.0,
        spacing: 0.0,
        solid_radius: 0.0,
        in_clearing: true,
    };

    fn config() -> WorldConfig {
        WorldConfig {
            seed: 7,
            half_extent: 50.0,
            margin: 30.0,
            clearing_radius: 16.0,
            hill_height: 1.5,
            hill_spacing: 14.0,
            rim_height: 10.0,
            trail_radius: 30.0,
            trail_width: 3.0,
            plantings: vec![TREE, GRASS],
        }
    }

    fn props_of(forest: &Forest, planting: usize) -> Vec<PlacedProp> {
        let mut props = forest.props().to_vec();
        props.retain(|prop| prop.planting == planting);
        props
    }

    #[spec]
    fn growing() {
        given("a config", || {
            let forest = Forest::new(config());

            when("it grows a second time", || {
                let again = Forest::new(config());

                test("the two forests match exactly", || {
                    assert_eq!(forest, again);
                });
            });

            when("its trees are inspected", || {
                let trees = props_of(&forest, 0);

                test("the forest has most of the trees asked for", || {
                    assert!(trees.len() > 45, "{} trees", trees.len());
                });

                test("the clearing stays open", || {
                    assert!(trees.iter().all(|t| t.position.xz().length() >= 16.0));
                });

                test("the trail stays open", || {
                    let on_trail = |t: &PlacedProp| forest.trail_distance(t.position.xz()) < 3.0;
                    assert!(!trees.iter().any(on_trail));
                });

                test("trees keep their distance from each other", || {
                    for (index, tree) in trees.iter().enumerate() {
                        for other in &trees[index + 1..] {
                            assert!(tree.position.distance(other.position) >= 4.0);
                        }
                    }
                });

                test("every tree stands on the ground", || {
                    for tree in &trees {
                        let ground = forest.ground_height(tree.position.xz());
                        assert_eq!(tree.position.y, ground);
                    }
                });

                test("trees vary in size", || {
                    let scales = trees.iter().map(|t| t.scale);
                    let largest = scales.clone().fold(0.0, f32::max);
                    let smallest = scales.fold(f32::MAX, f32::min);
                    assert!(largest - smallest > 0.5);
                });
            });

            when("its grass is inspected", || {
                let grass = props_of(&forest, 1);

                test("grass can grow in the clearing", || {
                    assert!(grass.iter().any(|g| g.position.xz().length() < 16.0));
                });

                test("grass stays within its reach", || {
                    let inside = |g: &PlacedProp| g.position.xz().abs().max_element() <= 40.0;
                    assert!(grass.iter().all(inside));
                });
            });

            when("its solid things are inspected", || {
                let trees = props_of(&forest, 0);

                test("only the trees are solid", || {
                    assert_eq!(forest.solids().len(), trees.len());
                });
            });
        });

        given("a world too small to fit what's asked for", || {
            let crowded = Planting {
                count: 500,
                spacing: 30.0,
                ..TREE
            };
            let forest = Forest::new(WorldConfig {
                plantings: vec![crowded],
                ..config()
            });

            when("it grows", || {
                let placed = forest.props().len();

                test("it places what fits and skips the rest", || {
                    assert!(placed > 0 && placed < 500, "placed {placed}");
                });
            });
        });
    }

    #[spec]
    fn walking_surface() {
        given("a forest with a single tree", || {
            let forest = Forest {
                config: config(),
                props: Vec::new(),
                solids: Solids::new(vec![Solid {
                    center: Vec2::new(10.0, 0.0),
                    radius: 1.0,
                }]),
            };

            when("a body walks into the tree", || {
                let pushed = forest.push_out(Vec2::new(9.5, 0.0), 0.5);

                test("it ends up touching the tree's edge", || {
                    assert_eq!(pushed, Vec2::new(8.5, 0.0));
                });
            });

            when("a body stands exactly at the tree's center", || {
                let pushed = forest.push_out(Vec2::new(10.0, 0.0), 0.5);

                test("it still gets pushed out", || {
                    assert_eq!(pushed, Vec2::new(11.5, 0.0));
                });
            });

            when("a body walks past the tree", || {
                let beside = Vec2::new(5.0, 0.0);

                test("it isn't moved", || {
                    assert_eq!(forest.push_out(beside, 0.5), beside);
                });
            });
        });
    }
}
