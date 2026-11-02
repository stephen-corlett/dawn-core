//! What players can't walk through, sorted into a grid so that a move only checks what's
//! nearby, however many solid things the world holds.

use bevy::prelude::*;
use std::collections::HashMap;

/// How wide one square of the grid is.
const CELL_SIZE: f32 = 8.0;

/// A circle on the XZ plane that players can't walk through.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Solid {
    pub center: Vec2,
    pub radius: f32,
}

/// Every solid thing in a world.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solids {
    all: Vec<Solid>,
    cells: HashMap<(i32, i32), Vec<u32>>,
    largest_radius: f32,
}

impl Solids {
    pub fn new(all: Vec<Solid>) -> Self {
        let mut cells: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for (index, solid) in all.iter().enumerate() {
            for cell in cells_within(solid.center, solid.radius) {
                cells.entry(cell).or_default().push(index as u32);
            }
        }
        let largest_radius = all.iter().map(|solid| solid.radius).fold(0.0, f32::max);
        Self {
            all,
            cells,
            largest_radius,
        }
    }

    pub fn all(&self) -> &[Solid] {
        &self.all
    }

    /// Moves a circle out of anything solid, and returns its new center. Solids are always
    /// tried in the same order, so the client and server get the same answer.
    pub fn push_out(&self, at: Vec2, radius: f32) -> Vec2 {
        // One push can carry the circle this far, so anything it could reach has to be tried.
        let longest_push = self.largest_radius + radius;
        let mut nearby: Vec<u32> = cells_within(at, radius + longest_push)
            .filter_map(|cell| self.cells.get(&cell))
            .flatten()
            .copied()
            .collect();
        nearby.sort_unstable();
        nearby.dedup();

        nearby.iter().fold(at, |at, &index| {
            let solid = self.all[index as usize];
            let offset = at - solid.center;
            let reach = solid.radius + radius;
            if offset.length_squared() < reach * reach {
                solid.center + offset.try_normalize().unwrap_or(Vec2::X) * reach
            } else {
                at
            }
        })
    }
}

/// Every cell that a circle could touch.
fn cells_within(center: Vec2, radius: f32) -> impl Iterator<Item = (i32, i32)> {
    let low = ((center - Vec2::splat(radius)) / CELL_SIZE)
        .floor()
        .as_ivec2();
    let high = ((center + Vec2::splat(radius)) / CELL_SIZE)
        .floor()
        .as_ivec2();
    (low.x..=high.x).flat_map(move |x| (low.y..=high.y).map(move |z| (x, z)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    /// Checks every solid in turn, which is what the grid has to match.
    fn push_out_of_all(all: &[Solid], at: Vec2, radius: f32) -> Vec2 {
        all.iter().fold(at, |at, solid| {
            let offset = at - solid.center;
            let reach = solid.radius + radius;
            if offset.length_squared() < reach * reach {
                solid.center + offset.try_normalize().unwrap_or(Vec2::X) * reach
            } else {
                at
            }
        })
    }

    /// A repeatable spread of points, about 40 meters across.
    fn spread(count: u32) -> impl Iterator<Item = Vec2> {
        (0..count).map(move |index| {
            let turn = index as f32 * 2.399;
            Vec2::from_angle(turn) * (index as f32).sqrt() * 20.0 / (count as f32).sqrt()
        })
    }

    #[spec]
    fn pushing_out() {
        given("a single solid", || {
            let solids = Solids::new(vec![Solid {
                center: Vec2::new(10.0, 0.0),
                radius: 1.0,
            }]);

            when("a body walks into it", || {
                let pushed = solids.push_out(Vec2::new(9.5, 0.0), 0.5);

                test("it ends up touching the edge", || {
                    assert_eq!(pushed, Vec2::new(8.5, 0.0));
                });
            });

            when("a body stands exactly at its center", || {
                let pushed = solids.push_out(Vec2::new(10.0, 0.0), 0.5);

                test("it still gets pushed out", || {
                    assert_eq!(pushed, Vec2::new(11.5, 0.0));
                });
            });

            when("a body walks past it", || {
                let beside = Vec2::new(5.0, 0.0);

                test("it isn't moved", || {
                    assert_eq!(solids.push_out(beside, 0.5), beside);
                });
            });

            when("a body touches it from the next grid cell", || {
                let touching = Vec2::new(8.4, 0.0);
                let across = Solids::new(vec![Solid {
                    center: Vec2::new(7.9, 0.0),
                    radius: 0.1,
                }]);

                test("it's still found", || {
                    assert_ne!(across.push_out(touching, 0.5), touching);
                });
            });
        });

        given("a large solid that spans many grid cells", || {
            let solids = Solids::new(vec![Solid {
                center: Vec2::ZERO,
                radius: 30.0,
            }]);

            when("a body walks into its far edge", || {
                let pushed = solids.push_out(Vec2::new(29.0, 0.0), 0.5);

                test("it's pushed out", || {
                    assert_eq!(pushed, Vec2::new(30.5, 0.0));
                });
            });
        });

        given("a crowd of solids", || {
            let all: Vec<Solid> = spread(400)
                .map(|center| Solid {
                    center,
                    radius: 0.9,
                })
                .collect();
            let solids = Solids::new(all.clone());

            when("bodies stand all over it", || {
                let bodies: Vec<Vec2> = spread(3000).map(|at| at * 1.1).collect();

                test(
                    "the grid gives the same answer as checking everything",
                    || {
                        for at in bodies {
                            assert_eq!(solids.push_out(at, 0.4), push_out_of_all(&all, at, 0.4));
                        }
                    },
                );
            });
        });

        given("no solids", || {
            let solids = Solids::default();

            when("a body moves", || {
                let at = Vec2::new(3.0, 4.0);

                test("it isn't moved", || {
                    assert_eq!(solids.push_out(at, 0.5), at);
                });
            });
        });
    }
}
