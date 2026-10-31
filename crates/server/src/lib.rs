//! The headless, authoritative game server. This crate assembles the game module
//! plugins and owns player connections. Game rules belong in game module crates.

pub mod network;

use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use config::{EnvValues, Environment, NetworkConfig, Secrets};
use core::net::SocketAddr;
use core::time::Duration;
use foundation::net::tick_duration;
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

impl ServerSettings {
    /// Settings for local development and tests.
    pub fn dev_default() -> Self {
        Self::from_parts(NetworkConfig::dev_default(), Secrets::dev_default())
    }

    /// Chooses settings for an environment. Fails if production has no real secrets.
    pub fn load(values: &EnvValues) -> Result<Self, String> {
        let environment = Environment::parse(values.environment.as_deref())?;
        Ok(Self::from_parts(
            NetworkConfig::for_environment(environment),
            Secrets::parse(environment, values.private_key.as_deref())?,
        ))
    }

    fn from_parts(config: NetworkConfig, secrets: Secrets) -> Self {
        Self {
            bind_addr: config.bind_addr,
            protocol_id: config.protocol_id,
            private_key: secrets.private_key,
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
                let settings = ServerSettings::load(&values).unwrap();

                test("the server only listens locally", || {
                    assert!(settings.bind_addr.ip().is_loopback());
                });

                test("it uses the development key", || {
                    assert_eq!(settings.private_key, Secrets::dev_default().private_key);
                });
            });
        });

        given("production with a real key", || {
            let key = "ab".repeat(32);
            let values = values(Some("production"), Some(&key));

            when("settings load", || {
                let settings = ServerSettings::load(&values).unwrap();

                test("the server listens publicly", || {
                    assert!(settings.bind_addr.ip().is_unspecified());
                });

                test("it uses the supplied key", || {
                    assert_eq!(settings.private_key, [0xab; 32]);
                });
            });
        });

        given("no environment and no key", || {
            let values = values(None, None);

            when("settings load", || {
                let settings = ServerSettings::load(&values);

                test("startup is refused", || {
                    assert!(settings.is_err());
                });
            });
        });

        given("an unknown environment", || {
            let values = values(Some("staging"), None);

            when("settings load", || {
                let settings = ServerSettings::load(&values);

                test("startup is refused", || {
                    assert!(settings.is_err());
                });
            });
        });
    }
}
