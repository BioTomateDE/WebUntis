use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client as HttpClient, Response};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use serde::de::DeserializeOwned;

mod entries;

const API_URL: &str = "webuntis.com/WebUntis/api/rest/view/v1";

pub struct Client {
    http_client: HttpClient,
    token: String,
    base_url: Url,
}

impl Client {
    pub fn new(token: String, school: &str) -> Result<Self> {
        const TOKEN_CHARSET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        const SCHOOL_CHARSET: &[u8; 27] = b"abcdefghijklmnopqrstuvwxyz-";

        if token.is_empty() {
            bail!("Token is empty");
        }
        if !token.bytes().all(|b| TOKEN_CHARSET.contains(&b)) {
            bail!("Token contains invalid character(s)");
        }

        if school.is_empty() {
            bail!("School is empty");
        }
        if !school.bytes().all(|b| SCHOOL_CHARSET.contains(&b)) {
            bail!("School contains invalid character(s)")
        }

        let http_client = HttpClient::builder()
            .build()
            .context("Could not build HTTP client")?;

        let base_url = format!("https://{school}.webuntis.com/WebUntis/api/rest/view/v1/");
        let base_url = Url::parse(&base_url)?;

        Ok(Self {
            http_client,
            token,
            base_url,
        })
    }

    fn send_get_request<T: DeserializeOwned>(
        &self,
        relative_url: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let url: Url = self
            .base_url
            .join(relative_url)
            .with_context(|| format!("Invalid URL for GET request: {relative_url:?}"))?;

        let resp: Response = self
            .http_client
            .get(url.clone())
            .bearer_auth(&self.token)
            .query(query)
            .send()
            .with_context(|| format!("Could not send GET request to {url}"))?;

        let status: StatusCode = resp.status();
        let text: String = resp.text().with_context(|| {
            format!("Could not extract text from GET request to {url} with status {status}")
        })?;

        if status.is_success() {
            let json: T = serde_json::from_str(&text).with_context(|| {
                format!("Could not extract JSON from success response from GET request to {url}")
            })?;
            return Ok(json);
        }

        // Request was not successful
        let message: String = if let Ok(json) = serde_json::from_str::<ErrorResponse>(&text) {
            json.error_message.unwrap_or_else(|| {
                json.validation_errors
                    .into_iter()
                    .map(|x| x.error_message)
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
        } else {
            // The request was so trash that serde could not even parse the json response
            text
        };

        bail!("GET request to {url} failed with status {status}: {message}");
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    /// Technically an enum but idc
    /// Probably corresponds to HTTP response reasons?
    error_code: String,
    error_message: Option<String>,
    validation_errors: Vec<ValidationError>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ValidationError {
    path: String,
    error_message: String,
}
