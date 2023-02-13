#![allow(dead_code, unused_imports, unused_variables)]
use climbsheet::{
    config, setup,
    sheets::{self, set_range_background_color},
};
use eyre::Result;
use google_sheets4::api::GridRange;
use tracing::*;

#[tokio::main]
async fn main() -> Result<()> {
    setup::setup()?;
    let config = config::read_config();
    let sheets = sheets::get_client(&config.service_account_credentials_path).await?;
    info!(?config.gyms, "starting with config");
    set_range_background_color(
        &sheets,
        &config.sheet_id,
        "#d3ffe2",
        GridRange {
            start_row_index: Some(1),
            end_row_index: Some(4),
            start_column_index: Some(config.grade_column_idx),
            end_column_index: Some(config.grade_column_idx + 1),
            sheet_id: Some(0),
        },
    )
    .await?;

    Ok(())
}
