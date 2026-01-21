use std::collections::HashSet;

use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client, Response};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use serde::de::DeserializeOwned;

mod entries;
mod login;

pub use login::login;

pub struct ApiClient {
    http_client: Client,
    pub token: String,
    base_url: Url,
}

impl ApiClient {
    pub fn new(http_client: Client, token: String, school: &str) -> Result<Self> {
        const TOKEN_CHARSET: &[u8; 65] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.";
        const SCHOOL_CHARSET: &[u8; 27] = b"abcdefghijklmnopqrstuvwxyz-";

        validate_charset("Token", &token, TOKEN_CHARSET)?;
        validate_charset("School Name", school, SCHOOL_CHARSET)?;

        let base_url = format!("https://{school}.webuntis.com/WebUntis/api/rest/view/v1/");
        let base_url = Url::parse(&base_url)?;

        Ok(Self {
            http_client,
            token,
            base_url,
        })
    }

    /// Sends a GET request to the relative URL with the given query parameters
    fn get(&self, relative_url: &str, query: &[(&str, &str)]) -> Result<String> {
        let url: Url = self
            .base_url
            .join(relative_url)
            .context("Could not create URL")?;
        let ctx = || format!("Could not send GET request to {url}");
        let resp: Response = self
            .http_client
            .get(url.clone())
            .bearer_auth(&self.token)
            .query(query)
            .send()
            .with_context(ctx)?;
        let text: String = handle_response(resp).with_context(ctx)?;
        Ok(text)
    }

    /// Sends a GET request to the relative URL with the given query parameters
    fn get_json<J>(&self, url: &str, query: &[(&str, &str)]) -> Result<J>
    where
        J: DeserializeOwned,
    {
        let text: String = self.get(url, query)?;
        let json: J = serde_json::from_str(&text).with_context(|| {
            format!("Could not extract JSON from success response from GET request to {url}")
        })?;
        Ok(json)
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

fn handle_response(response: Response) -> Result<String> {
    let status: StatusCode = response.status();
    let text: String = response
        .text()
        .with_context(|| format!("Could not extract text from response with status {status}"))?;

    if status.is_success() {
        return Ok(text);
    }

    // Request was not successful
    let message: String = if let Ok(json) = serde_json::from_str::<ErrorResponse>(&text) {
        json.error_message.unwrap_or_else(|| {
            json.validation_errors
                .into_iter()
                .map(|x| x.error_message)
                .collect::<Vec<_>>() // itertools is more efficient tbh
                .join(" | ")
        })
    } else {
        // The request was so trash that serde could not even parse the json response
        text
    };

    bail!("Request failed with status {status}: {message}");
}

fn validate_charset(description: &'static str, string: &str, charset: &'static [u8]) -> Result<()> {
    if string.is_empty() {
        bail!("{description} is empty");
    }
    if string.bytes().all(|b| charset.contains(&b)) {
        return Ok(());
    }
    let set = string
        .bytes()
        .filter(|b| !charset.contains(b))
        .map(char::from)
        .collect::<HashSet<char>>();
    bail!("{description} contains invalid characters: {set:?}");
}
