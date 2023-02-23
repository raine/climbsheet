use chrono::Timelike;
use eyre::Result;
use secrecy::ExposeSecret;
use tracing::*;

use climbsheet::{config, setup, vertical_life};

#[tokio::main]
async fn main() -> Result<()> {
    setup::setup()?;
    let config = config::read_config();
    info!(?config.gyms, "starting with config");
    let result = vertical_life::VerticalLifeAuthClient::do_auth_flow(
        &config.vertical_life_email,
        config.vertical_life_password.expose_secret(),
    )
    .await?;

    let mut client =
        vertical_life::VerticalLifeClient::new(result.access_token, result.refresh_token);

    let mut all_set_at = vec![];

    for gym_id in &config.gyms {
        info!(?gym_id, "getting gym details");
        let gym = client.get_gym_details(*gym_id).await?;
        info!(?gym.id, ?gym.name, ?gym.boulder_count, ?gym.route_count, "got gym");
        for gym_sector in gym.gym_sectors.iter() {
            let sector = client.get_gym_sector(gym_sector.id).await?;
            for wall in sector.walls.iter() {
                info!(?wall.name, ?wall.category, ?wall.height, "got wall");
                for climb in wall.climbs() {
                    all_set_at.push(climb.set_at);
                }
            }
        }
    }

    // Distribute datetimes in all_set_at to hourly buckets and print how many are in each bucket
    // Print hour as 00:00, zero-padded
    let mut buckets = vec![0; 24];
    for set_at in all_set_at {
        buckets[set_at.hour() as usize] += 1;
    }
    for (hour, count) in buckets.iter().enumerate() {
        println!("{:02}:00: {}", hour, count);
    }

    Ok(())
}
