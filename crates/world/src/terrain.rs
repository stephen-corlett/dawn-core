//! The shape of the ground, as pure functions of a position.

use crate::WorldConfig;
use bevy::math::ops;
use bevy::prelude::*;
use core::f32::consts::TAU;

/// How far past the clearing the ground takes to reach its full roughness, as a
/// multiple of the clearing's radius.
const CLEARING_BLEND: f32 = 1.8;
/// Where the rim starts to rise, as a share of how far players can walk.
const RIM_START: f32 = 0.7;
const SECOND_OCTAVE_WEIGHT: f32 = 0.5;

/// How high the ground is at a point. The clearing is exactly level at height zero.
pub fn height(config: &WorldConfig, at: Vec2) -> f32 {
    let from_center = at.length();
    let away_from_clearing = smoothstep(
        config.clearing_radius,
        config.clearing_radius * CLEARING_BLEND,
        from_center,
    );
    let rim = smoothstep(
        config.half_extent * RIM_START,
        config.half_extent + config.margin,
        at.abs().max_element(),
    );
    hills(config, at) * away_from_clearing + rim * config.rim_height
}

fn hills(config: &WorldConfig, at: Vec2) -> f32 {
    let broad = signed_noise(config.seed, at / config.hill_spacing);
    let fine = signed_noise(config.seed + 1, at * 2.0 / config.hill_spacing);
    (broad + fine * SECOND_OCTAVE_WEIGHT) * config.hill_height
}

/// How far a point is from the middle of the trail. The trail is a wobbly loop around
/// the clearing, with one straight path out to it along negative Z.
pub fn trail_distance(config: &WorldConfig, at: Vec2) -> f32 {
    let angle = ops::atan2(at.y, at.x);
    let wobble = ops::sin(angle * 3.0) * 4.0 + ops::sin(angle * 5.0 + 1.0) * 2.0;
    let around = (at.length() - config.trail_radius - wobble).abs();
    let path_end = -config.trail_radius;
    let along_path = (path_end..0.0).contains(&at.y);
    if along_path {
        around.min(at.x.abs())
    } else {
        around
    }
}

/// A smooth pattern between 0 and 1, for patches that shouldn't look random from
/// one point to the next.
pub fn noise(seed: u32, at: Vec2) -> f32 {
    let cell = at.floor();
    let within = at - cell;
    let (x, z) = (cell.x as i32, cell.y as i32);
    let corner = |dx, dz| random(seed, x.wrapping_add(dx) as u32, z.wrapping_add(dz) as u32);
    let (blend_x, blend_z) = (smooth(within.x), smooth(within.y));
    let near = lerp(corner(0, 0), corner(1, 0), blend_x);
    let far = lerp(corner(0, 1), corner(1, 1), blend_x);
    lerp(near, far, blend_z)
}

fn signed_noise(seed: u32, at: Vec2) -> f32 {
    noise(seed, at) * 2.0 - 1.0
}

/// A repeatable number from 0 up to 1. Integer math gives the client and server the
/// same result on any machine.
pub fn random(seed: u32, a: u32, b: u32) -> f32 {
    let mut hash = seed
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(a.wrapping_mul(0x85EB_CA6B))
        .wrapping_add(b.wrapping_mul(0xC2B2_AE35));
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x2C1B_3C6D);
    hash ^= hash >> 12;
    hash = hash.wrapping_mul(0x297A_2D39);
    hash ^= hash >> 15;
    (hash >> 8) as f32 / (1 << 24) as f32
}

/// A random direction, as radians.
pub fn random_angle(seed: u32, a: u32, b: u32) -> f32 {
    random(seed, a, b) * TAU
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn smoothstep(from: f32, to: f32, value: f32) -> f32 {
    smooth(((value - from) / (to - from)).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

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
            plantings: Vec::new(),
        }
    }

    #[spec]
    fn ground_shape() {
        given("a world with a clearing", || {
            let config = config();

            when("the ground in the clearing is measured", || {
                let edge = height(&config, Vec2::new(config.clearing_radius, 0.0));

                test("it is level", || {
                    assert_eq!(height(&config, Vec2::ZERO), 0.0);
                    assert_eq!(edge, 0.0);
                });
            });

            when("hills are measured outside the clearing", || {
                let samples: Vec<f32> = (0..40)
                    .map(|i| height(&config, Vec2::new(25.0 + i as f32, 3.0)))
                    .collect();
                let lowest = samples.iter().copied().fold(f32::MAX, f32::min);
                let highest = samples.iter().copied().fold(f32::MIN, f32::max);

                test("the ground rises and falls", || {
                    assert!(highest - lowest > 0.5, "range {lowest}..{highest}");
                });
            });

            when("the ground is measured at the far edge", || {
                let edge = config.half_extent + config.margin;
                let rim = height(&config, Vec2::new(edge, 0.0));

                test("it climbs to hide the end of the world", || {
                    assert!(rim > config.rim_height / 2.0, "rim {rim}");
                });
            });

            when("two neighboring points are compared", || {
                let first = height(&config, Vec2::new(30.0, 10.0));
                let second = height(&config, Vec2::new(30.1, 10.0));

                test("the ground has no cliffs", || {
                    assert!((first - second).abs() < 0.2);
                });
            });
        });

        given("two worlds with different seeds", || {
            let first = config();
            let second = WorldConfig {
                seed: 8,
                ..first.clone()
            };

            when("the same point is measured in each", || {
                let at = Vec2::new(33.0, -12.0);

                test("their ground differs", || {
                    assert_ne!(height(&first, at), height(&second, at));
                });
            });
        });
    }

    #[spec]
    fn trail_shape() {
        given("a world with a trail", || {
            let config = config();

            when(
                "a point on the path out of the clearing is measured",
                || {
                    let distance = trail_distance(&config, Vec2::new(0.5, -10.0));

                    test("it counts as the trail", || {
                        assert_eq!(distance, 0.5);
                    });
                },
            );

            when("a point north of the world origin is measured", || {
                let distance = trail_distance(&config, Vec2::new(0.5, 10.0));

                test("the path doesn't run that way", || {
                    assert!(distance > 5.0, "distance {distance}");
                });
            });

            when("a point at the world origin's far side is measured", || {
                let distance = trail_distance(&config, Vec2::new(0.0, 80.0));

                test("it is far from the loop", || {
                    assert!(distance > 30.0, "distance {distance}");
                });
            });
        });
    }

    #[spec]
    fn random_numbers() {
        given("a seed and two inputs", || {
            when("the same number is asked for twice", || {
                let first = random(3, 10, 20);
                let second = random(3, 10, 20);

                test("the numbers match", || {
                    assert_eq!(first, second);
                });
            });

            when("many numbers are drawn", || {
                let numbers: Vec<f32> = (0..1000).map(|i| random(3, i, 0)).collect();
                let mean = numbers.iter().sum::<f32>() / numbers.len() as f32;

                test("they stay in range", || {
                    assert!(numbers.iter().all(|n| (0.0..1.0).contains(n)));
                });

                test("they spread evenly", || {
                    assert!((mean - 0.5).abs() < 0.05, "mean {mean}");
                });
            });

            when("noise is sampled across a cell boundary", || {
                let below = noise(3, Vec2::new(1.999, 4.5));
                let above = noise(3, Vec2::new(2.001, 4.5));

                test("the pattern is continuous", || {
                    assert!((below - above).abs() < 0.01);
                });
            });
        });
    }
}
