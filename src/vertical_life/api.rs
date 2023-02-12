use eyre::Result;
use tracing::*;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};

use super::{types::Gym, GymSectorFull, VerticalLifeAuthClient};

pub const BASE_URL: &str = "https://vlcapi.vertical-life.info";
const USER_AGENT_VALUE: &str = "Vertical Life Climbing/6.14.0 (iPhone12,3; iOS 16.1.1; Scale/3.00)";
const MAX_ATTEMPTS: u8 = 3;

#[derive(Debug)]
pub struct VerticalLifeClient {
    pub client: reqwest::Client,
    pub access_token: String,
    pub refresh_token: String,
}

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

    async fn refresh_access_token(&mut self) -> Result<()> {
        let token_result = VerticalLifeAuthClient::refresh_token(&self.refresh_token).await?;
        self.access_token = token_result.access_token;
        self.refresh_token = token_result.refresh_token;
        Ok(())
    }

    async fn make_request<T>(&mut self, request_fn: T) -> Result<reqwest::Response>
    where
        T: FnOnce(&mut reqwest::Client) -> reqwest::RequestBuilder + std::marker::Copy,
    {
        let mut attempts = 0;
        loop {
            let headers = make_headers(&self.access_token);
            let result = request_fn(&mut self.client)
                .headers(headers)
                .send()
                .await?
                .error_for_status();
            match result {
                Ok(res) => return Ok(res),
                Err(err) => {
                    error!(?err, "failed to make request");
                    if err.status() == Some(StatusCode::UNAUTHORIZED) && attempts < MAX_ATTEMPTS {
                        attempts += 1;
                        self.refresh_access_token().await?;
                    } else {
                        return Err(err.into());
                    }
                }
            }
        }
    }

    pub async fn get_gym_details(&mut self, gym_id: u32) -> Result<Gym> {
        info!(?gym_id, "getting gym details");
        let res = self
            .make_request(|client| {
                let params = [("details", "overview")];
                client
                    .get(&format!("{}/gyms/{}", BASE_URL, gym_id))
                    .form(&params)
            })
            .await?;
        let gym_sector = res.json().await?;
        Ok(gym_sector)
    }

    pub async fn get_gym_sector(&mut self, gym_sector_id: u32) -> Result<GymSectorFull> {
        info!(?gym_sector_id, "getting gym sector");
        let res = self
            .make_request(|client| {
                client.get(&format!("{}/gym_sectors/{}", BASE_URL, gym_sector_id))
            })
            .await?;
        let gym_sector = res.json().await?;
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
