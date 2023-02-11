/// Returns for example "Ristikko - Reitit"
pub fn format_sheet_name(gym_location_name: &str, plural_human_item_type: &str) -> String {
    format!("{} - {}", gym_location_name, plural_human_item_type)
}

pub fn wall_category_to_plural_human_type(wall_category: &str) -> String {
    match wall_category {
        "gym_bouldering" => "Boulderit",
        "gym_sportclimbing" => "Reitit",
        _ => panic!("unknown wall category: {}", wall_category),
    }
    .to_string()
}

/// With input "Kiipeilyareena Ristikko" this should return "Ristikko"
pub fn parse_location_from_gym_name(gym_name: &str) -> &str {
    gym_name.split(' ').nth(1).unwrap()
}
