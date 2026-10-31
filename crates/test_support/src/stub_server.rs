//! A scripted server for testing the real client across its network boundary.
//! It spawns a player per client but runs no game rules, so positions only
//! change when a test changes them.

use crate::local_addr;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use foundation::config::{NetworkConfig, Secrets};
use foundation::net::{SEND_INTERVAL, tick_duration};
use foundation::player::PlayerId;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use movement::PlayerPosition;
use protocol::ProtocolPlugin;

/// Marks a scripted player that no connected client controls.
#[derive(Component)]
struct RemotePlayer;

pub fn stub_server_app(port: u16) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, StatesPlugin));
    app.add_plugins(ServerPlugins {
        tick_duration: tick_duration(),
    });
    app.add_plugins(ProtocolPlugin);
    app.insert_resource(ReplicationMetadata::new(SEND_INTERVAL));
    app.add_observer(handle_new_link);
    app.add_observer(spawn_player);

    let server = app
        .world_mut()
        .spawn((
            NetcodeServer::new(NetcodeConfig {
                protocol_id: NetworkConfig::dev_default().protocol_id,
                private_key: Secrets::dev_default().private_key,
                ..default()
            }),
            LocalAddr(local_addr(port)),
            ServerUdpIo,
        ))
        .id();
    app.add_systems(Startup, move |mut commands: Commands| {
        commands.trigger(Start { entity: server });
    });
    app
}

/// Moves every client-controlled player, as if the server corrected them.
pub fn set_controlled_positions(app: &mut App, position: Vec3) {
    let mut players = app
        .world_mut()
        .query_filtered::<&mut PlayerPosition, Without<RemotePlayer>>();
    for mut player in players.iter_mut(app.world_mut()) {
        player.0 = position;
    }
}

/// Adds a player that represents someone else in the world.
pub fn spawn_remote_player(app: &mut App, position: Vec3) -> Entity {
    app.world_mut()
        .spawn((
            RemotePlayer,
            PlayerId(PeerId::Netcode(u64::MAX)),
            PlayerPosition(position),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ))
        .id()
}

/// Moves a player added by `spawn_remote_player`.
pub fn move_remote_player(app: &mut App, player: Entity, position: Vec3) {
    app.world_mut()
        .get_mut::<PlayerPosition>(player)
        .expect("the remote player exists")
        .0 = position;
}

fn handle_new_link(trigger: On<Add<LinkOf>>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(ReplicationSender);
}

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
        PlayerId(client_id),
        PlayerPosition::default(),
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
        InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
        ControlledBy {
            owner: trigger.entity,
            lifetime: default(),
        },
    ));
}
