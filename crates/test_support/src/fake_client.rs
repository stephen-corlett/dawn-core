//! A scripted client for testing the real server across its network boundary.

use crate::local_addr;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use foundation::config::{NetworkConfig, Secrets};
use foundation::net::tick_duration;
use foundation::player::PlayerId;
use lightyear::prelude::client::input::*;
use lightyear::prelude::client::{ClientPlugins, NetcodeClient, NetcodeConfig};
use lightyear::prelude::input::native::*;
use lightyear::prelude::*;
use movement::MoveInput;
use protocol::ProtocolPlugin;

/// Connection details that tests may want to get wrong on purpose.
#[derive(Clone, Copy, Debug)]
pub struct FakeClientSettings {
    pub port: u16,
    pub client_id: u64,
    pub protocol_id: u64,
    pub private_key: [u8; 32],
}

impl FakeClientSettings {
    pub fn new(port: u16, client_id: u64) -> Self {
        Self {
            port,
            client_id,
            protocol_id: NetworkConfig::dev_default().protocol_id,
            private_key: Secrets::dev_default().private_key,
        }
    }
}

/// The input this client sends every tick. Tests change it to script behavior.
#[derive(Resource, Default)]
pub struct ScriptedInput(pub MoveInput);

pub fn fake_client_app(settings: FakeClientSettings) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, StatesPlugin));
    app.add_plugins(ClientPlugins {
        tick_duration: tick_duration(),
    });
    app.add_plugins(ProtocolPlugin);
    app.insert_resource(PredictionManager::default());
    app.init_resource::<ScriptedInput>();
    app.add_systems(
        FixedPreUpdate,
        write_scripted_input.in_set(InputSystems::WriteClientInputs),
    );
    app.add_observer(mark_controlled_player);

    let server_addr = local_addr(settings.port);
    let netcode = NetcodeClient::new(
        Authentication::Manual {
            server_addr,
            client_id: settings.client_id,
            private_key: settings.private_key,
            protocol_id: settings.protocol_id,
        },
        NetcodeConfig {
            token_expire_secs: -1,
            ..default()
        },
    )
    .expect("the fake client's settings are valid");
    let client = app
        .world_mut()
        .spawn((
            Client,
            ReplicationReceiver,
            Link::default(),
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
            PeerAddr(server_addr),
            netcode,
            UdpIo::default(),
        ))
        .id();
    app.add_systems(Startup, move |mut commands: Commands| {
        commands.trigger(Connect { entity: client });
    });
    app
}

/// Disconnects gracefully, the way a client does when a player logs out.
pub fn disconnect(app: &mut App) {
    let client = app
        .world_mut()
        .query_filtered::<Entity, With<Client>>()
        .single(app.world())
        .expect("the fake client has one connection");
    app.world_mut().trigger(Disconnect { entity: client });
}

fn write_scripted_input(
    script: Res<ScriptedInput>,
    mut players: Query<&mut ActionState<MoveInput>, With<InputMarker<MoveInput>>>,
) {
    if let Ok(mut action_state) = players.single_mut() {
        action_state.0 = script.0.clone();
    }
}

fn mark_controlled_player(
    trigger: On<Add<Controlled>>,
    players: Query<(), With<PlayerId>>,
    mut commands: Commands,
) {
    if players.contains(trigger.entity) {
        commands
            .entity(trigger.entity)
            .insert(InputMarker::<MoveInput>::default());
    }
}
