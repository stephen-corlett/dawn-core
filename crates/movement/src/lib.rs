//! Player movement, a supporting game module. It has library-style boundaries:
//! a small public API, with tuning values supplied by the caller.
//!
//! The code at this level is shared. The `server` and `client` features add
//! their plugins, and both run [`next_position`], so client prediction rarely
//! disagrees with the server.

#[cfg(feature = "client")]
pub mod client;
mod rules;
#[cfg(feature = "server")]
pub mod server;

pub use rules::next_position;

use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

/// Tuning values supplied by the game. The client and server must use the same
/// values, or prediction will constantly disagree with the server.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct MovementConfig {
    /// Walking speed in world units per second.
    pub speed: f32,
    /// Movement stays inside a square this far from the world origin.
    pub world_half_extent: f32,
}

/// An entity's position in the world. The server owns the true value.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect, Default)]
pub struct PlayerPosition(pub Vec3);

impl Ease for PlayerPosition {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        FunctionCurve::new(Interval::UNIT, move |t| {
            PlayerPosition(start.0.lerp(end.0, t))
        })
    }
}

/// What a player is asking to do this tick. Clients send requests like this,
/// never positions, so the server stays in charge of the result.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub struct MoveInput {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
}

impl MapEntities for MoveInput {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

/// Registers this game module's network types. Add it after lightyear's client or
/// server plugins.
pub struct MovementProtocolPlugin;

impl Plugin for MovementProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(input::native::InputPlugin::<MoveInput>::default());
        app.component::<PlayerPosition>()
            .replicate()
            .predict()
            .add_linear_interpolation();
    }
}
