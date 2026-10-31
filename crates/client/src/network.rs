//! Connects to the server. This works without a window, so tests can run it
//! headlessly.

use crate::config::ClientConfig;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use lightyear::prelude::client::{
    InputDelayConfig, InputTimelineConfig, NetcodeClient, NetcodeConfig,
};
use lightyear::prelude::*;

/// Lets the server notice a closed client quickly during development.
const CLIENT_TIMEOUT_SECS: i32 = 3;

pub struct ClientNetworkPlugin {
    pub config: ClientConfig,
}

impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings(self.config.clone()));
        app.insert_resource(PredictionManager::default());
        app.insert_resource(
            InputTimelineConfig::default().with_input_delay(InputDelayConfig::no_input_delay()),
        );
        app.add_systems(Startup, connect);
    }
}

#[derive(Resource)]
struct Settings(ClientConfig);

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
