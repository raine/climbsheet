use std::collections::HashSet;

use crate::{
    config,
    sheets::{self, SheetsClient, Spreadsheet},
    vertical_life,
};
use eyre::Result;
use google_sheets4::api::GridRange;
use tracing::*;

/// Spreadsheet rows of type Vec<String> are parsed to these to make them a bit
/// more comprehensible
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ClimbSheetRow {
    route_card_label: String,
    difficulty: String,
    set_at_human_date: String,
    route_setter: String,
    parent_name: String,
}

impl From<&vertical_life::Climb> for ClimbSheetRow {
    fn from(climb: &vertical_life::Climb) -> Self {
        climb.to_sheet_row().into()
    }
}

impl From<Vec<String>> for ClimbSheetRow {
    fn from(row: Vec<String>) -> Self {
        // Start from first non empty element
        // For some reason, row might not have the first column with background color as ""
        let row = row
            .into_iter()
            .skip_while(|s| s.is_empty())
            .collect::<Vec<_>>();
        Self {
            route_card_label: row[0].clone(),
            difficulty: row[1].clone(),
            set_at_human_date: row[2].clone(),
            route_setter: row[3].clone(),
            parent_name: row[4].clone(),
        }
    }
}

pub struct ClimbSheet<'a> {
    sheet_id: String,
    config: &'a config::Config,
    sheet_client: SheetsClient,
    spreadsheet: Spreadsheet,
}

impl<'a> ClimbSheet<'a> {
    pub async fn new(config: &'a config::Config) -> Result<ClimbSheet<'a>> {
        let sheet_client = sheets::get_client(&config.service_account_credentials_path).await?;
        let spreadsheet = sheets::get_spreadsheet(&sheet_client, &config.sheet_id).await?;

        Ok(Self {
            sheet_id: config.sheet_id.clone(),
            config,
            sheet_client,
            spreadsheet,
        })
    }

    /// For a gym, return rows from the spreadsheets all sheets (pages) that belong to the gym For
    /// example, for Ristikko, you would return rows from Ristikko - Reitit and Ristikko - Boulderit
    /// pages
    pub async fn get_gym_routes_from_sheet(
        &self,
        gym: &vertical_life::Gym,
    ) -> Result<Vec<ClimbSheetRow>> {
        info!(?gym, "getting gym routes from sheet");
        let gym_sheet_names = vertical_life::WALL_CATEGORIES.iter().map(|c| {
            format_sheet_name(
                parse_location_from_gym_name(&gym.name),
                &wall_category_to_plural_human_type(c),
            )
        });

        let sheets_rows =
            futures::future::join_all(gym_sheet_names.into_iter().map(|sheet_name| async move {
                sheets::get_sheet_rows(&self.sheet_client, &self.config.sheet_id, &sheet_name)
                    .await
                    .map(|rows| {
                        rows.into_iter()
                            // Skip the header row
                            .skip(1)
                            .map(ClimbSheetRow::from)
                            .collect::<Vec<_>>()
                    })
            }))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(sheets_rows.into_iter().flatten().collect())
    }

    pub async fn add_wall_to_sheet(
        &self,
        gym_sheet_routes: &HashSet<ClimbSheetRow>,
        gym: &vertical_life::Gym,
        wall: &vertical_life::Wall,
    ) -> Result<Vec<vertical_life::Climb>> {
        let mut new_climbs = vec![];
        let (sheet_name, sheet_id_num) =
            self.get_sheet_for_gym_name_and_wall_category(&gym.name, &wall.category);

        for climb in wall.climbs() {
            info!(?climb, "got climb");
            if gym_sheet_routes.contains(&climb.into()) {
                info!(?climb, "climb already exists in sheet, skipping");
                continue;
            }

            self.append_climb_to_sheet(&sheet_name, sheet_id_num, climb)
                .await?;
            new_climbs.push(climb.to_owned());
        }

        sheets::sort_sheet_by_column(
            &self.sheet_client,
            &self.sheet_id,
            sheet_id_num,
            self.config.date_column_idx,
        )
        .await?;
        Ok(new_climbs)
    }

    async fn append_climb_to_sheet(
        &self,
        sheet_name: &str,
        sheet_id_num: i32,
        climb: &vertical_life::Climb,
    ) -> Result<()> {
        let res = sheets::append_row(
            &self.sheet_client,
            &self.sheet_id,
            sheet_name,
            climb.to_sheet_row(),
        )
        .await?;
        tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
        let row_n = sheets::get_updated_row_from_update_values_response(&res);
        sheets::set_range_background_color(
            &self.sheet_client,
            &self.sheet_id,
            Some(sheets::color_from_hex(&climb.color)),
            GridRange {
                sheet_id: Some(sheet_id_num),
                start_row_index: Some(row_n),
                end_row_index: Some(row_n + 1),
                start_column_index: Some(self.config.climb_color_column_idx),
                end_column_index: Some(self.config.climb_color_column_idx + 1),
            },
        )
        .await?;
        Ok(())
    }

    /// Find sheet in spreadsheet that matches gym's name and the wall category
    /// For example, for gym_name "Kiipeilyareena Ristikko" and wall_category "gym_bouldering"
    /// this should return "Ristikko - Boulderit" and it's numeric zero-indexed sheet id
    fn get_sheet_for_gym_name_and_wall_category(
        &self,
        gym_name: &str,
        wall_category: &str,
    ) -> (String, i32) {
        let sheets = self.spreadsheet.sheets.as_ref().unwrap();
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

    pub async fn reset_grade_column_background(&self, sheet_id_num: i32) -> Result<()> {
        sheets::set_range_background_color(
            &self.sheet_client,
            &self.sheet_id,
            None,
            GridRange {
                sheet_id: Some(sheet_id_num),
                start_row_index: Some(1),
                end_row_index: None,
                start_column_index: Some(self.config.climb_color_column_idx),
                end_column_index: Some(self.config.climb_color_column_idx + 1),
            },
        )
        .await?;

        Ok(())
    }
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
