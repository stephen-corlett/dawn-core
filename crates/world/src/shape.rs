//! The authored shape of a zone: its ground and the land players can't enter, read from text.
//! Every stroke and blocked area has a name and a reason, because a zone is a list of decisions
//! that a person can read and change. See docs/zone-pipeline.md.

use crate::terrain;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;

/// A position on the XZ plane, in meters.
type Point = (f32, f32);

fn point(at: Point) -> Vec2 {
    Vec2::new(at.0, at.1)
}

/// The shape of one zone. Parse it with [`ZoneShape::parse`], which rejects anything that has no
/// reason to be there.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ZoneShape {
    pub name: String,
    /// Everything random about the zone's ground comes from this.
    pub seed: u32,
    /// How high the ground is before any stroke changes it.
    pub base_height: f32,
    /// Small rises and falls added over everything, so that nothing looks machined.
    pub detail: Detail,
    /// Applied in order, so that a later stroke can change what an earlier one made.
    pub strokes: Vec<Stroke>,
    pub blocked: Vec<BlockedLand>,
}

/// Fine roughness on top of the strokes.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Detail {
    /// How far the ground rises and falls.
    pub height: f32,
    /// The usual distance between neighboring rises.
    pub spacing: f32,
}

/// One named change to the ground, along a line.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Stroke {
    pub name: String,
    /// Why the land does this here. A stroke with no reason is rejected.
    pub why: String,
    pub action: StrokeAction,
    /// The middle of the stroke. One point makes a hill or a hollow.
    pub line: Vec<Point>,
    /// How far from the line the stroke keeps its full effect.
    pub flat: f32,
    /// How far past that the stroke takes to fade out.
    pub falloff: f32,
}

/// What a stroke does to the ground.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum StrokeAction {
    /// Adds this height. A negative height cuts down.
    Raise(f32),
    /// Blends the ground to exactly this height, for towns and roads.
    Level(f32),
}

/// An area players can't enter, however gentle its ground looks.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlockedLand {
    pub name: String,
    /// What stops a player here, as they'd see it. An area with no reason is rejected.
    pub why: String,
    /// The edge of the area. The last point joins back to the first.
    pub outline: Vec<Point>,
}

/// Why a zone's shape can't be used.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeError {
    /// The text isn't a shape at all.
    Unreadable(String),
    /// The text is readable, and these are all the problems with it.
    Invalid(Vec<String>),
}

impl fmt::Display for ShapeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(reason) => write!(formatter, "unreadable zone shape: {reason}"),
            Self::Invalid(problems) => {
                write!(formatter, "invalid zone shape: {}", problems.join("; "))
            }
        }
    }
}

impl std::error::Error for ShapeError {}

impl ZoneShape {
    /// Reads a shape from text, and reports every problem with it at once.
    pub fn parse(text: &str) -> Result<Self, ShapeError> {
        let shape: Self =
            ron::from_str(text).map_err(|error| ShapeError::Unreadable(error.to_string()))?;
        let problems = shape.problems();
        if problems.is_empty() {
            Ok(shape)
        } else {
            Err(ShapeError::Invalid(problems))
        }
    }

    fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.name.trim().is_empty() {
            problems.push("the zone has no name".to_string());
        }
        if self.detail.spacing <= 0.0 {
            problems.push("detail spacing must be above zero".to_string());
        }

        let mut names = HashSet::new();
        let reasons = self
            .strokes
            .iter()
            .map(|stroke| (&stroke.name, &stroke.why))
            .chain(self.blocked.iter().map(|land| (&land.name, &land.why)));
        for (name, why) in reasons {
            if name.trim().is_empty() {
                problems.push("something has no name".to_string());
            } else if !names.insert(name.as_str()) {
                problems.push(format!("{name}: the name is used twice"));
            }
            if why.trim().is_empty() {
                problems.push(format!("{name}: no reason given"));
            }
        }

