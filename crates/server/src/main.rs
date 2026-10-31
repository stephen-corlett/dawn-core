use bevy::log::LogPlugin;
use clap::Parser;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use foundation::net::DEFAULT_SERVER_PORT;
use server::{ServerSettings, build_app};

#[derive(Parser)]
#[command(about = "Project Dawn dedicated server")]
struct Args {
    /// UDP port to listen on
    #[arg(long, default_value_t = DEFAULT_SERVER_PORT)]
    port: u16,
}

fn main() {
    let args = Args::parse();
    let mut app = build_app(ServerSettings {
        bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), args.port),
        ..Default::default()
    });
    app.add_plugins(LogPlugin::default());
    app.run();
}
