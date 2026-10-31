//! The game client. This crate assembles the game module plugins and owns the
//! connection and world view. Game features belong in game module crates.

pub mod config;
pub mod network;
pub mod view;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use config::ClientConfig;
use foundation::net::tick_duration;
use lightyear::prelude::client::ClientPlugins;
use movement::client::{MoveBindings, MovementClientPlugin};
use network::ClientNetworkPlugin;
use protocol::{ProtocolPlugin, movement_config};

/// The movement plugin with this game's tuning and default keys.
pub fn movement_plugin() -> MovementClientPlugin {
    MovementClientPlugin {
        config: movement_config(),
        bindings: MoveBindings {
            forward: vec![KeyCode::KeyW, KeyCode::ArrowUp],
            back: vec![KeyCode::KeyS, KeyCode::ArrowDown],
            left: vec![KeyCode::KeyA, KeyCode::ArrowLeft],
            right: vec![KeyCode::KeyD, KeyCode::ArrowRight],
        },
    }
}

/// Builds the client without a window, for tests.
pub fn build_headless_app(config: ClientConfig) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, StatesPlugin));
    app.add_plugins(ClientPlugins {
        tick_duration: tick_duration(),
    });
    app.add_plugins(ProtocolPlugin);
    app.add_plugins((ClientNetworkPlugin { config }, movement_plugin()));
    app
}
