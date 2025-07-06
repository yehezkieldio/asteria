mod input;
mod keys;
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
        Some(("start", sub_m)) => {
            info!("Starting Asteria client...");

            // Parse the toggle key
            let toggle_key_str = sub_m.get_one::<String>("toggle-key").unwrap();
            let toggle_key = if toggle_key_str.starts_with("0x") {
                u32::from_str_radix(&toggle_key_str[2..], 16).map_err(|_| {
                    anyhow::anyhow!("Invalid hexadecimal key code: {}", toggle_key_str)
                })?
            } else {
                toggle_key_str
                    .parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("Invalid key code: {}", toggle_key_str))?
            };

            info!("=== Asteria Client Started ===");
            info!("Toggle key set to: 0x{:02x}", toggle_key);
            info!("Press the toggle key to enable/disable relay");
            info!("When relay is enabled:");
            info!("  - Your input is sent to Windows");
            info!("  - Local Linux input is suppressed");
            info!("Press the toggle key again to regain control of Linux");
            info!("================================");

            // Create network client and input capture
            let network_client = NetworkClient::new()?;
            let mut input_capture = InputCapture::new_with_toggle_key(toggle_key)?;

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
        .subcommand(
            Command::new("start").about("Start the Asteria client").arg(
                Arg::new("toggle-key")
                    .long("toggle-key")
                    .help("Hexadecimal key code for the toggle key (e.g., 0x1D for Left Ctrl)")
                    .value_name("KEY_CODE")
                    .default_value("0x1D"),
            ),
        )
        .subcommand(
            Command::new("ping")
                .about("Send a ping to test connectivity")
                .arg(Arg::new("host").help("Specific host to ping").index(1)),
        )
}
