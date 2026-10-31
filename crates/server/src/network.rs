//! Accepts client connections and spawns a player for each one.

use crate::config::ServerConfig;
use bevy::prelude::*;
use foundation::net::SEND_INTERVAL;
use foundation::player::PlayerId;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use movement::PlayerPosition;

/// Printed on stdout once the server is accepting connections. `xtask` watches
/// for this exact text, and keeps its own copy so it needn't compile the game.
const READY_LINE: &str = "DAWN_SERVER_READY";

pub struct ServerNetworkPlugin {
    pub config: ServerConfig,
}

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ReplicationMetadata::new(SEND_INTERVAL));
        app.insert_resource(Settings(self.config.clone()));
        app.add_systems(Startup, start_server);
        app.add_observer(announce_ready);
        app.add_observer(handle_new_link);
        app.add_observer(spawn_player);
    }
}

#[derive(Resource)]
struct Settings(ServerConfig);

fn start_server(mut commands: Commands, settings: Res<Settings>) {
    let settings = &settings.0;
    let server = commands
        .spawn((
            Name::from("Server"),
            NetcodeServer::new(NetcodeConfig {
                protocol_id: settings.protocol_id,
                private_key: settings.private_key,
                ..default()
            }),
            LocalAddr(settings.bind_addr),
            ServerUdpIo,
        ))
        .id();
    commands.trigger(Start { entity: server });
}

/// `cargo xtask dev` waits for this line before launching clients.
fn announce_ready(_trigger: On<Add<Started>>, settings: Res<Settings>) {
    info!("Listening on {}", settings.0.bind_addr);
    println!("{READY_LINE}");
}

/// A link exists as soon as a client makes contact, before it is verified.
fn handle_new_link(trigger: On<Add<LinkOf>>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert((ReplicationSender, Name::from("Client link")));
}

/// Players spawn only after the connection is verified as `Connected`.
fn spawn_player(
    trigger: On<Add<Connected>>,
    clients: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
) {
    let Ok(client_id) = clients.get(trigger.entity) else {
        return;
    };
    let client_id = client_id.0;
    commands.spawn((
        Name::from("Player"),
        PlayerId(client_id),
        PlayerPosition::default(),
        Replicate::to_clients(NetworkTarget::All),
        // The owner predicts their own character and interpolates everyone else's.
        PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
        InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
        ControlledBy {
            owner: trigger.entity,
            lifetime: default(),
        },
    ));
    info!("Spawned player for {client_id:?}");
}
