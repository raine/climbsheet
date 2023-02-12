extern crate google_sheets4 as sheets4;

use std::{collections::HashMap, sync::Arc};

use eyre::Result;
use lazy_static::lazy_static;

use regex::Regex;
use sheets4::{
    api::{
        AppendValuesResponse, BatchUpdateSpreadsheetRequest, CellData, CellFormat, Color,
        GridRange, RepeatCellRequest, Request, SortRangeRequest, SortSpec, ValueRange,
    },
    hyper::{self, client::HttpConnector},
    hyper_rustls::HttpsConnector,
    oauth2, Sheets,
};
use tokio::sync::Mutex;

pub type SheetsClient = Sheets<HttpsConnector<HttpConnector>>;
pub type Spreadsheet = sheets4::api::Spreadsheet;

pub async fn get_client() -> Result<SheetsClient> {
    let secret = sheets4::oauth2::read_service_account_key("credentials.json").await?;
    let connector = sheets4::hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .enable_http2()
        .build();
    let auth = oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await?;
    Ok(Sheets::new(hyper::Client::builder().build(connector), auth))
}

pub async fn get_spreadsheet(
    client: &SheetsClient,
    spreadsheet_id: &str,
) -> Result<sheets4::api::Spreadsheet> {
    let request = client.spreadsheets().get(spreadsheet_id);
    let response = request.doit().await?;
    Ok(response.1)
}

pub async fn append_row(
    sheets: &SheetsClient,
    sheet_id: &str,
    sheet_name: &str,
    row: Vec<String>,
) -> Result<sheets4::api::AppendValuesResponse> {
    let request = ValueRange {
        major_dimension: None,
        range: None,
        values: Some(vec![row]),
    };
    let range = sheet_name;
    let request = sheets
        .spreadsheets()
        .values_append(request, sheet_id, range)
        .insert_data_option("INSERT_ROWS")
        .value_input_option("USER_ENTERED");
    let (_, append_values_res) = request.doit().await?;
    Ok(append_values_res)
}

// Get numeric sheet id for sheet name
pub async fn get_sheet_id(sheets: &SheetsClient, sheet_id: &str, sheet_name: &str) -> Result<i32> {
    let request = sheets.spreadsheets().get(sheet_id);
    let (_, res) = request.doit().await?;
    let sheets = res.sheets.unwrap();
    let sheet = sheets
        .into_iter()
        .find(|s| s.properties.as_ref().unwrap().title == Some(sheet_name.to_string()))
        .unwrap();
    Ok(sheet.properties.unwrap().sheet_id.unwrap())
}

pub async fn memoized_get_sheet_id(
    sheets: &SheetsClient,
    sheet_id: &str,
    sheet_name: &str,
) -> Result<i32> {
    lazy_static! {
        static ref SHEET_ID_MAP: Arc<Mutex<HashMap<String, i32>>> =
            Arc::new(Mutex::new(HashMap::new()));
    }
    let mut sheet_id_map = SHEET_ID_MAP.lock().await;
    if let Some(sheet_id) = sheet_id_map.get(sheet_name) {
        return Ok(*sheet_id);
    }
    let sheet_id = get_sheet_id(sheets, sheet_id, sheet_name).await?;
    sheet_id_map.insert(sheet_name.to_string(), sheet_id);
    Ok(sheet_id)
}

pub async fn sort_sheet_by_column(
    sheets: &SheetsClient,
    sheet_id: &str,
    sheet_id_num: i32,
    column_idx: i32,
) -> Result<()> {
    let sort_range_request = SortRangeRequest {
        range: Some(GridRange {
            sheet_id: Some(sheet_id_num),
            start_column_index: Some(0),
            end_column_index: None,
            start_row_index: Some(1),
            end_row_index: None,
        }),
        sort_specs: Some(vec![SortSpec {
            dimension_index: Some(column_idx),
            sort_order: Some("DESCENDING".to_string()),
            ..Default::default()
        }]),
    };

    let sort_request = BatchUpdateSpreadsheetRequest {
        requests: Some(vec![Request {
            sort_range: Some(sort_range_request),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let request = sheets.spreadsheets().batch_update(sort_request, sheet_id);
    request.doit().await?;

    Ok(())
}

pub async fn set_cell_background_color(
    sheets: &SheetsClient,
    sheet_id: &str,
    sheet_id_num: i32,
    hex_color: &str,
    row: i32,
    column: i32,
) -> Result<()> {
    let style = CellFormat {
        background_color: Some(color_from_hex(hex_color)),
        ..Default::default()
    };

    let repeat_cell_req = RepeatCellRequest {
        range: Some(GridRange {
            sheet_id: Some(sheet_id_num),
            start_column_index: Some(column),
            end_column_index: Some(column + 1),
            start_row_index: Some(row),
            end_row_index: Some(row + 1),
        }),
        cell: Some(CellData {
            user_entered_format: Some(style),
            ..Default::default()
        }),
        fields: Some("*".to_string()),
    };

    let req = BatchUpdateSpreadsheetRequest {
        requests: Some(vec![Request {
            repeat_cell: Some(repeat_cell_req),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let _response = sheets
        .spreadsheets()
        .batch_update(req, sheet_id)
        .doit()
        .await?;

    Ok(())
}

pub async fn reset_row_format(
    sheets: &SheetsClient,
    sheet_id: &str,
    sheet_id_num: i32,
    row: i32,
) -> Result<()> {
    let style = CellFormat::default();
    let repeat_cell_req = RepeatCellRequest {
        range: Some(GridRange {
            sheet_id: Some(sheet_id_num),
            start_column_index: Some(0),
            end_column_index: None,
            start_row_index: Some(row),
            end_row_index: Some(row + 1),
        }),
        cell: Some(CellData {
            user_entered_format: Some(style),
            ..Default::default()
        }),
        fields: Some("*".to_string()),
    };

    let req = BatchUpdateSpreadsheetRequest {
        requests: Some(vec![Request {
            repeat_cell: Some(repeat_cell_req),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let _response = sheets
        .spreadsheets()
        .batch_update(req, sheet_id)
        .doit()
        .await?;

    Ok(())
}

pub async fn get_sheet_rows(
    sheets: &SheetsClient,
    sheet_id: &str,
    sheet_name: &str,
) -> Result<Vec<Vec<String>>> {
    let request = sheets
        .spreadsheets()
        .values_get(sheet_id, sheet_name)
        .major_dimension("ROWS");
    let (_, res) = request.doit().await?;
    Ok(res.values.unwrap_or_default())
}

/// Returns zero indexed row number
pub fn get_updated_row_from_update_values_response(
    append_values_res: &AppendValuesResponse,
) -> i32 {
    append_values_res
        .updates
        .as_ref()
        .and_then(|u| u.updated_range.as_ref())
        .map(|r| parse_row_from_range(r) - 1)
        .expect("expected row in update values response")
}

// Parse row number from range,
// for example "Ristikko - Reitit'!B12:F12"
fn parse_row_from_range(range: &str) -> i32 {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".+![A-Z]+(\d+):[A-Z]+\d+").unwrap();
    }

    let caps = RE.captures(range).unwrap();
    caps.get(1).unwrap().as_str().parse::<i32>().unwrap()
}

fn color_from_hex(hex: &str) -> Color {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap();
    Color {
        alpha: Some(1.0),
        blue: Some(b as f32 / 255.0),
        green: Some(g as f32 / 255.0),
        red: Some(r as f32 / 255.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_from_range_test() {
        let input = "'Ristikko - Reitit'!B12:F12";
        let expected = 12;
        let actual = parse_row_from_range(input);
        assert_eq!(expected, actual);
    }
}
