use crate::{
    config,
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
fn get_sheet_for_gym_name_and_wall_category(
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

async fn append_climb_to_sheet(
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

pub async fn add_wall_to_sheet(
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

/// Returns for example "Ristikko - Reitit"
fn format_sheet_name(gym_location_name: &str, plural_human_item_type: &str) -> String {
    format!("{} - {}", gym_location_name, plural_human_item_type)
}

fn wall_category_to_plural_human_type(wall_category: &str) -> String {
    match wall_category {
        "gym_bouldering" => "Boulderit",
        "gym_sportclimbing" => "Reitit",
        _ => panic!("unknown wall category: {}", wall_category),
    }
    .to_string()
}

/// With input "Kiipeilyareena Ristikko" this should return "Ristikko"
fn parse_location_from_gym_name(gym_name: &str) -> &str {
    gym_name.split(' ').nth(1).unwrap()
}
