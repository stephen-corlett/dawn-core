use bevy::gilrs::GilrsPlugin;
use bevy::prelude::*;
use clap::Parser;
use client::config::ClientConfig;
use client::look::{avatar_plugin, world_plugin};
use client::movement_plugin;
use client::network::ClientNetworkPlugin;
use core::net::SocketAddr;
use foundation::net::tick_duration;
use lightyear::prelude::client::ClientPlugins;
use protocol::{ProtocolPlugin, forest, movement_config};
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
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("Project Dawn - Client {}", args.client_id),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: config.asset_folder.clone(),
                ..default()
            })
            // Nothing uses gamepads yet, and this stops keyboards that claim to be one.
            .disable::<GilrsPlugin>(),
    );
    app.add_plugins(ClientPlugins {
        tick_duration: tick_duration(),
    });
    app.add_plugins(ProtocolPlugin);
    let forest = forest(&movement_config());
    app.add_plugins((
        ClientNetworkPlugin { config },
        movement_plugin(forest.clone()),
        world_plugin(forest),
        avatar_plugin(),
    ));
    app.run();
    ExitCode::SUCCESS
}
