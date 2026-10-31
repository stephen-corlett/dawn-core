use bevy::log::LogPlugin;
use clap::Parser;
use server::build_app;
use server::config::ServerConfig;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Project Dawn dedicated server")]
struct Args {
    /// UDP port to listen on, instead of the environment's default
    #[arg(long)]
    port: Option<u16>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    // `.env` is optional, because deployments usually set real environment variables.
    dotenvy::dotenv().ok();

    let mut config = match ServerConfig::from_process() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("server: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(port) = args.port {
        config.bind_addr.set_port(port);
    }

    let mut app = build_app(config);
    app.add_plugins(LogPlugin::default());
    app.run();
    ExitCode::SUCCESS
}
