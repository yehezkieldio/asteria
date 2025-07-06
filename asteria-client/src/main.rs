mod input;
mod network;

use anyhow::{Ok, Result};
use asteria_core::init_logging;
use clap::{Arg, ArgMatches, Command};
use tracing::{error, info};

use crate::input::InputCapture;
use crate::network::NetworkClient;

#[tokio::main]
async fn main() -> Result<()> {
    let matches: ArgMatches = build_cli().get_matches();

    init_logging();

    match matches.subcommand() {
        Some(("start", _)) => {
            info!("Starting Asteria client...");

            // Create network client and input capture
            let network_client = NetworkClient::new()?;
            let mut input_capture = InputCapture::new()?;

            // Start the client
            tokio::select! {
                result = input_capture.start_and_relay(network_client) => {
                    if let Err(e) = result {
                        error!("Input capture failed: {}", e);
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                }
            }
        }
        Some(("ping", sub_m)) => {
            let mut network_client = NetworkClient::new()?;
            let host = sub_m.get_one::<String>("host");

            if let Some(host) = host {
                info!("Pinging host: {}", host);
                // TODO: Override config with specific host
            }

            network_client.ping().await?;
        }
        _ => {
            error!("Invalid command. Use --help for usage information.");
        }
    }

    Ok(())
}

fn build_cli() -> Command {
    Command::new("asteria-client")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Asteria client application")
        .subcommand(Command::new("start").about("Start the Asteria client"))
        .subcommand(
            Command::new("ping")
                .about("Send a ping to test connectivity")
                .arg(Arg::new("host").help("Specific host to ping").index(1)),
        )
}
