//! Connection and timing settings shared by the client and server.

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;

/// How many simulation ticks run each second.
pub const TICK_RATE_HZ: f64 = 20.0;

/// How often the server sends world updates to clients.
pub const SEND_INTERVAL: Duration = Duration::from_millis(50);

/// Clients with a different protocol ID are refused when they connect.
/// Increase this whenever a protocol change breaks older clients.
pub const PROTOCOL_ID: u64 = 1;

/// Development key for signing connection tokens. Real deployments must load
/// a secret key from outside the repository instead.
pub const DEV_PRIVATE_KEY: [u8; 32] = [0; 32];

pub const DEFAULT_SERVER_PORT: u16 = 5000;

pub const DEFAULT_SERVER_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_SERVER_PORT);

pub fn tick_duration() -> Duration {
    Duration::from_secs_f64(1.0 / TICK_RATE_HZ)
}
