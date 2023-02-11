use std::{fs::File, io::Write};

use eyre::Result;
use reqwest::header::{HeaderMap, HeaderValue};

use super::{types::Gym, GymSectorFull};

#[derive(Debug)]
pub struct VerticalLifeClient {
    pub client: reqwest::Client,
    pub access_token: String,
    pub refresh_token: String,
}

const BASE_URL: &str = "https://vlcapi.vertical-life.info";
const USER_AGENT_VALUE: &str = "Vertical Life Climbing/6.14.0 (iPhone12,3; iOS 16.1.1; Scale/3.00)";

impl VerticalLifeClient {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .cookie_store(true)
                .build()
                .unwrap(),
            access_token,
            refresh_token,
        }
    }

    // Retry on unauthorized
    pub async fn get_gym_details(&self, gym_id: u32) -> Result<Gym> {
        let headers = make_headers(&self.access_token);
        let params = [("details", "overview")];
        let url = format!("{}/gyms/{}", BASE_URL, gym_id);
        let res = self
            .client
            .get(&url)
            .headers(headers)
            .query(&params)
            .send()
            .await?
            .error_for_status()?;
        let gym: Gym = res.json().await?;
        Ok(gym)
    }

    pub async fn get_gym_sector(&self, gym_sector_id: u32) -> Result<GymSectorFull> {
        let headers = make_headers(&self.access_token);
        let res = self
            .client
            .get(format!("{}/gym_sectors/{}", BASE_URL, gym_sector_id))
            .headers(headers)
            .send()
            .await?;

        let body = res.text().await?;
        // Write json body to file with gym sector id  formatted
        let mut file = File::create(format!("data/gym_sector_{}.json", gym_sector_id))?;
        file.write_all(body.as_bytes())?;
        let gym_sector = serde_json::from_str::<GymSectorFull>(&body)?;
        Ok(gym_sector)
    }
}

fn make_headers(access_token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
    );
    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert("accept-language", HeaderValue::from_static("en-FI"));
    headers.insert("x-app-version-code", HeaderValue::from_static("200"));
    headers.insert("x-app-id", HeaderValue::from_static("verticallife"));
    headers.insert("time-zone", HeaderValue::from_static("+0200"));
    headers.insert("user-agent", HeaderValue::from_static(USER_AGENT_VALUE));
    headers.insert("x-app-version", HeaderValue::from_static("6.14.0"));
    headers
}
