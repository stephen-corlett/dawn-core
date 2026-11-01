//! Player movement, a supporting game module. It has library-style boundaries:
//! a small public API, with tuning values supplied by the caller.
//!
//! The code at this level is shared. The `server` and `client` features add
//! their plugins, and both run [`next_body`], so client prediction rarely
//! disagrees with the server.

#[cfg(feature = "client")]
pub mod client;
mod rules;
#[cfg(feature = "server")]
pub mod server;
#[cfg(any(feature = "client", feature = "server"))]
mod simulation;
mod surface;

pub use rules::{Body, next_body};
pub use surface::{FlatSurface, Surface, WalkSurface};

use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

/// Tuning values supplied by the game. The client and server must use the same
/// values, or prediction will constantly disagree with the server.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct MovementConfig {
    /// Speed of each [`Gait`], in world units per second.
    pub sneak_speed: f32,
    pub run_speed: f32,
    pub sprint_speed: f32,
    /// Speed during a dodge roll, in world units per second.
    pub dodge_speed: f32,
    /// How long a dodge roll lasts.
    pub dodge_secs: f32,
    /// How long a player stays exposed after a dodge roll. They can't jump or roll again
    /// until it's over.
    pub dodge_recovery_secs: f32,
    /// The fastest a player can move during recovery, whatever gait they ask for.
    pub dodge_recovery_speed: f32,
    /// Upward speed at the start of a jump, in world units per second.
    pub jump_speed: f32,
    /// Downward acceleration in world units per second squared.
    pub gravity: f32,
    /// How far below a walking player the ground can drop in one tick before they
    /// fall instead of following it down.
    pub ground_snap: f32,
    /// How far a player's body keeps from anything solid.
    pub body_radius: f32,
    /// Movement stays inside a square this far from the world origin.
    pub world_half_extent: f32,
}

/// An entity's position in the world. The server owns the true value.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect, Default)]
#[require(VerticalSpeed, Dodge, Sneaking)]
pub struct PlayerPosition(pub Vec3);

/// How fast an entity is rising or falling. It's zero while standing on the ground.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect, Default)]
pub struct VerticalSpeed(pub f32);

/// An entity's dodge roll. The roll overrides the player's steering, and recovery slows
/// them down afterward, so committing to a roll carries a risk. Anything that rewards a
/// well-timed roll can end recovery early by clearing `recovery_secs`.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect, Default)]
pub struct Dodge {
    /// Where the roll is heading on the XZ plane.
    pub direction: Vec2,
    /// How much of the roll is left.
    pub remaining_secs: f32,
    /// How much of the recovery after a roll is left.
    pub recovery_secs: f32,
}

impl Dodge {
    pub fn is_rolling(&self) -> bool {
        self.remaining_secs > 0.0
    }

    pub fn is_recovering(&self) -> bool {
        self.recovery_secs > 0.0
    }

    /// Whether the player is tied up in a roll or the recovery from one.
    pub fn is_committed(&self) -> bool {
        self.is_rolling() || self.is_recovering()
    }
}

/// Whether an entity is moving in a crouch. Nothing about its position shows this, so
/// other players need it sent to them.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect, Default)]
pub struct Sneaking(pub bool);

/// The direction "forward" points for local input, as radians around the up axis.
/// Whatever frames the player, such as a camera, keeps this up to date.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct MoveHeading(pub f32);

impl Ease for PlayerPosition {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        FunctionCurve::new(Interval::UNIT, move |t| {
            PlayerPosition(start.0.lerp(end.0, t))
        })
    }
}

/// What a player is asking to do this tick. Clients send requests like this,
/// never positions, so the server stays in charge of the result.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Reflect)]
pub struct MoveInput {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub dodge: bool,
    pub gait: Gait,
    /// The direction `forward` points, as radians around the up axis. Zero faces
    /// negative Z. Clients choose this, so the rules can't trust it to be a number.
    pub yaw: f32,
}

/// How fast a player is asking to move.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum Gait {
    Sneak,
    #[default]
    Run,
    Sprint,
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
        app.component::<VerticalSpeed>().replicate().predict();
        app.component::<Dodge>().replicate().predict();
        app.component::<Sneaking>().replicate().predict();
    }
}
