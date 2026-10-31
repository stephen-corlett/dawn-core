//! The headless, authoritative game server. This crate assembles the game module
//! plugins and owns player connections. Game rules belong in game module crates.

pub mod network;

use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use foundation::net::{DEFAULT_SERVER_PORT, DEV_PRIVATE_KEY, PROTOCOL_ID, tick_duration};
use lightyear::prelude::server::ServerPlugins;
use movement::server::MovementServerPlugin;
use protocol::{ProtocolPlugin, movement_config};

/// How long the main loop sleeps between updates, so an idle server doesn't
/// spin a CPU core.
const LOOP_WAIT: Duration = Duration::from_millis(5);

#[derive(Clone, Debug)]
pub struct ServerSettings {
    pub bind_addr: SocketAddr,
    pub protocol_id: u64,
    pub private_key: [u8; 32],
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DEFAULT_SERVER_PORT),
            protocol_id: PROTOCOL_ID,
            private_key: DEV_PRIVATE_KEY,
        }
    }
}

/// Builds the server without starting it, so tests can step it by hand.
pub fn build_app(settings: ServerSettings) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(LOOP_WAIT)),
        TransformPlugin,
        StatesPlugin,
        DiagnosticsPlugin,
    ));
    app.add_plugins(ServerPlugins {
        tick_duration: tick_duration(),
    });
    app.add_plugins(ProtocolPlugin);
    app.add_plugins((
        MovementServerPlugin {
            config: movement_config(),
        },
        network::ServerNetworkPlugin { settings },
    ));
    app
}
