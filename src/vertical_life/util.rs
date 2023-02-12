use super::api::BASE_URL;

/// Returns https://vlcapi.vertical-life.info/images/<id>?width=3750
pub fn format_image_url(id: &str, width: i32) -> String {
    let width = match width {
        750 | 3750 => width,
        _ => panic!("invalid width: {}", width),
    };

    format!("{}/images/{}?width={}", BASE_URL, id, width)
}
