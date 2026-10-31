//! Tests the real server across its network boundary, using a scripted fake
//! client instead of the game client.

mod connections;
mod movement;

use bevy::prelude::*;
use server::build_app;
use server::config::ServerConfig;
use test_support::{local_addr, prepare};

/// Each test uses its own port, so tests can run at the same time.
fn server_app(port: u16) -> App {
    prepare(build_app(ServerConfig {
        bind_addr: local_addr(port),
        ..ServerConfig::dev_default()
    }))
}
