use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};
use eyre::Result;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue};
use scraper::{Html, Selector};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::*;

const BASE_URL: &str = "https://vlatka.vertical-life.info";

const NONCE_LENGTH: usize = 32;
const STATE_LENGTH: usize = 32;
const VERIFIER_LENGTH: usize = 32;

const BROWSER_USER_AGENT_VALUE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.1 Mobile/15E148 Safari/604.1";
const APP_USER_AGENT_VALUE: &str = "Vertical%20Life%20Climbing/4 CFNetwork/1399 Darwin/22.1.0";
const ACCEPT_VALUE: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE_VALUE: &str = "en-US,en;q=0.5";

const CLIENT_ID: &str = "vertical-life-ios";
const LOGIN_FORM_ID: &str = "#kc-form-login";
const REDIRECT_URI: &str = "vl-climbing://oauth2redirect";

#[derive(Debug)]
pub struct VerticalLifeAuthClient {
    pub client: reqwest::Client,
    pub code_verifier: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
}

impl VerticalLifeAuthClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .cookie_store(true)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            code_verifier: random_base64_string(VERIFIER_LENGTH),
        }
    }

    pub async fn authorize(&mut self) -> Result<String> {
        let url = format!(
            "{}/auth/realms/Vertical-Life/protocol/openid-connect/auth",
            BASE_URL
        );
        let state = random_base64_string(NONCE_LENGTH);
        let nonce = random_base64_string(STATE_LENGTH);
        let challenge = base64_encode(&sha256(self.code_verifier.to_string()));
        let headers = make_headers();
        let params = [
            ("response_type", "code"),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("scope", "openid profile email offline_access"),
            ("redirect_uri", REDIRECT_URI),
            ("client_id", CLIENT_ID),
            ("state", &state),
            ("nonce", &nonce),
        ];

        info!(url, state, nonce, challenge, ?params, "authorizing");
        let res = self
            .client
            .get(&url)
            .query(&params)
            .headers(headers)
            .send()
            .await?;

        let body = res.text().await?;
        let action_url = parse_action_url(&body)?;
        info!(action_url, "got action url");
        Ok(action_url)
    }

    pub async fn authenticate(
        &mut self,
        action_url: &str,
        username: &str,
        password: &str,
    ) -> Result<String> {
        let headers = make_headers();
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("username", username);
        params.insert("password", password);
        params.insert("rememberMe", "on");

        info!(url = action_url, "authenticating");
        let res = self
            .client
            .post(action_url)
            .headers(headers)
            .form(&params)
            .send()
            .await?;
        let redirect_url = res
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .expect("response should contain redirect_url");

        info!(redirect_url, "got redirect url");
        let parsed_url = url::Url::parse(redirect_url).unwrap();
        let query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
        let code = query
            .get("code")
            .expect("expected redirect url to have code query parameter");
        info!(code, "got code from redirect url");
        Ok(code.to_string())
    }

    pub async fn get_access_token(&mut self, code: &str) -> Result<TokenResult> {
        let url = format!(
            "{}/auth/realms/Vertical-Life/protocol/openid-connect/token",
            BASE_URL
        );
        let mut headers = make_headers();
        headers.insert("user-agent", HeaderValue::from_static(APP_USER_AGENT_VALUE));

        let mut params = HashMap::new();
        params.insert("client_id", CLIENT_ID);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", REDIRECT_URI);
        params.insert("code_verifier", &self.code_verifier);
        params.insert("code", code);

        info!(url, ?params, "getting access token");
        let res = self
            .client
            .post(&url)
            .headers(headers)
            .form(&params)
            .send()
            .await?;
        let body = res.text().await?;

        let token_result: TokenResult = serde_json::from_str(&body)?;
        Ok(token_result)
    }

    pub async fn refresh_token(refresh_token: &str) -> Result<TokenResult> {
        info!("refreshing access token");
        let auth_client = VerticalLifeAuthClient::new();
        let url = format!(
            "{}/auth/realms/Vertical-Life/protocol/openid-connect/token",
            BASE_URL
        );
        let mut headers = make_headers();
        headers.insert("user-agent", HeaderValue::from_static(APP_USER_AGENT_VALUE));

        let mut params = HashMap::new();
        params.insert("client_id", CLIENT_ID);
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);

        info!(url, ?params, "refreshing token");
        let res = auth_client
            .client
            .post(&url)
            .headers(headers)
            .form(&params)
            .send()
            .await?;
        let body = res.text().await?;
        dbg!(&body);

        let token_result: TokenResult = serde_json::from_str(&body)?;
        Ok(token_result)
    }

    pub async fn do_auth_flow(username: &str, password: &str) -> Result<TokenResult> {
        let mut client = VerticalLifeAuthClient::new();
        let action_url = client.authorize().await?;
        let code = client.authenticate(&action_url, username, password).await?;
        client.get_access_token(&code).await
    }
}

fn make_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "user-agent",
        HeaderValue::from_static(BROWSER_USER_AGENT_VALUE),
    );
    headers.insert("accept", HeaderValue::from_static(ACCEPT_VALUE));
    headers.insert(
        "accept-language",
        HeaderValue::from_static(ACCEPT_LANGUAGE_VALUE),
    );
    headers
}

fn parse_action_url(body: &str) -> Result<String> {
    let document = Html::parse_document(body);
    let selector = Selector::parse(LOGIN_FORM_ID).unwrap();
    let element = document.select(&selector).next().expect("no form on page");
    let action_url = element.value().attr("action").unwrap();
    Ok(action_url.to_string())
}

fn base64_encode(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(&data)
}

fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    rand::thread_rng().fill(&mut bytes[..]);
    bytes
}

fn random_base64_string(len: usize) -> String {
    base64_encode(&random_bytes(len))
}

fn sha256(data: String) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
