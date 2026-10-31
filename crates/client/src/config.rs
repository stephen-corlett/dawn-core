//! Environment variables for the client.
//!
//! A client can only sign its own connection in development, because the private key
//! must never ship to players. Production needs a login service to issue connection
//! tokens, so a production client refuses to start until one exists.

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use env_config::{Env, hex, parsed};
use foundation::net::{DEFAULT_SERVER_PORT, DEV_PRIVATE_KEY, ENVIRONMENT_VAR, PROTOCOL_ID};

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub client_id: u64,
    pub server_addr: SocketAddr,
    pub protocol_id: u64,
    pub private_key: [u8; 32],
}

impl ClientConfig {
    /// Settings for local development and tests.
    pub fn dev_default() -> Self {
        Self::load(Env::development(ENVIRONMENT_VAR)).expect("development defaults are valid")
    }

    pub fn from_process() -> Result<Self, String> {
        Self::load(Env::from_process(ENVIRONMENT_VAR))
    }

    fn load(mut env: Env) -> Result<Self, String> {
        if env.is_production() {
            return Err("production clients need a login service, which doesn't exist yet".into());
        }
        let local = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let server_addr = env.read(
            "DAWN_SERVER_ADDR",
            parsed::<SocketAddr>()
                .desc("Server to connect to")
                .default(SocketAddr::new(local, DEFAULT_SERVER_PORT)),
        );
        let private_key = env.read(
            "DAWN_PRIVATE_KEY",
            hex::<32>()
                .desc("Development signing key, as 64 hex characters")
                .dev_default(DEV_PRIVATE_KEY),
        );
        env.validate()?;

        Ok(Self {
            client_id: 1,
            server_addr: server_addr.value(),
            protocol_id: PROTOCOL_ID,
            private_key: private_key.value(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    fn env(vars: &[(&str, &str)]) -> Env {
        Env::from_vars(ENVIRONMENT_VAR, vars.iter().copied())
    }

    #[spec]
    fn client_config() {
        given("the development environment with no key", || {
            let env = env(&[("DAWN_ENV", "development")]);

            when("config loads", || {
                let config = ClientConfig::load(env).unwrap();

                test("it uses the development key", || {
                    assert_eq!(config.private_key, DEV_PRIVATE_KEY);
                });
            });
        });

        given("the development environment with a server address", || {
            let env = env(&[
                ("DAWN_ENV", "development"),
                ("DAWN_SERVER_ADDR", "127.0.0.1:7000"),
            ]);

            when("config loads", || {
                let config = ClientConfig::load(env).unwrap();

                test("it's used", || {
                    assert_eq!(config.server_addr.port(), 7000);
                });
            });
        });

        given("the development environment with an invalid key", || {
            let env = env(&[("DAWN_ENV", "development"), ("DAWN_PRIVATE_KEY", "bad")]);

            when("config loads", || {
                let config = ClientConfig::load(env);

                test("startup is refused", || {
                    assert!(config.is_err());
                });
            });
        });

        given("production with a key", || {
            let key = "ab".repeat(32);
            let env = env(&[("DAWN_ENV", "production"), ("DAWN_PRIVATE_KEY", &key)]);

            when("config loads", || {
                let config = ClientConfig::load(env);

                test("startup is refused", || {
                    assert!(config.is_err());
                });
            });
        });
    }
}
