//! What the client and server must agree on: the order network types are
//! registered in, and the tuning values both sides simulate with. Module types
//! and logic belong in their own crates.

mod forest;

pub use forest::forest;

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
        sneak_speed: 1.5,
        run_speed: 5.4,
        sprint_speed: 8.2,
        dodge_speed: 7.2,
        dodge_secs: 0.7,
        dodge_cooldown_secs: 0.4,
        jump_speed: 7.0,
        gravity: 18.0,
        ground_snap: 0.4,
        body_radius: 0.4,
        world_half_extent: 50.0,
    }
}
