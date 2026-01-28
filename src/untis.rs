use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client, Response};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use serde::de::DeserializeOwned;

pub mod entries;
mod login;

use crate::json_util::improve_json_error;

pub struct UntisClient {
    http_client: Client,
    token: String,
    base_url: Url,
}

impl UntisClient {
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
        let json: J = serde_json::from_str(&text)
            .map_err(|e| improve_json_error(&e, &text))
            .with_context(|| {
                format!("Could not extract JSON from success response from GET request to {url}")
            })?;
        Ok(json)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error_message: Option<String>,
    validation_errors: Option<Vec<ValidationError>>,
    error_code: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ValidationError {
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
    let message: String = match serde_json::from_str::<ErrorResponse>(&text) {
        Ok(json) => extract_error(json),
        Err(err) => {
            log::warn!("Could not parse error json response: {err}");
            text
        }
    };

    bail!("Request failed with status {status}: {message}");
}

fn extract_error(err: ErrorResponse) -> String {
    if let Some(msg) = err.error_message
        && !msg.is_empty()
    {
        return msg;
    }

    if let Some(errors) = err.validation_errors
        && !errors.is_empty()
    {
        return errors
            .into_iter()
            .map(|x| x.error_message)
            .collect::<Vec<_>>() // itertools is more efficient tbh
            .join(" | ");
    }

    if let Some(msg) = err.error_code
        && !msg.is_empty()
    {
        return msg;
    }

    String::from("<unknown>")
}
