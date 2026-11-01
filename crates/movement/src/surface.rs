use bevy::prelude::*;
use std::sync::Arc;

/// What movement needs to know about the ground it walks on. The game supplies this,
/// so movement knows nothing about how the world is built. The client and server must
/// use the same surface, or prediction will constantly disagree with the server.
pub trait WalkSurface: Send + Sync + 'static {
    /// How high the ground is at a point on the XZ plane.
    fn ground_height(&self, at: Vec2) -> f32;

    /// Moves a circle on the XZ plane out of anything solid, and returns its new center.
    fn push_out(&self, at: Vec2, radius: f32) -> Vec2;
}

/// The surface players walk on, shared by everything that needs it.
#[derive(Resource, Clone)]
pub struct Surface(pub Arc<dyn WalkSurface>);

/// Level ground at height zero with nothing on it.
pub struct FlatSurface;

impl WalkSurface for FlatSurface {
    fn ground_height(&self, _at: Vec2) -> f32 {
        0.0
    }

    fn push_out(&self, at: Vec2, _radius: f32) -> Vec2 {
        at
    }
}
