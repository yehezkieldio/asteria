use anyhow::{Ok, Result};
use asteria_core::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    Ok(())
}
