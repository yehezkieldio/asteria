use anyhow::{Ok, Result};
use asteria_core::init_logging;
use clap::{Arg, ArgMatches, Command};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let matches: ArgMatches = build_cli().get_matches();

    init_logging();

    match matches.subcommand() {
        Some(("start", _)) => {
            info!("Starting Asteria server...");
        }
        Some(("ping", sub_m)) => {
            let host = sub_m.get_one::<String>("host").cloned();
            if let Some(host) = host {
                info!("Pinging host: {}", host);
            } else {
                info!("Pinging using discovery...");
            }
        }
        _ => {
            error!("Invalid command. Use --help for usage information.");
        }
    }

    Ok(())
}

fn build_cli() -> Command {
    Command::new("asteria-server")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Asteria server application")
        .subcommand(Command::new("start").about("Start the Asteria server"))
        .subcommand(
            Command::new("ping")
                .about("Send a ping to test connectivity")
                .arg(
                    Arg::new("host")
                        .help("Specific host to ping (optional, will use discovery)")
                        .long("host")
                        .value_name("HOST"),
                ),
        )
}
