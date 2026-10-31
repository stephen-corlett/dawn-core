//! Connects to the server. This works without a window, so tests can run it
//! headlessly.

use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use foundation::config::{EnvValues, Environment, NetworkConfig, Secrets};
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

impl ClientSettings {
    /// Settings for local development and tests.
    pub fn dev_default() -> Self {
        let config = NetworkConfig::dev_default();
        Self {
            client_id: 1,
            server_addr: config.server_addr,
            protocol_id: config.protocol_id,
            private_key: Secrets::dev_default().private_key,
        }
    }

    /// Chooses settings for an environment. A client can only self-sign its
    /// connection in development, because the private key must never ship to players.
    /// Production needs a login service to issue connection tokens.
    pub fn load(values: &EnvValues) -> Result<Self, String> {
        match Environment::parse(values.environment.as_deref())? {
            Environment::Development => Ok(Self {
                private_key: Secrets::parse(
                    Environment::Development,
                    values.private_key.as_deref(),
                )?
                .private_key,
                ..Self::dev_default()
            }),
            Environment::Production => {
                Err("production clients need a login service, which doesn't exist yet".into())
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    fn values(environment: Option<&str>, private_key: Option<&str>) -> EnvValues {
        EnvValues {
            environment: environment.map(str::to_owned),
            private_key: private_key.map(str::to_owned),
        }
    }

    #[spec]
    fn loading_settings() {
        given("the development environment with no key", || {
            let values = values(Some("development"), None);

            when("settings load", || {
                let settings = ClientSettings::load(&values).unwrap();

                test("it uses the development key", || {
                    assert_eq!(settings.private_key, Secrets::dev_default().private_key);
                });
            });
        });

        given("the development environment with a key", || {
            let key = "ab".repeat(32);
            let values = values(Some("development"), Some(&key));

            when("settings load", || {
                let settings = ClientSettings::load(&values).unwrap();

                test("it uses the supplied key", || {
                    assert_eq!(settings.private_key, [0xab; 32]);
                });
            });
        });

        given("production with a key", || {
            let key = "ab".repeat(32);
            let values = values(Some("production"), Some(&key));

            when("settings load", || {
                let settings = ClientSettings::load(&values);

                test("startup is refused", || {
                    assert!(settings.is_err());
                });
            });
        });
    }
}
