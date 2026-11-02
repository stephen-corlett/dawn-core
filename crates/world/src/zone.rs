use crate::{Solid, Solids, ZoneShape};
use bevy::prelude::*;
use movement::WalkSurface;

/// A zone players can walk on: its authored shape, and the solid things standing on it.
#[derive(Clone, Debug, PartialEq)]
pub struct Zone {
    shape: ZoneShape,
    solids: Solids,
}

impl Zone {
    pub fn new(shape: ZoneShape, solids: Vec<Solid>) -> Self {
        Self {
            shape,
            solids: Solids::new(solids),
        }
    }

    pub fn shape(&self) -> &ZoneShape {
        &self.shape
    }
}

impl WalkSurface for Zone {
    fn ground_height(&self, at: Vec2) -> f32 {
        self.shape.height(at)
    }

    /// Blocked land goes last, so that nothing can push a player onto it.
    fn push_out(&self, at: Vec2, radius: f32) -> Vec2 {
        self.shape
            .push_out(self.solids.push_out(at, radius), radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const SHAPE: &str = r#"
        ZoneShape(
            name: "test vale",
            seed: 7,
            base_height: 10.0,
            detail: Detail(height: 0.0, spacing: 10.0),
            strokes: [],
            blocked: [
                BlockedLand(
                    name: "the wall",
                    why: "A sheer rock wall",
                    outline: [(10.0, -50.0), (20.0, -50.0), (20.0, 50.0), (10.0, 50.0)],
                ),
            ],
        )
    "#;

    #[spec]
    fn walking_a_zone() {
        given("a zone with a tree beside a wall", || {
            let zone = Zone::new(
                ZoneShape::parse(SHAPE).unwrap(),
                vec![Solid {
                    center: Vec2::new(8.5, 0.0),
                    radius: 1.0,
                }],
            );

            when("the ground is asked for", || {
                let height = zone.ground_height(Vec2::ZERO);

                test("it comes from the shape", || {
                    assert_eq!(height, zone.shape().base_height);
                });
            });

            when("the tree pushes a body towards the wall", || {
                let pushed = zone.push_out(Vec2::new(8.6, 0.0), 0.5);

                test("the body stays off the wall", || {
                    assert!(pushed.x <= 9.5, "{pushed}");
                });
            });

            when("a body walks into the tree away from the wall", || {
                let pushed = zone.push_out(Vec2::new(7.0, 0.0), 0.5);

                test("the tree stops it", || {
                    assert_eq!(pushed, Vec2::new(7.0, 0.0));
                });
            });
        });
    }
}
