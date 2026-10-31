//! Tests the real client against a stub server that runs no game rules. The
//! client has no window, so these tests cover its behavior, not its rendering.

mod connection;
mod players;
mod prediction;

use ::client::build_headless_app;
use ::client::config::ClientConfig;
use bevy::prelude::*;
use foundation::player::PlayerId;
use lightyear::prelude::*;
use test_support::stub_server::stub_server_app;
use test_support::{count, local_addr, prepare, run_until};

/// Starts a stub server and a client, and waits for the client's own player.
/// Each test uses its own port, so tests can run at the same time.
fn connected_client(port: u16) -> (App, App) {
    let mut server = prepare(stub_server_app(port));
    let mut client = client_app(port);
    run_until(&mut [&mut server, &mut client], "local player", |apps| {
        count::<(With<PlayerId>, With<Predicted>)>(apps[1]) == 1
    });
    (server, client)
}

fn client_app(port: u16) -> App {
    let mut app = build_headless_app(ClientConfig {
        server_addr: local_addr(port),
        ..ClientConfig::dev_default()
    });
    app.insert_resource(ButtonInput::<KeyCode>::default());
    prepare(app)
}
