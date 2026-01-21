use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client, Response};
use reqwest::{IntoUrl, Method, StatusCode, Url};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

mod entries;
pub mod login;

pub struct ApiClient {
    http_client: Client,
    token: String,
    base_url: Url,
}

impl ApiClient {
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

        let http_client = Client::builder()
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

    /// Sends a GET request to the relative URL with the given query parameters
    fn get(&self, url: impl IntoUrl, query: &[(&str, &str)]) -> Result<String> {
        let url: Url = url.into_url().context("Could not create URL")?;
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
    fn get_json<J>(&self, url: impl IntoUrl, query: &[(&str, &str)]) -> Result<J>
    where
        J: DeserializeOwned,
    {
        let url: Url = url.into_url().context("Could not create URL")?;
        let ctx = || format!("Could not send GET request to {url}");
        let resp: Response = self
            .http_client
            .get(url.clone())
            .bearer_auth(&self.token)
            .query(query)
            .send()
            .with_context(ctx)?;
        let text: String = handle_response(resp).with_context(ctx)?;
        let json: J = serde_json::from_str(&text).with_context(|| {
            format!("Could not extract JSON from success response from GET request to {url}")
        })?;
        Ok(json)
    }

    fn send_request<T: DeserializeOwned>(
        &self,
        method: Method,
        url: impl IntoUrl,
        query: &[(&str, &str)],
        body: Option<JsonValue>,
    ) -> Result<T> {
        let url: Url = url.into_url().context("Could not create URL")?;

        let resp: Response = self
            .http_client
            .request(method, url.clone())
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
