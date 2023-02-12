#![allow(dead_code, unused_imports, unused_variables)]
use climbsheet::{config, setup, sheets};
use eyre::Result;
use tracing::*;

#[tokio::main]
async fn main() -> Result<()> {
    setup::setup()?;
    let config = config::read_config();
    let sheets = sheets::get_client().await?;
    info!(?config.gyms, "starting with config");
    let rows = sheets::get_sheet_rows(&sheets, &config.sheet_id, "Ristikko - Reitit").await?;
    dbg!(rows);
    Ok(())
}
