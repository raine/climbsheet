use climbsheet::{climb_sheet::add_wall_to_sheet, config, setup, sheets, vertical_life};
use eyre::Result;
use tracing::*;

#[tokio::main]
async fn main() -> Result<()> {
    setup::setup()?;
    let config = config::read_config();
    let sheets = sheets::get_client().await?;
    info!(?config.gyms, "starting with config");

    let result = vertical_life::VerticalLifeAuthClient::do_auth_flow(
        &config.vertical_life_email,
        config.vertical_life_password.expose_secret(),
    )
    .await?;
    let mut client =
        vertical_life::VerticalLifeClient::new(result.access_token, result.refresh_token);
    let spreadsheet = sheets::get_spreadsheet(&sheets, &config.sheet_id).await?;

    for gym_id in &config.gyms {
        info!(?gym_id, "getting gym details");
        let gym = client.get_gym_details(*gym_id).await?;
        info!(?gym.id, ?gym.name, ?gym.boulder_count, ?gym.route_count, "got gym");
        for gym_sector in gym.gym_sectors.iter() {
            info!(?gym_sector.id, "getting gym sector");
            let sector = client.get_gym_sector(gym_sector.id).await?;
            for wall in sector.walls.iter() {
                info!(?wall.name, ?wall.category, ?wall.height, "got wall");
                add_wall_to_sheet(&config, &sheets, &spreadsheet, &gym, wall).await?;
            }
        }
    }

    Ok(())
}