        for stroke in &self.strokes {
            if stroke.line.is_empty() {
                problems.push(format!("{}: the line has no points", stroke.name));
            }
            if stroke.flat < 0.0 {
                problems.push(format!("{}: flat can't be negative", stroke.name));
            }
            if stroke.falloff <= 0.0 {
                problems.push(format!("{}: falloff must be above zero", stroke.name));
            }
        }
        for land in &self.blocked {
            if land.outline.len() < 3 {
                problems.push(format!("{}: an outline needs three points", land.name));
            }
        }
        problems
    }

    /// How high the ground is at a point.
    pub fn height(&self, at: Vec2) -> f32 {
        let shaped = self
            .strokes
            .iter()
            .fold(self.base_height, |ground, stroke| stroke.apply(ground, at));
        let at = at / self.detail.spacing;
        shaped + (terrain::noise(self.seed, at) * 2.0 - 1.0) * self.detail.height
    }

    /// Whether a point is on land players can't enter.
    pub fn is_blocked(&self, at: Vec2) -> bool {
        self.blocked.iter().any(|land| land.contains(at))
    }

    /// Moves a circle off any blocked land, and returns its new center.
    pub fn push_out(&self, at: Vec2, radius: f32) -> Vec2 {
        self.blocked
            .iter()
            .fold(at, |at, land| land.push_out(at, radius))
    }
}

impl Stroke {
    fn apply(&self, ground: f32, at: Vec2) -> f32 {
        let past_flat = (self.distance(at) - self.flat) / self.falloff;
        let strength = 1.0 - terrain::smooth(past_flat.clamp(0.0, 1.0));
        match self.action {
            StrokeAction::Raise(height) => ground + height * strength,
            StrokeAction::Level(height) => ground + (height - ground) * strength,
        }
    }

    fn distance(&self, at: Vec2) -> f32 {
        match self.line.as_slice() {
            [only] => at.distance(point(*only)),
            line => line
                .windows(2)
                .map(|pair| at.distance(nearest_on(point(pair[0]), point(pair[1]), at)))
                .fold(f32::MAX, f32::min),
        }
    }
}

impl BlockedLand {
    fn edges(&self) -> impl Iterator<Item = (Vec2, Vec2)> {
        let next = self.outline.iter().cycle().skip(1);
        self.outline
            .iter()
            .zip(next)
            .map(|(&from, &to)| (point(from), point(to)))
    }

    fn contains(&self, at: Vec2) -> bool {
        self.edges()
            .filter(|(from, to)| {
                (from.y > at.y) != (to.y > at.y)
                    && at.x < from.x + (at.y - from.y) / (to.y - from.y) * (to.x - from.x)
            })
            .count()
            % 2
            == 1
    }

    fn push_out(&self, at: Vec2, radius: f32) -> Vec2 {
        let Some((nearest, along)) = self
            .edges()
            .map(|(from, to)| (nearest_on(from, to, at), to - from))
            .min_by(|a, b| {
                a.0.distance_squared(at)
                    .total_cmp(&b.0.distance_squared(at))
            })
        else {
            return at;
        };
        let inside = self.contains(at);
        if !inside && at.distance_squared(nearest) >= radius * radius {
            return at;
        }
        let away = if inside { nearest - at } else { at - nearest };
        let away = away
            .try_normalize()
            .unwrap_or_else(|| self.outward(nearest, along, radius));
        nearest + away * radius
    }

    /// Which way is out from a point exactly on the edge, where the body's position gives no
    /// direction of its own.
    fn outward(&self, on_edge: Vec2, along: Vec2, step: f32) -> Vec2 {
        let across = along.perp().try_normalize().unwrap_or(Vec2::X);
        if self.contains(on_edge + across * step) {
            -across
        } else {
            across
        }
    }
}

