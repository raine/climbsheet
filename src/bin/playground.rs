use climbsheet::{climb_sheet::ClimbSheet, config, setup, vertical_life};
use eyre::Result;
use secrecy::ExposeSecret;
use tracing::*;

#[tokio::main]
async fn main() -> Result<()> {
    setup::setup()?;
    let config = config::read_config();
    let climbsheet = ClimbSheet::new(&config).await?;
    info!(?config.gyms, "starting with config");

    let result = vertical_life::VerticalLifeAuthClient::do_auth_flow(
        &config.vertical_life_email,
        config.vertical_life_password.expose_secret(),
    )
    .await?;
    let mut client =
        vertical_life::VerticalLifeClient::new(result.access_token, result.refresh_token);

    for gym_id in &config.gyms {
        info!(?gym_id, "getting gym details");
        let gym = client.get_gym_details(*gym_id).await?;
        climbsheet.highlight_new_routes(&gym).await?;
    }

    Ok(())
}
