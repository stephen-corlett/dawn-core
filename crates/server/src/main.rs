use bevy::log::LogPlugin;
use clap::Parser;
use foundation::config::EnvValues;
use server::{ServerSettings, build_app};
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

    let mut settings = match ServerSettings::load(&EnvValues::read()) {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("server: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(port) = args.port {
        settings.bind_addr.set_port(port);
    }

    let mut app = build_app(settings);
    app.add_plugins(LogPlugin::default());
    app.run();
    ExitCode::SUCCESS
}
