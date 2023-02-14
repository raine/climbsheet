#![allow(dead_code, unused_imports, unused_variables)]
use std::collections::HashSet;

use climbsheet::{climb_sheet::ClimbSheet, config, setup, sheets, vertical_life};
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
    let mut new_climbs = vec![];

    for gym_id in &config.gyms {
        info!(?gym_id, "getting gym details");
        let gym = client.get_gym_details(*gym_id).await?;
        // Get existing climbs from spreadsheet for the gym, so that we can check in
        // add_wall_to_sheet if the climb already exists in the sheet, and skip adding it
        let gym_sheet_routes = climbsheet.get_gym_routes_from_sheet(&gym).await?;
        let gym_sheet_routes_set: HashSet<_> = gym_sheet_routes.into_iter().collect();

        info!(?gym.id, ?gym.name, ?gym.boulder_count, ?gym.route_count, "got gym");
        for gym_sector in gym.gym_sectors.iter() {
            info!(?gym_sector.id, "getting gym sector");
            let sector = client.get_gym_sector(gym_sector.id).await?;
            for wall in sector.walls.iter() {
                info!(?wall.name, ?wall.category, ?wall.height, "got wall");

                new_climbs.extend_from_slice(
                    &climbsheet
                        .add_wall_to_sheet(&gym_sheet_routes_set, &gym, wall)
                        .await?,
                );
            }
        }
    }

    info!(?new_climbs, "done");
    Ok(())
}