/// The point on a line segment nearest to another point.
fn nearest_on(from: Vec2, to: Vec2, at: Vec2) -> Vec2 {
    let along = to - from;
    let length_squared = along.length_squared();
    if length_squared == 0.0 {
        return from;
    }
    from + along * ((at - from).dot(along) / length_squared).clamp(0.0, 1.0)
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
            strokes: [
                Stroke(
                    name: "the long ridge",
                    why: "It hides the far valley",
                    action: Raise(100.0),
                    line: [(0.0, 0.0), (100.0, 0.0)],
                    flat: 5.0,
                    falloff: 20.0,
                ),
                Stroke(
                    name: "the town green",
                    why: "People built where it was level",
                    action: Level(30.0),
                    line: [(50.0, 0.0)],
                    flat: 8.0,
                    falloff: 4.0,
                ),
                Stroke(
                    name: "the hollow",
                    why: "Water gathers here",
                    action: Raise(-5.0),
                    line: [(0.0, 500.0)],
                    flat: 0.0,
                    falloff: 10.0,
                ),
            ],
            blocked: [
                BlockedLand(
                    name: "the crag",
                    why: "A sheer rock wall",
                    outline: [(200.0, 0.0), (300.0, 0.0), (300.0, 100.0), (200.0, 100.0)],
                ),
            ],
        )
    "#;

    fn shape() -> ZoneShape {
        ZoneShape::parse(SHAPE).unwrap()
    }

    fn problems(text: &str) -> Vec<String> {
        match ZoneShape::parse(text) {
            Err(ShapeError::Invalid(problems)) => problems,
            other => panic!("expected problems, got {other:?}"),
        }
    }

    #[spec]
    fn reading_a_shape() {
        given("a shape that breaks several rules", || {
            let text = SHAPE
                .replace("It hides the far valley", " ")
                .replace("the town green", "the long ridge")
                .replace("falloff: 4.0", "falloff: 0.0")
                .replace("flat: 8.0", "flat: -1.0")
                .replace("line: [(0.0, 500.0)]", "line: []")
                .replace("name: \"test vale\"", "name: \"\"")
                .replace("spacing: 10.0", "spacing: 0.0")
                .replace("name: \"the crag\"", "name: \"\"")
                .replace(
                    "[(200.0, 0.0), (300.0, 0.0), (300.0, 100.0), (200.0, 100.0)]",
                    "[(200.0, 0.0)]",
                );

            when("it's read", || {
                let found = problems(&text).join("\n");

                test("a missing reason is reported", || {
                    assert!(found.contains("the long ridge: no reason given"), "{found}");
                });

                test("a repeated name is reported", || {
                    assert!(found.contains("used twice"), "{found}");
                });

                test("every other problem is reported with it", || {
                    for problem in [
                        "the zone has no name",
                        "detail spacing",
                        "something has no name",
                        "flat can't be negative",
                        "falloff must be above zero",
                        "the line has no points",
                        "an outline needs three points",
                    ] {
                        assert!(found.contains(problem), "{problem} in {found}");
                    }
                });
            });
        });

        given("text with a misspelled field", || {
            let text = SHAPE.replace("base_height", "base_hieght");

            when("it's read", || {
                let error = ZoneShape::parse(&text).unwrap_err();

                test("it's refused, and the message says why", || {
                    assert!(matches!(error, ShapeError::Unreadable(_)));
                    assert!(error.to_string().contains("base_hieght"), "{error}");
                });
            });
        });

        given("a list of problems", || {
            let error = ShapeError::Invalid(vec!["one".into(), "two".into()]);

            when("it's shown", || {
                let shown = error.to_string();

                test("it lists them all", || {
                    assert_eq!(shown, "invalid zone shape: one; two");
                });
            });
        });
    }

    #[spec]
    fn the_ground() {
        given("a shape", || {
            let shape = shape();

            when("a point is far from every stroke", || {
                let height = shape.height(Vec2::new(0.0, -900.0));

                test("the ground is at its base height", || {
                    assert_eq!(height, 10.0);
                });
            });

            when("a point is on the ridge's flat part", || {
                let height = shape.height(Vec2::new(20.0, 3.0));

                test("the ridge has its full height", || {
                    assert_eq!(height, 110.0);
                });
            });

            when("a point is partway down the ridge's side", || {
                let height = shape.height(Vec2::new(20.0, 15.0));

                test("the height is between the two", || {
                    assert!(height > 10.0 && height < 110.0, "{height}");
                });
            });

            when("a point is past the end of the ridge", || {
                let height = shape.height(Vec2::new(103.0, 0.0));

                test("the end of the line counts as the ridge", || {
                    assert_eq!(height, 110.0);
                });
            });

            when(
                "a later stroke levels ground that an earlier one raised",
                || {
                    let height = shape.height(Vec2::new(50.0, 0.0));

                    test("the later stroke wins", || {
                        assert_eq!(height, 30.0);
                    });
                },
            );

            when("a stroke has one point and a negative height", || {
                let height = shape.height(Vec2::new(0.0, 500.0));

                test("it cuts a hollow", || {
                    assert_eq!(height, 5.0);
                });
            });
        });

        given("a shape with detail", || {
            let mut shape = shape();
            shape.detail.height = 1.0;

            when("the same point is asked for twice", || {
                let at = Vec2::new(0.37, -900.2);

                test(
                    "the answer matches, because both sides have to agree",
                    || {
                        assert_eq!(shape.height(at), shape.height(at));
                    },
                );

                test("the ground stays within the detail's height", || {
                    assert!((shape.height(at) - 10.0).abs() <= 1.0);
                });
            });
        });

        given("a stroke whose line repeats a point", || {
            let mut shape = shape();
            shape.strokes[0].line = vec![(20.0, 0.0), (20.0, 0.0)];

            when("a point is on it", || {
                let height = shape.height(Vec2::new(20.0, 0.0));

                test("it still counts", || {
                    assert_eq!(height, 110.0);
                });
            });
        });
    }

    #[spec]
    fn blocked_land() {
        given("a shape with a blocked square", || {
            let shape = shape();

            when("points are checked", || {
                let inside = Vec2::new(250.0, 50.0);
                let outside = Vec2::new(150.0, 50.0);

                test("only the point inside is blocked", || {
                    assert!(shape.is_blocked(inside));
                    assert!(!shape.is_blocked(outside));
                });
            });

            when("a body is well clear of it", || {
                let at = Vec2::new(150.0, 50.0);

                test("it isn't moved", || {
                    assert_eq!(shape.push_out(at, 0.5), at);
                });
            });

            when("a body brushes its edge from outside", || {
                let pushed = shape.push_out(Vec2::new(199.8, 50.0), 0.5);

                test("it's held one radius away", || {
                    assert_eq!(pushed, Vec2::new(199.5, 50.0));
                });
            });

            when("a body has crossed into it", || {
                let pushed = shape.push_out(Vec2::new(202.0, 50.0), 0.5);

                test("it's put back out through the nearest edge", || {
                    assert_eq!(pushed, Vec2::new(199.5, 50.0));
                    assert!(!shape.is_blocked(pushed));
                });
            });

            when("a body is exactly on any of its edges", || {
                let west = shape.push_out(Vec2::new(200.0, 50.0), 0.5);
                let east = shape.push_out(Vec2::new(300.0, 50.0), 0.5);
                let south = shape.push_out(Vec2::new(250.0, 0.0), 0.5);
                let north = shape.push_out(Vec2::new(250.0, 100.0), 0.5);

                test("it's always moved out, never through", || {
                    assert_eq!(west, Vec2::new(199.5, 50.0));
                    assert_eq!(east, Vec2::new(300.5, 50.0));
                    assert_eq!(south, Vec2::new(250.0, -0.5));
                    assert_eq!(north, Vec2::new(250.0, 100.5));
                });
            });
        });
    }
}
