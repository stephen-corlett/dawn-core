//! Connects to the server. This works without a window, so tests can run it
//! headlessly.

use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use foundation::net::{DEFAULT_SERVER_ADDR, DEV_PRIVATE_KEY, PROTOCOL_ID};
use lightyear::prelude::client::{
    InputDelayConfig, InputTimelineConfig, NetcodeClient, NetcodeConfig,
};
use lightyear::prelude::*;

/// Lets the server notice a closed client quickly during development.
const CLIENT_TIMEOUT_SECS: i32 = 3;

#[derive(Clone, Debug)]
pub struct ClientSettings {
    pub client_id: u64,
    pub server_addr: SocketAddr,
    pub protocol_id: u64,
    pub private_key: [u8; 32],
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            client_id: 1,
            server_addr: DEFAULT_SERVER_ADDR,
            protocol_id: PROTOCOL_ID,
            private_key: DEV_PRIVATE_KEY,
        }
    }
}

pub struct ClientNetworkPlugin {
    pub settings: ClientSettings,
}

impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings(self.settings.clone()));
        app.insert_resource(PredictionManager::default());
        app.insert_resource(
            InputTimelineConfig::default().with_input_delay(InputDelayConfig::no_input_delay()),
        );
        app.add_systems(Startup, connect);
    }
}

#[derive(Resource)]
struct Settings(ClientSettings);

fn connect(mut commands: Commands, settings: Res<Settings>) -> Result {
    let settings = &settings.0;
    let auth = Authentication::Manual {
        server_addr: settings.server_addr,
        client_id: settings.client_id,
        private_key: settings.private_key,
        protocol_id: settings.protocol_id,
    };
    let netcode = NetcodeClient::new(
        auth,
        NetcodeConfig {
            client_timeout_secs: CLIENT_TIMEOUT_SECS,
            token_expire_secs: -1,
            ..default()
        },
    )?;
    let client = commands
        .spawn((
            Name::from("Client"),
            Client,
            ReplicationReceiver,
            Link::default(),
            // Port 0 lets the OS choose, so several clients can run on one machine.
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
            PeerAddr(settings.server_addr),
            netcode,
            UdpIo::default(),
        ))
        .id();
    commands.trigger(Connect { entity: client });
    Ok(())
}
