//! Environment variables for the server. Values that are safe to commit live here as
//! defaults, and secrets only come from the environment or a gitignored `.env` file.

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use env_config::{Env, hex, parsed};
use foundation::net::{DEFAULT_SERVER_PORT, DEV_PRIVATE_KEY, ENVIRONMENT_VAR, PROTOCOL_ID};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub protocol_id: u64,
    pub private_key: [u8; 32],
}

impl ServerConfig {
    /// Settings for local development and tests.
    pub fn dev_default() -> Self {
        Self::load(Env::development(ENVIRONMENT_VAR)).expect("development defaults are valid")
    }

    pub fn from_process() -> Result<Self, String> {
        Self::load(Env::from_process(ENVIRONMENT_VAR))
    }

    fn load(mut env: Env) -> Result<Self, String> {
        let public = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
        let local = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let bind_addr = env.read(
            "DAWN_BIND_ADDR",
            parsed::<SocketAddr>()
                .desc("Address the server listens on")
                .default(SocketAddr::new(public, DEFAULT_SERVER_PORT))
                .dev_default(SocketAddr::new(local, DEFAULT_SERVER_PORT)),
        );
        let private_key = env.read(
            "DAWN_PRIVATE_KEY",
            hex::<32>()
                .desc("Signs connection tokens, as 64 hex characters")
                .dev_default(DEV_PRIVATE_KEY)
                .forbid_in_production(DEV_PRIVATE_KEY),
        );
        env.validate()?;

        Ok(Self {
            bind_addr: bind_addr.value(),
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
    fn server_config() {
        given("the development environment with no key", || {
            let env = env(&[("DAWN_ENV", "development")]);

            when("config loads", || {
                let config = ServerConfig::load(env).unwrap();

                test("the server only listens locally", || {
                    assert!(config.bind_addr.ip().is_loopback());
                });

                test("it uses the development key", || {
                    assert_eq!(config.private_key, DEV_PRIVATE_KEY);
                });
            });
        });

        given("production with a real key", || {
            let key = "ab".repeat(32);
            let env = env(&[("DAWN_ENV", "production"), ("DAWN_PRIVATE_KEY", &key)]);

            when("config loads", || {
                let config = ServerConfig::load(env).unwrap();

                test("the server listens publicly", || {
                    assert!(config.bind_addr.ip().is_unspecified());
                });

                test("it uses the supplied key", || {
                    assert_eq!(config.private_key, [0xab; 32]);
                });
            });
        });

        given("a custom bind address", || {
            let key = "ab".repeat(32);
            let env = env(&[
                ("DAWN_BIND_ADDR", "127.0.0.1:7000"),
                ("DAWN_PRIVATE_KEY", &key),
            ]);

            when("config loads", || {
                let config = ServerConfig::load(env).unwrap();

                test("it's used", || {
                    assert_eq!(config.bind_addr.port(), 7000);
                });
            });
        });

        given("no environment and no key", || {
            let env = env(&[]);

            when("config loads", || {
                let config = ServerConfig::load(env);

                test("startup is refused", || {
                    assert!(config.is_err());
                });
            });
        });

        given("production with the development key", || {
            let key = "00".repeat(32);
            let env = env(&[("DAWN_PRIVATE_KEY", &key)]);

            when("config loads", || {
                let config = ServerConfig::load(env);

                test("startup is refused", || {
                    assert!(config.is_err());
                });
            });
        });
    }
}
