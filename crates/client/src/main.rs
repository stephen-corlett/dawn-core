use bevy::prelude::*;
use clap::Parser;
use client::config::ClientConfig;
use client::movement_plugin;
use client::network::ClientNetworkPlugin;
use client::view::ViewPlugin;
use core::net::SocketAddr;
use foundation::net::tick_duration;
use lightyear::prelude::client::ClientPlugins;
use protocol::ProtocolPlugin;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Project Dawn game client")]
struct Args {
    /// Unique ID for this client. Two clients can't share one.
    #[arg(long, default_value_t = 1)]
    client_id: u64,

    /// Server address to connect to, instead of the environment's default
    #[arg(long)]
    server: Option<SocketAddr>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    // `.env` is optional, and only development reads a key from it.
    dotenvy::dotenv().ok();

    let mut config = match ClientConfig::from_process() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("client: {error}");
            return ExitCode::FAILURE;
        }
    };
    config.client_id = args.client_id;
    if let Some(server) = args.server {
        config.server_addr = server;
    }

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
        ClientNetworkPlugin { config },
        movement_plugin(),
        ViewPlugin,
    ));
    app.run();
    ExitCode::SUCCESS
}
