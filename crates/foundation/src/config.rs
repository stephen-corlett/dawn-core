//! Environment configuration. Values that are safe to commit have a `default()` for
//! production and a `dev_default()` for development. Secrets only ever come from
//! environment variables, which a gitignored `.env` file can supply locally.

use crate::net::{DEFAULT_SERVER_PORT, PROTOCOL_ID};
use core::fmt;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::env;

pub const ENVIRONMENT_VAR: &str = "DAWN_ENV";
pub const PRIVATE_KEY_VAR: &str = "DAWN_PRIVATE_KEY";

const PRIVATE_KEY_BYTES: usize = 32;

/// Raw values read from the process. Loading code takes these as plain data, so
/// tests can exercise every case without changing real environment variables.
#[derive(Clone, Default)]
pub struct EnvValues {
    pub environment: Option<String>,
    pub private_key: Option<String>,
}

impl EnvValues {
    pub fn read() -> Self {
        Self {
            environment: env::var(ENVIRONMENT_VAR).ok(),
            private_key: env::var(PRIVATE_KEY_VAR).ok(),
        }
    }
}

/// Unset means production, so forgetting to configure a server fails safely.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn parse(value: Option<&str>) -> Result<Self, String> {
        match value {
            None | Some("production") => Ok(Self::Production),
            Some("development") => Ok(Self::Development),
            Some(other) => Err(format!(
                "{ENVIRONMENT_VAR} must be development or production, not {other}"
            )),
        }
    }
}

/// Network settings that are safe to commit.
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkConfig {
    pub bind_addr: SocketAddr,
    pub server_addr: SocketAddr,
    pub protocol_id: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DEFAULT_SERVER_PORT),
            server_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_SERVER_PORT),
            protocol_id: PROTOCOL_ID,
        }
    }
}

impl NetworkConfig {
    pub fn dev_default() -> Self {
        Self {
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_SERVER_PORT),
            ..Self::default()
        }
    }

    pub fn for_environment(environment: Environment) -> Self {
        match environment {
            Environment::Development => Self::dev_default(),
            Environment::Production => Self::default(),
        }
    }
}

/// Values that must never be committed. There's no `default()`, because production
/// has no safe value to fall back on.
#[derive(Clone, PartialEq)]
pub struct Secrets {
    pub private_key: [u8; PRIVATE_KEY_BYTES],
}

impl fmt::Debug for Secrets {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secrets { .. }")
    }
}

impl Secrets {
    /// An all-zero key for local development. Production refuses it.
    pub fn dev_default() -> Self {
        Self {
            private_key: [0; PRIVATE_KEY_BYTES],
        }
    }

    pub fn parse(environment: Environment, key: Option<&str>) -> Result<Self, String> {
        let secrets = match (key, environment) {
            (Some(key), _) => Self {
                private_key: parse_key(key)?,
            },
            (None, Environment::Development) => Self::dev_default(),
            (None, Environment::Production) => {
                return Err(format!("{PRIVATE_KEY_VAR} is required in production"));
            }
        };
        if environment == Environment::Production && secrets == Self::dev_default() {
            return Err(format!(
                "{PRIVATE_KEY_VAR} can't be the development key in production"
            ));
        }
        Ok(secrets)
    }
}

fn parse_key(hex: &str) -> Result<[u8; PRIVATE_KEY_BYTES], String> {
    let invalid = || format!("{PRIVATE_KEY_VAR} must be {PRIVATE_KEY_BYTES} bytes of hex");
    if hex.len() != PRIVATE_KEY_BYTES * 2 || !hex.is_ascii() {
        return Err(invalid());
    }
    let mut key = [0; PRIVATE_KEY_BYTES];
    for (byte, pair) in key.iter_mut().zip(hex.as_bytes().chunks(2)) {
        let pair = core::str::from_utf8(pair).map_err(|_| invalid())?;
        *byte = u8::from_str_radix(pair, 16).map_err(|_| invalid())?;
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const REAL_KEY: &str = "0102030405060708091011121314151617181920212223242526272829303132";
    const ZERO_KEY: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[spec]
    fn environment_config() {
        given("the current process environment", || {
            let expected = env::var(ENVIRONMENT_VAR).ok();

            when("its values are read", || {
                let values = EnvValues::read();

                test("they match the process", || {
                    assert_eq!(values.environment, expected);
                });
            });
        });

        given("no environment name", || {
            let value = None;

            when("the environment is read", || {
                let environment = Environment::parse(value);

                test("it's production", || {
                    assert_eq!(environment, Ok(Environment::Production));
                });
            });
        });

        given("the development environment name", || {
            let value = Some("development");

            when("the environment is read", || {
                let environment = Environment::parse(value);

                test("it's development", || {
                    assert_eq!(environment, Ok(Environment::Development));
                });
            });
        });

        given("an unknown environment name", || {
            let value = Some("staging");

            when("the environment is read", || {
                let environment = Environment::parse(value);

                test("it's rejected", || {
                    assert!(environment.is_err());
                });
            });
        });

        given("the development environment", || {
            let environment = Environment::Development;

            when("network settings are chosen", || {
                let config = NetworkConfig::for_environment(environment);

                test("the server only listens locally", || {
                    assert!(config.bind_addr.ip().is_loopback());
                });
            });

            when("no key is set", || {
                let secrets = Secrets::parse(environment, None);

                test("it falls back to the development key", || {
                    assert_eq!(secrets, Ok(Secrets::dev_default()));
                });
            });
        });

        given("the production environment", || {
            let environment = Environment::Production;

            when("network settings are chosen", || {
                let config = NetworkConfig::for_environment(environment);

                test("the server listens publicly", || {
                    assert!(config.bind_addr.ip().is_unspecified());
                });
            });

            when("no key is set", || {
                let secrets = Secrets::parse(environment, None);

                test("startup is refused", || {
                    assert!(secrets.is_err());
                });
            });

            when("the development key is set", || {
                let secrets = Secrets::parse(environment, Some(ZERO_KEY));

                test("startup is refused", || {
                    assert!(secrets.is_err());
                });
            });

            when("a real key is set", || {
                let secrets = Secrets::parse(environment, Some(REAL_KEY)).unwrap();

                test("its bytes are decoded", || {
                    assert_eq!(secrets.private_key[..3], [0x01, 0x02, 0x03]);
                });
            });

            when("the key is too short", || {
                let secrets = Secrets::parse(environment, Some("0102"));

                test("it's rejected", || {
                    assert!(secrets.is_err());
                });
            });

            when("the key isn't hex", || {
                let invalid = "zz".repeat(PRIVATE_KEY_BYTES);
                let secrets = Secrets::parse(environment, Some(&invalid));

                test("it's rejected", || {
                    assert!(secrets.is_err());
                });
            });
        });

        given("loaded secrets", || {
            let secrets = Secrets::dev_default();

            when("they're printed for debugging", || {
                let printed = format!("{secrets:?}");

                test("the key isn't shown", || {
                    assert_eq!(printed, "Secrets { .. }");
                });
            });
        });
    }
}
