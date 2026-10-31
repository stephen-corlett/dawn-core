//! Connection and timing settings shared by the client and server.

use core::time::Duration;

/// How many simulation ticks run each second.
pub const TICK_RATE_HZ: f64 = 20.0;

/// How often the server sends world updates to clients.
pub const SEND_INTERVAL: Duration = Duration::from_millis(50);

/// Clients with a different protocol ID are refused when they connect.
/// Increase this whenever a protocol change breaks older clients.
pub const PROTOCOL_ID: u64 = 1;

/// Selects `development` or `production`. Unset means production.
pub const ENVIRONMENT_VAR: &str = "DAWN_ENV";

/// Signs connections in development, and the client and server must agree on it. The
/// server's config rejects this key in production.
pub const DEV_PRIVATE_KEY: [u8; 32] = [0; 32];

pub const DEFAULT_SERVER_PORT: u16 = 5000;

pub fn tick_duration() -> Duration {
    Duration::from_secs_f64(1.0 / TICK_RATE_HZ)
}
