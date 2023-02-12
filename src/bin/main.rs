use climbsheet::{
    config, setup,
    sheet_util::{
        format_sheet_name, parse_location_from_gym_name, wall_category_to_plural_human_type,
    },
    sheets::{
        self, get_updated_row_from_update_values_response, set_cell_background_color,
        sort_sheet_by_column, SheetsClient, Spreadsheet,
    },
    vertical_life,
};
use eyre::Result;
use tracing::*;

/// Find sheet in spreadsheet that matches gym's name and the wall category
/// For example, for gym_name "Kiipeilyareena Ristikko" and wall_category "gym_bouldering"
/// this should return "Ristikko - Boulderit" and it's numeric zero-indexed sheet id
pub fn get_sheet_for_gym_name_and_wall_category(
    spreadsheet: &Spreadsheet,
    gym_name: &str,
    wall_category: &str,
) -> (String, i32) {
    let sheets = spreadsheet.sheets.as_ref().unwrap();
    let location_name = parse_location_from_gym_name(gym_name);
    let sheet_name = format_sheet_name(
        location_name,
        &wall_category_to_plural_human_type(wall_category),
    );
    let sheet = sheets
        .iter()
        .find(|s| s.properties.as_ref().unwrap().title == Some(sheet_name.to_string()))
        .unwrap_or_else(|| panic!("sheet '{sheet_name}' not found in spreadsheet"));

    let sheet_id_num = sheet.properties.as_ref().unwrap().sheet_id.unwrap();
    (sheet_name, sheet_id_num)
}

pub async fn append_climb_to_sheet(
    config: &config::Config,
    sheets: &SheetsClient,
    sheet_id: &str,
    sheet_name: &str,
    sheet_id_num: i32,
    climb: &vertical_life::Climb,
) -> Result<()> {
    let res = sheets::append_row(sheets, sheet_id, sheet_name, climb.to_sheet_row()).await?;
    tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
    let row_n = get_updated_row_from_update_values_response(&res);
    set_cell_background_color(
        sheets,
        sheet_id,
        sheet_id_num,
        &climb.color,
        row_n,
        config.climb_color_column_idx,
    )
    .await?;
    Ok(())
}

async fn add_wall_to_sheet(
    config: &config::Config,
    sheets: &SheetsClient,
    spreadsheet: &Spreadsheet,
    gym: &vertical_life::Gym,
    wall: &vertical_life::Wall,
) -> Result<()> {
    let sheet_id = spreadsheet.spreadsheet_id.as_ref().unwrap();
    let (sheet_name, sheet_id_num) =
        get_sheet_for_gym_name_and_wall_category(spreadsheet, &gym.name, &wall.category);

    for climb in wall.climbs() {
        info!(?climb, "got climb");
        append_climb_to_sheet(config, sheets, sheet_id, &sheet_name, sheet_id_num, climb).await?;
    }

    sort_sheet_by_column(sheets, sheet_id, sheet_id_num, config.date_column_idx).await?;

    Ok(())
}

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
