//! The world players walk through, a supporting game module: the shape of the ground,
//! what grows on it, and what's solid. It has library-style boundaries, with its size,
//! seed, and contents supplied by the caller.
//!
//! The code at this level is shared. A [`Forest`] grows the same way from the same
//! [`WorldConfig`], so the client and server agree on it without sending it over the
//! network. The `client` feature adds what's only for looking at.

#[cfg(feature = "client")]
pub mod client;
mod forest;
mod terrain;

pub use forest::{Forest, PlacedProp, Solid};

use bevy::prelude::*;

/// Describes a world. The client and server must use the same values, or they'll
/// disagree about the ground and everything on it.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldConfig {
    /// Everything random about the world comes from this.
    pub seed: u32,
    /// Players stay inside a square this far from the world origin.
    pub half_extent: f32,
    /// How far the world carries on past where players can reach.
    pub margin: f32,
    /// How far the level, treeless ground reaches from the world origin.
    pub clearing_radius: f32,
    /// How far the ground rises and falls away from the clearing.
    pub hill_height: f32,
    /// The usual distance between neighboring hills.
    pub hill_spacing: f32,
    /// How far the ground climbs at the edges of the world.
    pub rim_height: f32,
    /// How far the trail around the clearing is from the world origin.
    pub trail_radius: f32,
    /// How wide the bare ground of the trail is.
    pub trail_width: f32,
    pub plantings: Vec<Planting>,
}

/// One kind of thing scattered around the world.
#[derive(Clone, Debug, PartialEq)]
pub struct Planting {
    /// The glTF file to draw, relative to the asset folder.
    pub model: &'static str,
    /// How many to place. Fewer appear if there's no room for them.
    pub count: u32,
    /// The smallest and largest size, as multiples of the model's size.
    pub scale: Vec2,
    /// How far from the world origin these can grow.
    pub reach: f32,
    /// How close the nearest other spaced thing can be. Zero allows any distance.
    pub spacing: f32,
    /// The radius players can't walk through. Zero means they walk straight through.
    pub solid_radius: f32,
    /// Whether these can grow in the clearing.
    pub in_clearing: bool,
}
