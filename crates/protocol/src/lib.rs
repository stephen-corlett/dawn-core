//! What the client and server must agree on: the order network types are
//! registered in, and the tuning values both sides simulate with. Module types
//! and logic belong in their own crates.

use bevy::prelude::*;
use foundation::player::PlayerId;
use lightyear::prelude::*;
use movement::{MovementConfig, MovementProtocolPlugin};

/// Add this after lightyear's client or server plugins.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<PlayerId>().replicate();
        app.add_plugins(MovementProtocolPlugin);
    }
}

pub fn movement_config() -> MovementConfig {
    MovementConfig {
        speed: 7.0,
        world_half_extent: 50.0,
    }
}
