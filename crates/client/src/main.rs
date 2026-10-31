use bevy::prelude::*;
use clap::Parser;
use client::movement_plugin;
use client::network::{ClientNetworkPlugin, ClientSettings};
use client::view::ViewPlugin;
use core::net::SocketAddr;
use foundation::net::{DEFAULT_SERVER_ADDR, tick_duration};
use lightyear::prelude::client::ClientPlugins;
use protocol::ProtocolPlugin;

#[derive(Parser)]
#[command(about = "Project Dawn game client")]
struct Args {
    /// Unique ID for this client. Two clients can't share one.
    #[arg(long, default_value_t = 1)]
    client_id: u64,

    /// Server address to connect to
    #[arg(long, default_value_t = DEFAULT_SERVER_ADDR)]
    server: SocketAddr,
}

fn main() {
    let args = Args::parse();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("Project Dawn - Client {}", args.client_id),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(ClientPlugins {
        tick_duration: tick_duration(),
    });
    app.add_plugins(ProtocolPlugin);
    app.add_plugins((
        ClientNetworkPlugin {
            settings: ClientSettings {
                client_id: args.client_id,
                server_addr: args.server,
                ..default()
            },
        },
        movement_plugin(),
        ViewPlugin,
    ));
    app.run();
}
