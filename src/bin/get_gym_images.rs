use std::io::Write;

use eyre::Result;
use tracing::*;

use climbsheet::{config, setup, vertical_life};

async fn download_image(url: &str, path: &str) -> Result<()> {
    let mut image_file = std::fs::File::create(path)?;
    let image_response = reqwest::get(url).await?;
    let image_bytes = image_response.bytes().await?;
    image_file.write_all(&image_bytes)?;
    Ok(())
}

#[tokio::main]
// Save overview images for each gym sector to images/
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

    for gym_id in &config.gyms {
        info!(?gym_id, "getting gym details");
        let gym = client.get_gym_details(*gym_id).await?;
        info!(?gym.id, ?gym.name, ?gym.boulder_count, ?gym.route_count, "got gym");
        for gym_sector in gym.gym_sectors.iter() {
            dbg!(&gym_sector);
            let image_id = &gym_sector.overview;
            let image_url = vertical_life::format_image_url(image_id, 3750);
            let path = format!(
                "images/{}-{}-{}.jpg",
                gym.name.replace(' ', "-"),
                gym_sector.name.replace(' ', "-"),
                gym_sector.category
            );
            download_image(&image_url, &path).await?;
            info!(?image_id, ?image_url, ?path, "downloaded image");
        }
    }

    Ok(())
}
