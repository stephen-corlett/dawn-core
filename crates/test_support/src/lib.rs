//! Test helpers. Only tests may depend on this crate.
//!
//! The fake client and stub server share the real network protocol, but none
//! of the real client or server code. That keeps integration tests honest:
//! each one exercises a single real program across its network boundary.

pub mod fake_client;
pub mod stub_server;

use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use std::thread::sleep;
use std::time::Instant;

const TIMEOUT: Duration = Duration::from_secs(15);
const UPDATE_PAUSE: Duration = Duration::from_millis(2);

pub fn local_addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

/// Finishes plugin setup, which `App::run` normally does.
pub fn prepare(mut app: App) -> App {
    app.finish();
    app.cleanup();
    app
}

/// Updates every app until `done` passes, panicking if it never does.
pub fn run_until(apps: &mut [&mut App], what: &str, mut done: impl FnMut(&mut [&mut App]) -> bool) {
    let deadline = Instant::now() + TIMEOUT;
    while Instant::now() < deadline {
        update_all(apps);
        if done(apps) {
            return;
        }
        sleep(UPDATE_PAUSE);
    }
    panic!("timed out waiting for: {what}");
}

/// Updates every app for a fixed time, for checking that nothing happens.
pub fn run_for(apps: &mut [&mut App], duration: Duration) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        update_all(apps);
        sleep(UPDATE_PAUSE);
    }
}

fn update_all(apps: &mut [&mut App]) {
    for app in apps.iter_mut() {
        app.update();
    }
}

/// Counts the entities that match a filter.
pub fn count<F: QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), F>()
        .iter(app.world())
        .count()
}
